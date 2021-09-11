use std::io::Result;
use std::sync::mpsc;
use std::thread;

use crate::{CallFinish,Timings};

struct ReverseCallbackCallFinish<T,U> {
    send: Option<mpsc::SyncSender<T>>,
    p: std::marker::PhantomData<U>
}

impl<T,U> CallFinish for ReverseCallbackCallFinish<T,U>
where
    T: Send + 'static,
    U: Sync + Send + 'static
{
    type CallType = T;
    type ReturnType = Timings<U>;
    fn call(&mut self, t: T) {
        match &self.send {
            Some(s) => {
                s.send(t).expect("failed to send");
            }
            _ => {}
        }
    }

    fn finish(&mut self) -> Result<Timings<U>> {
        self.send = None;
        Ok(Timings::new())
    }
}


pub struct ReverseCallback<T, U: Sync+Send+'static> {
    recv: mpsc::Receiver<T>,
    result: Option<thread::JoinHandle<Result<Timings<U>>>>,
    p: std::marker::PhantomData<U>
}


impl<T, U> ReverseCallback<T, U>
where
    T: Send + 'static,
    U: Sync + Send + 'static,
{
    pub fn new<F: 'static + FnOnce(Box<dyn CallFinish<CallType = T, ReturnType = Timings<U>>>) -> Result<Timings<U>>  + Send >(caller: F) -> ReverseCallback<T, U> {
        let (send, recv) = mpsc::sync_channel(1);
        
        let cb = Box::new(ReverseCallbackCallFinish{send: Some(send),p:std::marker::PhantomData});
        
        let result = thread::spawn(move || caller(cb));

        ReverseCallback {
            recv: recv,
            result: Some(result),
            p:std::marker::PhantomData
        }
    }
    
    pub fn collect_result(&mut self) -> Result<Timings<U>> {
        self.result.take().unwrap().join().expect("!!")
    }
    
}


impl<T,U> Iterator for ReverseCallback<T, U> 
where
    T: Send + 'static,
    U: Sync + Send + 'static,
{
    type Item = T;
    
    fn next(&mut self) -> Option<T> {
        match self.recv.recv() {
            Ok(x) => Some(x),
            Err(e) => {
                println!("{:?}", e);
                None
            }
        }
    }
}



