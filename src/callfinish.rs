use std::io::Result;


pub trait CallFinish: Sync + Send + 'static {
    type CallType;
    type ReturnType;
    fn call(&mut self, f: Self::CallType);
    fn finish(&mut self) -> Result<Self::ReturnType>;
}




pub trait CollectResult: Sync + Send + 'static {
    type InType;
    type OutType;
    fn collect(&self, a: Vec<Self::InType>) -> Self::OutType;
}
