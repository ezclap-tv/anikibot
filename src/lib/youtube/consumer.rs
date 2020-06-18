use super::communication::{APIRequestKind, APIResponse, RequestSender};

#[derive(Debug, Clone)]
pub struct ConsumerYouTubePlaylistAPI {
    tx: RequestSender,
}

impl ConsumerYouTubePlaylistAPI {
    pub fn new(tx: RequestSender) -> Self {
        Self { tx }
    }

    pub async fn set_playlist<S: Into<String>>(&self, playlist_id: S) -> APIResponse {
        yt_api_send!(
            self,
            APIRequestKind::Playlist_Set {
                id: playlist_id.into()
            }
        )
    }

    pub async fn set_page_size(&self, page_size: usize) -> APIResponse {
        yt_api_send!(self, APIRequestKind::Playlist_SetPageSize(page_size))
    }

    pub async fn configure<S: Into<String>>(
        &self,
        playlist_id: S,
        page_size: usize,
    ) -> APIResponse {
        yt_api_send!(
            self,
            APIRequestKind::Playlist_Configure {
                id: playlist_id.into(),
                page_size
            }
        )
    }

    pub async fn get_playlist(&self) -> APIResponse {
        yt_api_send!(self, APIRequestKind::Playlist_Get)
    }

    pub async fn get_page_size(&self) -> APIResponse {
        yt_api_send!(self, APIRequestKind::Playlist_GetPageSize)
    }

    pub async fn get_config(&self) -> APIResponse {
        yt_api_send!(self, APIRequestKind::Playlist_GetConfig)
    }

    pub async fn get_playlist_videos(&self) -> APIResponse {
        yt_api_send!(self, APIRequestKind::Playlist_GetPlaylistVideos)
    }
}
