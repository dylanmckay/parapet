use mio::tcp::*;
use uuid::Uuid;

pub struct Connection
{
    pub socket: TcpStream,
}

pub struct Node
{
    pub uuid: Uuid,
    pub connection: Option<Connection>,
}

