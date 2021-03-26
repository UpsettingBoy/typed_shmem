//! # typed_shmem
//! Provides the [ShMem](ShMem) and [ShMemCfg](ShMemCfg) types for creating a shared memory region.
//!
//! In order of a type `T` to be compatible with the shared memory implementation here present it must be
//! `T: zerocopy::AsBytes + zerocopy::FromBytes + Default`.
//!
//! Since there is no synchronization when reading/mutating the shared data, the programmer has to be
//! responsible of how to do so in order to not corrupt said data.
//!
//! # Example
//! ## Owner process
//! ```no_run
//! use typed_shmem as sh;
//! use typed_shmem::error::ShMemErr;
//!
//! fn main() -> Result<(), ShMemErr> {
//!     let mut mem = sh::ShMemCfg::<u32>::default()
//!          .set_owner()
//!          .on_file("test_program")
//!          .build()?;
//!
//!     // ShMem<T> implements Deref and DerefMut.
//!     *mem = 10; //Write.
//!     assert_eq!(*mem, 10); //Read.
//!
//!     loop {} //Used to keep the process alive, thus the allocated shared memory too.
//!     
//!     Ok(())
//! }
//! ```
//! ## Any other process
//! ```no_run
//! use typed_shmem as sh;
//! use typed_shmem::error::ShMemErr;
//!
//! fn main() -> Result<(), ShMemErr> {
//!     let mut mem = sh::ShMemCfg::<u32>::default()
//!              .on_file("test_program")
//!              .build()?;
//!
//!     assert_eq!(*mem, 10); //Read.
//!
//!     Ok(())
//! }
//! ```
//!
//! # Panics
//! If the platform on which this crate is compiled does not comply with cfg(unix) nor with cfg(windows),
//! the program will **panic**.

use std::{
    convert::TryFrom,
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use common::ShMemOps;
use error::Result;
use zerocopy::{AsBytes, FromBytes};

mod common;
pub mod error;

cfg_if::cfg_if! {
    if #[cfg(unix)] {
        mod unix;
        use unix as sh;
    } else if #[cfg(windows)] {
        mod windows;
        use windows as sh;
    } else {
        panic!("No shared memory model available for this platform.");
    }
}

/// Configures and initilizes a shared memory region.
/// By default, the segment name is ramdomly created and this instance is not the owner of the memory object.
/// # Example
/// ```no_compile
/// let memory = ShMemCfg::<u32>::default().build().unwrap();
///
///
pub struct ShMemCfg<T>
where
    T: AsBytes + FromBytes + Default,
{
    owner: bool,
    file_name: String,
    init_value: Option<T>,
    _marker: PhantomData<T>,
}

impl<T> Default for ShMemCfg<T>
where
    T: AsBytes + FromBytes + Default,
{
    fn default() -> Self {
        let mut seed = [0_u8; 8];
        getrandom::getrandom(&mut seed).expect("Error on getrandom!");

        let mut rnd = oorandom::Rand32::new(u64::from_ne_bytes(seed));

        cfg_if::cfg_if! {
            if #[cfg(unix)] {
                let name = format!("/shmem_{}", rnd.rand_u32());
            } else if #[cfg(windows)] {
                let name = format!("{}", rnd.rand_u32());
            } else {
                let name = String::new();
                panic!();
            }
        };

        Self {
            owner: false,
            file_name: name,
            init_value: None,
            _marker: PhantomData,
        }
    }
}

impl<T> ShMemCfg<T>
where
    T: AsBytes + FromBytes + Default,
{
    /// Name of the segment of the shared memory.
    /// # Params
    /// `name`: Name of the segment.
    /// # Returns
    /// Mutable reference to the configurator.
    pub fn on_file(mut self, name: &str) -> Self {
        cfg_if::cfg_if! {
            if #[cfg(unix)] {
                let p_name = format!("/shmem_{}", name);
            } else if #[cfg(windows)] {
                let p_name = name.to_string();
            } else {
                let p_name = String::new();
                panic!();
            }
        };

        self.file_name = p_name;
        self
    }

    /// Makes this instance the owner of the shared memory object. Only **one** instance referencing the same
    /// segmente can be the owner or the segment could be double freed.
    /// # Returns
    /// Mutable reference to the configurator.
    pub fn set_owner(mut self) -> Self {
        self.owner = true;
        self
    }

    /// Sets the initial value of the shared memory region. If skipped, `T::default()` will be used.
    /// # Params
    /// `init`: Initial value
    /// # Returns
    /// Mutable reference to the configurator.
    pub fn with_initial_value(mut self, init: T) -> Self {
        self.init_value = Some(init);
        self
    }

    /// Builds a [ShMem](ShMem) with the configuration of this instance of [ShMemCfg](ShMemCfg).
    /// # Returns
    /// A result wrapping the memory segment.
    pub fn build(self) -> Result<ShMem<T>> {
        let map = sh::ShObj::try_from(self)?;

        Ok(ShMem { map })
    }
}

/// Contains the platform-specific implementation details for shared memory. The memory itself it is accessed
/// through the `Deref` and `DerefMut` traits.
///
/// It must be created using [ShMemCfg](ShMemCfg) or _Shared memory configurator_.
///
/// # Drop
/// When `ShMem` is dropped, it removes the handle to the shared memory file. If the dropped instance of `ShMem` is _owner_, it will
/// try to remove the shared memory file too (on *nix).
///
/// # To keep in mind
/// The memory does not implement any form of synchronization. It also draw on UBs to glue `Deref` and `DerefMut` traits to the internal implementation.
pub struct ShMem<T>
where
    T: AsBytes + FromBytes + Default,
{
    map: sh::ShObj<T>,
}

impl<T> ShMemOps<T> for ShMem<T>
where
    T: AsBytes + FromBytes + Default,
{
    unsafe fn get_t(&self) -> &T {
        self.map.get_t()
    }

    unsafe fn get_t_mut(&mut self) -> &mut T {
        self.map.get_t_mut()
    }
}
