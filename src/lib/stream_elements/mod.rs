#[macro_use]
mod macros;
pub mod api;
pub mod channels;
pub mod communication;
pub mod config;
pub mod consumer;
pub mod song_requests;

#[cfg(test)]
mod tests {
    #[test]
    fn test_api_thread() {
        pretty_env_logger::init();
        let mut runtime = tokio::runtime::Runtime::new().unwrap();
        let handle = runtime.handle().clone();
        runtime.block_on(api_main(handle))
    }

    async fn api_main(runtime: tokio::runtime::Handle) {
        let token = crate::Secrets::get().stream_elements_jwt_token;
        let (api, _handle) =
            crate::StreamElementsAPI::new(crate::StreamElementsConfig::with_token(token).unwrap())
                .start(runtime)
                .await
                .unwrap();

        let result = api.channels().my_id().await;
        log::trace!("Received {:#?} from the API", result);
    }
}
