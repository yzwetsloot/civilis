use reqwest::Client;
use std::error::Error;
use std::time::Duration;

pub fn configure_client() -> Result<Client, Box<dyn Error>> {
    let client = Client::builder().timeout(Duration::from_secs(10)).build()?;
    Ok(client)
}

pub async fn get(client: &Client, url: String) -> Result<String, Box<dyn Error>> {
    eprintln!("Fetch {}", url);

    let res = client.get(url).send().await?;
    eprintln!("Got {}", res.status());

    let body = res.text().await?;
    Ok(body)
}
