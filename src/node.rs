use mio::tcp::*;
use uuid::Uuid;
use io;
use std;

use std::io::prelude::*;

pub struct Connection
{
    pub socket: TcpStream,
    pub builder: io::Builder,
}

impl Connection
{
    pub fn process_incoming_data(&mut self) -> Result<(), std::io::Error> {
        let mut array = [0; 10000];
        let bytes_read = self.socket.read(&mut array)?;

        self.builder.give_bytes(&array[0..bytes_read]);

        Ok(())
    }
}

pub struct Node
{
    pub uuid: Uuid,
    pub connection: Option<Connection>,
}

