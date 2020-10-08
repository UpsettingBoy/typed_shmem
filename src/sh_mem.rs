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

pub(crate) trait DerefAble<T> {
    fn get_t(&self) -> &T;
    fn get_t_mut(&mut self) -> &mut T;
}

#[derive(Debug)]
pub struct ShMem<T: Default + Copy> {
    map: sh::ShObj<T>,
    _marker: PhantomData<T>,
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

#[derive(Debug)]
pub struct ShMemCfg<T: Default> {
    file_name: String,
    owner: bool,

    _marker: PhantomData<T>,
}

impl<T: Default> Default for ShMemCfg<T> {
    fn default() -> Self {
        cfg_if::cfg_if! {
            if #[cfg(unix)] {
                let name = "/shmem_{}";
            } else if #[cfg(windows)] {
                let name = "Global\\{}";
            } else {
                let name = "placeholder".to_string();
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
    pub fn owner(&mut self) -> &mut Self {
        self.owner = true;
        self
    }

    pub fn on_file(&mut self, name: String) -> &mut Self {
        cfg_if::cfg_if! {
            if #[cfg(unix)] {
                let p_name = format!("/shmem_{}", name);
            } else if #[cfg(windows)] {
                let p_name = format!("Global\\{}", name);
            } else {
                let p_name = "placeholder".to_string();
                panic!();
            }
        };

        self.file_name = p_name;
        self
    }

    pub fn build(&mut self) -> Result<ShMem<T>, Box<dyn Error>> {
        let obj = sh::ShObj::try_from(self)?;

        Ok(ShMem {
            map: obj,
            _marker: PhantomData,
        })
    }
}
