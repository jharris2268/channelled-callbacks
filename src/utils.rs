use crate::callback::{CallFinish, CollectResult};
use std::collections::HashMap;
use std::fmt;
use std::io::{Error, ErrorKind, Result};
use std::marker::PhantomData;

pub struct MergeTimings<U: Sync + Send + 'static>(PhantomData<U>);

impl<U: Sync + Send + 'static> MergeTimings<U> {
    pub fn new() -> MergeTimings<U> {
        MergeTimings(PhantomData)
    }
}

impl<U> CollectResult for MergeTimings<U>
where
    U: Sync + Send + 'static,
{
    type InType = Timings<U>;
    type OutType = Timings<U>;

    fn collect(&self, vv: Vec<Self::InType>) -> Self::OutType {
        let mut vv = vv;
        if vv.is_empty() {
            return Timings::new();
        }

        let mut r = vv.pop().unwrap();
        if vv.len() > 0 {
            for v in vv {
                r.combine(v);
            }
        }
        r
    }
}

pub struct ReplaceNoneWithTimings<T> {
    out: Box<T>,
}
impl<T> ReplaceNoneWithTimings<T> {
    pub fn new(out: Box<T>) -> ReplaceNoneWithTimings<T> {
        ReplaceNoneWithTimings { out }
    }
}

impl<T, U> CallFinish for ReplaceNoneWithTimings<T>
where
    T: CallFinish<ReturnType = Option<Timings<U>>>,
    U: Sync + Send + 'static,
{
    type CallType = T::CallType;
    type ReturnType = Timings<U>;

    fn call(&mut self, c: Self::CallType) {
        self.out.call(c);
    }

    fn finish(&mut self) -> Result<Self::ReturnType> {
        let x = self.out.finish()?;
        match x {
            None => Ok(Timings::new()),
            Some(y) => Ok(y),
        }
    }
}

pub struct CallAll<
    T: CallFinish + ?Sized,
    U: Sync + Send + 'static,
    W: Fn(U) -> T::CallType,
    V,
> {
    out: Box<T>,
    tm: f64,
    msg: String,
    callfunc: Box<W>,
    x: PhantomData<U>,
    y: PhantomData<V>,
}

impl<T, U, W, V> CallAll<T, U, W, V>
where
    T: CallFinish<ReturnType = Timings<V>> + ?Sized,
    U: Sync + Send + 'static,
    W: Fn(U) -> T::CallType + Sync + Send + 'static,
    V: Sync + Send + 'static,
{
    pub fn new(out: Box<T>, msg: &str, callfunc: Box<W>) -> CallAll<T, U, W, V> {
        CallAll {
            out: out,
            msg: String::from(msg),
            tm: 0.0,
            callfunc: callfunc,
            x: PhantomData,
            y: PhantomData,
        }
    }
}

impl<T, U, W, V> CallFinish for CallAll<T, U, W, V>
where
    T: CallFinish<ReturnType = Timings<V>> + ?Sized,
    U: Sync + Send + 'static,
    W: Fn(U) -> T::CallType + Sync + Send + 'static,
    V: Sync + Send + 'static,
{
    type CallType = U;
    type ReturnType = Timings<V>;

    fn call(&mut self, c: U) {
        let tx = ThreadTimer::new();
        let r = (self.callfunc)(c);
        self.tm += tx.since();
        self.out.call(r);
    }

    fn finish(&mut self) -> Result<Timings<V>> {
        let mut t = self.out.finish()?;
        t.add(self.msg.as_str(), self.tm);
        Ok(t)
    }
}


pub struct Timings<T: Sync + Send + 'static> {
    pub timings: HashMap<String, f64>,
    pub others: Vec<(String, T)>,
}

impl<T> Timings<T>
where
    T: Sync + Send + 'static,
{
    pub fn new() -> Timings<T> {
        Timings {
            timings: HashMap::new(),
            others: Vec::new(),
        }
    }

    pub fn add(&mut self, k: &str, v: f64) {
        self.timings.insert(String::from(k), v);
    }
    pub fn add_other(&mut self, k: &str, v: T) {
        self.others.push((String::from(k), v));
    }

    pub fn combine(&mut self, mut other: Self) {
        for (k, v) in other.timings {
            if self.timings.contains_key(&k) {
                *self.timings.get_mut(&k).unwrap() += v;
            } else {
                self.timings.insert(k, v);
            }
        }
        for (a, b) in std::mem::take(&mut other.others) {
            self.others.push((a, b));
        }
    }
}
impl<T> fmt::Display for Timings<T>
where
    T: Sync + Send + 'static,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut fs = String::new();
        for (k, v) in &self.timings {
            fs = format!("{}\n{}: {:0.1}s", fs, k, v);
        }
        write!(f, "Timings: {}", fs)
    }
}
