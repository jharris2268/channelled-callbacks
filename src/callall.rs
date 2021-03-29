use crate::{CallFinish,Timings};
use std::marker::PhantomData;
use std::io::Result;
use cpu_time::ThreadTime;



fn as_secs(dur: std::time::Duration) -> f64 {
    (dur.as_secs() as f64) * 1.0 + (dur.subsec_nanos() as f64) * 0.000000001
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
        let tx = ThreadTime::now();
        let r = (self.callfunc)(c);
        self.tm += as_secs(tx.elapsed());
        self.out.call(r);
    }

    fn finish(&mut self) -> Result<Timings<V>> {
        let mut t = self.out.finish()?;
        t.add(self.msg.as_str(), self.tm);
        Ok(t)
    }
}
