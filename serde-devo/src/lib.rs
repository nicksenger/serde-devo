use std::fmt;

extern crate serde_devo_derive;
use serde::Deserialize;
pub use serde_devo_derive::Devolve;

#[derive(Debug, Clone, Deserialize)]
pub enum Error {
    UnknownVariant {
        #[serde(borrow)]
        ty: &'static str,
        #[serde(borrow)]
        path: Vec<&'static str>,
    },
}
impl std::error::Error for Error {}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::UnknownVariant { ty, path } => {
                write!(
                    f,
                    "evolution failed: {ty}.{}",
                    path.iter().fold("[unknown_variant]".to_string(), |a, b| {
                        format!("{b}.{a}")
                    })
                )
            }
        }
    }
}

impl Error {
    pub fn extend(self, ty: &'static str, ext: &'static str) -> Self {
        match self {
            Self::UnknownVariant { mut path, .. } => {
                path.push(ext);
                Self::UnknownVariant { ty, path }
            }
        }
    }
}

/// A **data structure** which represents the complete, or latest known form of another
/// devolved **data structure**, and which may be converted into this "devolved" form.
///
/// Additionally, serde-devo provides a procedural macro called [`serde_devo_derive`]
/// to automatically implement `Devolvable` for structs and enums in your program.
///
/// In rare cases it may be necessary to implement `Devolvable` manually for some
/// type in your program.
#[cfg(feature = "json")]
pub trait Devolve<T = serde_json::Value> {
    type Devolved: Evolve<T, Evolved = Self>;
    fn into_devolved(self) -> Self::Devolved;
}

/// A **data structure** which represents the complete, or latest known form of another
/// devolved **data structure**, and which may be converted into this "devolved" form.
///
/// Additionally, serde-devo provides a procedural macro called [`serde_devo_derive`]
/// to automatically implement `Devolvable` for structs and enums in your program.
///
/// In rare cases it may be necessary to implement `Devolvable` manually for some
/// type in your program.
#[cfg(not(feature = "json"))]
pub trait Devolve<T> {
    type Devolved: Evolve<T, Evolved = Self>;
    fn into_devolved(self) -> Self::Devolved;
}

/// A **data structure** which represents the potentially incomplete form of another
/// evolving **data structure**, and which may be converted into this "evolved" form
/// with possibility for error.
///
/// Additionally, serde-devo provides a procedural macro called [`serde_devo_derive`]
/// to automatically generate `Evolvable` types for structs and enums in your program.
///
/// In rare cases it may be necessary to implement `Evolvable` manually for some
/// type in your program.
pub trait Evolve<T>: Sized {
    type Evolved: Devolve<T, Devolved = Self>;
    fn try_into_evolved(self) -> Result<Self::Evolved, Error>;
}
