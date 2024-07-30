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

const PLAYLIST_ID: &str = "PLsPUh22kYmNAmjsHke4pd8S9z6m_hVRur";

async fn get_video_urls() -> anyhow::Result<Vec<String>> {
    dotenv().ok();
    let api_key = env::var("YOUTUBE_API_KEY").expect("failed to get youtube api key");

    let client = Client::new();
    let url = format!(
        "https://www.googleapis.com/youtube/v3/playlistItems?part=snippet&playlistId={}&maxResults=50&key={}",
        PLAYLIST_ID, api_key
    );

    let response = client.get(&url).send().await?;
    let body: ApiResponse = response.json().await?;

    debug!("Got response: {:?}", body);
    let video_urls: Vec<String> = body
        .items
        .into_iter()
        .map(|item| {
            format!(
                "https://www.youtube.com/embed/{}",
                item.snippet.resourceId.videoId
            )
        })
        .collect();

    debug!("Got urls: {:?}", video_urls);
    Ok(video_urls)
}

#[derive(Template, Debug)]
#[template(path = "pages/videos.html")]
pub struct VideosTemplate {
    video_urls: Vec<String>,
}

pub async fn videos() -> Html<String> {
    let tmpl = VideosTemplate {
        video_urls: get_video_urls().await.expect("Failed to get video urls"),
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
