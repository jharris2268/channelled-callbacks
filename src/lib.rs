//! Channelled Callbacks is a library for Rust to help to simplifiy concurrent programming involving
//! streams of data. It is based on the
//! [message-passing utilities](https://doc.rust-lang.org/stable/std/sync/mpsc/index.html)
//! provided by Rust, but wrapped with data passed through callback functions.
//!



mod callfinish;
mod callback;
mod callbackmerge;
mod callbacksync;

mod mergetimings;
mod callall;
mod replacenonewithtimings;
mod timings;

mod reversecallback;

pub use callfinish::{CallFinish,CollectResult};
pub use callback::Callback;
pub use callbackmerge::CallbackMerge;
pub use callbacksync::CallbackSync;
pub use mergetimings::MergeTimings;
pub use callall::CallAll;
pub use replacenonewithtimings::ReplaceNoneWithTimings;
pub use timings::Timings;
pub use reversecallback::ReverseCallback;



#[derive(Debug)]
pub enum Error<E: std::error::Error + Send + 'static>  {
    ChannelledCallbackError(std::string::String),
    OtherError(E)
}




impl<E> std::error::Error for Error<E> 
    where E: std::error::Error + Send + 'static
{
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::ChannelledCallbackError(_) => None,
            Error::OtherError(e) => Some(e)
        }
    }
}

impl<E> std::fmt::Display for Error<E>
    where E: std::error::Error + Send + 'static
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::ChannelledCallbackError(e) => write!(f, "ChannelledCallbackError {}", e),
            Error::OtherError(e) => write!(f, "{:?}", e)
        }
    }
}

impl<E> std::convert::From<E> for Error<E>
    where E: std::error::Error + Send + 'static
{
    fn from(e: E) -> Self {
        Error::OtherError(e)
    }
}

pub type Result<T, E> = std::result::Result<T, Error<E>>;


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

