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
    Other,
}

struct Scanner<'a> {
    line: &'a str,
    chars: Vec<char>,
    column: usize,
}

impl<'a> Scanner<'a> {
    pub const fn empty() -> Self {
        Scanner {
            line: "",
            chars: vec![],
            column: 0,
        }
    }

    pub fn reset(&mut self, new_line: &'a str) {
        self.line = new_line;
        self.chars = new_line.chars().collect();
        self.column = 0;
    }

    pub fn take(&mut self, token: &str) -> bool {
        let found = self
            .line
            .get(self.column..self.column + token.len())
            .map_or(false, |slc| slc.eq(token));
        if found {
            self.column += token.len();
        }
        found
    }

    pub fn take_and_ws(&mut self, token: &str) -> bool {
        let mut found = self
            .line
            .get(self.column..self.column + token.len())
            .map_or(false, |slc| slc.eq(token));
        if found {
            found = false;
            let mut count = 0;
            while let Some(chr) = self.chars.get(self.column + token.len() + count) {
                if chr.is_whitespace() {
                    count += 1;
                } else {
                    break;
                }
            }
            if count > 0 || self.is_done() {
                self.column += token.len();
                found = true;
            }
        }
        found
    }

    pub fn take_single(&mut self, token: char) -> bool {
        let found = self
            .chars
            .get(self.column)
            .map_or(false, |slc| slc.eq(&token));
        if found {
            self.column += 1;
        }
        found
    }

    pub fn take_any(&mut self) -> Option<char> {
        let chr = self.chars.get(self.column);
        if chr.is_some() {
            self.column += 1;
        }
        chr.copied()
    }

    pub fn skip_ws(&mut self) -> bool {
        let mut count = 0;
        while let Some(chr) = self.chars.get(self.column + count) {
            if chr.is_whitespace() {
                count += 1;
            } else {
                break;
            }
        }
        self.column += count;
        count > 0
    }

    pub fn is_non_ws(&self) -> bool {
        if let Some(chr) = self.chars.get(self.column) {
            if chr.is_whitespace() {
                return false;
            }
        }
        true
    }

