use crate::{CollectResult,Timings};
use std::marker::PhantomData;



pub struct MergeTimings<U: Sync + Send + 'static>(PhantomData<U>);

impl<U: Sync + Send + 'static> MergeTimings<U> {
    pub fn new() -> MergeTimings<U> {
        MergeTimings(PhantomData)
    }
}

impl<U> CollectResult for MergeTimings<U>
where
    U: Sync + Send + 'static,
{
    type InType = Timings<U>;
    type OutType = Timings<U>;

    fn collect(&self, vv: Vec<Self::InType>) -> Self::OutType {
        let mut vv = vv;
        if vv.is_empty() {
            return Timings::new();
        }

        let mut r = vv.pop().unwrap();
        if vv.len() > 0 {
            for v in vv {
                r.combine(v);
            }
        }
        r
    }
}
