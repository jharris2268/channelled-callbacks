use crate::{Error, Result};
use std::sync::mpsc;
use std::thread;

use crate::CallFinish;



fn call_all_sync<T: Send + 'static, U: Send + 'static, E: std::error::Error+Send+'static>(
    recvs: Vec<mpsc::Receiver<T>>,
    mut cf: Box<impl CallFinish<CallType = T, ReturnType = U, ErrorType = E> + ?Sized>,
) -> Result<U, E> {
    let mut i = 0;
    let l = recvs.len();
    let mut nf = 0;
    loop {
        match recvs[i % l].recv() {
            Ok(m) => cf.call(m),

            Err(_) => {
                nf += 1;
                if nf == l {
                    return cf.finish();
                }
            }
        }
        i += 1;
    }
}

static MAXNUMCHAN: usize = 32;

pub struct CallbackSync<T, U,E: std::error::Error + Send + 'static> {
    send: Option<mpsc::SyncSender<T>>,
    result: Option<thread::JoinHandle<Result<U,E>>>,
    expectresult: bool,
    //th: usize
}

impl<T, U, E> CallbackSync<T, U, E>
where
    T: Send + 'static,
    U: Send + 'static,
    E: std::error::Error + Send + 'static
{
    pub fn new(
        cf: Box<impl CallFinish<CallType = T, ReturnType = U, ErrorType = E> + ?Sized>,
        numchan: usize,
    ) -> Vec<Box<CallbackSync<T, U, E>>> {
        if numchan == 0 || numchan > MAXNUMCHAN {
            panic!(
                "wrong numchan {}: must between 1 and {}",
                numchan, MAXNUMCHAN
            );
        }
        let mut sends = Vec::new();
        let mut recvs = Vec::new();

        for _ in 0..numchan {
            let (send, recv) = mpsc::sync_channel(1);
            sends.push(send);
            recvs.push(recv);
        }

        let mut res = Vec::new();

        let result = thread::spawn(move || call_all_sync(recvs, cf));
        res.push(Box::new(CallbackSync {
            send: sends.pop(),
            result: Some(result),
            expectresult: true,
        }));

        for _ in 1..numchan {
            res.push(Box::new(CallbackSync {
                send: sends.pop(),
                result: None,
                expectresult: false,
            }));
        }
        res.reverse();
        res
    }
}

impl<T, U, E> CallFinish for CallbackSync<T, U, E>
where
    T: Send + 'static,
    U: Send + 'static,
    E: std::error::Error + Send + 'static
{
    type CallType = T;
    type ReturnType = Option<U>;
    type ErrorType = E;

    fn call(&mut self, t: T) {
        match &self.send {
            Some(s) => {
                s.send(t).expect("failed to send");
            }
            _ => {}
        }
    }

    fn finish(&mut self) -> Result<Option<U>,E> {
        self.send = None;

        if !self.expectresult {
            return Ok(None);
        }

        let r = std::mem::replace(&mut self.result, None);

        match r {
            Some(r) => match r.join() {
                Ok(p) => match p {
                    Ok(q) => Ok(Some(q)),

                    Err(e) => Err(e),
                },
                Err(e) => Err(Error::ChannelledCallbackError(
                    format!("failed to join {:?}", e),
                )),
            },
            None => Err(Error::ChannelledCallbackError("already called finish".to_string())),
        }
    }
}
