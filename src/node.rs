use {Packet, Error};
use mio::tcp::*;
use uuid::Uuid;
use proto;

pub struct Connection
{
    pub token: ::mio::Token,
    pub protocol: proto::wire::stream::Connection<Packet, TcpStream>,
}

impl Connection
{
    pub fn process_incoming_data(&mut self) -> Result<(), Error> {
        Ok(self.protocol.process_incoming_data()?)
    }

    pub fn receive_packet(&mut self) -> Result<Option<Packet>, Error> {
        Ok(self.protocol.receive_packet()?)
    }
}

pub struct Node
{
    pub uuid: Uuid,
    pub connection: Option<Connection>,
}

