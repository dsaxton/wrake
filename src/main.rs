use clap::{App, AppSettings, Arg};
use lazy_static::lazy_static;
use regex::Regex;
use select::document::Document;
use select::predicate::Name;

// Recursive search
// Parse more tags
// Complete relative links
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

    let url = app_matches.value_of("url").unwrap();
    let proxy = app_matches.value_of("proxy");
    let mut client = reqwest::blocking::Client::builder();
    if let Some(p) = proxy {
        client = client
            .proxy(reqwest::Proxy::all(p).unwrap())
            .danger_accept_invalid_certs(true)
    }
    let client = client.build().unwrap();
    let initial_result = client.get(url).send().unwrap();
    let body = initial_result.text().unwrap();
    let doc = Document::from(body.as_str());

    println!("Body:");
    println!();
    println!("{:?}", body);
    println!();
    // println!("Document:");
    // println!();
    // println!("{:?}", doc);
    // println!();
    println!("a:");
    println!();
    print_attributes(&doc, "a", "href");
    println!();
    println!("script:");
    println!();
    print_attributes(&doc, "a", "src");
    println!("link:");
    println!();
    print_attributes(&doc, "a", "href");
}

fn print_attributes(document: &Document, tag: &str, attr: &str) {
    document
        .find(Name(tag))
        .filter_map(|n| n.attr(attr))
        .for_each(|x| {
            if let Some(link) = sanitize_link(x) {
                println!("{}", link)
            }
        });
}

fn sanitize_link(link: &str) -> Option<&str> {
    lazy_static! {
        static ref INVALID_LINK_REGEX: Regex = Regex::new("^(mailto|#|tel|javascript)").unwrap();
    }
    if INVALID_LINK_REGEX.is_match(link) {
        return None;
    }
    Some(link)
}
