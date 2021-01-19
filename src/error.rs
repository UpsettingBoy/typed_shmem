use core::panic;

pub type Result<T> = std::result::Result<T, ShMemErr>;

cfg_if::cfg_if! {
    if #[cfg(windows)] {
        #[derive(Debug)]
        pub enum ShMemErr {
            Windows(String),
        }
    } else if #[cfg(unix)] {
        #[derive(Debug)]
        pub enum ShMemErr {
            Unix(String),
        }

        impl From<nix::Error> for ShMemErr {
            fn from(e: nix::Error) -> Self {
                ShMemErr::Unix(e)
            }
        }
    } else {
        panic!("Invalid platform!");
    }
}
