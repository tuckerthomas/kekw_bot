use serde::{Deserialize, Serialize};
use reqwest::Url;
use std::env;

static OMDB_ENDPOINT: &'static str = "http://www.omdbapi.com/";

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Movie {
    pub title: String,
    pub year: String,
    pub rated: String,
    pub released: String,
    pub runtime: String,
    pub genre: String,
    pub director: String,
    pub writer: String,
    pub actors: String,
    pub plot: String,
    pub language: String,
    pub country: String,
    pub awards: String,
    pub poster: String,
    pub ratings: Vec<MovieRating>,
    pub metascore: String,
    #[serde(alias = "imdbRating")]
    pub imdb_rating: String,
    #[serde(alias = "imdbVotes")]
    pub imdb_votes: String,
    #[serde(alias = "imdbID")]
    pub imdb_id: String,
    pub r#type: String,
    #[serde(alias = "DVD")]
    pub dvd: String,
    pub box_office: String,
    pub production: String,
    pub website: String,
    pub response: String
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct MovieRating {
    source: String,
    value: String
}

pub fn build_base_url() -> Url {
    let mut omdb_url = Url::parse(OMDB_ENDPOINT).unwrap();
    return omdb_url
}

pub async fn query_by_title(title: String) -> Result<Option<Movie>, Box<dyn std::error::Error + Send + Sync>> {
    let mut omdb_url = build_base_url();

    omdb_url.set_query(Some(&format!("t={}", title)));
    
    let body = reqwest::get(omdb_url).await?.text().await?;

    let movie: Movie = serde_json::from_str(&body)?;

    return Ok(Some(movie));
}

pub async fn query_by_id(id: String) -> Result<Option<Movie>, Box<dyn std::error::Error + Send + Sync>> {
    let OMDB_API_KEY = env::var("OMDB_API_KEY").expect("Expected OMDB_API_KEY to be set");
    let mut omdb_url = build_base_url();

    let omdb_params = format!("apikey={}", OMDB_API_KEY) + "&" + &format!("i={}", id);

    omdb_url.set_query(Some(&omdb_params));
    
    let body = reqwest::get(omdb_url.clone()).await?.text().await?;

    println!("Url {}, Got body {}", omdb_url, body);

    let movie: Movie = serde_json::from_str(&body)?;

    return Ok(Some(movie));
}