use std::marker::PhantomData;

use error::Result;
use zerocopy::{AsBytes, FromBytes};

mod common;
pub mod error;
pub mod unix;

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
        todo!()
    }
}

impl<T> ShMemCfg<T>
where
    T: AsBytes + FromBytes + Default,
{
    pub fn build(mut self) -> Result<ShMem<T>> {
        todo!()
    }
}

pub struct ShMem<T>
where
    T: AsBytes + FromBytes + Default,
{
    _marker: PhantomData<T>,
}
