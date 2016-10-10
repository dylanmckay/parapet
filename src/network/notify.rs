use protocol;

use std::time::{SystemTime, Duration};
use std::collections::VecDeque;

/// How often to notify of work.
const WORK_NOTIFY_FREQUENCY_MS: u64 = 1000;

/// Keeps track of notifications to broadcast.
#[derive(Clone, Debug)]
pub struct Notify
{
    pub work: Work,
}

#[derive(Clone, Debug)]
pub enum Work
{
    /// No work is available.
    Unavailable,
    /// There is work currently available.
    Available {
        last_sent_at: SystemTime,
    },
    /// We have completed all work, but still need to notify everybody.
    Complete,
}

impl Notify
{
    pub fn new() -> Self {
        Notify {
            work: Work::Unavailable,
        }
    }

    pub fn notify(&mut self) -> VecDeque<protocol::PacketKind> {
        let mut packets = VecDeque::new();

        packets.extend(self.work.notify());

        packets
    }
}

impl Work
{
    pub fn available(&mut self) {
        match *self {
            Work::Unavailable | Work::Complete => {
                *self = Work::Available { last_sent_at: SystemTime::now() - Duration::from_secs(1000)};
            },
            Work::Available { .. } => (),
        }
    }

    pub fn complete(&mut self) {
        match *self {
            Work::Available { .. } => {
                *self = Work::Complete;
            },
            Work::Complete | Work::Unavailable => (),
        }
    }

    fn notify(&mut self) -> Option<protocol::PacketKind> {
        match *self {
            Work::Unavailable => None,
            Work::Available { ref mut last_sent_at } => {
                let now = SystemTime::now();

                if now.duration_since(*last_sent_at).unwrap() >= Duration::from_millis(WORK_NOTIFY_FREQUENCY_MS) {
                    *last_sent_at = now;

                    Some(protocol::PacketKind::WorkAvailable(protocol::WorkAvailable))
                } else {
                    None
                }
            },
            Work::Complete => {
                *self = Work::Unavailable;
                Some(protocol::PacketKind::WorkComplete(protocol::WorkComplete))
            },
        }
    }
}

