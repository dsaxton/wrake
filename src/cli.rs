use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, disable_help_flag = true, arg_required_else_help = true)]
pub struct Args {
    #[arg(short, long, help = "Target URL")]
    pub url: String,

    #[arg(short, long, default_value = "wrake", help = "User-Agent header")]
    pub user_agent: String,

    #[arg(short, long, help = "HTTP/HTTPS proxy")]
    pub proxy: Option<String>,

    #[arg(short, long, default_value_t = 2, help = "Recursion depth")]
    pub depth: u8,

    #[arg(short = 'n', long, help = "Do not restrict to original domain")]
    pub no_domain_filter: bool,

    #[arg(short = 'i', long, help = "Accept invalid TLS certs for proxy")]
    pub insecure_proxy: bool,
}
