/// Stores the JWT token and channel_id required by the StreamElements API.
#[derive(Clone)]
pub struct StreamElementsConfig {
    pub jwt_token: String,
    pub channel_id: String,
}

impl StreamElementsConfig {
    // XXX: maybe unpepega this error type
    /// Create a new config using the given token. The token must be a valid ASCII string.
    pub fn with_token(jwt_token: String) -> Result<Self, String> {
        if jwt_token.is_ascii() {
            Ok(Self {
                jwt_token,
                channel_id: String::new(),
            })
        } else {
            Err(String::from("The JWT token must be a valid ASCII string."))
        }
    }

    /// Set a channel id.
    pub fn channel_id(self, channel_id: String) -> Self {
        Self { channel_id, ..self }
    }
}
