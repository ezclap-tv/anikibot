//! Implements the API methods from the [`StreamElement's API reference`].
//!
//! [`StreamElement's API reference`]: https://docs.streamelements.com/reference/
use crate::stream_elements::communication::{APIRequestKind, APIResponse, RequestSender};

/// Implements the `SongRequest` API methods.
pub struct SongRequests {
    tx: RequestSender,
}
