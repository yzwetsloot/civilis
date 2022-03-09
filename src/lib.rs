use async_recursion::async_recursion;
use reqwest::Client;
use std::error::Error;
use std::time::Instant;

mod args;
mod history;
mod parser;
mod request;

pub use args::Args;
pub use history::History;

pub static mut SIGTERM: bool = false;

pub async fn run(args: Args) -> Result<(), Box<dyn Error>> {
    let history = History::new(args.shards);

    let client = request::configure_client(args.timeout)?;

    let start = Instant::now();
    visit(args.url, &history, &client, 0, args.depth).await?;

    let duration = start.elapsed();

    let domain_count = history.len() as u64;
    let reqs = domain_count / duration.as_secs();
    println!(
        "\nfound {} domains in {:?} ({} req/s)",
        domain_count, duration, reqs,
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

    if unsafe { SIGTERM } {
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
