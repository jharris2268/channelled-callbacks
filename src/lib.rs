mod callback;
mod utils;

pub use callback::{CallFinish, Callback, CallbackMerge, CallbackSync};
pub use utils::{MergeTimings, CallAll, ReplaceNoneWithTimings, Timings};


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
