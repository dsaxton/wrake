use lazy_static::lazy_static;
use regex::Regex;
use reqwest::blocking::Client;
use reqwest::{Proxy, Url};
use select::document::Document;
use select::predicate::Name;

pub fn build_client(proxy: Option<&str>) -> Client {
    let mut client = Client::builder();
    if let Some(p) = proxy {
        client = client
            .proxy(Proxy::all(p).unwrap())
            .danger_accept_invalid_certs(true)
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

pub fn extract_document_from_url(client: &Client, url: &Url) -> Result<Document, reqwest::Error> {
    let response = client.get(url.as_str()).send()?;
    let body = response.text()?;
    Ok(Document::from(body.as_str()))
}

pub fn collect_links_from_tags(
    document: &Document,
    url: &Url,
    tag: &str,
    attribute: &str,
) -> Vec<String> {
    document
        .find(Name(tag))
        .filter_map(|n| {
            if let Some(link) = n.attr(attribute) {
                sanitize_link(url, link)
            } else {
                None
            }
        })
        .collect::<Vec<String>>()
}

pub fn sanitize_link(url: &Url, link: &str) -> Option<String> {
    lazy_static! {
        static ref BAD_LINK: Regex = Regex::new("^(mailto|#|tel|javascript|^\\s*$)").unwrap();
        static ref RELATIVE_LINK: Regex = Regex::new("^\\.?/").unwrap();
        static ref HTTP_PREFIX: Regex = Regex::new("^http").unwrap();
    }

    if BAD_LINK.is_match(link) {
        return None;
    }

    let result: String;
    if RELATIVE_LINK.is_match(link) {
        result = url.join(link).unwrap().to_string();
    } else if !HTTP_PREFIX.is_match(link) {
        return None;
    } else {
        result = String::from(link);
    }
    Some(result)
}

pub fn share_same_domain(left: &Url, right: &Url) -> bool {
    if let (Some(left_domain), Some(right_domain)) = (left.domain(), right.domain()) {
        return left_domain == right_domain;
    }
    false
}