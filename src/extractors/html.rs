// SPDX-FileCopyrightText: 2022 Robin Vobruba <hoijui.quaero@gmail.com>
// SPDX-FileCopyrightText: 2020 Armin Becher <becherarmin@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::anchor::{self, Anchor};
use crate::config::Config;
use crate::link::{self, Link, Position};
use crate::markup::File;

pub struct LinkExtractor();

#[derive(Clone, Copy, Debug)]
enum ParserState {
    Text,
    Comment,
    Element,
    EqualSign,
    Attribute,
}

#[derive(Clone, Copy, Debug)]
enum Attribute {
    Href,
    Name,
    Id,
}

impl super::LinkExtractor for LinkExtractor {
    fn find_links_and_anchors(
        &self,
        file: &File,
        conf: &Config,
    ) -> std::io::Result<(Vec<Link>, Vec<Anchor>)> {
        let mut links: Vec<Link> = Vec::new();
        let mut anchors: Vec<Anchor> = Vec::new();
        let mut attribute: Option<Attribute> = None;
        let mut state: ParserState = ParserState::Text;
        let mut is_anchor = false;
        // let mut element_part: Option<Attribute>;
        for (line, line_str) in file.content.fetch()?.as_ref().lines().enumerate() {
            let line_chars: Vec<char> = line_str.chars().collect();
            let mut column: usize = 0;
            while line_chars.get(column).is_some() {
                match state {
                    ParserState::Comment => {
                        if line_chars.get(column) == Some(&'-')
                            && line_chars.get(column + 1) == Some(&'-')
                            && line_chars.get(column + 2) == Some(&'>')
                        {
                            column += 2;
                            state = ParserState::Text;
                        }
                    }
                    ParserState::Text => {
                        if line_chars.get(column) == Some(&'<')
                            && line_chars.get(column + 1) == Some(&'!')
                            && line_chars.get(column + 2) == Some(&'-')
                            && line_chars.get(column + 3) == Some(&'-')
                        {
                            column += 3;
                            state = ParserState::Comment;
                        } else if line_chars.get(column) == Some(&'<')
                            && line_chars.get(column + 1) == Some(&'a')
                        {
                            column += 1;
                            state = ParserState::Element;
                            is_anchor = true;
                        } else if line_chars.get(column) == Some(&'<')
                            && line_chars.get(column + 1) != Some(&'a')
                        {
                            column += 1;
                            while line_chars.get(column).is_some()
                                && !line_chars[column].is_whitespace()
                            {
                                column += 1;
                            }
                            state = ParserState::Element;
                            is_anchor = false;
                        }
                    }
                    ParserState::Element => {
                        if is_anchor {
                            if conf.links
                                && line_chars.get(column) == Some(&'h')
                                && line_chars.get(column + 1) == Some(&'r')
                                && line_chars.get(column + 2) == Some(&'e')
                                && line_chars.get(column + 3) == Some(&'f')
                            {
                                column += 3;
                                state = ParserState::EqualSign;
                                attribute = Some(Attribute::Href);
                            } else if conf.anchors
                                && line_chars.get(column) == Some(&'n')
                                && line_chars.get(column + 1) == Some(&'a')
                                && line_chars.get(column + 2) == Some(&'m')
                                && line_chars.get(column + 3) == Some(&'e')
                            {
                                column += 3;
                                state = ParserState::EqualSign;
                                attribute = Some(Attribute::Name);
                            }
                        }
                        if conf.anchors
                            && line_chars.get(column) == Some(&'i')
                            && line_chars.get(column + 1) == Some(&'d')
                        {
                            column += 1;
                            state = ParserState::EqualSign;
                            attribute = Some(Attribute::Id);
                        }
                    }
                    ParserState::EqualSign => {
                        match line_chars.get(column) {
                            Some(x) if x.is_whitespace() => {}
                            Some(x) if x == &'=' => state = ParserState::Attribute,
                            Some(_) => state = ParserState::Element,
                            None => {}
                        };
                    }
                    ParserState::Attribute => match attribute {
                        Some(attrib_cont) => {
                            match line_chars.get(column) {
                                Some(x) if !x.is_whitespace() && x != &'"' => {
                                    let attrib_column = column;
                                    while line_chars.get(column).is_some()
                                        && !line_chars[column].is_whitespace()
                                        && line_chars[column] != '"'
                                    {
                                        column += 1;
                                    }
                                    while let Some(c) = line_chars.get(column) {
                                        if c.is_whitespace() || c == &'"' {
                                            break;
                                        }
                                        column += 1;
                                    }
                                    let attrib_target = &(&line_chars[attrib_column..column])
                                        .iter()
                                        .collect::<String>();
                                    let pos = Position {
                                        line: line + 1,
                                        column: attrib_column + 1,
                                    } + &file.start;
                                    match attrib_cont {
                                        Attribute::Href => links.push(Link::new(
                                            file.locator.clone(),
                                            pos,
                                            attrib_target,
                                        )),
                                        Attribute::Name => anchors.push(Anchor {
                                            source: link::Locator {
                                                file: file.locator.clone(),
                                                pos,
                                            },
                                            name: attrib_target.clone(),
                                            r#type: anchor::Type::Direct,
                                        }),
                                        Attribute::Id => anchors.push(Anchor {
                                            source: link::Locator {
                                                file: file.locator.clone(),
                                                pos,
                                            },
                                            name: attrib_target.clone(),
                                            r#type: anchor::Type::ElementId,
                                        }),
                                    }
                                    state = ParserState::Text;
                                }
                                Some(_) | None => {}
                            };
                        }
                        None => {}
                    },
                }
                column += 1;
            }
        }
        Ok((links, anchors))
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        anchor,
        link::{self, FileLoc},
        markup::Type,
    };

