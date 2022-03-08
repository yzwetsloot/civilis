use async_recursion::async_recursion;
use reqwest::Client;
use std::collections::HashSet;
use std::error::Error;
use std::sync::{Arc, Mutex};
use std::time::Instant;

mod args;
mod parser;
mod request;

pub use args::Args;

type History = Arc<Mutex<HashSet<String>>>;

pub async fn run(args: Args) -> Result<(), Box<dyn Error>> {
    let history = Arc::new(Mutex::new(HashSet::new()));

    let client = request::configure_client(args.timeout)?;

    let start = Instant::now();
    visit(args.url, &history, &client, 0, args.depth).await?;

    let duration = start.elapsed();

    let history = history.lock().unwrap();

    let reqs = history.len() as u64 / duration.as_secs();
    println!(
        "\nfound {} domains in {:?} ({} req/s)",
        history.len(),
        duration,
        reqs,
    );

    Ok(())
}

#[async_recursion]
async fn visit(
    url: String,
    history: &History,
    client: &Client,
    depth: u16,
    max_depth: u16,
) -> Result<(), Box<dyn Error>> {
    if depth == max_depth {
        return Ok(());
    }

    let mut tasks = vec![];

    let body = request::get(client, url).await?;

    for domain in parser::parse_unique_domains(body, history) {
        let history = history.clone();

        let client = client.clone();

        let handle = tokio::spawn(async move {
            let _ = visit(domain, &history, &client, depth + 1, max_depth).await;
        });

        tasks.push(handle);
    }

    for task in tasks {
        let _ = task.await;
    }

    Ok(())
}
