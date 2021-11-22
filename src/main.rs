use rayon::prelude::*;
use reqwest::Url;
use tools::{build_client, collect_links, share_same_domain};

mod app;
mod tools;

fn main() {
    let app_matches = app::build_app().get_matches();
    let url = Url::parse(app_matches.value_of("url").unwrap()).expect("Cannot parse url argument");
    let proxy = app_matches.value_of("proxy");
    let no_restrict_domain = app_matches.is_present("no-restrict-domain");
    let mut depth = app_matches
        .value_of("depth")
        .unwrap()
        .parse::<u8>()
        .expect("Cannot parse depth argument");
    let client = build_client(proxy);
    let links = collect_links(&client, &url);
    links.iter().for_each(|link| {
        println!("{}", link);
    });
    while depth > 1 {
        let links = links
            .par_iter()
            .filter(|link| {
                no_restrict_domain || share_same_domain(&url, &Url::parse(link).unwrap())
            })
            .flat_map(|link| collect_links(&client, &Url::parse(link).unwrap()))
            .collect::<Vec<String>>();
        links.iter().for_each(|link| {
            println!("{}", link);
        });
        depth -= 1;
    }
}
