use async_recursion::async_recursion;
use psl::{List, Psl};
use scraper::{Html, Selector};
use std::collections::HashSet;
use std::error::Error;
use std::str;
use std::sync::{Arc, Mutex};
use url::Url;

type History = Arc<Mutex<HashSet<String>>>;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let history = Arc::new(Mutex::new(HashSet::new()));

    let url = match std::env::args().nth(1) {
        Some(url) => url,
        None => {
            println!("No CLI URL provided, using default.");
            "https://hyper.rs".into()
        }
    };

    visit(url, history).await;

    Ok(())
}

#[async_recursion]
async fn visit(url: String, history: History) {
    let body = get(url).await.unwrap();

    let domains = {
        let document = Html::parse_document(&body);
        let selector = Selector::parse("a").unwrap();

        let domains: HashSet<_> = document
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
            .collect();

        domains
    };

    let mut tasks = vec![];

    for domain in domains {
        let history = history.clone();

        let handle = tokio::spawn(async move {
            visit(format!("https://{}", domain), history).await;
        });

        tasks.push(handle);
    }

    for task in tasks {
        task.await.unwrap();
    }
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
