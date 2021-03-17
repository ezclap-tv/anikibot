pub type Videos = Vec<YouTubeVideo>;

#[derive(Debug)]
pub struct PlaylistPage {
    pub kind: String,
    pub next_page_token: Option<String>,
    pub videos: Videos,
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct YouTubeVideo {
    pub id: String,
}

impl YouTubeVideo {
    pub fn into_url(self) -> String {
        format!("https://www.youtube.com/watch?v={}", self.id)
    }
}
