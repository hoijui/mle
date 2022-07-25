use super::html_link_extractor::HtmlLinkExtractor;
use super::markdown_link_extractor::MarkdownLinkExtractor;
use crate::link::{Link, MarkupAnchorTarget, MarkupAnchorType};
use crate::markup::{MarkupFile, MarkupType};

pub fn remove_anchor(link: &mut String) -> Option<String> {
    match link.find('#') {
        Some(anchor_pos) => {
            // let anchor = link.rsplit(pat: P)(suffix: P)(new_len: usize)
            let anchor: String = link.drain(anchor_pos..).skip(1).collect();
            // link.truncate(anchor_pos);
            Some(anchor)
        }
        None => None,
    }
}

/// Finds links (and optionally anchors),
/// using the markup file specific link extractor internally.
///
/// # Errors
///
/// If fetching the markup file content failed.
pub fn find_links(
    file: &MarkupFile,
    anchors_only: bool,
) -> std::io::Result<(Vec<Link>, Vec<MarkupAnchorTarget>)> {
    let link_extractor = link_extractor_factory(file.markup_type);

    info!(
        "Scannig file at location '{:#?}' for links ...",
        file.locator
    );
    link_extractor.find_links_and_anchors(file, anchors_only)
    // match file.content.fetch() {
    //     Ok(text) => {
    //         // let (mut links, anchor_targets) =
    //             link_extractor.find_links_and_anchors(&file, anchors_only)//;
    //         // for l in &mut links {
    //         //     l.source = file.locator.to_string();
    //         //     l.anchor = remove_anchor(&mut l.target);
    //         //     //println!("XXX {:?}", l);
    //         // }
    //         // (links, anchor_targets)
    //     }
    //     Err(e) => {
    //         warn!(
    //             "File '{:#?}'. IO Error: '{}'. Check your file encoding.",
    //             file.locator, e
    //         );
    //         // (vec![], vec![])
    //         Err(...)
    //     }
    // }
}

fn link_extractor_factory(markup_type: MarkupType) -> Box<dyn LinkExtractor> {
    match markup_type {
        MarkupType::Markdown => Box::new(MarkdownLinkExtractor()),
        MarkupType::Html => Box::new(HtmlLinkExtractor()),
    }
}

pub trait LinkExtractor {
    /// Finds links (and optionally anchors),
    /// using the markup file specific link extractor internally.
    ///
    /// # Errors
    /// If fetching the markup file content failed.
    fn find_links_and_anchors(
        &self,
        // text: &str,
        file: &MarkupFile,
        anchors_only: bool,
    ) -> std::io::Result<(Vec<Link>, Vec<MarkupAnchorTarget>)>;

    /// Finds links only, using the markup file specific link extractor internally.
    ///
    /// # Errors
    /// If fetching the markup file content failed.
    fn find_links(&self, file: &MarkupFile) -> std::io::Result<Vec<Link>> {
        let (result, _) = self.find_links_and_anchors(file, true)?;
        Ok(result)
    }
}
