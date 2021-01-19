use std::{
    convert::TryFrom,
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use common::ShMemOps;
use error::Result;
use unix::ShObj;
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

pub struct ShMemCfg<T>
where
    T: AsBytes + FromBytes + Default,
{
    owner: bool,
    file_name: String,
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
                let name = format!("Global\\{}", rnd.rand_u32());
            } else {
                let name = String::new();
                panic!();
            }
        };

        Self {
            owner: false,
            file_name: name,
            _marker: PhantomData,
        }
    }
}

impl<T> ShMemCfg<T>
where
    T: AsBytes + FromBytes + Default,
{
    pub fn with_filename(&mut self, name: &str) -> &mut Self {
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

    pub fn as_owner(&mut self) -> &mut Self {
        self.owner = true;
        self
    }

    pub fn build(self) -> Result<ShMem<T>> {
        let map = sh::ShObj::try_from(self)?;

        Ok(ShMem { map })
    }
}

pub struct ShMem<T>
where
    T: AsBytes + FromBytes + Default,
{
    map: ShObj<T>,
}

impl<T> Deref for ShMem<T>
where
    T: AsBytes + FromBytes + Default,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.map.get_t()
    }
}

impl<T> DerefMut for ShMem<T>
where
    T: AsBytes + FromBytes + Default,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.map.get_t_mut()
    }
}
