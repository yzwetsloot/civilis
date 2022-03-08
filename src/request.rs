use reqwest::Client;
use std::error::Error;
use std::time::Duration;

pub fn configure_client(timeout: u64) -> Result<Client, Box<dyn Error>> {
    let client = Client::builder()
        .timeout(Duration::from_secs(timeout))
        .build()?;
    Ok(client)
}

pub async fn get(client: &Client, url: String) -> Result<String, Box<dyn Error>> {
    let res = client.get(url).send().await?;

    let body = res.text().await?;
    Ok(body)
}
