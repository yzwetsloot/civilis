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
        .filter_map(|href| is_new_link(href, history))
        .collect()
}

fn is_new_link(url: &str, history: &super::History) -> Option<String> {
    if let Some(tld) = parse_tld(url) {
        let mut history = history.lock().unwrap();
        if !history.contains(&tld) {
            println!("{}", tld);
            history.insert(tld.to_string());
            return Some(url.to_string());
        }
    }
    None
}

fn parse_tld(url: &str) -> Option<String> {
    let url = Url::parse(url).ok()?;
    let host = url.host_str()?;
    let domain = List.domain(host.as_bytes())?;
    let domain = str::from_utf8(domain.as_bytes()).unwrap().to_string();
    Some(domain)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[test]
    fn parse_unique_no_anchor_tags() {
        let body = "
<html>
    <body>
        <h1>test</h1>
        <p>lorem ipsum</p>
    </body>
</html>
";
        let empty_set: HashSet<String> = HashSet::new(); // empty set
        let history = Arc::new(Mutex::new(HashSet::new())); // empty history

        assert_eq!(empty_set, parse_unique_domains(body.to_string(), &history));
    }

    #[test]
    fn parse_unique_without_href() {
        let body = "
<html>
    <body>
        <h1>test</h1>
        <a>test</a>
    </body>
</html>
";
        let empty_set: HashSet<String> = HashSet::new(); // empty set
        let history = Arc::new(Mutex::new(HashSet::new())); // empty history

        assert_eq!(empty_set, parse_unique_domains(body.to_string(), &history));
    }

    #[test]
    fn parse_unique_multi_domain() {
        let body = r#"
<html>
    <body>
        <h1>test</h1>
        <a href="/">test</a>
        <div>
            <a href="https://www.google.com">Google</a>
            <a href="https://github.com">Github</a>
            <a href="/aboutus">About Us</a>
        </div>
    </body>
</html>
"#;
        let mut set = HashSet::new();
        set.insert("https://www.google.com".to_string());

        let history = Arc::new(Mutex::new(HashSet::new()));
        {
            let mut history = history.lock().unwrap();
            history.insert("github.com".to_string());
        }

        assert_eq!(set, parse_unique_domains(body.to_string(), &history));
    }

    #[test]
    fn is_new_link_relative_url() {
        let history = Arc::new(Mutex::new(HashSet::new()));
        assert_eq!(None, is_new_link("/", &history));
    }

    #[test]
    fn is_new_link_unseen() {
        let history = Arc::new(Mutex::new(HashSet::new()));
        {
            let mut history = history.lock().unwrap();
            history.insert("github.com".to_string());
        }
        assert_eq!(None, is_new_link("https://test.github.com", &history));
    }

    #[test]
    fn parse_tld_relative_url() {
        let url = "/test";
        assert_eq!(None, parse_tld(url));
    }

    #[test]
    fn parse_tld_blogspot_domain() {
        let url = "https://www.test.blogspot.com";
        assert_eq!(Some("test.blogspot.com".to_string()), parse_tld(url));
    }

    #[test]
    fn parse_tld_google_domain() {
        let url = "https://myaccount.google.com";
        assert_eq!(Some("google.com".to_string()), parse_tld(url));
    }
}
