use std::convert::TryFrom;

use winapi::{
    ctypes::c_void,
    um::{memoryapi::UnmapViewOfFile, winnt::HANDLE},
};
use zerocopy::{AsBytes, FromBytes};

use crate::{common::ShMemOps, error::ShMemErr, ShMemCfg};

#[allow(dead_code)]
pub(super) struct ShObj<T>
where
    T: AsBytes + FromBytes + Default,
{
    data: *mut T,
    handle: HANDLE,
    file_name: Vec<u16>,
    owner: bool,
}

impl<T> TryFrom<ShMemCfg<T>> for ShObj<T>
where
    T: AsBytes + FromBytes + Default,
{
    type Error = ShMemErr;

    fn try_from(value: ShMemCfg<T>) -> Result<Self, Self::Error> {
        let mut name_utf16 = value.file_name.encode_utf16().collect::<Vec<u16>>();

        let file_map_handle = if value.owner {
            windows_fn::create_handle::<T>(name_utf16.as_mut_ptr())?
        } else {
            windows_fn::open_handle(name_utf16.as_mut_ptr())?
        };

        let ptr = windows_fn::map_file_view::<T>(file_map_handle)? as *mut T;

        if value.owner {
            let init = match value.init_value {
                Some(v) => v,
                None => T::default(),
            };

            unsafe {
                *data_ptr = init;
            }
        }

        Ok(ShObj {
            data: ptr,
            handle: file_map_handle,
            file_name: name_utf16,
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
            UnmapViewOfFile(self.data as *mut c_void);
            windows_fn::close_handle(self.handle);
        }
    }
}

impl<T> ShMemOps<T> for ShObj<T>
where
    T: AsBytes + FromBytes + Default,
{
    unsafe fn get_t(&self) -> &T {
        &(*self.data)
    }

    unsafe fn get_t_mut(&mut self) -> &mut T {
        &mut (*self.data)
    }
}

mod windows_fn {
    use crate::error::{Result, ShMemErr};
    use winapi::{
        ctypes::c_void,
        shared::minwindef::FALSE,
        shared::winerror::ERROR_ALREADY_EXISTS,
        um::{
            errhandlingapi::GetLastError,
            handleapi::{CloseHandle, INVALID_HANDLE_VALUE},
            memoryapi::{CreateFileMappingW, OpenFileMappingW, FILE_MAP_ALL_ACCESS},
            winnt::{HANDLE, PAGE_READWRITE},
        },
    };
    use zerocopy::{AsBytes, FromBytes};

    pub fn create_handle<T>(name: *mut u16) -> Result<HANDLE>
    where
        T: AsBytes + FromBytes + Default,
    {
        let handle = unsafe {
            CreateFileMappingW(
                INVALID_HANDLE_VALUE,
                std::ptr::null_mut(),
                PAGE_READWRITE,
                0,
                std::mem::size_of::<T>() as u32,
                name,
            )
        };

        let error_code = unsafe { GetLastError() };

        if handle.is_null() {
            return Err(ShMemErr::Windows(format!(
                "Creating HANDLE error: {}",
                error_code
            )));
        } else if error_code == ERROR_ALREADY_EXISTS {
            return Err(ShMemErr::Windows("HANDLE already exists!".to_string()));
        };

        Ok(handle)
    }

    pub fn close_handle(handle: HANDLE) {
        unsafe {
            CloseHandle(handle);
        }
    }

    pub fn map_file_view<T>(handle: HANDLE) -> Result<*mut c_void>
    where
        T: AsBytes + FromBytes + Default,
    {
        let map = unsafe {
            winapi::um::memoryapi::MapViewOfFile(
                handle,
                FILE_MAP_ALL_ACCESS,
                0,
                0,
                std::mem::size_of::<T>(),
            )
        };

        if map.is_null() {
            let error_code = unsafe { GetLastError() };
            return Err(ShMemErr::Windows(format!(
                "Mapping HANDLE error: {}",
                error_code
            )));
        }

        Ok(map)
    }

    pub fn open_handle(name: *mut u16) -> Result<HANDLE> {
        let handle = unsafe { OpenFileMappingW(FILE_MAP_ALL_ACCESS, FALSE, name) };

        if handle.is_null() {
            let error_code = unsafe { GetLastError() };
            return Err(ShMemErr::Windows(format!(
                "Opening HANDLE error: {}",
                error_code
            )));
        }

        Ok(handle)
    }
}
