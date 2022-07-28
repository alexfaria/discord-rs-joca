use reqwest::Error;
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Meme {
    pub post_link: String,
    pub subreddit: String,
    pub title: String,
    pub url: String,
    pub nsfw: bool,
    pub spoiler: bool,
    pub author: String,
    pub ups: i64,
    pub preview: Vec<String>,
}

pub async fn gimme(subreddit: Option<&String>) -> Result<Meme, Error> {
    let request_url = format!(
        "https://meme-api.herokuapp.com/gimme/{}",
        subreddit.unwrap_or(&String::from(""))
    );

    return reqwest::get(request_url).await?.json().await
}
