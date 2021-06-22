use reqwest::{header, Client, Error};
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GalleryResponse {
    pub data: Gallery,
    pub success: bool,
    pub status: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Gallery {
    pub id: String,
    pub title: String,
    pub description: ::serde_json::Value,
    pub datetime: i64,
    pub cover: String,
    #[serde(rename = "account_url")]
    pub account_url: String,
    #[serde(rename = "account_id")]
    pub account_id: i64,
    pub privacy: String,
    pub layout: String,
    pub views: i64,
    pub link: String,
    pub ups: i64,
    pub downs: i64,
    pub points: i64,
    pub score: i64,
    #[serde(rename = "is_album")]
    pub is_album: bool,
    pub vote: ::serde_json::Value,
    #[serde(rename = "comment_count")]
    pub comment_count: i64,
    #[serde(rename = "images_count")]
    pub images_count: i64,
    pub images: Vec<Image>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Image {
    pub id: String,
    pub title: ::serde_json::Value,
    pub description: ::serde_json::Value,
    pub datetime: i64,
    #[serde(rename = "type")]
    pub type_field: String,
    pub animated: bool,
    pub width: i64,
    pub height: i64,
    pub size: i64,
    pub views: i64,
    pub bandwidth: i64,
    pub link: String,
}

pub struct ImgurClient {
    http_client: Client,
    base_url: String,
}

impl ImgurClient {
    pub fn new(client_id: String) -> Self {
        let auth_header = format!("Client-ID {}", client_id);
        let mut headers = header::HeaderMap::new();
        headers.insert(
            "Authorization",
            header::HeaderValue::from_str(auth_header.as_str()).unwrap(),
        );

        let http_client = Client::builder().default_headers(headers).build().unwrap();

        Self {
            http_client,
            base_url: String::from("https://api.imgur.com"),
        }
    }

    pub async fn get_gallery(&self, gallery: String) -> Result<Gallery, Error> {
        let request_url = format!("{}/3/gallery/album/{}", self.base_url, gallery);
        let response = self.http_client.get(request_url).send().await?;
        let subreddit_gallery: GalleryResponse = response.json().await?;
        Ok(subreddit_gallery.data)
    }
}
