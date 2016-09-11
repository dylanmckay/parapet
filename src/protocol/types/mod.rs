pub mod numerics;
#[macro_use]
pub mod composite;
pub mod array;
pub mod map;
pub mod string;

use protocol;

use std::io::prelude::*;
use std::fmt;

/// The default byte ordering.
pub type ByteOrder = ::byteorder::BigEndian;

/// A type which can be read or written.
pub trait Type : Clone + fmt::Debug
{
    /// Reads a type for a stream.
    fn read(read: &mut Read) -> Result<Self, protocol::Error>;

    /// Writes a type to a stream.
    fn write(&self, write: &mut Write) -> Result<(), protocol::Error>;
}

