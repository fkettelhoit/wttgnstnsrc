mod downloader;
mod scraper;

use std::path::PathBuf;

use clap::Parser;
use futures::stream::{self, StreamExt};

#[derive(Parser)]
#[command(about = "Download Wittgenstein Nachlass facsimiles from wittgensteinsource.org")]
struct Args {
    /// Maximum image width in pixels
    #[arg(long, default_value_t = 2000)]
    max_width: u32,

    /// Destination directory for downloaded facsimiles [default: facsimiles-{max_width}px]
    #[arg(long)]
    destination: Option<PathBuf>,

    /// Download all facsimiles, not just CC-licensed ones from the Wren Library
    #[arg(long)]
    all: bool,

    /// Skip these documents (comma-separated names, e.g. Ms-107,Ms-108)
    #[arg(long, value_delimiter = ',')]
    skip: Vec<String>,

    /// Only download these documents (comma-separated names, e.g. Ms-107,Ms-108)
    #[arg(long, value_delimiter = ',')]
    only: Vec<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let max_width = args.max_width;
    let destination = args.destination
        .unwrap_or_else(|| PathBuf::from(format!("facsimiles-{max_width}px")));

    println!("Fetching document names from wittgensteinsource.org...");
    let doc_links = scraper::fetch_document_links(!args.all).await?;
    let total_docs = doc_links.len();
    println!("Found {total_docs} documents.");

    for (doc_idx, doc_url) in doc_links.iter().enumerate() {
        let doc_num = doc_idx + 1;

        let mut doc_ok = false;
        for doc_attempt in 1..=3 {
            let pages = match scraper::fetch_pages_for_doc(doc_url).await {
                Ok(pages) => pages,
                Err(e) => {
                    eprintln!("Error fetching pages for {doc_url} (attempt {doc_attempt}/3): {e}");
                    continue;
                }
            };

            if pages.is_empty() {
                doc_ok = true;
                break;
            }

            let doc_name = &pages[0].0;

            if !args.skip.is_empty() && args.skip.iter().any(|s| s == doc_name) {
                println!("[{doc_num}/{total_docs}] Skipping {doc_name}");
                doc_ok = true;
                break;
            }
            if !args.only.is_empty() && !args.only.iter().any(|s| s == doc_name) {
                println!("[{doc_num}/{total_docs}] Skipping {doc_name} (not in --only list)");
                doc_ok = true;
                break;
            }

            let total_pages = pages.len();

            let results: Vec<bool> = stream::iter(pages.iter().enumerate())
                .map(|(page_idx, (doc, page))| {
                    let dzi_url = scraper::build_dzi_url(doc, page);
                    let output_path = destination.join(doc).join(format!("{page}.png"));
                    async move {
                        if output_path.exists() {
                            return true;
                        }
                        println!("[{doc_num}/{total_docs}] {doc}/{page} ({}/{})", page_idx + 1, total_pages);
                        for retry in 1..=5 {
                            match downloader::download_dzi(&dzi_url, &output_path, max_width).await {
                                Ok(_) => return true,
                                Err(e) => {
                                    let msg = e.to_string();
                                    if msg.contains("none succeeded") {
                                        println!("  No zoomable image found for {doc}/{page}, skipping.");
                                        return true;
                                    }
                                    eprintln!("  Retry {retry}/5 failed for {doc}/{page}: {e}");
                                }
                            }
                        }
                        eprintln!("Failed {doc}/{page} after 5 retries");
                        false
                    }
                })
                .buffer_unordered(3)
                .collect()
                .await;

            let all_pages_ok = results.iter().all(|&ok| ok);
            if all_pages_ok {
                println!("Completed {doc_name} ({total_pages} pages)");
                doc_ok = true;
                break;
            } else {
                eprintln!("Some pages failed, retrying whole document (attempt {doc_attempt}/3)");
            }
        }

        if !doc_ok {
            eprintln!("Giving up on {doc_url} after 3 document-level attempts");
        }
    }

    println!("Done.");
    Ok(())
}
