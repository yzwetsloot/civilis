use async_recursion::async_recursion;
use psl::{List, Psl};
use reqwest::Client;
use scraper::{Html, Selector};
use std::collections::HashSet;
use std::error::Error;
use std::str;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use url::Url;

type History = Arc<Mutex<HashSet<String>>>;

const MAX_DEPTH: u16 = 5;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let url = match std::env::args().nth(1) {
        Some(url) => url,
        None => {
            println!("No CLI URL provided, using default.");
            "https://hyper.rs".into()
        }
    };

    let history = Arc::new(Mutex::new(HashSet::new()));

    let client = Client::builder().timeout(Duration::from_secs(10)).build()?;

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

    let body = get(client, url).await?;

    for domain in parse_unique_domains(body, history) {
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

async fn get(client: &Client, url: String) -> Result<String, Box<dyn Error>> {
    eprintln!("Fetch {}", url);

    let res = client.get(url).send().await?;
    eprintln!("Got {}", res.status());

    let body = res.text().await?;
    Ok(body)
}

fn parse_unique_domains(body: String, history: &History) -> HashSet<String> {
    let document = Html::parse_document(&body);
    let selector = Selector::parse("a").unwrap();

    document
        .select(&selector)
        .filter_map(|el| el.value().attr("href"))
        .filter_map(|href| parse_tld(href))
        .filter(|tld| {
            let mut history = history.lock().unwrap();
            if history.contains(tld) {
                false
            } else {
                history.insert(tld.to_string());
                true
            }
        })
        .collect()
}

fn parse_tld(url: &str) -> Option<String> {
    let url = Url::parse(url).ok()?;
    let host = url.host_str()?;
    let domain = List.domain(host.as_bytes())?;
    let domain = str::from_utf8(domain.as_bytes()).unwrap().to_string();
    Some(domain)
}
