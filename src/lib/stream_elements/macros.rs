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
