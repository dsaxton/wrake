use clap::Parser;
use reqwest::Url;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::task;

mod cli;
mod crawler;

use cli::Args;
use crawler::Crawler;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let start = Url::parse(&args.url)?;
    let crawler = Arc::new(Crawler::new(&args)?);

    let mut seen = HashSet::new();
    let mut queue = vec![start.as_str().to_owned()];

    for _ in 0..=args.depth {
        let batch: Vec<_> = queue.drain(..).collect();
        let tasks: Vec<_> = batch
            .into_iter()
            .map(|u| {
                let crawler = Arc::clone(&crawler);
                task::spawn(async move { crawler.fetch_links(&u).await })
            })
            .collect();

        queue.clear();
        for job in tasks {
            for link in job.await?? {
                if seen.insert(link.clone()) {
                    println!("{link}");
                    if crawler.same_domain(&link) {
                        queue.push(link);
                    }
                }
            }
        }
        if queue.is_empty() {
            break;
        }
    }
    Ok(())
}
