use std::default::Default;

#[derive(Clone, Debug)]
pub struct Status
{
    pub work_available: bool,
}

impl Default for Status
{
    fn default() -> Self {
        Status {
            work_available: false,
        }
    }
}

