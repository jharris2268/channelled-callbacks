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
