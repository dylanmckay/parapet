#![feature(question_mark)]
#![feature(associated_consts)]
#![feature(const_fn)]
#![feature(conservative_impl_trait)]
#![feature(plugin)]

#![cfg_attr(test, plugin(stainless))]

extern crate mio;
extern crate slab;
extern crate uuid;
extern crate byteorder;
extern crate graphsearch;
extern crate dot;
extern crate twox_hash;
extern crate walkdir;
#[macro_use]
extern crate protocol as proto;
extern crate itertools;

pub use self::parapet::Parapet;
pub use self::interactive::Interactive;
pub use self::network::Network;
pub use self::error::Error;
pub use self::protocol::{Packet, PacketKind};
pub use self::job::Job;

pub mod parapet;
pub mod interactive;
pub mod network;
pub mod error;
pub mod protocol;
pub mod job;
pub mod ci;

pub const SERVER_PORT: u16 = 53371;
pub const SERVER_ADDRESS: (&'static str, u16) = ("127.0.0.1", SERVER_PORT);

pub const CLIENT_NAME: &'static str = "vanilla";
pub const CLIENT_VERSION: &'static str = env!("CARGO_PKG_VERSION");

pub const PROTOCOL_MAJOR: u16 = 0;
pub const PROTOCOL_REVISION: u16 = 0;

pub fn user_agent() -> protocol::UserAgent {
    protocol::UserAgent {
        client: format!("{} v{}", CLIENT_NAME, CLIENT_VERSION),
        protocol_major: PROTOCOL_MAJOR,
        protocol_revision: PROTOCOL_REVISION,
    }
}

