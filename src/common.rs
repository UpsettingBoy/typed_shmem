use zerocopy::{AsBytes, FromBytes};

pub(crate) trait ShMemOps<T>
where
    T: AsBytes + FromBytes + Default,
{
    unsafe fn get_t(&self) -> &T;
    unsafe fn get_t_mut(&mut self) -> &mut T;
}
