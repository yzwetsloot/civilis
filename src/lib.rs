use async_recursion::async_recursion;
use lazy_static::lazy_static;
use reqwest::Client;
use std::error::Error;
use std::time::Instant;

mod args;
mod graph;
mod parser;
mod request;

pub use args::Args;
pub use graph::{Graph, Vertex};

pub static mut SIGTERM: bool = false;

lazy_static! {}

pub async fn run(args: Args) -> Result<(), Box<dyn Error>> {
    let g = Graph::new(args.shards);

    let client = request::configure_client(args.timeout)?;

    let start = Instant::now();

    let domain = parser::parse_root_domain(&args.url)
        .expect(format!("unable to parse root domain from argument: {}", args.url).as_str());
    g.add_vertex(Vertex::new(domain.clone()));

    println!("{} - {}", g.size(), domain);

    visit(args.url, &g, &client, 0, args.depth).await?;

    let duration = start.elapsed();

    let domain_count = g.size() as u64;
    let reqs = domain_count / duration.as_secs();
    println!(
        "\nfound {} domains in {:?} ({} req/s)",
        domain_count, duration, reqs,
    );

    g.serialize().unwrap();

    Ok(())
}

#[async_recursion]
async fn visit(
    url: String,
    graph: &Graph,
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

    let body = request::get(client, url.clone()).await?;

    for domain in parser::parse_unique_domains(body, url, graph) {
        let graph = graph.clone();

        let client = client.clone();

        let handle = tokio::spawn(async move {
            let _ = visit(domain, &graph, &client, depth + 1, max_depth).await;
        });

        tasks.push(handle);
    }

    for task in tasks {
        let _ = task.await;
    }

    Ok(())
}