    pub fn take_non_ws(&mut self) -> &'a str {
        let mut count = 0;
        while let Some(chr) = self.chars.get(self.column + count) {
            if chr.is_whitespace() {
                break;
            }
            count += 1;
        }
        self.column += count;
        &self.line[self.column - count..self.column]
    }

    pub fn take_non_ws_or(&mut self, token: char) -> &'a str {
        let mut count = 0;
        while let Some(chr) = self.chars.get(self.column + count) {
            if chr.is_whitespace() || chr == &token {
                break;
            }
            count += 1;
        }
        self.column += count;
        &self.line[self.column - count..self.column]
    }

    pub fn take_non(&mut self, token: char) -> Option<&'a str> {
        let mut count = 0;
        let mut found = false;
        while let Some(chr) = self.chars.get(self.column + count) {
            if chr == &token {
                found = true;
                break;
            }
            count += 1;
        }
        if found || count > 0 {
            self.column += count;
            Some(&self.line[self.column - count..self.column])
        } else {
            None
        }
    }

    pub fn is_done(&self) -> bool {
        self.column >= self.chars.len()
    }
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
        let mut scanner = Scanner::empty();
        for (line, line_str) in file.content.fetch()?.as_ref().lines().enumerate() {
            scanner.reset(line_str);
            while !scanner.is_done() {
                match state {
                    ParserState::Comment => {
                        if scanner.take("-->") {
                            state = ParserState::Text;
                        } else {
                            scanner.take_any();
                        }
                    }
                    ParserState::Text => {
                        if scanner.take("<!--") {
                            state = ParserState::Comment;
                        } else if scanner.take_single('<') {
                            // not "<a"!
                            scanner.skip_ws();
                            let _end = scanner.take_single('/');
                            let elem = scanner.take_non_ws_or('>');
                            is_anchor = elem == "a";
                            if !scanner.take_single('>') {
                                state = ParserState::Element;
                                scanner.skip_ws();
                            }
                        } else {
                            scanner.take_any();
                        }
                    }
                    ParserState::Element => {
                        scanner.skip_ws();
                        if scanner.take_single('>') {
                            state = ParserState::Text;
                        } else if let Some(attrib_name) = scanner.take_non('=') {
                            debug!("attrib_name: '{attrib_name}'");
                            match attrib_name {
                                "href" if is_anchor && conf.extract_links() => {
                                    state = ParserState::EqualSign;
                                    attribute = Some(Attribute::Href);
                                }
                                "name" if is_anchor && conf.extract_anchors() => {
                                    state = ParserState::EqualSign;
                                    attribute = Some(Attribute::Name);
                                }
                                "id" if conf.extract_anchors() => {
                                    state = ParserState::EqualSign;
                                    attribute = Some(Attribute::Id);
                                }
                                _ => {
                                    state = ParserState::EqualSign;
                                    attribute = Some(Attribute::Other);
                                }
                            }
                            scanner.skip_ws();
                        } else {
                            panic!("Bad HTML: Can't have empty attribute name!");
                        }
                    }
                    ParserState::EqualSign => {
                        scanner.skip_ws();
                        if scanner.take_single('=') {
                            state = ParserState::Attribute;
                        } else if scanner.is_non_ws() {
                            panic!(
                                "Bad character encountered while in state {state:#?}: {:#?}",
                                scanner.take_any()
                            );
                        }
                    }
                    ParserState::Attribute => {
                        scanner.skip_ws();
                        if let Some(attrib_cont) = attribute {
                            match scanner.take_any() {
                                Some('"') => {
                                    let attrib_column = scanner.column;
                                    let attrib_target = scanner.take_non('"').expect(
                                        "Bad HTML! need to finnish attribute value with '\"' (Note: We do not support multi-line attribute values (yet)!)",
                                    );
                                    scanner.take_single('"');
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
                                            name: attrib_target.to_string(),
                                            r#type: anchor::Type::Direct,
                                        }),
                                        Attribute::Id => anchors.push(Anchor {
                                            source: link::Locator {
                                                file: file.locator.clone(),
                                                pos,
                                            },
                                            name: attrib_target.to_string(),
                                            r#type: anchor::Type::ElementId,
                                        }),
                                        Attribute::Other => {}
                                    }
                                    state = ParserState::Element;
                                    attribute = None;
                                }
                                Some(_) | None => {}
                            };
                        }
                    }
                }
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
            links: None,
            anchors: Some(None),
            ..Config::default()
        };
        let markup_file = File::dummy(content, Type::Html);
        super::super::find_links(&markup_file, &conf).map(|(_links, anchors)| anchors)
    }

    #[test]
    fn sc_take() -> std::io::Result<()> {
        let mut scanner = Scanner::empty();
        scanner.reset("Hello World");
        assert!(scanner.take("Hello"));
        assert!(scanner.take_single(' '));
        assert!(scanner.take("World"));
        assert!(scanner.is_done());
        Ok(())
    }

    #[test]
    fn sc_take_any() -> std::io::Result<()> {
        let mut scanner = Scanner::empty();
        scanner.reset("Hello World");
        assert!(scanner.take("Hello"));
        assert_eq!(scanner.take_any(), Some(' '));
        assert!(scanner.take("World"));
        assert_eq!(scanner.take_any(), None);
        assert!(scanner.is_done());
        Ok(())
    }

    #[test]
    fn sc_skip_ws() -> std::io::Result<()> {
        let mut scanner = Scanner::empty();
        scanner.reset("Hello \t \n\r\n \t World");
        assert!(scanner.take("Hello"));
        assert!(scanner.skip_ws());
        assert!(scanner.take("World"));
        assert!(scanner.is_done());
        Ok(())
    }

    #[test]
    fn sc_non_ws() -> std::io::Result<()> {
        let mut scanner = Scanner::empty();
        scanner.reset("Hello World");
        assert_eq!(scanner.take_non_ws(), "Hello");
        assert!(scanner.take_single(' '));
        assert_eq!(scanner.take_non_ws(), "World");
        assert!(scanner.is_done());
        Ok(())
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

    #[test]
    fn empty_attrib() {
        let input = r#"<img src="img/file.jpg" alt="" width="800" />"#;
        let result = find_links(input).expect("No error");
        assert!(result.is_empty());
    }

    #[test]
    fn two_with_extra_attrib() {
        let input = r#"
        <a href="https://www.w3schools.com" target="_blank">Visit W3Schools.com!</a>
        <a href="https://www.w3schools.com" target="_blank">Visit W3Schools.com!</a>
        "#;
        let result = find_links(input).expect("No error");
        let expected1 = Link::new(
            FileLoc::dummy(),
            Position {
                line: 2,
                column: 18,
            },
            "https://www.w3schools.com",
        );
        let expected2 = Link::new(
            FileLoc::dummy(),
            Position {
                line: 3,
                column: 18,
            },
            "https://www.w3schools.com",
        );
        assert_eq!(vec![expected1, expected2], result);
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
    #[test_case(
        r#"<a href="https://www.w3schools.com" target="_blank">Visit W3Schools.com!</a>"#,
        1,
        10
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
    #[test_case(
        r#"<!--comment--><abc id="the_anchor">Visit W3Schools.com!</a>"#,
        false,
        1,
        24
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
    #[test_case(r#"<!--comment--><abc name="the_anchor">Visit W3Schools.com!</abc>"#)]
    fn no_anchors(input: &str) {
        let result = find_anchors(input).expect("No error");
        assert_eq!(Vec::<Anchor>::new(), result);
    }
}
