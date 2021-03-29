use std::io::{Error, ErrorKind, Result};
use std::sync::mpsc;
use std::thread;

use crate::CallFinish;

fn call_all<T: Send + 'static, U: Send + 'static>(
    recv: mpsc::Receiver<T>,
    mut cf: Box<impl CallFinish<CallType = T, ReturnType = U>>,
) -> Result<U> {
    for m in recv.iter() {
        cf.call(m);
    }

    cf.finish()
}



pub struct Callback<T, U> {
    send: Option<mpsc::SyncSender<T>>,
    result: Option<thread::JoinHandle<Result<U>>>,
}
impl<T, U> Callback<T, U>
where
    T: Send + 'static,
    U: Send + 'static,
{
    pub fn new(cf: Box<impl CallFinish<CallType = T, ReturnType = U>>) -> Callback<T, U> {
        let (send, recv) = mpsc::sync_channel(1);

        let result = thread::spawn(move || call_all(recv, cf));

        Callback {
            send: Some(send),
            result: Some(result),
        }
    }
}

impl<T, U> CallFinish for Callback<T, U>
where
    T: Send + 'static,
    U: Send + 'static,
{
    type CallType = T;
    type ReturnType = U;
    fn call(&mut self, t: T) {
        match &self.send {
            Some(s) => {
                s.send(t).expect("failed to send");
            }
            _ => {}
        }
    }

    fn finish(&mut self) -> Result<U> {
        self.send = None;

        let r = std::mem::replace(&mut self.result, None);

        match r {
            Some(r) => match r.join() {
                Ok(p) => p,
                Err(e) => Err(Error::new(
                    ErrorKind::Other,
                    format!("failed to join {:?}", e),
                )),
            },
            None => Err(Error::new(ErrorKind::Other, "already called finish")),
        }
    }
}


