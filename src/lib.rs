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
extern crate gaol;
#[macro_use]
extern crate protocol as proto;

pub use self::parapet::Parapet;
pub use self::interactive::Interactive;
pub use self::connection::Connection;
pub use self::network::Network;
pub use self::error::Error;
pub use self::protocol::{Packet, PacketKind};
pub use self::job::Job;
pub use self::pending_state::PendingState;
pub use self::builder::Builder;
pub use self::workspace::Workspace;

pub mod parapet;
pub mod interactive;
pub mod connection;
pub mod network;
pub mod error;
pub mod protocol;
pub mod job;
pub mod pending_state;
pub mod builder;
pub mod workspace;

pub mod graphviz;
pub mod local;
pub mod remote;

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

