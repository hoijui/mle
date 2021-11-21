use super::html_link_extractor::HtmlLinkExtractor;
use super::markdown_link_extractor::MarkdownLinkExtractor;
use crate::markup::{MarkupFile, MarkupType};
use std::fmt;

/// Link found in markup files
#[derive(PartialEq, Clone)]
pub struct MarkupLink {
    /// The source file of the link
    pub source: String,
    /// The target the link points to
    pub target: String,
    /// The target the link points to
    pub anchor: Option<String>,
    /// The line number were the link was found
    pub line: usize,
    /// The column number were the link was found
    pub column: usize,
}

impl fmt::Debug for MarkupLink {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} => {}{}{} (line {}, column {})",
            self.source,
            self.target,
            self.anchor_sep(),
            self.anchor.as_ref().unwrap_or(&String::default()),
            self.line,
            self.column
        )
    }
}

impl MarkupLink {
    fn anchor_sep(&self) -> &str {
        match &self.anchor {
            Some(_) => "#",
            None => "",
        }
    }

    /// Splitting `link` of form `"file#anchor"` into `"file"` and `Option("anchor")`.
    /// TODO tests/samples here
    pub fn split(link: &str) -> (&str, Option<String>) {
        match link.find('#') {
            Some(anchor_sep) => (
                &link[..anchor_sep],
                Some(link[(anchor_sep + 1)..].to_string()),
            ),
            None => (link, None),
        }
    }

    pub fn new_src(source: String, target: &str, line: usize, column: usize) -> MarkupLink {
        let (target, anchor) = MarkupLink::split(target);

        MarkupLink {
            source,
            target: target.to_string(),
            anchor,
            line,
            column,
        }
    }

    pub fn new(target: &str, line: usize, column: usize) -> MarkupLink {
        MarkupLink::new_src(String::new(), target, line, column)
    }
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum MarkupAnchorType {
    /// An anchor associated to a title, auto generated from the title
    TitleAuto,
    /// An anchor associated to a title, manually defined for the title
    TitleManual,
    /// A dedicated anchor, defined as such (`<a name="..."/>` or `<a id="..."/>`)
    Direct,
    /// An anchor associated to an HTML element (e.g. a div)
    ElementId,
}

/// Anchor target found in markup files
///
/// In HTML, these look like:
/// <a name="manual-anchor">target part</a>
/// <a id="manual-anchor">target part</a>
/// <p id="manual-anchor">target part</p>
/// <div id="manual-anchor">target part</div>
/// <... id="manual-anchor">target part</...>
///
/// In Markdown - in addition to the HTML form -
/// different MD flavors support different anchors:
/// * GFM supplies auto-generated anchors for headers,
///   using the following rules:
///   1. downcase the headline
///   2. remove anything that is not a letter, number, space or hyphen
///   3. change any space to a hyphen
///   so `# My 1. @#%^$^-cool header!!` will have the anchor `my-1--cool-header`
/// * Pandoc MD supports similar (but sadly not equal) auto-generated anchors,
///   or additionally manually set anchors for headers,
///   using the following syntax:
///   `# My header {#manual-anchor}`
///
#[derive(PartialEq, Clone)]
pub struct MarkupAnchorTarget {
    /// The source file of the anchor
    pub source: String,
    /// The anchor name (the thing one links to)
    pub name: String,
    /// The anchor type
    pub r#type: MarkupAnchorType,
    /// The line number were the anchor was found
    pub line: usize,
    /// The column number were the link was found
    pub column: usize,
}

impl fmt::Debug for MarkupAnchorTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}{} (line {}, column {}, type {:?})",
            self.source, self.name, self.line, self.column, self.r#type
        )
    }
}

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
