use std::fs;
use std::path::Path;

use anyhow::Context;

pub fn convert_to_webp(
    pages: &[(String, String)],
    png_dir: &Path,
    webp_dir: &Path,
    quality: f32,
) -> anyhow::Result<()> {
    for (_, page_name) in pages {
        let png_path = png_dir.join(format!("{page_name}.png"));
        let webp_path = webp_dir.join(format!("{page_name}.webp"));

        if !png_path.exists() || webp_path.exists() {
            continue;
        }

        let img = ::image::open(&png_path)
            .with_context(|| format!("Failed to open {}", png_path.display()))?;
        let rgb = img.into_rgb8();
        let (w, h) = (rgb.width(), rgb.height());

        let encoder = webp::Encoder::from_rgb(rgb.as_raw(), w, h);
        let webp_data = encoder.encode(quality);

        fs::create_dir_all(webp_dir)
            .with_context(|| format!("Failed to create {}", webp_dir.display()))?;
        fs::write(&webp_path, &*webp_data)
            .with_context(|| format!("Failed to write {}", webp_path.display()))?;
    }

    Ok(())
}
