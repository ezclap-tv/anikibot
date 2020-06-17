use super::communication::RequestSender;
use channels::Channels;

pub mod channels;
pub mod song_requests;

#[derive(Debug, Clone)]
pub struct ConsumeStreamElementsAPI {
    tx: RequestSender,
}

impl ConsumeStreamElementsAPI {
    pub fn new(tx: RequestSender) -> Self {
        Self { tx }
    }

    pub fn channels(&self) -> Channels {
        Channels::new(self.tx.clone())
    }
}
