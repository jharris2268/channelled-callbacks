use crate::{Error, Result};
use std::sync::mpsc;
use std::thread;

use crate::CallFinish;

fn call_all<T: Send + 'static, U: Send + 'static, E: std::error::Error + Send + 'static>(
    recv: mpsc::Receiver<T>,
    mut cf: Box<impl CallFinish<CallType = T, ReturnType = U, ErrorType = E>>,
) -> Result<U, E> {
    for m in recv.iter() {
        cf.call(m);
    }
    cf.finish()
}



pub struct Callback<T, U, E: std::error::Error + Send + 'static> {
    send: Option<mpsc::SyncSender<T>>,
    result: Option<thread::JoinHandle<Result<U, E>>>,
}
impl<T, U, E> Callback<T, U, E>
where
    T: Send + 'static,
    U: Send + 'static,
    E: std::error::Error + Send + 'static
{
    pub fn new(cf: Box<impl CallFinish<CallType = T, ReturnType = U, ErrorType = E>>) -> Callback<T, U, E> {
        let (send, recv) = mpsc::sync_channel(1);

        let result = thread::spawn(move || call_all(recv, cf));

        Callback {
            send: Some(send),
            result: Some(result)
        }
    }
}

impl<T, U, E> CallFinish for Callback<T, U, E>
where
    T: Send + 'static,
    U: Send + 'static,
    E: std::error::Error + Send + 'static
{
    type CallType = T;
    type ReturnType = U;
    type ErrorType = E;
    
    fn call(&mut self, t: T) {
        match &self.send {
            Some(s) => {
                s.send(t).expect("failed to send");
            }
            _ => {}
        }
    }

    fn finish(&mut self) -> Result<U, E> {
        self.send = None;

        let r = std::mem::replace(&mut self.result, None);

        match r {
            Some(r) => match r.join() {
                Ok(p) => {
                    match p {
                        Ok(t) => Ok(t),
                        Err(e) => Err(e)
                    }
                },
                        
                Err(e) => Err(Error::ChannelledCallbackError(
                    format!("failed to join {:?}", e),
                )),
            },
            None => Err(Error::ChannelledCallbackError("already called finish".to_string())),
        }
    }
}


