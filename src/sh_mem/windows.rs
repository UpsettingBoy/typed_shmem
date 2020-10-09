use std::{convert::TryFrom, error::Error};

use winapi::{
    ctypes::c_void,
    um::{memoryapi::UnmapViewOfFile, winnt::HANDLE},
};

use super::DerefAble;

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
    handle: HANDLE,
    file_name: Vec<u16>,
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
        let mut name_utf16 = winapi::um::winnt::SE_CREATE_GLOBAL_NAME
            .encode_utf16()
            .collect::<Vec<u16>>();

        let file_map_handle = if value.owner {
            windows_fn::create_handle::<T>(name_utf16.as_mut_ptr())?
        } else {
            windows_fn::open_handle(name_utf16.as_mut_ptr())?
        };

        let buf = windows_fn::map_file_view::<T>(file_map_handle)? as *mut TWrap<T>;

        if value.owner {
            unsafe {
                (*buf).ptr = T::default();
            }
        }

        Ok(ShObj {
            data: buf,
            handle: file_map_handle,
            file_name: name_utf16,
            owner: value.owner,
        })
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
            UnmapViewOfFile(self.data as *mut c_void);
            windows_fn::close_handle(self.handle);
        }
    }
}

mod windows_fn {
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

    use crate::sh_mem::IResult;

    use super::TWrap;

    pub fn create_handle<T: Default + Copy>(name: *mut u16) -> IResult<HANDLE> {
        let handle = unsafe {
            CreateFileMappingW(
                INVALID_HANDLE_VALUE,
                std::ptr::null_mut(),
                PAGE_READWRITE,
                0,
                std::mem::size_of::<TWrap<T>>() as u32,
                name,
            )
        };

        let error_code = unsafe { GetLastError() };

        if handle.is_null() {
            return Err(format!("Creating HANDLE error: {}", error_code).into());
        } else if error_code == ERROR_ALREADY_EXISTS {
            return Err(format!("HANDLE already exists!").into());
        };

        Ok(handle)
    }

    pub fn close_handle(handle: HANDLE) {
        unsafe {
            CloseHandle(handle);
        }
    }

    pub fn map_file_view<T: Default + Copy>(handle: HANDLE) -> IResult<*mut c_void> {
        let map = unsafe {
            winapi::um::memoryapi::MapViewOfFile(
                handle,
                FILE_MAP_ALL_ACCESS,
                0,
                0,
                std::mem::size_of::<TWrap<T>>(),
            )
        };

        if map.is_null() {
            let error_code = unsafe { GetLastError() };
            return Err(format!("Mapping HANDLE error: {}", error_code).into());
        }

        Ok(map)
    }

    pub fn open_handle(name: *mut u16) -> IResult<HANDLE> {
        let handle = unsafe { OpenFileMappingW(FILE_MAP_ALL_ACCESS, FALSE, name) };

        if handle.is_null() {
            let error_code = unsafe { GetLastError() };
            return Err(format!("Opening HANDLE error: {}", error_code).into());
        }

        Ok(handle)
    }
}
