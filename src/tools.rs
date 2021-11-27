use lazy_static::lazy_static;
use regex::Regex;
use reqwest::blocking::Client;
use reqwest::{Error, Proxy, Url};
use select::document::Document;
use select::predicate::Name;

pub fn build_client(proxy: Option<&str>, insecure_proxy: bool, user_agent: &str) -> Client {
    let mut client = Client::builder().user_agent(user_agent);
    if let Some(p) = proxy {
        let p = Proxy::all(p).expect("Invalid proxy string");
        client = client.proxy(p).danger_accept_invalid_certs(insecure_proxy)
    }
    client.build().unwrap()
}

pub fn collect_links(client: &Client, url: &Url) -> Vec<String> {
    let document = match extract_document_from_url(client, url) {
        Ok(d) => d,
        Err(_) => return vec![],
    };
    let a_tag_links = collect_links_from_tags(&document, url, "a", "href");
    let script_tag_links = collect_links_from_tags(&document, url, "script", "src");
    let link_tag_links = collect_links_from_tags(&document, url, "link", "href");
    [&a_tag_links[..], &script_tag_links[..], &link_tag_links[..]].concat()
}

fn extract_document_from_url(client: &Client, url: &Url) -> Result<Document, Error> {
    let response = client.get(url.as_str()).send()?;
    let body = response.text()?;
    Ok(Document::from(body.as_str()))
}

fn collect_links_from_tags(
    document: &Document,
    url: &Url,
    tag: &str,
    attribute: &str,
) -> Vec<String> {
    document
        .find(Name(tag))
        .filter_map(|n| {
            if let Some(link) = n.attr(attribute) {
                format_link(url, link)
            } else {
                None
            }
        })
        .collect::<Vec<String>>()
}

fn format_link(url: &Url, link: &str) -> Option<String> {
    lazy_static! {
        static ref BAD_LINK: Regex = Regex::new("^(mailto|#|tel|javascript|\\s*$)").unwrap();
        static ref RELATIVE_LINK: Regex = Regex::new("^\\.?/").unwrap();
        static ref HTTP_PREFIX: Regex = Regex::new("^http").unwrap();
    }

    if BAD_LINK.is_match(link) {
        return None;
    }
    if RELATIVE_LINK.is_match(link) {
        return Some(url.join(link).unwrap().to_string());
    }
    if !HTTP_PREFIX.is_match(link) {
        return None;
    }
    Some(String::from(link))
}

pub fn share_same_domain(left: &Url, right: &Url) -> bool {
    if let (Some(left_domain), Some(right_domain)) = (left.domain(), right.domain()) {
        return left_domain == right_domain;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::format_link;
    use reqwest::Url;

    #[test]
    fn test_format_link_relative() {
        let url = Url::parse("https://example.com").unwrap();
        for (link, expected) in [
            ("./hello", "https://example.com/hello"),
            ("/hello", "https://example.com/hello"),
            ("/hello.js", "https://example.com/hello.js"),
            ("./hello.js", "https://example.com/hello.js"),
        ] {
            let result = format_link(&url, link).unwrap();
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn test_format_link_absolute() {
        let url = Url::parse("https://example.com").unwrap();
        for (link, expected) in [
            ("//hello.com", "https://hello.com/"),
            ("//hello.com/some/path", "https://hello.com/some/path"),
            ("//hello.com/some/path/", "https://hello.com/some/path/"),
        ] {
            let result = format_link(&url, link).unwrap();
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn test_format_link_invalid() {
        let url = Url::parse("https://example.com").unwrap();
        for link in [
            "#some-anchor",
            "mailto:bob@example.com",
            "javascript:something",
            "tel:1234567",
        ] {
            let result = format_link(&url, link);
            assert!(result.is_none());
        }
    }
}
