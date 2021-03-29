use std::io::{Error, ErrorKind, Result};
use std::sync::mpsc;
use std::thread;

use crate::CallFinish;



fn call_all_sync<T: Send + 'static, U: Send + 'static>(
    recvs: Vec<mpsc::Receiver<T>>,
    mut cf: Box<impl CallFinish<CallType = T, ReturnType = U> + ?Sized>,
) -> Result<U> {
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

static MAXNUMCHAN: usize = 8;

pub struct CallbackSync<T, U> {
    send: Option<mpsc::SyncSender<T>>,
    result: Option<thread::JoinHandle<Result<U>>>,
    expectresult: bool,
    //th: usize
}

impl<T, U> CallbackSync<T, U>
where
    T: Send + 'static,
    U: Send + 'static,
{
    pub fn new(
        cf: Box<impl CallFinish<CallType = T, ReturnType = U> + ?Sized>,
        numchan: usize,
    ) -> Vec<Box<CallbackSync<T, U>>> {
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

impl<T, U> CallFinish for CallbackSync<T, U>
where
    T: Send + 'static,
    U: Send + 'static,
{
    type CallType = T;
    type ReturnType = Option<U>;

    fn call(&mut self, t: T) {
        match &self.send {
            Some(s) => {
                s.send(t).expect("failed to send");
            }
            _ => {}
        }
    }

    fn finish(&mut self) -> Result<Option<U>> {
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
                Err(e) => Err(Error::new(
                    ErrorKind::Other,
                    format!("failed to join {:?}", e),
                )),
            },
            None => Err(Error::new(ErrorKind::Other, "already called finish")),
        }
    }
}
