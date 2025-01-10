use derive_more::{derive::Display, From};

#[derive(Debug, From, Display)]
pub enum Error {
    #[from]
    Io(std::io::Error),
    #[from]
    TryFromInt(std::num::TryFromIntError),
}

pub type Result<T> = std::result::Result<T, Error>;
