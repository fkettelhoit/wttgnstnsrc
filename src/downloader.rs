use std::path::Path;

use anyhow::{Context, Result};
use image::imageops::FilterType;
use structopt::StructOpt;

/// Download a single DZI image using dezoomify-rs, writing the result to `output_path`.
/// Skips download if the output file already exists. Uses a temporary file to avoid partial writes.
pub async fn download_dzi(dzi_url: &str, output_path: &Path, max_width: u32) -> Result<()> {
    if output_path.exists() {
        return Ok(());
    }

    if let Some(parent) = output_path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .with_context(|| format!("Failed to create directory {}", parent.display()))?;
    }

    let tmp_path = output_path.with_extension("png-tmp.png");
    let tmp_str = tmp_path
        .to_str()
        .context("Temp path is not valid UTF-8")?;

    let args = dezoomify_rs::Arguments::from_iter([
        "dezoomify-rs",
        "--max-width",
        &max_width.to_string(),
        "--retries",
        "2",
        "--timeout",
        "60s",
        dzi_url,
        tmp_str,
    ]);

    match dezoomify_rs::dezoomify(&args).await {
        Ok(_) => {
            tokio::fs::rename(&tmp_path, output_path)
                .await
                .with_context(|| {
                    format!(
                        "Failed to rename {} to {}",
                        tmp_path.display(),
                        output_path.display()
                    )
                })?;
        }
        Err(e) => {
            // Clean up tmp file on failure
            let _ = tokio::fs::remove_file(&tmp_path).await;
            let msg = if e.to_string().contains("none succeeded") {
                "couldn't find a zoomable image".to_string()
            } else {
                e.to_string().lines().next().unwrap_or("unknown error").to_string()
            };
            anyhow::bail!("{msg}");
        }
    }

    Ok(())
}

/// Try downloading at max_width first. On any failure, retry at full resolution
/// and scale down to target_width.
pub async fn download_dzi_with_fallback(
    dzi_url: &str,
    output_path: &Path,
    max_width: u32,
    target_width: u32,
    fallback_width: u32,
) -> Result<()> {
    if output_path.exists() {
        return Ok(());
    }

    // Try at requested max_width first
    match download_dzi(dzi_url, output_path, max_width).await {
        Ok(_) => return Ok(()),
        Err(e) if e.to_string().contains("couldn't find a zoomable image") => return Err(e),
        Err(_) => {
            // Fall through to full-resolution fallback
        }
    }

    let page_name = output_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown");
    println!("  Retrying {page_name} at --max-width={fallback_width} (will scale to {target_width}px)...");

    if let Some(parent) = output_path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .with_context(|| format!("Failed to create directory {}", parent.display()))?;
    }

    let fullres_path = output_path.with_extension("png-fullres.png");
    let fullres_str = fullres_path
        .to_str()
        .context("Fullres path is not valid UTF-8")?;

    let fallback_width_str = fallback_width.to_string();
    let args = dezoomify_rs::Arguments::from_iter([
        "dezoomify-rs",
        "--max-width",
        &fallback_width_str,
        "--retries",
        "2",
        "--timeout",
        "60s",
        dzi_url,
        fullres_str,
    ]);

    match dezoomify_rs::dezoomify(&args).await {
        Ok(_) => {
            // Resize to target_width
            let fullres_path_clone = fullres_path.clone();
            let output_path_owned = output_path.to_path_buf();
            tokio::task::spawn_blocking(move || {
                let img = image::open(&fullres_path_clone)
                    .with_context(|| format!("Failed to open {}", fullres_path_clone.display()))?;
                let (w, h) = (img.width(), img.height());
                let new_height = (h as f64 * target_width as f64 / w as f64).round() as u32;
                let resized = image::imageops::resize(&img, target_width, new_height, FilterType::Lanczos3);
                resized.save(&output_path_owned)
                    .with_context(|| format!("Failed to save resized image to {}", output_path_owned.display()))?;
                std::fs::remove_file(&fullres_path_clone).ok();
                Ok::<_, anyhow::Error>(())
            })
            .await
            .context("Resize task panicked")??;
            Ok(())
        }
        Err(e) => {
            let _ = tokio::fs::remove_file(&fullres_path).await;
            let msg = if e.to_string().contains("none succeeded") {
                "couldn't find a zoomable image".to_string()
            } else {
                e.to_string().lines().next().unwrap_or("unknown error").to_string()
            };
            anyhow::bail!("{msg}");
        }
    }
}
