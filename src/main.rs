use clap::{App, AppSettings, Arg};
use lazy_static::lazy_static;
use regex::Regex;
use reqwest::blocking::Client;
use reqwest::{Proxy, Url};
use select::document::Document;
use select::predicate::Name;

// Recursive search
// Parse more tags
// Correctly handle relative links (what if we are not at the root level?)
// Parse URLs and use to optionally exclude certain links from recursion
// Use a domain name instead of full URL, use reqwest::Url methods: https://docs.rs/reqwest/0.3.0/reqwest/struct.Url.html

fn main() {
    let app_matches = build_app().get_matches();
    let base_url =
        Url::parse(app_matches.value_of("url").unwrap()).expect("Cannot parse url argument");
    let proxy = app_matches.value_of("proxy");
    let _depth = app_matches
        .value_of("depth")
        .unwrap()
        .parse::<u8>()
        .expect("Cannot parse depth argument");
    let mut client = Client::builder();
    if let Some(p) = proxy {
        client = client
            .proxy(Proxy::all(p).unwrap())
            .danger_accept_invalid_certs(true)
    }
    let client = client.build().unwrap();
    let document = extract_document_from_url(&client, base_url.as_str());
    let result = collect_links(&document, &base_url);
    for link in result {
        println!("{}", link);
    }
}

fn build_app() -> App<'static> {
    App::new("wrake")
        .version("0.1.0")
        .about("Collect links from the given URL")
        .setting(AppSettings::ArgRequiredElseHelp)
        .setting(AppSettings::DisableHelpSubcommand)
        .arg(
            Arg::new("url")
                .short('u')
                .long("url")
                .value_name("string")
                .about("Target URL")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::new("proxy")
                .short('p')
                .long("proxy")
                .value_name("string")
                .about("Proxy through which to send requests")
                .takes_value(true)
                .required(false),
        )
        .arg(
            Arg::new("depth")
                .short('d')
                .long("depth")
                .value_name("integer")
                .about("Recursion depth")
                .takes_value(true)
                .required(false)
                .default_value("3"),
        )
}

fn extract_document_from_url(client: &Client, url: &str) -> Document {
    let response = client.get(url).send().unwrap();
    let body = response.text().unwrap();
    Document::from(body.as_str())
}

fn collect_links(document: &Document, base_url: &Url) -> Vec<String> {
    let a_tag_links = collect_links_from_tags(document, base_url, "a", "href");
    let script_tag_links = collect_links_from_tags(document, base_url, "script", "src");
    let link_tag_links = collect_links_from_tags(document, base_url, "link", "href");
    [&a_tag_links[..], &script_tag_links[..], &link_tag_links[..]].concat()
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

fn sanitize_link(base_url: &Url, link: &str) -> Option<String> {
    lazy_static! {
        static ref BAD_LINK: Regex = Regex::new("^(mailto|#|tel|javascript)").unwrap();
        static ref DOUBLE_SLASH_PREFIX: Regex = Regex::new("^//").unwrap();
        static ref RELATIVE_LINK_PREFIX: Regex = Regex::new("^\\.?/").unwrap();
        // FIXME: need to handle these cases separately and fix RELATIVE_LINK_PREFIX
        // static ref ABSOLUTE_LINK_PREFIX: Regex = Regex::new("^/[^/]").unwrap();
        static ref HTTP_PREFIX: Regex = Regex::new("^http").unwrap();
    }

    if BAD_LINK.is_match(link) {
        return None;
    }

    let sanitized: String;
    if DOUBLE_SLASH_PREFIX.is_match(link) {
        sanitized = format!("https:{}", link);
    } else if RELATIVE_LINK_PREFIX.is_match(link) {
        sanitized = base_url
            .join(&RELATIVE_LINK_PREFIX.replace(link, ""))
            .unwrap()
            .to_string();
    } else if !HTTP_PREFIX.is_match(link) {
        sanitized = base_url.join(link).unwrap().to_string();
    } else {
        sanitized = String::from(link);
    }
    Some(sanitized)
}
