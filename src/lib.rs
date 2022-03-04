use async_recursion::async_recursion;
use reqwest::Client;
use std::collections::HashSet;
use std::error::Error;
use std::sync::{Arc, Mutex};
use std::time::Instant;

mod parser;
mod request;

type History = Arc<Mutex<HashSet<String>>>;

const MAX_DEPTH: u16 = 5;

pub async fn run(url: String) -> Result<(), Box<dyn Error>> {
    let history = Arc::new(Mutex::new(HashSet::new()));

    let client = request::configure_client()?;

    let start = Instant::now();
    visit(url, &history, &client, 0).await?;

    let duration = start.elapsed();

    let history = history.lock().unwrap();
    println!("found {} tlds in {:?}", history.len(), duration);

    Ok(())
}

#[async_recursion]
async fn visit(
    url: String,
    history: &History,
    client: &Client,
    depth: u16,
) -> Result<(), Box<dyn Error>> {
    if depth == MAX_DEPTH {
        return Ok(());
    }

    let mut tasks = vec![];

    let body = request::get(client, url).await?;

    for domain in parser::parse_unique_domains(body, history) {
        let history = history.clone();

        let client = client.clone();

        let handle = tokio::spawn(async move {
            if let Err(error) =
                visit(format!("https://{}", domain), &history, &client, depth + 1).await
            {
                eprintln!("{}", error);
            }
        });

        tasks.push(handle);
    }

    for task in tasks {
        task.await.unwrap();
    }

    Ok(())
}
