extern crate log;
extern crate reqwest;
extern crate serde;
extern crate serde_json;
extern crate tokio;
extern crate twitchchat;

pub mod bot;
pub mod secrets;
pub mod stream_elements;
pub mod youtube;

pub use bot::Bot;
pub use secrets::Secrets;
pub use stream_elements::api::StreamElementsAPI;
pub use stream_elements::config::StreamElementsConfig;

pub(crate) type BoxedError = Box<dyn std::error::Error + Send + Sync>;

#[derive(Debug)]
pub struct BackendError {
    pub inner: BoxedError,
}

impl From<String> for BackendError {
    fn from(s: String) -> Self {
        Self {
            inner: BoxedError::from(s),
        }
    }
}

impl From<BoxedError> for BackendError {
    fn from(inner: BoxedError) -> Self {
        Self { inner }
    }
}

impl std::fmt::Display for BackendError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "BackendError {{ Error = {} }}", self.inner)
    }
}

impl std::error::Error for BackendError {}
unsafe impl Send for BackendError {}
unsafe impl Sync for BackendError {}
