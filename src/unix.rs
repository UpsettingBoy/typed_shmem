use std::{convert::TryFrom, ffi::c_void};

use nix::sys::mman::{self, MapFlags, ProtFlags};
use zerocopy::{AsBytes, FromBytes};

use crate::{common::ShMemOps, error::ShMemErr, ShMemCfg};

type RawFd = i32;

pub(super) struct ShObj<T>
where
    T: AsBytes + FromBytes + Default,
{
    data: *mut T,
    file_name: String,
    owner: bool,
}

impl<T> TryFrom<ShMemCfg<T>> for ShObj<T>
where
    T: AsBytes + FromBytes + Default,
{
    type Error = ShMemErr;

    fn try_from(value: ShMemCfg<T>) -> Result<Self, Self::Error> {
        let size = std::mem::size_of::<T>();
        let prot = ProtFlags::PROT_READ | ProtFlags::PROT_WRITE | ProtFlags::PROT_EXEC;
        let flags = MapFlags::MAP_SHARED;

        let fd = if value.owner {
            unix_fn::create_fd(size, &value.file_name)?
        } else {
            unix_fn::open_fd(&value.file_name)?
        };

        let map = unsafe { mman::mmap(std::ptr::null_mut(), size, prot, flags, fd, 0)? };
        let data_ptr = map as *mut T;

        if value.owner {
            unsafe {
                *data_ptr = T::default();
            }
        }

        Ok(Self {
            data: data_ptr,
            file_name: value.file_name.clone(),
            owner: value.owner,
        })
    }
}

impl<T> Drop for ShObj<T>
where
    T: AsBytes + FromBytes + Default,
{
    fn drop(&mut self) {
        unsafe {
            mman::munmap(self.data as *mut c_void, std::mem::size_of::<T>())
                .expect("Error Drop munmap!");
        }

        if self.owner {
            unix_fn::delete_fd(&self.file_name).expect("Error Drop shm_unlink!");
        }
    }
}

impl<T> ShMemOps<T> for ShObj<T>
where
    T: AsBytes + FromBytes + Default,
{
    fn get_t(&self) -> &T {
        unsafe { &(*self.data) }
    }

    fn get_t_mut(&mut self) -> &mut T {
        unsafe { &mut *self.data }
    }
}

mod unix_fn {
    use nix::{
        fcntl::OFlag,
        sys::{mman, stat::Mode},
        unistd,
    };

    use crate::error::Result;

    use super::RawFd;

    pub fn create_fd(size: usize, name: &str) -> Result<RawFd> {
        let flag =
            OFlag::O_TRUNC | OFlag::O_CREAT | OFlag::O_RDWR | OFlag::O_DSYNC | OFlag::O_RSYNC;
        let mode = Mode::S_IRWXU | Mode::S_IRWXG | Mode::S_IRWXO;

        let fd = mman::shm_open(name, flag, mode)?;

        unistd::ftruncate(fd, size as i64)?;

        Ok(fd)
    }

    pub fn open_fd(name: &str) -> Result<RawFd> {
        let flag = OFlag::O_RDWR | OFlag::O_DSYNC | OFlag::O_RSYNC;
        let mode = Mode::S_IRWXU | Mode::S_IRWXG | Mode::S_IRWXO;

        let fd = mman::shm_open(name, flag, mode)?;

        Ok(fd)
    }

    pub fn delete_fd(name: &str) -> Result<()> {
        Ok(mman::shm_unlink(name)?)
    }
}
