use psl::{List, Psl};
use reqwest::Client;
use scraper::{Html, Selector};
use std::collections::HashSet;
use std::str;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::mpsc;
use url::Url;

struct Count(u32);

impl Count {
    fn increment(&mut self) {
        self.0 += 1;
    }

    fn decrement(&mut self) {
        self.0 -= 1;
    }
}

const WORKER_COUNT: u32 = 100;

#[tokio::main]
async fn main() {
    let (tx, mut rx): (mpsc::Sender<String>, mpsc::Receiver<String>) = mpsc::channel(1024);
    tx.send("https://www.github.com".to_string()).await.unwrap();

    let mut links: HashSet<String> = HashSet::new();

    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .unwrap();

    let count = Arc::new(Mutex::new(Count(1)));

    while let Some(link) = rx.recv().await {
        if let Some(domain) = parse_root_domain(&link) {
            if !links.contains(&domain) {
                {
                    let mut value = count.lock().unwrap();

                    if value.0 == WORKER_COUNT {
                        continue;
                    }

                    value.increment();
                    println!("{} running tasks ({}) - {}", value.0, links.len(), link);
                }

                links.insert(domain);

                let client = client.clone();
                let tx = tx.clone();

                let count = count.clone();

                tokio::spawn(async move {
                    get(link, &client, tx, &count).await;
                });
            }
        }
    }
}

async fn get(
    url: String,
    client: &reqwest::Client,
    tx: mpsc::Sender<String>,
    count: &Arc<Mutex<Count>>,
) {
    match client.get(url).send().await {
        Ok(res) => {
            let body = res.text().await.unwrap();
            let links = parse_anchor_tags(body);

            for link in links {
                tx.send(link).await.unwrap();
            }
        }
        Err(error) => eprintln!("{}", error),
    };

    let mut value = count.lock().unwrap();
    value.decrement();
}

fn parse_anchor_tags(body: String) -> HashSet<String> {
    let document = Html::parse_document(&body);
    let selector = Selector::parse("a").unwrap();

    document
        .select(&selector)
        .filter_map(|el| el.value().attr("href"))
        .map(|href| href.to_string())
        .collect()
}

fn parse_root_domain(url: &str) -> Option<String> {
    let url = Url::parse(url).ok()?;
    let host = url.host_str()?;
    let domain = List.domain(host.as_bytes())?;
    let domain = str::from_utf8(domain.as_bytes()).unwrap().to_string();
    Some(domain)
}
