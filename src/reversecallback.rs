use crate::Result;
use std::sync::mpsc;
use std::thread;

use crate::{CallFinish,Timings};

struct ReverseCallbackCallFinish<T,U,E> {
    send: Option<mpsc::SyncSender<T>>,
    p: std::marker::PhantomData<U>,
    z: std::marker::PhantomData<E>
}

impl<T,U,E> CallFinish for ReverseCallbackCallFinish<T,U,E>
where
    T: Send + 'static,
    U: Sync + Send + 'static, 
    E: std::error::Error + Sync + Send + 'static
{
    type CallType = T;
    type ReturnType = Timings<U>;
    type ErrorType = E;
    
    fn call(&mut self, t: T) {
        match &self.send {
            Some(s) => {
                s.send(t).expect("failed to send");
            }
            _ => {}
        }
    }

    fn finish(&mut self) -> Result<Timings<U>, E> {
        self.send = None;
        Ok(Timings::new())
    }
}


pub struct ReverseCallback<T, U: Sync+Send+'static, E: std::error::Error + Sync + Send + 'static > {
    recv: mpsc::Receiver<T>,
    result: Option<thread::JoinHandle<std::result::Result<Timings<U>, E>>>,
    p: std::marker::PhantomData<U>
}


impl<T, U, E> ReverseCallback<T, U, E>
where
    T: Send + 'static,
    U: Sync + Send + 'static,
    E: std::error::Error + Sync + Send + 'static
{
    pub fn new<F: 'static + FnOnce(Box<dyn CallFinish<CallType = T, ReturnType = Timings<U>, ErrorType = E>>) -> std::result::Result<Timings<U>, E>  + Send >(caller: F) -> ReverseCallback<T, U, E> {
        let (send, recv) = mpsc::sync_channel(1);
        
        let cb = Box::new(ReverseCallbackCallFinish{
                send: Some(send),
                p: std::marker::PhantomData,
                z: std::marker::PhantomData
            });
        
        let result = thread::spawn(move || caller(cb));

        ReverseCallback {
            recv: recv,
            result: Some(result),
            p: std::marker::PhantomData
        }
    }
    
    pub fn collect_result(&mut self) -> Result<Timings<U>, E> {
        Ok(self.result.take().unwrap().join().expect("!!")?)
    }
    
    /*
    pub fn iter<'a>(&'a mut self) -> Box<dyn Iterator<Item=T> + 'a> {
        Box::new(self.recv.iter())
    }*/
    
}


impl<T,U, E> Iterator for ReverseCallback<T, U, E> 
where
    T: Send + 'static,
    U: Sync + Send + 'static,
    E: std::error::Error + Sync + Send + 'static
{
    type Item = T;
    
    fn next(&mut self) -> Option<T> {
        match self.recv.recv() {
            Ok(x) => Some(x),
            Err(_) => {
                //println!("{:?}", e);
                //only error is to indicate channel is closed...
                None
            }
        }
    }
}



