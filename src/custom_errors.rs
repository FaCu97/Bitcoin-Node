use std::{error::Error, fmt};

#[derive(Debug, PartialEq, Eq, Clone)]
/// Representa los distintos errores genericos que pueden llegar a ocurrir
/// durante el programa
pub enum NodeCustomErrors {
    ThreadJoinError(String),
    LockError(String),
    ReadNodeError(String),
    WriteNodeError(String),
    CanNotRead(String),
    ThreadChannelError(String),
    UnmarshallingError(String),
}

impl fmt::Display for NodeCustomErrors {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            NodeCustomErrors::ThreadJoinError(msg) => {
                write!(f, "ThreadJoinError Error: {}", msg)
            }
            NodeCustomErrors::LockError(msg) => write!(f, "LockError Error: {}", msg),
            NodeCustomErrors::ReadNodeError(msg) => {
                write!(f, "Can not read from socket Error: {}", msg)
            }
            NodeCustomErrors::WriteNodeError(msg) => {
                write!(f, "Can not write in socket Error: {}", msg)
            }
            NodeCustomErrors::CanNotRead(msg) => {
                write!(f, "No more elements in list Error: {}", msg)
            }
            NodeCustomErrors::ThreadChannelError(msg) => {
                write!(f, "Can not send elements to channel Error: {}", msg)
            }
            NodeCustomErrors::UnmarshallingError(msg) => {
                write!(f, "Can not unmarshall bytes Error: {}", msg)
            }
        }
    }
}

impl Error for NodeCustomErrors {}
