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
use log::{debug, info, warn};
use reqwest::{
    header::{HeaderMap, HeaderValue},
    Client, Error as ReqwestError, RequestBuilder,
};

use super::channels::Channels;
use super::config::StreamElementsConfig;

/// The base StreamElements' Kappa API URL.
pub const BASE_API_URL: &'static str = "https://api.streamelements.com/kappa/v2";

/// Ensures that the API is properly configured.
pub struct StreamElementsAPIGuard {
    api: StreamElementsAPI,
}

impl StreamElementsAPIGuard {
    /// Checks that the channel_id is present in the config. If not, requests it from the StreamElements API via `GET: channels/me/`.
    pub async fn finalize(mut self) -> Result<StreamElementsAPI, ReqwestError> {
        match &self.api.config.channel_id[..] {
            "" => {
                warn!("Missing the channel id, attempting to GET");
                self.api.config.channel_id = self.api.channels().my_id().await?;
            }
            _ => (),
        }
        info!("Channel id appears to be correctly configured.");
        Ok(self.api)
    }
}

///! Provides a Rust interface to the StreamElements API.
pub struct StreamElementsAPI {
    config: StreamElementsConfig,
    client: Client,
}

impl StreamElementsAPI {
    /// Creates a new `StreamElementsAPI` instance wrapped in the guard type.
    /// To obtain the wrapped API object, the user must call [`finalize()`] and verify that the API is properly configured.
    ///
    /// [`finalize()`]: StreamElementsAPIGuard::finalize
    pub fn new(config: StreamElementsConfig) -> StreamElementsAPIGuard {
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

    /// Formats `BASE_API_URL` with the given `channel_id`, `method`, and `endpoint` to obtain an API method URL.
    ///
    /// ```
    /// # use backend::stream_elements::StreamElementsAPI;
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
    /// # use backend::stream_elements::StreamElementsAPI;
    /// let url = StreamElementsAPI::get_endpoint_url("songrequest/playing?provider=provider");
    /// assert_eq!(url, "https://api.streamelements.com/kappa/v2/songrequest/playing?provider=provider");
    /// ```
    #[inline]
    pub fn get_endpoint_url(endpoint: &str) -> String {
        format!("{}/{}", BASE_API_URL, endpoint)
    }

    /// Builds a request for the given API method.
    #[allow(unused)]
    #[inline]
    pub(crate) fn get_method(&self, method: &str, endpoint: &str) -> RequestBuilder {
        let url =
            StreamElementsAPI::get_method_endpoint_url(&self.config.channel_id, method, endpoint);
        debug!("GET: {}", url);
        self.client.get(&url)
    }

    /// Builds a request for the given API endpoint.
    #[inline]
    pub(crate) fn get(&self, endpoint: &str) -> RequestBuilder {
        let url = StreamElementsAPI::get_endpoint_url(endpoint);
        debug!("GET: {}", url);
        self.client.get(&url)
    }

    /// Returns the [`Channels`] API subset.
    ///
    /// [`Channels`]: crate::methods::Channels
    #[inline(always)]
    pub fn channels<'a>(&'a mut self) -> Channels<'a> {
        Channels::new(self)
    }
}
