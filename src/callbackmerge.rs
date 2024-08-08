
use crate::{CallFinish,CollectResult};
use crate::{Error,Result};

pub struct CallbackMerge<T, U, V, E> {
    callbacks: Vec<Box<dyn CallFinish<CallType = T, ReturnType = U, ErrorType = E>>>,
    collect: Box<dyn CollectResult<InType = U, OutType = V>>,
    idx: usize,
}

impl<T, U, V, E> CallbackMerge<T, U, V, E>
where
    T: Send + 'static,
    U: Send + 'static,
    V: Send + 'static,
    E: std::error::Error + Send + 'static
{
    pub fn new(
        callbacks: Vec<Box<dyn CallFinish<CallType = T, ReturnType = U, ErrorType = E>>>,
        collect: Box<dyn CollectResult<InType = U, OutType = V>>,
    ) -> CallbackMerge<T, U, V, E> {
        CallbackMerge {
            callbacks: callbacks,
            collect: collect,
            idx: 0,
        }
    }
}

impl<T, U, V, E> CallFinish for CallbackMerge<T, U, V, E>
where
    T: Send + 'static,
    U: Send + 'static,
    V: Send + 'static,
    E: std::error::Error + Send + 'static
{
    type CallType = T;
    type ReturnType = V;
    type ErrorType = E;

    fn call(&mut self, t: T) {
        let l = self.callbacks.len();
        self.callbacks[self.idx % l].call(t);
        self.idx += 1;
    }

    fn finish(&mut self) -> Result<Self::ReturnType, Self::ErrorType> {
        let mut r = Vec::new();
        let mut err: Option<Error<E>> = None;
        for c in self.callbacks.iter_mut() {
            match c.finish() {
                Ok(s) => {
                    r.push(s);
                }
                Err(e) => {
                    err = Some(e);
                }
            }
        }

        match err {
            Some(e) => Err(e),
            None => Ok(self.collect.collect(r)),
        }
    }
}
