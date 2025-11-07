use anyhow::{Result};
use reqwest::{Client, Proxy, Url};
use scraper::{Html, Selector};
use std::time::Duration;

pub struct Crawler {
    client:       Client,
    start_domain: String,
    filter:       bool,
}

impl Crawler {
    pub fn new(args: &crate::cli::Args) -> Result<Self> {
        let mut builder = Client::builder()
            .user_agent(&args.user_agent)
            .timeout(Duration::from_secs(10))
            .gzip(true);

        if let Some(p) = &args.proxy {
            let proxy = Proxy::all(p).map_err(|_| anyhow::anyhow!("invalid proxy"))?;
            builder = builder.proxy(proxy).danger_accept_invalid_certs(args.insecure_proxy);
        }
        let client = builder.build()?;

        let start_domain = Url::parse(&args.url)?
            .domain()
            .ok_or_else(|| anyhow::anyhow!("cannot extract domain"))?
            .to_owned();

        Ok(Self { client, start_domain, filter: !args.no_domain_filter })
    }

    pub async fn fetch_links(&self, url: &str) -> Result<Vec<String>> {
        let body = self.client.get(url).send().await?.text().await?;
        let doc = Html::parse_document(&body);
        let selector = Selector::parse("a[href], link[href], script[src]").unwrap();

        let mut links = Vec::new();
        for node in doc.select(&selector) {
            let raw = match node.value().attr("href").or_else(|| node.value().attr("src")) {
                Some(v) => v.trim(),
                None => continue,
            };
            let abs = self.abs_url(url, raw)?;
            if self.filter && !self.same_domain(&abs) {
                continue;
            }
            links.push(abs);
        }
        Ok(links)
    }

    pub fn same_domain(&self, url: &str) -> bool {
        Url::parse(url)
            .ok()
            .and_then(|u| u.domain().map(str::to_owned))
            .map_or(false, |d| d == self.start_domain)
    }

    fn abs_url(&self, base: &str, link: &str) -> Result<String> {
        if link.starts_with("http") {
            return Ok(link.to_owned());
        }
        let base = Url::parse(base)?;
        Ok(base.join(link)?.to_string())
    }
}
