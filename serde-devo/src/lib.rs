use std::fmt;

extern crate serde_devo_derive;
use serde::Deserialize;
pub use serde_devo_derive::Devolve;

#[derive(Debug, Clone, Copy, Deserialize, Default)]
pub enum Error {
    #[default]
    Evolution,
}
impl std::error::Error for Error {}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Evolution => {
                write!(f, "evolution failed")
            }
        }
    }
}

#[cfg(feature = "json")]
pub trait Devolvable<T = serde_json::Value> {
    type Devolved: Evolvable<T, Evolved = Self>;
    fn devolve(self) -> Self::Devolved;
}

#[cfg(not(feature = "json"))]
pub trait Devolvable<T> {
    type Devolved: Evolvable<T, Evolved = Self>;
    fn devolve(self) -> Self::Devolved;
}

pub trait Evolvable<T>: Sized {
    type Evolved: Devolvable<T, Devolved = Self>;
    fn try_evolve(self) -> Result<Self::Evolved, Error>;
}
