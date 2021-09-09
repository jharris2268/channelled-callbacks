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

pub use callfinish::{CallFinish,CollectResult};
pub use callback::Callback;
pub use callbackmerge::CallbackMerge;
pub use callbacksync::CallbackSync;
pub use mergetimings::MergeTimings;
pub use callall::CallAll;
pub use replacenonewithtimings::ReplaceNoneWithTimings;
pub use timings::Timings;




#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
