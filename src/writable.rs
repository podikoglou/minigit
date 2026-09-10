use std::io::{self, Write};

pub trait Writable {
    fn write<W: Write>(&self, writer: &mut W) -> io::Result<()>;
}
