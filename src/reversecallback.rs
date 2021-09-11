use std::io::{Error, ErrorKind, Result};
use std::sync::mpsc;
use std::thread;

use crate::CallFinish;

struct ReverseCallbackCallFinish<T> {
    send: Option<mpsc::SyncSender<T>>,
    
}

impl<T> CallFinish for Callback<T>
where
    T: Send + 'static,
    
{
    type CallType = T;
    type ReturnType = ();
    fn call(&mut self, t: T) {
        match &self.send {
            Some(s) => {
                s.send(t).expect("failed to send");
            }
            _ => {}
        }
    }

    fn finish(&mut self) -> Result<()> {
        self.send = None;
        Ok(())
    }
}


pub struct ReverseCallback<T, U> {
    recv: Option<mpsc::Receiver<T>>,
    result: Option<thread::JoinHandle<Result<U>>>,
}


impl<T, U> ReverseCallback<T, U>
where
    T: Send + 'static,
    U: Send + 'static,
{
    pub fn new(caller: Fn(Box<impl CallFinish<CallType = T, ReturnType = ()>>) -> Result<U>) -> ReverseCallback<T, U> {
        let (send, recv) = mpsc::sync_channel(1);
        
        let cb = Box::new(ReverseCallbackCallFinish{send: send});
        
        let result = thread::spawn(move || caller(cb));

        Callback {
            recv: Some(recv),
            result: Some(result),
        }
    }
    
    pub fn collect_result(&self) -> Result<U> {
        self.result.join()
    }
    
}


impl Iterator for ReverseCallback<T, U> 
where
    T: Send + 'static,
    U: Send + 'static,
{
    type Item = T;
    
    fn next(&mut self) -> Option<T> {
        self.recv.recv()
    }
}



