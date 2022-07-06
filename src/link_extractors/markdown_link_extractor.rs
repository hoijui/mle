use super::html_link_extractor::HtmlLinkExtractor;
use crate::link_extractors::link_extractor::LinkExtractor;
use crate::types::MarkupAnchorTarget;
use crate::types::MarkupAnchorType;
use crate::types::MarkupLink;
use lazy_static::lazy_static;
use pulldown_cmark::{BrokenLink, Event, Options, Parser, Tag};
use regex::Regex;

pub struct MarkdownLinkExtractor();

lazy_static! {
    static ref NON_ID_CHARS: Regex = Regex::new(r"[^A-Za-z0-9 -]").unwrap();
    static ref LEADING_NUMBER: Regex = Regex::new(r"^[0-9]+").unwrap();
}

/// 1. downcase the headline
/// 2. remove anything that is not a letter, number, space or hyphen
/// 3. change any space to a hyphen
/// 4. if pandoc-style, remove leading numbers
fn generate_id(text: &str, gfm_style: bool) -> String {
    let mut id = NON_ID_CHARS
        .replace_all(&text.to_lowercase(), "")
        .replace(" ", "-");
    if !gfm_style {
        id = LEADING_NUMBER.replace(&id, "").to_string();
    }
    id
}

impl LinkExtractor for MarkdownLinkExtractor {
    fn find_links_and_anchors(
        &self,
        text: &str,
        anchors_only: bool,
    ) -> (Vec<MarkupLink>, Vec<MarkupAnchorTarget>) {
        let html_extractor = HtmlLinkExtractor();

        // Setup callback that sets the URL and title when it encounters
        // a reference to our home page.
        let callback = &mut |broken_link: BrokenLink| {
            warn!("Broken reference link: {:?}", broken_link.reference);
            //TODO: Return error state
            None
        };

        let parser = Parser::new_with_broken_link_callback(
            text,
            Options::ENABLE_HEADING_ATTRIBUTES,
            Some(callback),
        );

        let line_lengths: Vec<usize> = text.lines().map(str::len).collect();
        let line_column_from_idx = |idx: usize| -> (usize, usize) {
            let mut line = 1;
            let mut column = idx + 1;
            for line_length in &line_lengths {
                if *line_length >= column {
                    return (line, column);
                }
                column -= line_length + 1;
                line += 1;
            }
            (line, column)
        };

        let mut links: Vec<MarkupLink> = Vec::new();
        let mut anchors: Vec<MarkupAnchorTarget> = Vec::new();
        let gathering_anchors = true; // TODO Configuration setting needed for this
        let mut gathering_for_header = false;
        let mut header_content: Vec<String> = Vec::new();
        for (evt, range) in parser.into_offset_iter() {
            match evt {
                Event::Start(tag) => {
                    match tag {
                        Tag::Heading(_level, id, _classes) => {
                            if gathering_anchors && id.is_none() {
                                gathering_for_header = true;
                            }
                        },
                        _ => (),
                    }
                }
                Event::End(tag) => {
                    match tag {
                        Tag::Link(_link_type, destination, _title)
                        | Tag::Image(_link_type, destination, _title) => {
                            let line_col = line_column_from_idx(range.start);
                            links.push(MarkupLink::new(
                                &destination,
                                line_col.0,
                                line_col.1,
                            ))
                        }
                        Tag::Heading(_level, id, _classes) => {
                            let line_col = line_column_from_idx(range.start);
                            let r#type : MarkupAnchorType;
                            let id_str : String = match id {
                                Some(id_cont) => {
                                    r#type = MarkupAnchorType::TitleManual;
                                    // eprint!("XXX Title with manual id!: '{}'\n", id_cont);
                                    id_cont.to_owned()
                                },
                                None => {
                                    r#type = MarkupAnchorType::TitleAuto;
                                    gathering_for_header = false;
                                    // let header_text = header_content.iter().filter(|txt| txt.is_empty()).collect::<Vec<&String>>().join(" ");
                                    // let header_text = header_content.iter()
                                    //         //.filter(|txt| !txt.is_empty())
                                    //         .fold(String::new(), |acc, s| {
                                    //             eprint!("YYY part: '{}'\n", s);
                                    //             acc + "" + s
                                    //         });
                                    //         // .collect::<Vec<_>>()
                                    //         // .join(" ");
                                    let header_text = header_content.join("");
                                    let id = generate_id(&header_text, true); // TODO need config setting for GFM vs Pandoc rules
                                    // eprint!(fmt"XXX Title with auto id!: '{header_text}'\n");
                                    // eprint!("XXX Title with auto id!: '{}'\n", id);
                                    id
                                },
                            };
                            header_content.clear();
                            anchors.push(MarkupAnchorTarget {
                                source: String::new(),
                                name: id_str,
                                r#type: r#type,
                                line: line_col.0,
                                column: line_col.1,
                            })
                        }
                        _ => (),
                    };
                }
                Event::Html(cont) /* TODO FALL_THROUGH_TO_NEXT_THREE, OR ... (see TODO below) */ => {
                    let line_col = line_column_from_idx(range.start);
                    let mut html_result = html_extractor.find_links(cont.as_ref());
                    html_result = html_result
                        .iter()
                        .map(|md_link| {
                            let line = line_col.0 + md_link.line - 1;
                            let column = if md_link.line > 1 {
                                md_link.column
                            } else {
                                line_col.1 + md_link.column - 1
                            };
                            MarkupLink::new_src(
                                md_link.source.clone(),
                                &md_link.target,
                                line,
                                column,
                            )
                        })
                        .collect();
                    links.append(&mut html_result);



                    if gathering_for_header { // TODO ... OR_THIS (see TODO above)
                        header_content.push(cont.into_string());
                    }
                }
                Event::Text(cont)
                | Event::Code(cont)
                | Event::FootnoteReference(cont) => {
                    if gathering_for_header {
                        header_content.push(cont.into_string());
                    }
                }
                _ => (),
            };
        }
        (links, anchors)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ntest::test_case;

    #[test]
    fn inline_no_link() {
        let le = MarkdownLinkExtractor();
        let input = "]This is not a () link](! has no title attribute.";
        let result = le.find_links(&input);
        assert!(result.is_empty());
    }

    #[test]
    fn commented_link() {
        let le = MarkdownLinkExtractor();
        let input = "]This is not a () <!--[link](link)-->.";
        let result = le.find_links(&input);
        assert!(result.is_empty());
    }

    #[test]
    fn nested_links() {
        let le = MarkdownLinkExtractor();
        let input =
            "\n\r\t\n[![](http://meritbadge.herokuapp.com/mle)](https://crates.io/crates/mlc)";
        let result = le.find_links(&input);
        let img = MarkupLink::new("http://meritbadge.herokuapp.com/mle", 3, 2);
        let link = MarkupLink::new("https://crates.io/crates/mle", 3, 1);
        assert_eq!(vec![img, link], result);
    }

    #[test]
    fn link_escaped() {
        let le = MarkdownLinkExtractor();
        let input = "This is not a \\[link\\](random_link).";
        let result = le.find_links(&input);
        assert!(result.is_empty());
    }

    #[test]
    fn link_in_headline() {
        let le = MarkdownLinkExtractor();
        let input = "  # This is a [link](http://example.net/).";
        let result = le.find_links(&input);
        assert_eq!(result[0].column, 15);
    }

    #[test]
    fn no_link_colon() {
        let le = MarkdownLinkExtractor();
        let input = "This is not a [link]:bla.";
        let result = le.find_links(&input);
        assert!(result.is_empty());
    }

    #[test]
    fn inline_code() {
        let le = MarkdownLinkExtractor();
        let input = " `[code](http://example.net/)`, no link!.";
        let result = le.find_links(&input);
        assert!(result.is_empty());
    }

    #[test]
    fn link_near_inline_code() {
        let le = MarkdownLinkExtractor();
        let input = " `bug` [code](http://example.net/), link!.";
        let result = le.find_links(&input);
        let expected = MarkupLink::new("http://example.net/", 1, 8);
        assert_eq!(vec![expected], result);
    }

    #[test]
    fn link_very_near_inline_code() {
        let le = MarkdownLinkExtractor();
        let input = "`bug`[code](http://example.net/)";
        let result = le.find_links(&input);
        let expected = MarkupLink::new("http://example.net/", 1, 6);
        assert_eq!(vec![expected], result);
    }

    #[test]
    fn code_block() {
        let le = MarkdownLinkExtractor();
        let input = " ``` js\n[code](http://example.net/)```, no link!.";
        let result = le.find_links(&input);
        assert!(result.is_empty());
    }

    #[test]
    fn html_code_block() {
        let le = MarkdownLinkExtractor();
        let input = "<script>\n[code](http://example.net/)</script>, no link!.";
        let result = le.find_links(&input);
        assert!(result.is_empty());
    }

    #[test]
    fn escaped_code_block() {
        let le = MarkdownLinkExtractor();
        let input = "   klsdjf \\`[escape](http://example.net/)\\`, no link!.";
        let result = le.find_links(&input);
        let expected = MarkupLink::new("http://example.net/", 1, 13);
        assert_eq!(vec![expected], result);
    }

    #[test]
    fn link_in_code_block() {
        let le = MarkdownLinkExtractor();
        let input = "```\n[only code](http://example.net/)\n```.";
        let result = le.find_links(&input);
        assert!(result.is_empty());
    }

    #[test]
    fn image_reference() {
        let le = MarkdownLinkExtractor();
        let link_str = "http://example.net/";
        let input = format!("\n\nBla ![This is an image link]({})", link_str);
        let result = le.find_links(&input);
        let expected = MarkupLink::new(link_str, 3, 5);
        assert_eq!(vec![expected], result);
    }

    #[test]
    fn link_no_title() {
        let le = MarkdownLinkExtractor();
        let link_str = "http://example.net/";
        let input = format!("[This link]({}) has no title attribute.", link_str);
        let result = le.find_links(&input);
        let expected = MarkupLink::new(link_str, 1, 1);
        assert_eq!(vec![expected], result);
    }

    #[test]
    fn link_with_title() {
        let le = MarkdownLinkExtractor();
        let link_str = "http://example.net/";
        let input = format!("\n123[This is a link]({} \"with title\") oh yea.", link_str);
        let result = le.find_links(&input);
        let expected = MarkupLink::new(link_str, 2, 4);
        assert_eq!(vec![expected], result);
    }

    #[test_case("<http://example.net/>", 1)]
    // TODO GitHub Link style support
    //#[test_case("This is a short link http://example.net/", 22)]
    //#[test_case("http://example.net/", 1)]
    #[test_case("This is a short link <http://example.net/>", 22)]
    fn inline_link(input: &str, column: usize) {
        let le = MarkdownLinkExtractor();
        let result = le.find_links(&input);
        let expected = MarkupLink::new("http://example.net/", 1, column);
        assert_eq!(vec![expected], result);
    }

    #[test_case(
        "<a href=\"http://example.net/\"> target=\"_blank\">Visit W3Schools!</a>",
        test_name = "html_link_with_target"
    )]
    #[test_case(
        "<a href=\"http://example.net/\"> link text</a>",
        test_name = "html_link_no_target"
    )]
    fn html_link(input: &str) {
        let le = MarkdownLinkExtractor();
        let result = le.find_links(&input);
        let expected = MarkupLink::new("http://example.net/", 1, 1);
        assert_eq!(vec![expected], result);
    }

    #[test]
    fn html_link_ident() {
        let le = MarkdownLinkExtractor();
        let result = le.find_links(&"123<a href=\"http://example.net/\"> link text</a>");
        let expected = MarkupLink::new("http://example.net/", 1, 4);
        assert_eq!(vec![expected], result);
    }

    #[test]
    fn html_link_new_line() {
        let le = MarkdownLinkExtractor();
        let result = le.find_links(&"\n123<a href=\"http://example.net/\"> link text</a>");
        let expected = MarkupLink::new("http://example.net/", 2, 4);
        assert_eq!(vec![expected], result);
    }

    #[test]
    fn raw_html_issue_31() {
        let le = MarkdownLinkExtractor();
        let result = le.find_links(&"Some text <a href=\"some_url\">link text</a> more text.");
        let expected = MarkupLink::new("some_url", 1, 11);
        assert_eq!(vec![expected], result);
    }

    #[test]
    fn referenced_link() {
        let le = MarkdownLinkExtractor();
        let link_str = "http://example.net/";
        let input = format!(
            "This is [an example][arbitrary case-insensitive reference text] reference-style link.\n\n[Arbitrary CASE-insensitive reference text]: {}",
            link_str
        );
        let result = le.find_links(&input);
        let expected = MarkupLink::new(link_str, 1, 9);
        assert_eq!(vec![expected], result);
    }

    #[test]
    fn referenced_link_tag_only() {
        let le = MarkdownLinkExtractor();
        let link_str = "http://example.net/";
        let input = format!(
            "Foo Bar\n\n[Arbitrary CASE-insensitive reference text]: {}",
            link_str
        );
        let result = le.find_links(&input);
        assert_eq!(0, result.len());
    }

    #[test]
    fn referenced_link_no_tag_only() {
        let le = MarkdownLinkExtractor();
        let input = "[link][reference]";
        let result = le.find_links(&input);
        assert_eq!(0, result.len());
        // TODO: Check broken links
    }
}