    use super::*;
    use ntest::test_case;

    fn find_links(content: &str) -> std::io::Result<Vec<Link>> {
        let conf = Config::default();
        let markup_file = File::dummy(content, Type::Html);
        super::super::find_links(&markup_file, &conf).map(|(links, _anchors)| links)
    }

    fn find_anchors(content: &str) -> std::io::Result<Vec<Anchor>> {
        let conf = Config {
            links: false,
            anchors: true,
            ..Config::default()
        };
        let markup_file = File::dummy(content, Type::Html);
        super::super::find_links(&markup_file, &conf).map(|(_links, anchors)| anchors)
    }

    #[test]
    fn no_link() -> std::io::Result<()> {
        let input = "]This is not a <has> no link <h1>Bla</h1> attribute.";
        let result = find_links(input)?;
        assert!(result.is_empty());
        Ok(())
    }

    #[test]
    fn commented() -> std::io::Result<()> {
        let input = "df <!-- <a href=\"http://wiki.selfhtml.org\"> haha</a> -->";
        let result = find_links(input)?;
        assert!(result.is_empty());
        Ok(())
    }

    #[test_case(
        "<a href=\"https://www.w3schools.com\">Visit W3Schools.com!</a>",
        1,
        10
    )]
    #[test_case(
        "<a\nhref\n=\n  \"https://www.w3schools.com\">\nVisit W3Schools.com!\n</a>",
        4,
        4
    )]
    #[test_case(
        "<a hreflang=\"en\" href=\"https://www.w3schools.com\">Visit W3Schools.com!</a>",
        1,
        24
    )]
    #[test_case(
        "<!--comment--><a href=\"https://www.w3schools.com\">Visit W3Schools.com!</a>",
        1,
        24
    )]
    fn links(input: &str, line: usize, column: usize) {
        let result = find_links(input).expect("No error");
        let expected = Link::new(
            FileLoc::dummy(),
            Position { line, column },
            "https://www.w3schools.com",
        );
        assert_eq!(vec![expected], result);
    }

    #[test_case(
        r#"<!--comment--><a href="https://www.w3schools.com" name="the_anchor">Visit W3Schools.com!</a>"#,
        true,
        1,
        57,
    )]
    #[test_case(
        r#"<!--comment--><a href="https://www.w3schools.com" id="the_anchor">Visit W3Schools.com!</a>"#,
        false,
        1,
        55,
    )]
    #[test_case(
        r#"<!--comment--><a name="the_anchor">Visit W3Schools.com!</a>"#,
        true,
        1,
        24
    )]
    #[test_case(
        r#"<!--comment--><table id="the_anchor">Visit W3Schools.com!</a>"#,
        false,
        1,
        26
    )]
    fn anchors(input: &str, direct: bool, line: usize, column: usize) {
        let result = find_anchors(input).expect("No error");
        let expected = Anchor {
            source: link::Locator {
                file: FileLoc::dummy(),
                pos: Position { line, column },
            },
            name: "the_anchor".to_owned(),
            r#type: if direct {
                anchor::Type::Direct
            } else {
                anchor::Type::ElementId
            },
        };
        assert_eq!(vec![expected], result);
    }

    #[test_case(r#"<!--comment--><table idid="the_anchor">Visit W3Schools.com!</a>"#)]
    #[test_case(r#"<!--comment--><a namename="the_anchor">Visit W3Schools.com!</a>"#)]
    fn no_anchors(input: &str) {
        let result = find_anchors(input).expect("No error");
        assert_eq!(Vec::<Anchor>::new(), result);
    }
}
