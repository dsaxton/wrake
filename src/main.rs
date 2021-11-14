use clap::{App, AppSettings, Arg};
use select::document::Document;
use select::predicate::Name;

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

    doc.find(Name("a"))
        .filter_map(|n| n.attr("href"))
        .for_each(|x| println!("{}", x));
}
