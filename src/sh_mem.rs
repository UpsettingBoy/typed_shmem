//! # Shared memory
//! This module contains all the structs and functions needed to create shared memory objects under Unix and Windows.
//!
//! In order of a type `T` to be compatible with the shared memory implementation here present it must be
//! `T: Default + Copy`.
//!
//! # Panics
//! If the platform of which this crate is compiled does not comply with cfg(unix) nor with cfg(windows),
//! the program will **panic** at runtime.
//!
//! # Example
//! ## Owner process
//! ```no_run
//! use std::error::Error;
//! use safe_ipc::sh_mem;
//!
//! fn main() -> Result<(), Box<dyn Error>> {
//!     let mut mem = sh_mem::ShMemCfg::<u32>::default()
//!          .owner()
//!          .on_file("test_program".to_string())
//!          .build()?;
//!
//!     // ShMem<T> implements Deref and DerefMut for T.
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
//! use std::error::Error;
//! use safe_ipc::sh_mem;
//!
//! fn main() -> Result<(), Box<dyn Error>> {
//!     let mut mem = sh_mem::ShMemCfg::<u32>::default()
//!              .on_file("test_program".to_string())
//!              .build()?;
//!
//!     assert_eq!(*mem, 10); //Read.
//!
//!     Ok(())
//! }
//! ```

use std::{
    convert::TryFrom, error::Error, fmt::Debug, marker::PhantomData, ops::Deref, ops::DerefMut,
};

cfg_if::cfg_if! {
    if #[cfg(unix)] {
        mod unix;
        use unix as sh;
    } else if #[cfg(windows)] {
        mod windows;
        use windows as sh;
    } else {
        panic!("No shared memory model available.");
    }
}

type IResult<T> = std::result::Result<T, Box<dyn Error>>;

/// Don't forget to implement this trait on each platform for `ShObj`.
trait DerefAble<T> {
    fn get_t(&self) -> &T;
    fn get_t_mut(&mut self) -> &mut T;
}

/// Contains the platform-specific implementation details for the shared memory. The memory itself it is accessed
/// through the `Deref` and `DerefMut` traits.
///
/// It must be created using [ShMemCfg](ShMemCfg) or _Shared memory configurator_.
///
/// # To keep in mind
/// The memory does not implement any form of between-process synchronization.
#[derive(Debug)]
pub struct ShMem<T: Default + Copy> {
    map: sh::ShObj<T>,
}

impl<T: Default + Copy> Deref for ShMem<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.map.get_t()
    }
}

impl<T: Default + Copy> DerefMut for ShMem<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.map.get_t_mut()
    }
}

/// Configures and initilizes a shared memory region.
/// By default, the segment name is ramdomly created and this instance is not the owner of the memory object.
/// # Example
/// ```no_compile
/// let memory = ShMemCfg::<u32>::default().build().unwrap();
///
/// ```
#[derive(Debug)]
pub struct ShMemCfg<T: Default + Copy> {
    file_name: String,
    owner: bool,

    _marker: PhantomData<T>,
}

impl<T: Default + Copy> Default for ShMemCfg<T> {
    fn default() -> Self {
        let mut seed = [0_u8; 8];
        getrandom::getrandom(&mut seed).expect("Error on getrandom!");

        let mut rnd = oorandom::Rand32::new(u64::from_ne_bytes(seed));

        cfg_if::cfg_if! {
            if #[cfg(unix)] {
                let name = format!("/shmem_{}", rnd.rand_u32());
            } else if #[cfg(windows)] {
                let name = format!("Global\\{}", rnd.rand_u32());
            } else {
                let name = String::new();
                panic!();
            }
        };

        Self {
            file_name: name.to_string(),
            owner: false,
            _marker: PhantomData,
        }
    }
}

impl<T: Default + Copy> ShMemCfg<T> {
    /// Makes this instance the owner of the shared memory object. Only **one** instance referencing the same
    /// segmente can be the owner or the segment could be double freed.
    /// # Returns
    /// Mutable reference to the configurator.
    pub fn owner(&mut self) -> &mut Self {
        self.owner = true;
        self
    }

    /// Name of the segment of the shared memory.
    /// # Params
    /// `name`: Name of the segment.
    /// # Returns
    /// Mutable reference to the configurator.
    pub fn on_file(&mut self, name: String) -> &mut Self {
        cfg_if::cfg_if! {
            if #[cfg(unix)] {
                let p_name = format!("/shmem_{}", name);
            } else if #[cfg(windows)] {
                let p_name = format!("Global\\{}", name);
            } else {
                let p_name = String::new();
                panic!();
            }
        };

        self.file_name = p_name;
        self
    }

    /// Builds a [ShMem](ShMem) with the configuration of this instance of [ShMemCfg](ShMemCfg).
    /// # Returns
    /// A result wrapping the memory segment.
    pub fn build(&mut self) -> Result<ShMem<T>, Box<dyn Error>> {
        let obj = sh::ShObj::try_from(self)?;

        Ok(ShMem { map: obj })
    }
}
