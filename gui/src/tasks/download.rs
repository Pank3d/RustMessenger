use reqwest::{Client, Url};

pub(crate) type DownloadTaskError = reqwest::Error;

pub(crate) async fn run(client: Client, url: Url) -> Result<Vec<u8>, DownloadTaskError> {
	Ok(client.get(url).send().await?.bytes().await?.to_vec())
}
