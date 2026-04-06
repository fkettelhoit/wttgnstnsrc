use std::fs::File;
use std::io::{BufWriter, Cursor};
use std::path::Path;

use anyhow::Context;
use printpdf::*;

const PPI: f32 = 150.0;

pub fn generate_pdf(
    doc_name: &str,
    pages: &[(String, String)],
    image_dir: &Path,
    output_path: &Path,
    jpeg_quality: Option<u8>,
) -> anyhow::Result<()> {
    // Collect existing image paths with their page names
    let image_pages: Vec<_> = pages
        .iter()
        .filter_map(|(_, page_name)| {
            let p = image_dir.join(format!("{page_name}.png"));
            p.exists().then_some((p, page_name.as_str()))
        })
        .collect();

    anyhow::ensure!(!image_pages.is_empty(), "No images found for {doc_name}");

    // Read first image to get dimensions for initial page
    let first_img = ::image::open(&image_pages[0].0)
        .with_context(|| format!("Failed to open {}", image_pages[0].0.display()))?;
    let (w, h) = ::image::GenericImageView::dimensions(&first_img);
    let width_mm = Mm(w as f32 / PPI * 25.4);
    let height_mm = Mm(h as f32 / PPI * 25.4);

    let (doc, first_page_idx, first_layer_idx) =
        PdfDocument::new(doc_name, width_mm, height_mm, "");
    let font = doc.add_builtin_font(BuiltinFont::Helvetica)?;

    // Embed first image
    embed_image(
        &doc,
        first_page_idx,
        first_layer_idx,
        first_img,
        w,
        h,
        jpeg_quality,
    );
    add_page_label(&doc, first_page_idx, first_layer_idx, doc_name, image_pages[0].1, width_mm, w, &font);
    doc.add_bookmark(image_pages[0].1, first_page_idx);

    // Process remaining images
    for &(ref img_path, page_name) in &image_pages[1..] {
        let img = ::image::open(img_path)
            .with_context(|| format!("Failed to open {}", img_path.display()))?;
        let (w, h) = ::image::GenericImageView::dimensions(&img);
        let width_mm = Mm(w as f32 / PPI * 25.4);
        let height_mm = Mm(h as f32 / PPI * 25.4);

        let (page_idx, layer_idx) = doc.add_page(width_mm, height_mm, "");
        embed_image(&doc, page_idx, layer_idx, img, w, h, jpeg_quality);
        add_page_label(&doc, page_idx, layer_idx, doc_name, page_name, width_mm, w, &font);
        doc.add_bookmark(page_name, page_idx);
    }

    let mut file = BufWriter::new(
        File::create(output_path)
            .with_context(|| format!("Failed to create {}", output_path.display()))?,
    );
    doc.save(&mut file)
        .with_context(|| format!("Failed to save {}", output_path.display()))?;

    Ok(())
}

fn add_page_label(
    doc: &PdfDocumentReference,
    page_idx: PdfPageIndex,
    layer_idx: PdfLayerIndex,
    doc_name: &str,
    page_name: &str,
    page_width: Mm,
    page_width_px: u32,
    font: &IndirectFontRef,
) {
    let label = format!("{doc_name},{page_name}");
    let font_size = page_width_px as f32 * 0.0075;
    // Approximate Helvetica average character width as 0.5 * font_size (in pt), converted to mm
    let char_width_mm = font_size * 0.5 * 25.4 / 72.0;
    let text_width = char_width_mm * label.len() as f32;
    let x = Mm((page_width.0 - text_width) / 2.0);
    let layer = doc.get_page(page_idx).get_layer(layer_idx);

    let padding = Mm(1.5);
    let rect_x = x - padding;
    let rect_y = Mm(2.0);
    let rect_w = Mm(text_width) + padding + padding;
    let rect_h = Mm(font_size * 25.4 / 72.0) + padding;

    layer.set_fill_color(Color::Rgb(Rgb::new(0.0, 0.0, 0.0, None)));
    layer.add_rect(Rect::new(rect_x, rect_y, rect_x + rect_w, rect_y + rect_h));

    layer.set_fill_color(Color::Rgb(Rgb::new(1.0, 1.0, 1.0, None)));
    layer.use_text(&label, font_size, x, Mm(3.0), font);
}

fn embed_image(
    doc: &PdfDocumentReference,
    page_idx: PdfPageIndex,
    layer_idx: PdfLayerIndex,
    img: ::image::DynamicImage,
    width_px: u32,
    height_px: u32,
    jpeg_quality: Option<u8>,
) {
    let rgb = img.into_rgb8();

    let (image_data, image_filter) = match jpeg_quality {
        Some(q) => {
            let mut buf = Cursor::new(Vec::new());
            let encoder = ::image::codecs::jpeg::JpegEncoder::new_with_quality(&mut buf, q);
            ::image::ImageEncoder::write_image(
                encoder,
                rgb.as_raw(),
                width_px,
                height_px,
                ::image::ExtendedColorType::Rgb8,
            )
            .expect("JPEG encoding failed");
            (buf.into_inner(), Some(ImageFilter::DCT))
        }
        None => (rgb.into_raw(), None),
    };

    let pdf_image = Image::from(ImageXObject {
        width: Px(width_px as usize),
        height: Px(height_px as usize),
        color_space: ColorSpace::Rgb,
        bits_per_component: ColorBits::Bit8,
        interpolate: true,
        image_data,
        image_filter,
        smask: None,
        clipping_bbox: None,
    });

    let layer = doc.get_page(page_idx).get_layer(layer_idx);
    pdf_image.add_to_layer(layer, ImageTransform {
        dpi: Some(PPI),
        ..Default::default()
    });
}
