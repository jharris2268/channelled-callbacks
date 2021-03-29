use crate::{CallFinish,Timings};
use std::io::Result;


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
