use std::io::prelude::*;
use std;
use io;

use byteorder::{WriteBytesExt, ReadBytesExt};
use serde_json;
use uuid::Uuid;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum Packet
{
    Hello(Hello),
}

impl Packet
{
    pub fn packet_id(&self) -> u16 {
        match *self {
            Packet::Hello(..) => Hello::ID,
        }
    }
}

pub trait PacketTrait
{
    const ID: u16;

    fn packet_id(&self) -> u16 { Self::ID }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Hello
{
    pub uuid: Uuid,
    pub sibling_uuids: Vec<Uuid>,
}

impl PacketTrait for Hello
{
    const ID: u16 = 0;
}

impl Packet
{
    pub fn write(&self, write: &mut Write) -> Result<(), std::io::Error> {
        let body: Vec<u8> = serde_json::to_string(self).unwrap().bytes().collect();

        write.write_u16::<io::ByteOrder>(self.packet_id())?;
        write.write(&body)?;

        Ok(())
    }

    pub fn read(read: &mut Read) -> Result<Self, std::io::Error> {
        let packet_id = read.read_u16::<io::ByteOrder>()?;

        let body_bytes: Vec<u8> = read.bytes().map(|a| a.unwrap()).collect();
        let body = String::from_utf8(body_bytes).unwrap();

        match packet_id {
            Hello::ID => {
                let p: Hello = serde_json::from_str(&body).unwrap();
                Ok(Packet::Hello(p))
            },
            _ => {
                panic!("unknown packet id {:?}", packet_id);
            },
        }
    }
}

