use askama::Template;
use axum::response::Html;
use dotenv::dotenv;
use reqwest::Client;
use serde::Deserialize;
use std::env;
use tracing::debug;

#[derive(Debug, Deserialize)]
struct PlaylistItem {
    snippet: Snippet,
}

#[allow(non_snake_case)]
#[derive(Debug, Deserialize)]
struct Snippet {
    resourceId: ResourceId,
    title: String,
    description: String,
}

#[allow(non_snake_case)]
#[derive(Debug, Deserialize)]
struct ResourceId {
    videoId: String,
}

#[allow(non_snake_case)]
#[derive(Debug, Deserialize)]
struct ApiResponse {
    items: Vec<PlaylistItem>,
}

#[derive(Debug)]
struct VideoInfo {
    url: String,
    title: String,
    description: String,
}

impl From<Snippet> for VideoInfo {
    fn from(value: Snippet) -> Self {
        Self {
            url: format!("https://www.youtube.com/embed/{}", value.resourceId.videoId),
            title: value.title,
            description: value.description,
        }
    }
}

async fn get_video_urls() -> anyhow::Result<Vec<VideoInfo>> {
    dotenv().ok();
    let api_key = env::var("YOUTUBE_API_KEY").expect("failed to get youtube api key");
    let playlist_id = env::var("YOUTUBE_PLAYLIST_ID").expect("failed to get youtube api key");

    let client = Client::new();
    let url = format!(
        "https://www.googleapis.com/youtube/v3/playlistItems?part=snippet&playlistId={}&maxResults=50&key={}",
        playlist_id, api_key
    );

    let response = client.get(&url).send().await?;
    let body: ApiResponse = response.json().await?;

    debug!("Got response: {:?}", body);
    let videos: Vec<VideoInfo> = body
        .items
        .into_iter()
        .map(|item| VideoInfo::from(item.snippet))
        .collect();

    debug!("Got infos: {:?}", videos);
    Ok(videos)
}

#[derive(Template, Debug)]
#[template(path = "pages/videos.html")]
pub struct VideosTemplate {
    videos: Vec<VideoInfo>,
}

pub async fn videos() -> Html<String> {
    let tmpl = VideosTemplate {
        videos: get_video_urls().await.expect("Failed to get videos"),
    };
    match tmpl.render() {
        Ok(r) => Html(r),
        Err(err) => Html(format!("Error rendering Layout: {}", err.to_string())),
    }
}

mod tests {
    use std::sync::LazyLock;

    use crate::TRACING;

    #[tokio::test]
    async fn get_urls_works() {
        LazyLock::force(&TRACING);
        assert!(super::get_video_urls().await.is_ok());
    }
}
