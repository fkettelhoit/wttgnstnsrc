mod compact;
mod downloader;
mod pdf;
mod scraper;

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};

#[derive(Clone, Copy, PartialEq)]
enum PageResult { Ok, Failed, NoImage }

use clap::Parser;
use futures::stream::{self, StreamExt};

#[derive(Parser)]
#[command(about = "Download Wittgenstein Nachlass facsimiles from wittgensteinsource.org")]
struct Args {
    /// Maximum image width in pixels
    #[arg(long, default_value_t = 2000)]
    max_width: u32,

    /// Base output directory [default: facsimiles/{max_width}px]
    #[arg(long)]
    destination: Option<PathBuf>,

    /// Download all facsimiles, not just CC-licensed ones from the Wren Library
    #[arg(long)]
    all: bool,

    /// Skip these documents (comma-separated names, e.g. Ms-107,Ms-108)
    #[arg(long, value_delimiter = ',', conflicts_with = "only")]
    skip: Vec<String>,

    /// Only download these documents (comma-separated names, e.g. Ms-107,Ms-108)
    #[arg(long, value_delimiter = ',')]
    only: Vec<String>,

    /// Number of images to download in parallel
    #[arg(long, default_value_t = 1)]
    parallel: usize,

    /// Generate a PDF for each document. Use --pdf for JPEG q90 (default),
    /// --pdf=75 for custom quality, or --pdf=uncompressed for raw RGB.
    #[arg(long, default_missing_value = "90", num_args = 0..=1)]
    pdf: Option<String>,

    /// Disable parallel WebP conversion
    #[arg(long)]
    webp_slow: bool,
}

/// Look at existing PNGs in doc_dir and return the most common width.
/// Falls back to max_width if no PNGs exist yet.
fn detect_target_width(doc_dir: &Path, max_width: u32) -> u32 {
    let entries = match std::fs::read_dir(doc_dir) {
        Ok(e) => e,
        Err(_) => return max_width,
    };

    let mut width_counts: HashMap<u32, usize> = HashMap::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("png")
            && let Ok((w, _)) = image::image_dimensions(&path)
        {
            *width_counts.entry(w).or_insert(0) += 1;
        }
    }

    width_counts
        .into_iter()
        .max_by_key(|&(_, count)| count)
        .map(|(w, _)| w)
        .unwrap_or(max_width)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let max_width = args.max_width;
    let base_dir = args.destination
        .unwrap_or_else(|| PathBuf::from(format!("facsimiles/{max_width}px")));
    let png_dir = base_dir.join("png");
    let webp_dir = base_dir.join("webp");
    let pdf_dir = base_dir.join("pdf");

    println!("Fetching document names from wittgensteinsource.org...");
    let doc_links = scraper::fetch_document_links(!args.all).await?;
    let total_docs = doc_links.len();
    println!("Found {total_docs} documents.");

    let total_failed = AtomicU32::new(0);
    let total_no_image = AtomicU32::new(0);

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
            let doc_dir = png_dir.join(doc_name);
            let target_width = detect_target_width(&doc_dir, max_width);

            let results: Vec<PageResult> = stream::iter(pages.iter().enumerate())
                .map(|(page_idx, (doc, page))| {
                    let dzi_url = scraper::build_dzi_url(doc, page);
                    let output_path = png_dir.join(doc).join(format!("{page}.png"));
                    let total_failed = &total_failed;
                    let total_no_image = &total_no_image;
                    async move {
                        if output_path.exists() {
                            return PageResult::Ok;
                        }
                        println!("[{doc_num}/{total_docs}] {doc}/{page} ({}/{})", page_idx + 1, total_pages);
                        for retry in 1..=3 {
                            match downloader::download_dzi_with_fallback(&dzi_url, &output_path, max_width, target_width).await {
                                Ok(_) => return PageResult::Ok,
                                Err(e) => {
                                    let msg = shorten_error(&e);
                                    if msg == "couldn't find a zoomable image" {
                                        eprintln!("  No zoomable image for {doc}/{page}");
                                        total_no_image.fetch_add(1, Ordering::Relaxed);
                                        return PageResult::NoImage;
                                    }
                                    eprintln!("  Retry {retry}/3 failed for {doc}/{page}: {msg}");
                                }
                            }
                        }
                        eprintln!("Failed {doc}/{page} after 3 retries");
                        total_failed.fetch_add(1, Ordering::Relaxed);
                        PageResult::Failed
                    }
                })
                .buffer_unordered(args.parallel)
                .collect()
                .await;

            let has_real_failures = results.contains(&PageResult::Failed);
            if !has_real_failures {
                println!("Completed {doc_name} ({total_pages} pages)");

                // WebP conversion (always runs)
                let webp_doc_dir = webp_dir.join(doc_name);
                println!("Converting to WebP: {doc_name}...");
                match compact::convert_to_webp(&pages, &png_dir.join(doc_name), &webp_doc_dir, 80.0, args.webp_slow) {
                    Ok(_) => {},
                    Err(e) => eprintln!("Warning: WebP conversion failed for {doc_name}: {e}"),
                }

                // PDF generation (if --pdf is set)
                if let Some(ref quality_str) = args.pdf {
                    let jpeg_quality = if quality_str == "uncompressed" {
                        None
                    } else {
                        Some(quality_str.parse::<u8>().unwrap_or(90))
                    };

                    std::fs::create_dir_all(&pdf_dir)?;
                    let pdf_path = pdf_dir.join(format!("{doc_name}.pdf"));
                    if pdf_path.exists() {
                        println!("PDF already exists: {}", pdf_path.display());
                    } else {
                        println!("Generating PDF for {doc_name}...");
                        match pdf::generate_pdf(doc_name, &pages, &png_dir.join(doc_name), &pdf_path, jpeg_quality) {
                            Ok(_) => println!("Created {}", pdf_path.display()),
                            Err(e) => eprintln!("Warning: PDF generation failed for {doc_name}: {e}"),
                        }
                    }
                }

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

    let failed = total_failed.load(Ordering::Relaxed);
    let no_image = total_no_image.load(Ordering::Relaxed);
    match (failed, no_image) {
        (0, 0) => println!("Done. All pages downloaded successfully."),
        (0, n) => println!("Done. {n} pages had no zoomable image."),
        (f, 0) => println!("Done. {f} pages failed to download."),
        (f, n) => println!("Done. {f} pages failed to download. {n} pages had no zoomable image."),
    }
    Ok(())
}

fn shorten_error(e: &anyhow::Error) -> String {
    let text = e.to_string();
    if text.contains("none succeeded") || text.contains("Tried all") {
        return "couldn't find a zoomable image".to_string();
    }
    let first_line = text.lines().next().unwrap_or("unknown error");
    first_line.to_string()
}
