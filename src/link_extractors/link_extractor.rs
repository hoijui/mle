use super::html_link_extractor::HtmlLinkExtractor;
use super::markdown_link_extractor::MarkdownLinkExtractor;
use crate::markup::{MarkupFile, MarkupType};
use crate::types::{MarkupLink, MarkupAnchorType, MarkupAnchorTarget};

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

pub fn find_links(
    file: &MarkupFile,
    anchors_only: bool,
) -> (Vec<MarkupLink>, Vec<MarkupAnchorTarget>) {
    let link_extractor = link_extractor_factory(file.markup_type);

    info!("Scannig file at location '{}' for links ...", file.locator);
    match file.content.fetch() {
        Ok(text) => {
            let (mut links, anchor_targets) =
                link_extractor.find_links_and_anchors(&text, anchors_only);
            for l in &mut links {
                l.source = file.locator.to_string();
                l.anchor = remove_anchor(&mut l.target);
                //println!("XXX {:?}", l);
            }
            (links, anchor_targets)
        }
        Err(e) => {
            warn!(
                "File '{}'. IO Error: '{}'. Check your file encoding.",
                file.locator, e
            );
            (vec![], vec![])
        }
    }
}

fn link_extractor_factory(markup_type: MarkupType) -> Box<dyn LinkExtractor> {
    match markup_type {
        MarkupType::Markdown => Box::new(MarkdownLinkExtractor()),
        MarkupType::Html => Box::new(HtmlLinkExtractor()),
    }
}

pub trait LinkExtractor {
    fn find_links_and_anchors(
        &self,
        text: &str,
        anchors_only: bool,
    ) -> (Vec<MarkupLink>, Vec<MarkupAnchorTarget>);

    fn find_links(&self, text: &str) -> Vec<MarkupLink> {
        let (result, _) = self.find_links_and_anchors(text, true);
        result
    }
}
