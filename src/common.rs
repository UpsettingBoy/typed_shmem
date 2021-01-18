use zerocopy::{AsBytes, FromBytes};

pub(crate) trait ShMemOps<T>: Drop
where
    T: AsBytes + FromBytes + Default,
{
    fn get_t(&self) -> &T;
    fn get_t_mut(&mut self) -> &mut T;
}
