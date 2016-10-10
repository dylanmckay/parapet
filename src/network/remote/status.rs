use std::default::Default;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Status
{
    pub work: Work,
}

impl Default for Status
{
    fn default() -> Self {
        Status {
            work: Work::Unavailable,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Work
{
    Unavailable,
    Available {
        have_asked_for_work: bool,
    },
}

