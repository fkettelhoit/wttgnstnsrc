use std::path::Path;

use anyhow::{Context, Result};
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
            anyhow::bail!(
                "Failed to download {} to {}: {e}",
                dzi_url,
                output_path.display()
            );
        }
    }

    Ok(())
}
