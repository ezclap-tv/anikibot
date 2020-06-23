//! Provides a basic StreamElements API.
//!
//! ```ignore
//! extern crate tokio;
//! extern crate backend;
//!
//! use tokio::stream::StreamExt as _;
//! use backend::*;
//!
//!
//! #[tokio::main]
//! async fn main() {
//!     let config = StreamElementsConfig::with_token("JWT_TOKEN_HERE").unwrap();
//!     let api = StreamElementsAPI::new(config).finalize().await.unwrap();
//!     println!("My id is {}", api.channel().my_id().await.unwrap());
//! }
//! ```
use super::channels::Channels;
use super::{
    communication::spawn_api_thread, config::StreamElementsConfig,
    consumer::ConsumerStreamElementsAPI, song_requests::SongRequests, stats::Stats,
};
use crate::BackendError;
use reqwest::{
    header::{HeaderMap, HeaderValue},
    Client, Error as ReqwestError, RequestBuilder,
};
use tokio::runtime;

/// The base StreamElements' Kappa API URL.
pub const BASE_API_URL: &str = "https://api.streamelements.com/kappa/v2";

/// An alias for `Result<T, Reqwest::Error>`.
pub type APIResult<T> = Result<T, ReqwestError>;

/// Ensures that the API is properly configured.
pub struct StreamElementsAPIGuard {
    api: StreamElementsAPI,
}

impl StreamElementsAPIGuard {
    /// Stars the API thread and returns its sender and thread handle.
    pub async fn start(
        self,
        runtime: runtime::Handle,
    ) -> Result<(ConsumerStreamElementsAPI, std::thread::JoinHandle<()>), BackendError> {
        let api = self.finalize().await?;
        let (tx, handle) = spawn_api_thread(api, runtime);
        Ok((ConsumerStreamElementsAPI::new(tx), handle))
    }

    /// Checks that the channel_id is present in the config. If not, requests it from the StreamElements API via `GET: channels/me/`.
    async fn finalize(mut self) -> Result<StreamElementsAPI, BackendError> {
        if self.api.config.channel_id[..].is_empty() {
            log::warn!("Missing the channel id, attempting to GET");
            self.api.config.channel_id = self.api.channels().my_id().await?;
        }
        log::info!("Channel id appears to be correctly configured.");
        Ok(self.api)
    }
}

/// Provides a Rust interface to the StreamElements API.
pub struct StreamElementsAPI {
    config: StreamElementsConfig,
    client: Client,
}

impl StreamElementsAPI {
    /// Creates a new `StreamElementsAPI` instance wrapped in the guard type.
    /// To obtain a usable API object, the user must call [`start()`] and verify that the API is properly configured.
    ///
    /// [`start()`]: StreamElementsAPIGuard::start
    pub fn with_config(config: StreamElementsConfig) -> StreamElementsAPIGuard {
        let mut headers = HeaderMap::new();
        headers.insert("accept", HeaderValue::from_static("application/json"));
        headers.insert(
            "Authorization",
            HeaderValue::from_str(&format!("Bearer {}", config.jwt_token)).unwrap(),
        );
        StreamElementsAPIGuard {
            api: Self {
                config,
                client: Client::builder().default_headers(headers).build().unwrap(), // Only fails if the TLS backend or config is invalid
            },
        }
    }

    /// Makes a deep copy of the API. Useful for sending API objects to different threads.
    #[inline]
    pub(crate) fn deep_clone(&self) -> Option<Self> {
        if self.config.channel_id.is_empty() {
            None
        } else {
            Some(Self::with_config(self.config.clone()).api)
        }
    }

    /// Formats `BASE_API_URL` with the given `channel_id`, `method`, and `endpoint` to obtain an API method URL.
    ///
    /// ```
    /// # use backend::stream_elements::api::StreamElementsAPI;
    /// let url = StreamElementsAPI::get_method_endpoint_url("xxx", "songrequest", "player");
    /// assert_eq!(url, "https://api.streamelements.com/kappa/v2/songrequest/xxx/player");
    /// ```
    #[inline]
    pub fn get_method_endpoint_url(channel_id: &str, method: &str, endpoint: &str) -> String {
        format!("{}/{}/{}/{}", BASE_API_URL, method, channel_id, endpoint)
    }

    /// Formats `BASE_API_URL` with the given `endpoint` to obtain an API method URL.
    ///
    /// ```
    /// # use backend::stream_elements::api::StreamElementsAPI;
    /// let url = StreamElementsAPI::get_endpoint_url("songrequest/playing?provider=provider");
    /// assert_eq!(url, "https://api.streamelements.com/kappa/v2/songrequest/playing?provider=provider");
    /// ```
    #[inline]
    pub fn get_endpoint_url(endpoint: &str) -> String {
        format!("{}/{}", BASE_API_URL, endpoint)
    }

    /// Returns the configured channel id.
    #[inline(always)]
    pub fn channel_id(&self) -> &str {
        &self.config.channel_id
    }

    /// Builds a request for the given API method.
    #[inline]
    pub(crate) fn get_method(&self, method: &str, endpoint: &str) -> RequestBuilder {
        self.get_method_for_channel_id(&self.config.channel_id, method, endpoint)
    }

    /// Builds a request for the given API method.
    #[inline]
    pub(crate) fn get_method_for_channel_id(
        &self,
        channel_id: &str,
        method: &str,
        endpoint: &str,
    ) -> RequestBuilder {
        let url = StreamElementsAPI::get_method_endpoint_url(channel_id, method, endpoint);
        log::debug!("GET: {}", url);
        self.client.get(&url)
    }

    /// Builds a request for the given API endpoint.
    #[inline]
    pub(crate) fn get(&self, endpoint: &str) -> RequestBuilder {
        let url = StreamElementsAPI::get_endpoint_url(endpoint);
        log::debug!("GET: {}", url);
        self.client.get(&url)
    }

    /// Builds a POST request for the given API method.
    pub(crate) fn post_method_for_channel_id(
        &self,
        channel_id: &str,
        method: &str,
        endpoint: &str,
    ) -> RequestBuilder {
        let url = StreamElementsAPI::get_method_endpoint_url(channel_id, method, endpoint);
        log::debug!("POST: {}", url);
        self.client.post(&url)
    }

    /// Builds a POST request for the given API method.
    #[allow(unused)]
    #[inline]
    pub(crate) fn post_method(&self, method: &str, endpoint: &str) -> RequestBuilder {
        self.post_method_for_channel_id(&self.config.channel_id, method, endpoint)
    }

    /// Builds a POST request for the given API endpoint.
    #[allow(unused)]
    #[inline]
    pub(crate) fn post(&self, endpoint: &str) -> RequestBuilder {
        let url = StreamElementsAPI::get_endpoint_url(endpoint);
        log::debug!("POST: {}", url);
        self.client.post(&url)
    }

    /// Returns the [`Channels`] API subset.
    ///
    /// [`Channels`]: crate::stream_elements::channels::Channels
    #[inline(always)]
    pub fn channels(&self) -> Channels<'_> {
        Channels::new(self)
    }

    /// Returns the [`SongRequests`] API subset.
    ///
    /// [`SongRequests`]: crate::stream_elements::song_requests::SongRequests
    #[inline(always)]
    pub fn song_requests(&self) -> SongRequests<'_> {
        SongRequests::new(self)
    }

    /// Returns a request builder for the stats API subset.
    ///
    /// [`StatsSettings`]: crate::stream_elements::stats::StatsSettings
    #[inline(always)]
    pub fn stats(&self) -> Stats<'_> {
        Stats::new(self)
    }
}
