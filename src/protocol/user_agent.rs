
define_composite_type!(UserAgent {
    // The client name.
    client: String,

    // Any breaking changes should increment the major number.
    protocol_major: u16,
    // The revision number. All clients of the same build but different
    // revisons should have no problem communicating.
    protocol_revision: u16
});

impl UserAgent
{
    pub fn is_compatible(&self, other: &Self) -> bool {
        self.protocol_major == other.protocol_major
    }
}

