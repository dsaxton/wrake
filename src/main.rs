use clap::{App, AppSettings, Arg};
use lazy_static::lazy_static;
use regex::Regex;
use select::document::Document;
use select::predicate::Name;

// Recursive search
// Parse more tags
// Correctly handle relative links (what if we are not at the root level?)
// Parse URLs and use to optionally exclude certain links from recursion

fn main() {
    let app_matches = App::new("wrake")
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
        .get_matches();

    let base_url = app_matches.value_of("url").unwrap();
    println!("base_url: {}", base_url);
    let proxy = app_matches.value_of("proxy");
    let mut client = reqwest::blocking::Client::builder();
    if let Some(p) = proxy {
        client = client
            .proxy(reqwest::Proxy::all(p).unwrap())
            .danger_accept_invalid_certs(true)
    }
    let client = client.build().unwrap();
    let initial_result = client.get(base_url).send().unwrap();
    let body = initial_result.text().unwrap();
    let doc = Document::from(body.as_str());

    println!("a");
    print_links(base_url, &doc, "a", "href");
    println!("script");
    print_links(base_url, &doc, "script", "src");
    println!("link");
    print_links(base_url, &doc, "link", "href");
}

fn print_links(base_url: &str, document: &Document, tag: &str, attr: &str) {
    document
        .find(Name(tag))
        .filter_map(|n| n.attr(attr))
        .for_each(|x| {
            if let Some(link) = sanitize_link(base_url, x) {
                println!("{}", link)
            }
        });
}

fn sanitize_link(base_url: &str, link: &str) -> Option<String> {
    lazy_static! {
        static ref INVALID_LINK_REGEX: Regex = Regex::new("^(mailto|#|tel|javascript)").unwrap();
        static ref DOUBLE_SLASH_PREFIX: Regex = Regex::new("^//").unwrap();
        static ref SINGLE_SLASH_PREFIX: Regex = Regex::new("^/").unwrap();
        static ref DOT_SLASH_PREFIX: Regex = Regex::new("^./").unwrap();
        static ref HTTP_PREFIX: Regex = Regex::new("^http").unwrap();
    }
    let maybe_slash = if base_url.ends_with('/') { "" } else { "/" };
    if INVALID_LINK_REGEX.is_match(link) {
        println!("Invalid link: {}", link);
        return None;
    }
    if DOUBLE_SLASH_PREFIX.is_match(link) {
        println!("Double slash prefix: {}", link);
        return Some(format!("https:{}", link));
    }
    if SINGLE_SLASH_PREFIX.is_match(link) {
        println!("Single slash prefix: {}", link);
        return Some(format!(
            "{}{}{}",
            base_url,
            maybe_slash,
            SINGLE_SLASH_PREFIX.replace_all(link, "")
        ));
    }
    if DOT_SLASH_PREFIX.is_match(link) {
        println!("Dot slash prefix: {}", link);
        return Some(format!(
            "{}{}{}",
            base_url,
            maybe_slash,
            DOT_SLASH_PREFIX.replace_all(link, "")
        ));
    }
    if !HTTP_PREFIX.is_match(link) {
        println!("No HTTP prefix: {}", link);
        return Some(format!("{}{}{}", base_url, maybe_slash, link));
    }
    Some(String::from(link))
}
