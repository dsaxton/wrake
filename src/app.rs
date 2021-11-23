use clap::{App, AppSettings, Arg};

pub fn build_app() -> App<'static> {
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
            Arg::new("user-agent")
                .short('a')
                .long("user-agent")
                .value_name("string")
                .about("User agent header when sending requests")
                .takes_value(true)
                .default_value("wrake")
                .required(false),
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
                .default_value("2"),
        )
        .arg(
            Arg::new("no-restrict-domain")
                .short('n')
                .long("no-restrict-domain")
                .about("Do not restrict search to original domain")
                .takes_value(false)
                .required(false),
        )
        .arg(
            Arg::new("insecure-proxy")
                .short('k')
                .long("insecure-proxy")
                .about("Accept invalid certificates for proxy")
                .takes_value(false)
                .required(false),
        )
}
