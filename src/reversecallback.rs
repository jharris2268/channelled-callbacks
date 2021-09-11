use std::io::Result;
use std::sync::mpsc;
use std::thread;

use crate::CallFinish;

struct ReverseCallbackCallFinish<T> {
    send: Option<mpsc::SyncSender<T>>,
    
}

impl<T> CallFinish for ReverseCallbackCallFinish<T>
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
    recv: mpsc::Receiver<T>,
    result: Option<thread::JoinHandle<Result<U>>>,
}


impl<T, U> ReverseCallback<T, U>
where
    T: Send + 'static,
    U: Send + 'static,
{
    pub fn new<F: 'static + Fn(Box<dyn CallFinish<CallType = T, ReturnType = ()>>) -> Result<U>  + Send >(caller: F) -> ReverseCallback<T, U> {
        let (send, recv) = mpsc::sync_channel(1);
        
        let cb = Box::new(ReverseCallbackCallFinish{send: Some(send)});
        
        let result = thread::spawn(move || caller(cb));

        ReverseCallback {
            recv: recv,
            result: Some(result),
        }
    }
    
    pub fn collect_result(&mut self) -> Result<U> {
        self.result.take().unwrap().join().expect("!!")
    }
    
}


impl<T,U> Iterator for ReverseCallback<T, U> 
where
    T: Send + 'static,
    U: Send + 'static,
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



