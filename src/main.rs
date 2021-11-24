use rayon::prelude::*;
use reqwest::Url;
use std::collections::HashSet;
use tools::{build_client, collect_links, share_same_domain};

mod app;
mod tools;

fn main() {
    let app_matches = app::build_app().get_matches();
    let url = Url::parse(app_matches.value_of("url").unwrap()).expect("Cannot parse url argument");
    let user_agent = app_matches.value_of("user-agent").unwrap();
    let proxy = app_matches.value_of("proxy");
    let no_domain_filter = app_matches.is_present("no-domain-filter");
    let insecure_proxy = app_matches.is_present("insecure-proxy");
    let mut depth = app_matches
        .value_of("depth")
        .unwrap()
        .parse::<i8>()
        .expect("Cannot parse depth argument");
    let client = build_client(proxy, insecure_proxy, user_agent);
    let mut shown_links: HashSet<String> = HashSet::new();
    let links = collect_links(&client, &url);
    links.iter().for_each(|link| {
        if !shown_links.contains(link) {
            println!("{}", link);
            shown_links.insert(link.clone());
        }
    });
    while depth > 0 {
        let links = links
            .par_iter()
            .map(|link| Url::parse(link).unwrap())
            .filter(|link| no_domain_filter || share_same_domain(&url, link))
            .flat_map(|link| collect_links(&client, &link))
            .collect::<Vec<String>>();
        links.iter().for_each(|link| {
            if !shown_links.contains(link) {
                println!("{}", link);
                shown_links.insert(link.clone());
            }
        });
        depth -= 1;
    }
}
