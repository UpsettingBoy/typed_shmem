use zerocopy::{AsBytes, FromBytes};

/// Controls how a memory region can be accessed.
/// See the implementation details of `ShObj` under *nix and Windows
/// for hints on how this can backfire.
pub trait ShMemOps<T>
where
    T: AsBytes + FromBytes + Default,
{
    /// Gets a reference to the shared memory region data.
    /// # Returns
    /// Immutable reference.
    unsafe fn get_t(&self) -> &T;
    
    /// Gets a mutable reference to the shared memory region data.
    /// # Returns
    /// Mutable reference.
    unsafe fn get_t_mut(&mut self) -> &mut T;
}
