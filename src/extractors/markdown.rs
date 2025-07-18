// SPDX-FileCopyrightText: 2022 Robin Vobruba <hoijui.quaero@gmail.com>
// SPDX-FileCopyrightText: 2020 Armin Becher <becherarmin@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::LazyLock;

use crate::anchor;
use crate::anchor::Anchor;
use crate::config::Config;
use crate::link::Link;
use crate::link::Locator;
use crate::link::Position;
use crate::markup;
use crate::markup::Content;
use crate::markup::File;
use pulldown_cmark::{BrokenLink, Event, Options, Parser, Tag};
use regex::Regex;

pub struct LinkExtractor();

static NON_ID_CHARS: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"[^A-Za-z0-9 -]").unwrap());
static LEADING_NUMBER: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[0-9]+").unwrap());
static CHECK_BOX_VALUES: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[ xX]?$").unwrap());

/// 1. downcase the headline
/// 2. remove anything that is not a letter, number, space or hyphen
/// 3. change any space to a hyphen
/// 4. if pandoc-style, remove leading numbers
fn generate_id(text: &str, gfm_style: bool) -> String {
    let mut id = NON_ID_CHARS
        .replace_all(&text.to_lowercase(), "")
        .replace(' ', "-");
    if !gfm_style {
        id = LEADING_NUMBER.replace(&id, "").to_string();
    }
    id
}

impl LinkExtractor {
    fn create_pos_from_idx(content: &str) -> impl Fn(usize) -> Position {
        let line_lengths: Vec<usize> = content.lines().map(str::len).collect();
        move |idx: usize| -> Position {
            let mut line = 1;
            let mut column = idx + 1;
            for line_length in &line_lengths {
                if *line_length >= column {
                    return Position { line, column };
                }
                column -= line_length + 1;
                line += 1;
            }
            Position { line, column }
        }
    }
}

impl super::LinkExtractor for LinkExtractor {
    async fn find_links_and_anchors(
        &self,
        file: &File<'_>,
        conf: &Config,
    ) -> std::io::Result<super::ParseRes> {
        let html_le = super::html::LinkExtractor();

        // let line_lengths: Vec<usize> = file.content.fetch()?.lines().map(str::len).collect();
        let pos_from_idx = Self::create_pos_from_idx(file.content.fetch().await?.as_ref());

        let callback = &mut |broken_link: BrokenLink| {
            let refrnc = broken_link.reference.as_ref();
            if CHECK_BOX_VALUES.is_match(refrnc) {
                // As we will not be able to get everyone to use the correct:
                //
                // ```
                // - \[x\] bli`
                // - \[ \] bla
                // ```
                //
                // instead of:
                //
                // ```
                // - [x] bli`
                // - [ ] bla
                // ```
                //
                // Both because it is much uglier
                // and because not all renderers support it,
                // let's ignore the later,
                // if they appear here as invalid reference links.
                log::debug!("Broken reference link detected for link reference: {refrnc:#?}");
                return None;
                // let pos = pos_from_idx(broken_link.span.start) + &file.start;
                // let annotated_target = format!("MISSING_REF-{refrnc}");
                // let bad_link = Link::new(file.locator.clone(), pos, &annotated_target);
                // log::warn!("Bad reference link: {bad_link}");
                // return Some((
                //     CowStr::from(annotated_target),
                //     CowStr::from(refrnc.to_owned()),
                // ));
            }
            // TODO Add bad_link to a list and return that list from this function for more flexibility on the library users side.
            None
        };

        let text = file.content.fetch().await?;
        let parser = Parser::new_with_broken_link_callback(
            &text,
            Options::ENABLE_HEADING_ATTRIBUTES,
            Some(callback),
        );

        let mut links: Vec<Link> = Vec::new();
        let mut anchors: Vec<Anchor> = Vec::new();
        let mut gathering_for_header = false;
        let mut header_content: Vec<String> = Vec::new();
        for (evt, range) in parser.into_offset_iter() {
            match evt {
                Event::Start(Tag::Heading {level: _, id, classes: _, attrs: _})
                if conf.extract_anchors() && id.is_none() => {
                    // if let Tag::Heading(_level, id, _classes) = tag {
                    gathering_for_header = true;
                    // }
                    // match tag {
                    //     Tag::Heading(_level, id, _classes) => {
                    //         if gathering_anchors && id.is_none() {
                    //             gathering_for_header = true;
                    //         }
                    //     },
                    //     _ => (),
                    // }
                }
                Event::Start(tag) => {
                    match tag {
                        Tag::Link {link_type: _, dest_url, title: _, id: _}
                        | Tag::Image {link_type: _, dest_url, title: _, id: _}
                        if conf.extract_links() => {
                            let pos = pos_from_idx(range.start) + &file.start;
                            links.push(Link::new(
                                file.locator.clone(),
                                pos,
                                &dest_url,
                            ));
                        }
                        Tag::Heading {level: _, id, classes: _, attrs: _}
                        if conf.extract_anchors() => {
                            let pos = pos_from_idx(range.start) + &file.start;
                            let source = Locator {
                                file: file.locator.clone(),
                                pos,
                            };
                            let r#type : anchor::Type;
                            let id_str : String = if let Some(id_cont) = id {
                                    r#type = anchor::Type::TitleManual;
                                    // eprint!("XXX Title with manual id!: '{}'\n", id_cont);
                                    id_cont.to_string()
                                } else {
                                    r#type = anchor::Type::TitleAuto;
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
                                    /*let id = */generate_id(&header_text, true)//; // TODO need config setting for GFM vs Pandoc rules
                                    // eprint!(fmt"XXX Title with auto id!: '{header_text}'\n");
                                    // eprint!("XXX Title with auto id!: '{}'\n", id);
                                    // id
                            };
                            header_content.clear();
                            anchors.push(Anchor {
                                source,
                                name: id_str,
                                r#type,
                            });
                        }
                        _ => (),
                    }
                }
                Event::Html(content) | Event::InlineHtml(content) /* TODO FALL_THROUGH_TO_NEXT_THREE, OR ... (see TODO below) */ => {
                    let cur_pos = pos_from_idx(range.start) + &file.start - Position { line: 1, column: 0 };
                    let sub_markup = File {
                        markup_type: markup::Type::Html,
                        locator: file.locator.clone(),
                        content: Content::InMemory(content.as_ref()),
                        start: cur_pos,
                    };
                    let mut sub_parsed = html_le.find_links_and_anchors(&sub_markup, conf).await?;
                    if conf.extract_links() {
                        links.append(&mut sub_parsed.links);
                    }
                    if conf.extract_anchors() {
                        anchors.append(&mut sub_parsed.anchors);
                    }

                    if gathering_for_header { // TODO ... OR_THIS (see TODO above)
                        header_content.push(content.into_string());
                    }
                }
                Event::Text(content)
                | Event::Code(content)
                | Event::FootnoteReference(content) => {
                    if gathering_for_header {
                        header_content.push(content.into_string());
                    }
                }
                _ => (),
            }
        }
        Ok(super::ParseRes { links, anchors })
    }
}

