use crate::{AsSize, Image};
use std::io::{self, Read};

pub trait Decoder where Self: Sized {
    type Key: AsSize + Send + Sync;

    fn read<R: Read>(r: R) -> io::Result<Self>;
    
    fn entry(key: &Self::Key) -> Option<&Image>;
}