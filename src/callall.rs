use crate::{CallFinish,Timings};
use std::marker::PhantomData;

use crate::{Result};
use cpu_time::ThreadTime;



fn as_secs(dur: std::time::Duration) -> f64 {
    (dur.as_secs() as f64) * 1.0 + (dur.subsec_nanos() as f64) * 0.000000001
}

pub struct CallAll<
    T: CallFinish + ?Sized,
    U: Sync + Send + 'static,
    W: Fn(U) -> T::CallType,
    V,
    E,
> {
    out: Box<T>,
    tm: f64,
    msg: String,
    callfunc: Box<W>,
    x: PhantomData<U>,
    y: PhantomData<V>,
    z: PhantomData<E>
}

impl<T, U, W, V, E> CallAll<T, U, W, V, E>
where
    T: CallFinish<ReturnType = Timings<V>, ErrorType = E> + ?Sized,
    U: Sync + Send + 'static,
    W: Fn(U) -> T::CallType + Sync + Send + 'static,
    V: Sync + Send + 'static,
{
    pub fn new(out: Box<T>, msg: &str, callfunc: Box<W>) -> CallAll<T, U, W, V, E> {
        CallAll {
            out: out,
            msg: String::from(msg),
            tm: 0.0,
            callfunc: callfunc,
            x: PhantomData,
            y: PhantomData,
            z: PhantomData
        }
    }
}

impl<T, U, W, V, E> CallFinish for CallAll<T, U, W, V, E>
where
    T: CallFinish<ReturnType = Timings<V>, ErrorType = E> + ?Sized,
    U: Sync + Send + 'static,
    W: Fn(U) -> T::CallType + Sync + Send + 'static,
    V: Sync + Send + 'static,
    E: std::error::Error + Sync + Send + 'static
{
    type CallType = U;
    type ReturnType = Timings<V>;
    type ErrorType = E;

    fn call(&mut self, c: U) {
        let tx = ThreadTime::now();
        let r = (self.callfunc)(c);
        self.tm += as_secs(tx.elapsed());
        self.out.call(r);
    }

    fn finish(&mut self) -> Result<Timings<V>, E> {
        match self.out.finish() {
            Ok(mut t) => {
                t.add(self.msg.as_str(), self.tm);
                Ok(t)
            },
            Err(e) => Err(e)
        }
    }
}
