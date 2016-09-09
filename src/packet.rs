use uuid::Uuid;

#[derive(RustcEncodable, RustcDecodable, PartialEq)]
pub enum Packet
{
    Hello(Hello),
}

#[derive(RustcEncodable, RustcDecodable, PartialEq)]
pub struct Hello
{
    pub uuid: String,
    pub sibling_uuids: Vec<String>,
}

