use super::{Graph, Vertex};
use psl::{List, Psl};
use scraper::{Html, Selector};
use std::collections::HashSet;
use std::str;
use url::Url;

pub fn parse_unique_domains(body: String, src: String, g: &Graph) -> HashSet<String> {
    let document = Html::parse_document(&body);
    let selector = Selector::parse("a").unwrap();

    document
        .select(&selector)
        .filter_map(|el| el.value().attr("href"))
        .filter_map(|href| is_new_link(href, src.clone(), g))
        .collect()
}

fn is_new_link(url: &str, src: String, g: &Graph) -> Option<String> {
    let src = parse_root_domain(&src).unwrap();
    if let Some(domain) = parse_root_domain(url) {
        if !g.contains(&domain) {
            g.add_vertex(Vertex::new(domain.clone()));
            println!("{} - {}", g.size(), domain);
            g.add_edge(&src, &domain);
            return Some(url.to_string());
        }
    }
    None
}

pub fn parse_root_domain(url: &str) -> Option<String> {
    let url = Url::parse(url).ok()?;
    let host = url.host_str()?;
    let domain = List.domain(host.as_bytes())?;
    let domain = str::from_utf8(domain.as_bytes()).unwrap().to_string();
    Some(domain)
}

#[cfg(test)]
mod tests {
    use crate::{Graph, Vertex};

    use super::*;

    const NUM_SHARDS: u64 = 1;
    const ROOT_DOMAIN: &str = "https://github.com";

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
        let graph = Graph::new(NUM_SHARDS); // empty graph

        assert_eq!(
            empty_set,
            parse_unique_domains(body.to_string(), ROOT_DOMAIN.to_string(), &graph)
        );
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
        let graph = Graph::new(NUM_SHARDS); // empty graph

        assert_eq!(
            empty_set,
            parse_unique_domains(body.to_string(), ROOT_DOMAIN.to_string(), &graph)
        );
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

        let graph = Graph::new(NUM_SHARDS);
        graph.add_vertex(Vertex::new("github.com".to_string()));
        assert_eq!(
            set,
            parse_unique_domains(body.to_string(), ROOT_DOMAIN.to_string(), &graph)
        );
    }

    #[test]
    fn is_new_link_relative_url() {
        let graph = Graph::new(NUM_SHARDS);
        assert_eq!(None, is_new_link("/", ROOT_DOMAIN.to_string(), &graph));
    }

    #[test]
    fn is_new_link_unseen() {
        let graph = Graph::new(NUM_SHARDS);
        graph.add_vertex(Vertex::new("github.com".to_string()));

        assert_eq!(
            None,
            is_new_link("https://test.github.com", ROOT_DOMAIN.to_string(), &graph)
        );
    }

    #[test]
    fn parse_root_domain_relative_url() {
        let url = "/test";
        assert_eq!(None, parse_root_domain(url));
    }

    #[test]
    fn parse_root_domain_blogspot_domain() {
        let url = "https://www.test.blogspot.com";
        assert_eq!(
            Some("test.blogspot.com".to_string()),
            parse_root_domain(url)
        );
    }

    #[test]
    fn parse_root_domain_google_domain() {
        let url = "https://myaccount.google.com";
        assert_eq!(Some("google.com".to_string()), parse_root_domain(url));
    }
}
