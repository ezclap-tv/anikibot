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
            .map(crate::stream_elements::communication::APIResponseMessage::Str)
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
            .map(crate::stream_elements::communication::APIResponseMessage::Json)
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
macro_rules! resp_json_from_struct {
    ($resp:expr) => {
        resp_json_from_struct!(json => match $resp {
            Ok(res) => serde_json::to_value(res)
                .map_err(|e| crate::BackendError::from(Box::new(e) as crate::BoxedError)),
            Err(e) => Err(crate::BackendError::from(Box::new(e) as crate::BoxedError)),
        })
    };
    (json => $resp:expr) => {
        $resp
            .map(crate::stream_elements::communication::APIResponseMessage::Json)
            .map_err(|e| {
                log::error!(
                    "Caught an error while processing a StreamElements API request: {:#?}",
                    e
                );
                crate::BackendError::from(Box::new(e) as crate::BoxedError)
            })
    };
}
