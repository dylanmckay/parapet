use {Packet, PacketKind, Error};
use network;
use protocol;

use proto;
use mio::tcp::*;

#[derive(Debug)]
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

    /// Terminate the connection with a reason.
    pub fn terminate<S>(mut self, reason: S) -> Result<(), Error>
        where S: Into<String> {
        self.protocol.send_packet(&Packet {
            path: network::Path::empty(),
            kind: PacketKind::Terminate(protocol::Terminate {
                reason: reason.into(),
            }),
        })?;

        // Connection is dropped when connection goes out of scope

        Ok(())
    }
}

