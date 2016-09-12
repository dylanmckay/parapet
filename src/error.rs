use std;
use proto;

#[derive(Debug)]
pub enum Error
{
    Io(std::io::Error),
    Protocol(proto::Error),
}

impl From<std::io::Error> for Error
{
    fn from(e: std::io::Error) -> Self {
        Error::Io(e)
    }
}

impl From<proto::Error> for Error
{
    fn from(e: proto::Error) -> Self {
        Error::Protocol(e)
    }
}

