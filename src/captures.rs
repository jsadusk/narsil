#[allow(unused_lifetimes)]
pub trait Captures<'a> {}
impl<'a, T: ?Sized> Captures<'a> for T {}
