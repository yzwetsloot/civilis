use async_recursion::async_recursion;
use civilis::{Graph, Vertex};
use futures::future::join_all;
use psl::{List, Psl};
use scraper::{Html, Selector};
use std::collections::HashSet;
use std::error::Error;
use std::str;
use url::Url;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let url = match std::env::args().nth(1) {
        Some(url) => url,
        None => {
            println!("No CLI URL provided, using default.");
            "https://hyper.rs".into()
        }
    };

    visit(url).await?;

    Ok(())
}

#[async_recursion(?Send)]
async fn visit(url: String) -> Result<(), Box<dyn Error>> {
    let body = get(url).await?;

    let document = Html::parse_document(&body);
    let selector = Selector::parse("a").unwrap();

    let domains: HashSet<_> = document
        .select(&selector)
        .filter_map(|el| el.value().attr("href"))
        .filter_map(|href| parse_tld(href))
        .collect();

    join_all(
        domains
            .iter()
            .map(|domain| visit(format!("https://{}", domain))),
    )
    .await;

    Ok(())
}

async fn get(url: String) -> Result<String, Box<dyn Error>> {
    eprintln!("Fetch {}", url);

    let res = reqwest::get(url).await?;
    eprintln!("Got {}", res.status());

    let body = res.text().await?;
    Ok(body)
}

fn parse_tld(url: &str) -> Option<String> {
    let url = Url::parse(url).ok()?;
    let host = url.host_str()?;
    let domain = List.domain(host.as_bytes())?;
    let domain = str::from_utf8(domain.as_bytes()).unwrap().to_string();
    Some(domain)
}
