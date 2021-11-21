use lazy_static::lazy_static;
use regex::Regex;
use reqwest::blocking::Client;
use reqwest::{Proxy, Url};
use select::document::Document;
use select::predicate::Name;

mod app;

// Recursive search
// Parse more tags
// Correctly handle relative links (what if we are not at the root level?)
// Parse URLs and use to optionally exclude certain links from recursion
// Use a domain name instead of full URL, use reqwest::Url methods: https://docs.rs/reqwest/0.3.0/reqwest/struct.Url.html

fn main() {
    let app_matches = app::build_app().get_matches();
    let base_url =
        Url::parse(app_matches.value_of("url").unwrap()).expect("Cannot parse url argument");
    let proxy = app_matches.value_of("proxy");
    let _depth = app_matches
        .value_of("depth")
        .unwrap()
        .parse::<u8>()
        .expect("Cannot parse depth argument");
    let client = build_client(proxy);
    let links = collect_links(&client, &base_url);
    for link in links {
        println!("{}", link);
        let new_url = Url::parse(&link).unwrap();
        if share_same_domain(&base_url, &new_url) {
            let new_links = collect_links(&client, &base_url);
            for new_link in new_links {
                println!("{}", new_link);
            }
        }
    }
}

fn build_client(proxy: Option<&str>) -> Client {
    let mut client = Client::builder();
    if let Some(p) = proxy {
        client = client
            .proxy(Proxy::all(p).unwrap())
            .danger_accept_invalid_certs(true)
    }
    client.build().unwrap()
}

fn collect_links(client: &Client, url: &Url) -> Vec<String> {
    let document = extract_document_from_url(client, url);
    let a_tag_links = collect_links_from_tags(&document, url, "a", "href");
    let script_tag_links = collect_links_from_tags(&document, url, "script", "src");
    let link_tag_links = collect_links_from_tags(&document, url, "link", "href");
    [&a_tag_links[..], &script_tag_links[..], &link_tag_links[..]].concat()
}

fn extract_document_from_url(client: &Client, url: &Url) -> Document {
    let response = client.get(url.as_str()).send().unwrap();
    let body = response.text().unwrap();
    Document::from(body.as_str())
}

fn collect_links_from_tags(
    document: &Document,
    base_url: &Url,
    tag: &str,
    attribute: &str,
) -> Vec<String> {
    document
        .find(Name(tag))
        .filter_map(|n| {
            if let Some(link) = n.attr(attribute) {
                sanitize_link(base_url, link)
            } else {
                None
            }
        })
        .collect::<Vec<String>>()
}

fn sanitize_link(current_url: &Url, link: &str) -> Option<String> {
    lazy_static! {
        static ref BAD_LINK: Regex = Regex::new("^(mailto|#|tel|javascript|^\\s*$)").unwrap();
        static ref RELATIVE_LINK: Regex = Regex::new("^\\.?/").unwrap();
        static ref HTTP_PREFIX: Regex = Regex::new("^http").unwrap();
    }

    if BAD_LINK.is_match(link) {
        return None;
    }

    let sanitized: String;
    if RELATIVE_LINK.is_match(link) {
        sanitized = current_url.join(link).unwrap().to_string();
    } else if !HTTP_PREFIX.is_match(link) {
        return None;
    } else {
        sanitized = String::from(link);
    }
    Some(sanitized)
}

fn share_same_domain(left: &Url, right: &Url) -> bool {
    if let (Some(left_domain), Some(right_domain)) = (left.domain(), right.domain()) {
        return left_domain == right_domain;
    }
    false
}
