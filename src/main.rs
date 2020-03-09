#[macro_use]
extern crate failure;
use lazy_static::lazy_static;
use log::debug;
use reqwest::Client;
use scraper::{Html, Selector};
use tempfile::tempfile;
use url::Url;

use rodio::Source;
use std::fs::File;
use std::io::{Seek, SeekFrom, Write};

mod error;

use error::Error;

lazy_static! {
    /// Base URL of the moanmyip website.
    static ref BASE_URL: Url = Url::parse("https://www.moanmyip.com").unwrap();
    /// HTTP client.
    static ref CLIENT: Client = Client::new();
}

async fn get_front_page() -> Result<Html, Error> {
    let body = CLIENT.get(BASE_URL.as_str()).send().await?.text().await?;

    Ok(Html::parse_document(&body))
}

/// Extracts the users external IP address from the given HTML document.
fn get_ip_address(document: &Html) -> Result<String, Error> {
    let selector = Selector::parse(".content .ip").unwrap();

    document
        .select(&selector)
        .next()
        .map(|element| {
            element
                .text()
                .collect::<Vec<_>>()
                .join(" ")
                .trim()
                .to_string()
        })
        .ok_or(Error::ExternalIpMissingError)
}

fn get_audio_clip_url(document: &Html) -> Result<Url, Error> {
    let selector = Selector::parse("#audio-container audio[src]").unwrap();

    let element = document
        .select(&selector)
        .next()
        .ok_or(Error::AudioClipSrcMissingError)?;

    let src = element
        .value()
        .attr("src")
        .ok_or(Error::AudioClipSrcMissingError)?;

    BASE_URL.join(src).map_err(|e| e.into())
}

async fn download_audio_clip(document: &Html, file: &mut File) -> Result<usize, Error> {
    let audio_clip_url = get_audio_clip_url(&document)?;
    let mut res = CLIENT.get(audio_clip_url.as_str()).send().await?;
    let mut size = 0;

    while let Some(chunk) = res.chunk().await? {
        size += chunk.len();
        file.write_all(&chunk)?;
    }

    Ok(size)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    env_logger::init();
    eprintln!("This program is made possible thanks to https://www.moanmyip.com");
    eprintln!("The audio clip is entirely generated and hosted by said website.");

    let device = rodio::default_output_device().unwrap();
    let sink = rodio::Sink::new(&device);
    let document = get_front_page().await?;
    let ip_address = get_ip_address(&document)?;

    println!("{}", ip_address);

    let mut file = tempfile()?;
    let size = download_audio_clip(&document, &mut file).await?;
    file.seek(SeekFrom::Start(0))?;

    debug!("Downloaded audio clip size: {}", size);

    let source = rodio::Decoder::new(file).unwrap();
    sink.append(source.convert_samples::<f32>());
    sink.sleep_until_end();

    Ok(())
}
