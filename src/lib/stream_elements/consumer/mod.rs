use super::communication::RequestSender;
use channels::Channels;
use song_requests::SongRequests;

pub mod channels;
pub mod song_requests;

#[derive(Debug, Clone)]
pub struct ConsumerStreamElementsAPI {
    tx: RequestSender,
}

impl ConsumerStreamElementsAPI {
    pub fn new(tx: RequestSender) -> Self {
        Self { tx }
    }

    #[must_use = "Calling channels() does nothing"]
    pub fn channels(&self) -> Channels {
        Channels::new(self.tx.clone())
    }

    #[must_use = "Calling song_requests() does nothing"]
    pub fn song_requests(&self) -> SongRequests {
        SongRequests::new(self.tx.clone())
    }
}
