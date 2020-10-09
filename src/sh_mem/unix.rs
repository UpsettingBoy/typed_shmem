use std::{borrow::Cow, convert::TryFrom, error::Error};

use nix::{
    libc::c_void,
    sys::mman::{self, MapFlags, ProtFlags},
};

use super::DerefAble;

type RawFd = i32;

#[repr(C)]
#[derive(Debug)]
struct TWrap<T: Default + Copy> {
    ptr: T,
}

impl<T: Default + Copy> Clone for TWrap<T> {
    fn clone(&self) -> Self {
        Self { ..*self }
    }
}

impl<T: Default + Copy> Copy for TWrap<T> {}

#[derive(Debug)]
pub(super) struct ShObj<T: Default + Copy> {
    data: *mut TWrap<T>,
    file_name: Cow<'static, str>,
    owner: bool,
}

impl<T: Default + Copy> DerefAble<T> for ShObj<T> {
    fn get_t(&self) -> &T {
        unsafe { &(*self.data).ptr }
    }

    fn get_t_mut(&mut self) -> &mut T {
        unsafe { &mut (*self.data).ptr }
    }
}

impl<T: Default + Copy> TryFrom<&mut super::ShMemCfg<T>> for ShObj<T> {
    type Error = Box<dyn Error>;

    fn try_from(value: &mut super::ShMemCfg<T>) -> Result<Self, Self::Error> {
        let size = std::mem::size_of::<T>();
        let prot = ProtFlags::PROT_READ | ProtFlags::PROT_WRITE | ProtFlags::PROT_EXEC;
        let flags = MapFlags::MAP_SHARED;

        let fd = if value.owner {
            unix_fn::create_fd::<T>(&value.file_name)?
        } else {
            unix_fn::open_fd(&value.file_name)?
        };

        unsafe {
            let map = mman::mmap(std::ptr::null_mut(), size, prot, flags, fd, 0)?;
            let wrap_ptr = map as *mut TWrap<T>;

            if value.owner {
                *wrap_ptr = TWrap { ptr: T::default() };
            }

            Ok(Self {
                data: wrap_ptr,
                file_name: Cow::from(value.file_name.clone()),
                owner: value.owner,
            })
        }
    }
}

impl<T: Default + Copy> Clone for ShObj<T> {
    fn clone(&self) -> Self {
        Self {
            file_name: self.file_name.clone(),
            owner: false,
            ..*self
        }
    }
}

impl<T: Default + Copy> Drop for ShObj<T> {
    fn drop(&mut self) {
        unsafe {
            mman::munmap(self.data as *mut c_void, std::mem::size_of::<T>())
                .expect("Error Drop munmap");

            if self.owner {
                unix_fn::delete_fd(&self.file_name).expect("Error Drop shm_unlink");
            }
        }
    }
}

mod unix_fn {
    use nix::{fcntl::OFlag, sys::mman, sys::stat::Mode, unistd};

    use crate::sh_mem::IResult;

    use super::RawFd;

    pub fn create_fd<T>(name: &str) -> IResult<RawFd> {
        let size = std::mem::size_of::<T>();
        let flag =
            OFlag::O_TRUNC | OFlag::O_CREAT | OFlag::O_RDWR | OFlag::O_DSYNC | OFlag::O_RSYNC;
        let mode = Mode::S_IRWXU | Mode::S_IRWXG | Mode::S_IRWXO;

        let fd = mman::shm_open(name, flag, mode)?;

        unistd::ftruncate(fd, size as i64)?;

        Ok(fd)
    }

    pub fn open_fd(name: &str) -> IResult<RawFd> {
        let flag = OFlag::O_RDWR | OFlag::O_DSYNC | OFlag::O_RSYNC;
        let mode = Mode::S_IRWXU | Mode::S_IRWXG | Mode::S_IRWXO;

        let fd = mman::shm_open(name, flag, mode)?;

        Ok(fd)
    }

    pub fn delete_fd(name: &str) -> IResult<()> {
        Ok(mman::shm_unlink(name)?)
    }
}
