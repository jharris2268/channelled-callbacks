use crate::{CallFinish,Timings};
use crate::Result;


pub struct ReplaceNoneWithTimings<T: ?Sized> {
    out: Box<T>,
}
impl<T> ReplaceNoneWithTimings<T>
where T: ?Sized
{
    pub fn new(out: Box<T>) -> ReplaceNoneWithTimings<T> {
        ReplaceNoneWithTimings { out }
    }
}

impl<T, U, E> CallFinish for ReplaceNoneWithTimings<T>
where
    T: CallFinish<ReturnType = Option<Timings<U>>, ErrorType = E> + ?Sized,
    U: Sync + Send + 'static,
    E: std::error::Error + Send + 'static
{
    type CallType = T::CallType;
    type ReturnType = Timings<U>;
    type ErrorType = E;

    fn call(&mut self, c: Self::CallType) {
        self.out.call(c);
    }

    fn finish(&mut self) -> Result<Self::ReturnType, Self::ErrorType> {
        let x = self.out.finish()?;
        match x {
            None => Ok(Timings::new()),
            Some(y) => Ok(y),
        }
    }
}
