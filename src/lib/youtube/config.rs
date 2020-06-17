#[derive(Debug, Clone)]
pub struct YouTubeAPIConfig {
    pub number_of_videos: Option<usize>,
    pub playlist_id: Option<String>,
    pub items_per_page: usize,
    pub next_page: String,
}
