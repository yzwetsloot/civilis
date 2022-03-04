use psl::{List, Psl};
use scraper::{Html, Selector};
use std::collections::HashSet;
use std::str;
use url::Url;

pub fn parse_unique_domains(body: String, history: &super::History) -> HashSet<String> {
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
