use crate::ShMem;

pub type Result<T> = std::result::Result<T, ShMemErr>;

#[derive(Debug)]
pub enum ShMemErr {
    Unix(nix::Error),
}

impl From<nix::Error> for ShMemErr {
    fn from(e: nix::Error) -> Self {
        ShMemErr::Unix(e)
    }
}