#[cfg(test)]
mod tests {
    use crate::link::{FileLoc, Target};

    use super::*;
    use ntest::test_case;
    use url::Url;

    macro_rules! aw_through_engine {
        ($e:expr) => {
            tokio_test::block_on($e)
        };
    }

    async fn find_links(content: &str) -> Vec<Link> {
        let markup_file = File::dummy(content, markup::Type::Markdown);
        let conf = Config::default();
        super::super::find_links(&markup_file, &conf)
            .await
            .map(|parsed| parsed.links)
            .expect("No error")
    }

    fn link_new_http_no_anchor(url: &str, line: usize, column: usize) -> Link {
        Link {
            source: Locator {
                file: FileLoc::dummy(),
                pos: Position { line, column },
            },
            target: Target::Http(Url::parse(url).expect("Test specified non-valid HTTP(S) URL")),
        }
    }

    #[tokio::test]
    async fn inline_no_link() {
        let input = "]This is not a () link](! has no title attribute.";
        let result = find_links(input).await;
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn commented_link() {
        let input = "]This is not a () <!--[link](link)-->.";
        let result = find_links(input).await;
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn nested_links() {
        let input =
            "\n\r\t\n[![](http://meritbadge.herokuapp.com/mle)](https://crates.io/crates/mle)";
        let result = find_links(input).await;
        let img = link_new_http_no_anchor("http://meritbadge.herokuapp.com/mle", 3, 2);
        let link = link_new_http_no_anchor("https://crates.io/crates/mle", 3, 1);
        assert_eq!(vec![link, img], result);
    }

    #[tokio::test]
    async fn link_escaped() {
        let input = "This is not a \\[link\\](random_link).";
        let result = find_links(input).await;
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn link_in_headline() {
        let input = "  # This is a [link](http://example.net/).";
        let result = find_links(input).await;
        assert_eq!(result[0].source.pos.column, 15);
    }

    #[tokio::test]
    async fn link_with_newline() {
        let input = "This is a [link](\nhttp://example.net/)";
        let result = find_links(input).await;
        assert_eq!(result[0].source.pos.column, 11);
        assert_eq!(result[0].target.to_string(), "http://example.net/");
    }

    #[tokio::test]
    async fn link_relative_with_newline_and_space_wrong() {
        let input = "[link](doc/spaced name.md)";
        let result = find_links(input).await;
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn link_relative_with_newline_and_space() {
        let input = "[link](<doc/spaced name.md>)";
        let result = find_links(input).await;
        assert_eq!(result[0].source.pos.column, 1);
        assert_eq!(result[0].target.to_string(), "doc/spaced name.md");
    }

    #[tokio::test]
    async fn link_relative_with_newline() {
        let input = "Download the file following the [assembly guide](
../../doc/assembly/Production_Guide.md)";
        let result = find_links(input).await;
        assert_eq!(result[0].source.pos.line, 1);
        assert_eq!(result[0].source.pos.column, 33);
        assert_eq!(
            result[0].target.to_string(),
            "../../doc/assembly/Production_Guide.md"
        );
    }

    #[tokio::test]
    async fn link_target_on_new_line() {
        let input = "bla [Solar Pura](\nhttp://example.net/) work,";
        let result = find_links(input).await;
        assert_eq!(result[0].source.pos.column, 5);
        assert_eq!(result[0].target.to_string(), "http://example.net/");
    }

    #[tokio::test]
    async fn no_link_colon() {
        let input = "This is not a [link]:bla.";
        let result = find_links(input).await;
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn inline_code() {
        let input = " `[code](http://example.net/)`, no link!.";
        let result = find_links(input).await;
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn link_near_inline_code() {
        let input = " `bug` [code](http://example.net/), link!.";
        let result = find_links(input).await;
        let expected = link_new_http_no_anchor("http://example.net/", 1, 8);
        assert_eq!(vec![expected], result);
    }

    #[tokio::test]
    async fn link_very_near_inline_code() {
        let input = "`bug`[code](http://example.net/)";
        let result = find_links(input).await;
        let expected = link_new_http_no_anchor("http://example.net/", 1, 6);
        assert_eq!(vec![expected], result);
    }

    #[tokio::test]
    async fn code_block() {
        let input = " ``` js\n[code](http://example.net/)```, no link!.";
        let result = find_links(input).await;
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn html_code_block() {
        let input = "<script>\n[code](http://example.net/)</script>, no link!.";
        let result = find_links(input).await;
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn escaped_code_block() {
        let input = "   klsdjf \\`[escape](http://example.net/)\\`, no link!.";
        let result = find_links(input).await;
        let expected = link_new_http_no_anchor("http://example.net/", 1, 13);
        assert_eq!(vec![expected], result);
    }

    #[tokio::test]
    async fn link_in_code_block() {
        let input = "```\n[only code](http://example.net/)\n```.";
        let result = find_links(input).await;
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn image_reference() {
        let link_str = "http://example.net/";
        let input = &format!("\n\nBla ![This is an image link]({link_str})");
        let result = find_links(input).await;
        let expected = link_new_http_no_anchor(link_str, 3, 5);
        assert_eq!(vec![expected], result);
    }

    #[tokio::test]
    async fn link_no_title() {
        let link_str = "http://example.net/";
        let input = &format!("[This link]({link_str}) has no title attribute.");
        let result = find_links(input).await;
        let expected = link_new_http_no_anchor(link_str, 1, 1);
        assert_eq!(vec![expected], result);
    }

    #[tokio::test]
    async fn link_with_title() {
        let link_str = "http://example.net/";
        let input = &format!("\n123[This is a link]({link_str} \"with title\") oh yea.");
        let result = find_links(input).await;
        let expected = link_new_http_no_anchor(link_str, 2, 4);
        assert_eq!(vec![expected], result);
    }

    // #[tokio::test]
    #[test_case("<http://example.net/>", 1)]
    #[test_case("This is a short link <http://example.net/>", 22)]
    fn inline_link(input: &str, column: usize) {
        let result = aw_through_engine!(find_links(input));
        let expected = link_new_http_no_anchor("http://example.net/", 1, column);
        assert_eq!(vec![expected], result);
    }

    // /// TODO GitHub link style support
    // #[test_case("http://example.net/", 1)]
    // #[test_case("This is a short link http://example.net/", 22)]
    // #[test_case("This is a short link https://example.net/", 22)]
    // fn inline_link_github(input: &str, column: usize) {
    //     let result = find_links(input).await;
    //     let expected = link_new_http_no_anchor("http://example.net/", 1, column);
    //     assert_eq!(vec![expected], result);
    // }

    #[test_case(
        "<a href=\"http://example.net/\"> target=\"_blank\">Visit W3Schools!</a>",
        test_name = "html_link_with_target"
    )]
    #[test_case(
        "<a href=\"http://example.net/\"> link text</a>",
        test_name = "html_link_no_target"
    )]
    fn html_link(input: &str) {
        let result = aw_through_engine!(find_links(input));
        let expected = link_new_http_no_anchor("http://example.net/", 1, 11);
        assert_eq!(vec![expected], result);
    }

    #[tokio::test]
    async fn html_link_ident() {
        let input = "123<a href=\"http://example.net/\"> link text</a>";
        let result = find_links(input).await;
        let expected = link_new_http_no_anchor("http://example.net/", 1, 14);
        assert_eq!(vec![expected], result);
    }

    #[tokio::test]
    async fn html_link_new_http_no_anchor_line() {
        let input = "\n123<a href=\"http://example.net/\"> link text</a>";
        let result = find_links(input).await;
        let expected = link_new_http_no_anchor("http://example.net/", 2, 14);
        assert_eq!(vec![expected], result);
    }

    #[tokio::test]
    async fn raw_html_issue_31() {
        let input = "Some text <a href=\"http://example.net/\">link text</a> more text.";
        let result = find_links(input).await;
        let expected = link_new_http_no_anchor("http://example.net/", 1, 21);
        assert_eq!(vec![expected], result);
    }

    #[tokio::test]
    async fn referenced_link() {
        let link_str = "http://example.net/";
        let input =
            &format!("This is [an example][myref] reference-style link.\n\n[myref]: {link_str}");
        let result = find_links(input).await;
        let expected = link_new_http_no_anchor(link_str, 1, 9);
        assert_eq!(vec![expected], result);
    }

    #[tokio::test]
    async fn referenced_link_with_spaces() {
        let link_str = "http://example.net/";
        let input = &format!(
            "This is [an example][space containing reference text] reference-style link.\n\n[space containing reference text]: {link_str}"
        );
        let result = find_links(input).await;
        let expected = link_new_http_no_anchor(link_str, 1, 9);
        assert_eq!(vec![expected], result);
    }

    #[tokio::test]
    async fn referenced_link_case_insensitive() {
        let link_str = "http://example.net/";
        let input = &format!(
            "This is [an example][case-insensitive reference text] reference-style link.\n\n[CASE-insensitive Reference Text]: {link_str}"
        );
        let result = find_links(input).await;
        let expected = link_new_http_no_anchor(link_str, 1, 9);
        assert_eq!(vec![expected], result);
    }

    #[tokio::test]
    async fn referenced_link_tag_only() {
        let link_str = "http://example.net/";
        let input = &format!("Foo Bar\n\n[tag-without-reference]: {link_str}");
        let result = find_links(input).await;
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn referenced_link_no_tag_only_reference() {
        let input = "[link][reference]";
        let result = find_links(input).await;
        assert!(result.is_empty());
        // TODO: Check broken links
    }

    #[tokio::test]
    async fn checkboxes() {
        let input = "
- [ ] unchecked
- [x] checked lower
- [X] checked upper

* [ ] unchecked
* [x] checked lower
* [X] checked upper

1. [ ] unchecked
2. [x] checked lower
3. [X] checked upper

1. [ ] unchecked
1. [x] checked lower
1. [X] checked upper

a) [ ] unchecked
b) [x] checked lower
c) [X] checked upper

[ ] unchecked
[x] checked lower
[X] checked upper
";
        let result = find_links(input).await;
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn checkboxes_with_tags() {
        let input = "
- [ ] unchecked
- [x] checked lower
- [X] checked upper

* [ ] unchecked
* [x] checked lower
* [X] checked upper

1. [ ] unchecked
2. [x] checked lower
3. [X] checked upper

1. [ ] unchecked
1. [x] checked lower
1. [X] checked upper

a) [ ] unchecked
b) [x] checked lower
c) [X] checked upper

[ ] unchecked
[x] checked lower
[X] checked upper

# Hack

Define referenceable link tags,
so the above would be valid links,
if (wrongly) detected as such.

[ ]: unchecked
[x]: checked-lower
[X]: checked-upper
";
        let result = find_links(input).await;
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn checkboxes_quoted() {
        let input = r"
- \[ \] unchecked
- \[x\] checked lower
- \[X\] checked upper

* \[ \] unchecked
* \[x\] checked lower
* \[X\] checked upper

1. \[ \] unchecked
2. \[x\] checked lower
3. \[X\] checked upper

1. \[ \] unchecked
1. \[x\] checked lower
1. \[X\] checked upper

a) \[ \] unchecked
b) \[x\] checked lower
c) \[X\] checked upper

\[ \] unchecked
\[x\] checked lower
\[X\] checked upper

# Hack

Define referencable link tags,
so the above woudl be valid links,
if (wrongly) detected as such.

[ ]: unchecked
[x]: checked-lower
[X]: checked-upper
";
        let result = find_links(input).await;
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn non_checkboxes() {
        let input = "
- [ ](a)
- [x](b)
- [X]()

* [ ](a)
* [x](b)
* [X](c)

1. [ ](a)
2. [x](b)
3. [X](c)

1. [ ](a)
1. [x](b)
1. [X](c)

a) [ ](a)
b) [x](b)
c) [X](c)

[ ](a)
[x](b)
[X](c)
";
        let result = find_links(input).await;
        assert_eq!(result.len(), 18);
    }
}
