use {Packet, Error};
use mio::tcp::*;
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

    pub fn send_packet(&mut self, packet: &Packet) -> Result<(), Error> {
        self.protocol.send_packet(packet)?;
        Ok(())
    }
}

