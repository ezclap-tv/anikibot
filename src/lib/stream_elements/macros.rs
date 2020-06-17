#[macro_export]
macro_rules! api_send {
    ($self:ident, $kind:expr) => {{
        let (tx, rx) = tokio::sync::oneshot::channel();
        $self
            .tx
            .send(crate::stream_elements::communication::APIRequestMessage {
                kind: $kind,
                output: tx,
            })
            .expect("The API thread receiver was dropped.");
        rx.await.expect("The API thread oneshot sender was dropped")
    }};
}

#[macro_export]
macro_rules! resp_str {
    ($resp:expr) => {
        $resp
            .map(|res| crate::stream_elements::communication::APIResponseMessage::Str(res))
            .map_err(|e| {
                log::error!(
                    "Caught an error while processing a StreamElements API request: {:#?}",
                    e
                );
                crate::BackendError::from(Box::new(e) as crate::BoxedError)
            })
    };
}

#[macro_export]
macro_rules! resp_json {
    ($resp:expr) => {
        resp_json!(json => match $resp {
            Ok(res) => res.json::<serde_json::Value>().await,
            Err(e) => Err(e),
        })
    };
    (json => $resp:expr) => {
        $resp
            .map(|res| crate::stream_elements::communication::APIResponseMessage::Json(res))
            .map_err(|e| {
                log::error!(
                    "Caught an error while processing a StreamElements API request: {:#?}",
                    e
                );
                crate::BackendError::from(Box::new(e) as crate::BoxedError)
            })
    };
}
