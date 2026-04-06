use anyhow::{Context, Result};
use scraper::{Html, Selector};

const WITTGENSTEIN_SOURCE: &str = "http://www.wittgensteinsource.org";
const BERGEN_NACHLASS_EDITION: &str = "/agora_show_collection_list/1?customMenu=1";

async fn fetch_html(url: &str) -> Result<String> {
    let response = reqwest::get(url)
        .await
        .with_context(|| format!("Failed to fetch {url}"))?;
    let text = response
        .text()
        .await
        .with_context(|| format!("Failed to read response from {url}"))?;
    Ok(text)
}

/// Fetch the list of document links from wittgensteinsource.org.
/// Returns relative URLs for facsimile pages (links with text "F").
/// When `wren_library_only` is true, only includes items containing "(WL)".
pub async fn fetch_document_links(wren_library_only: bool) -> Result<Vec<String>> {
    let url = format!("{WITTGENSTEIN_SOURCE}{BERGEN_NACHLASS_EDITION}");
    let body = fetch_html(&url).await?;
    let document = Html::parse_document(&body);

    let li_selector = Selector::parse("li").unwrap();
    let a_selector = Selector::parse("a").unwrap();

    let mut links = Vec::new();
    for li in document.select(&li_selector) {
        if wren_library_only {
            let li_text = li.text().collect::<String>();
            if !li_text.contains("(WL)") {
                continue;
            }
        }
        for a in li.select(&a_selector) {
            if a.text().collect::<String>().trim() == "F"
                && let Some(href) = a.value().attr("href")
            {
                links.push(href.to_string());
            }
        }
    }

    Ok(links)
}

/// Fetch the page list for a given document URL.
/// Returns a list of (doc_name, page_name) pairs extracted from `data-title` attributes.
pub async fn fetch_pages_for_doc(relative_url: &str) -> Result<Vec<(String, String)>> {
    let url = format!("{WITTGENSTEIN_SOURCE}{relative_url}");
    let body = fetch_html(&url).await?;
    let document = Html::parse_document(&body);

    let a_selector = Selector::parse("a").unwrap();

    let mut pages = Vec::new();
    for a in document.select(&a_selector) {
        if let Some(data_title) = a.value().attr("data-title")
            && let Some((doc, page)) = data_title.split_once(',')
        {
            pages.push((doc.to_string(), page.to_string()));
        }
    }

    Ok(pages)
}

/// Build the DZI URL for a given document and page.
pub fn build_dzi_url(doc: &str, page: &str) -> String {
    let doc = if doc == "Ts-309" {
        "Ts-309-Stonborough"
    } else {
        doc
    };
    format!(
        "http://www.wittgensteinsource.org/fcgi-bin/iipsrv.fcgi\
         ?DeepZoom=/var/www/wab/web/uploads/flexip_viewer_images/iip/\
         {doc},{page}.tif.dzi"
    )
}
