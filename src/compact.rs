use std::fs;
use std::path::Path;

use anyhow::Context;
use rayon::prelude::*;
use webp::WebPConfig;

fn convert_page(page_name: &str, png_dir: &Path, webp_dir: &Path, quality: f32) -> anyhow::Result<()> {
    let png_path = png_dir.join(format!("{page_name}.png"));
    let webp_path = webp_dir.join(format!("{page_name}.webp"));

    if !png_path.exists() || webp_path.exists() {
        return Ok(());
    }

    let img = ::image::open(&png_path)
        .with_context(|| format!("Failed to open {}", png_path.display()))?;
    let rgb = img.into_rgb8();
    let (w, h) = (rgb.width(), rgb.height());

    let encoder = webp::Encoder::from_rgb(rgb.as_raw(), w, h);
    let mut config = WebPConfig::new().unwrap();
    config.quality = quality;
    config.thread_level = 1;
    let webp_data = encoder
        .encode_advanced(&config)
        .map_err(|e| anyhow::anyhow!("WebP encoding failed for {page_name}: {e:?}"))?;

    fs::write(&webp_path, &*webp_data)
        .with_context(|| format!("Failed to write {}", webp_path.display()))?;

    Ok(())
}

pub fn convert_to_webp(
    pages: &[(String, String)],
    png_dir: &Path,
    webp_dir: &Path,
    quality: f32,
    sequential: bool,
) -> anyhow::Result<()> {
    fs::create_dir_all(webp_dir)
        .with_context(|| format!("Failed to create {}", webp_dir.display()))?;

    if sequential {
        pages.iter().try_for_each(|(_, page_name)| {
            convert_page(page_name, png_dir, webp_dir, quality)
        })
    } else {
        pages.par_iter().try_for_each(|(_, page_name)| {
            convert_page(page_name, png_dir, webp_dir, quality)
        })
    }
}
