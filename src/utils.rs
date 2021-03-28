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

