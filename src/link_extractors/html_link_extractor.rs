use crate::config::Config;
use crate::link::MarkupAnchorTarget;
use crate::link::{Link, Position};
use crate::link_extractors::link_extractor::LinkExtractor;
// use crate::link::MarkupLink;
use crate::markup::MarkupFile;

pub struct HtmlLinkExtractor();

#[derive(Clone, Copy, Debug)]
enum ParserState {
    Text,
    Comment,
    Element,
    EqualSign,
    Link,
}

// #[derive(Clone, Copy, Debug)]
// enum Attribute {
//     Href,
//     Name,
//     Id,
// }

impl LinkExtractor for HtmlLinkExtractor {
    fn find_links_and_anchors(
        &self,
        file: &MarkupFile,
        conf: &Config,
    ) -> std::io::Result<(Vec<Link>, Vec<MarkupAnchorTarget>)> {
        let mut links: Vec<Link> = Vec::new();
        let mut anchors: Vec<MarkupAnchorTarget> = Vec::new(); // TODO FIXME This is never added to!
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
                        if is_anchor
                            && line_chars.get(column) == Some(&'h')
                            && line_chars.get(column + 1) == Some(&'r')
                            && line_chars.get(column + 2) == Some(&'e')
                            && line_chars.get(column + 3) == Some(&'f')
                        {
                            column += 3;
                            state = ParserState::EqualSign;
                        }
                    }
                    ParserState::EqualSign => {
                        match line_chars.get(column) {
                            Some(x) if x.is_whitespace() => {}
                            Some(x) if x == &'=' => state = ParserState::Link,
                            Some(_) => state = ParserState::Element,
                            None => {}
                        };
                    }
                    ParserState::Link => {
                        match line_chars.get(column) {
                            Some(x) if !x.is_whitespace() && x != &'"' => {
                                let link_column = column;
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
                                let link_target = &(&line_chars[link_column..column])
                                    .iter()
                                    .collect::<String>();
                                // let link_target = line_chars[link_column..column]; // TODO FIXME Do it somehow like this (should be faster)
                                let pos = Position {
                                    line: line + 1,
                                    column: link_column + 1,
                                } + &file.start;
                                links.push(Link::new(file.locator.clone(), pos, link_target));
                                state = ParserState::Text;
                            }
                            Some(_) | None => {}
                        };
                    }
                }
                column += 1;
            }
        }
        Ok((links, anchors))
    }
}

#[cfg(test)]
mod tests {
    use crate::{link::FileLoc, markup::MarkupType};

    use super::*;
    use ntest::test_case;

    fn find_links(content: &str) -> std::io::Result<Vec<Link>> {
        let le = HtmlLinkExtractor();
        let conf = Config::default();
        let markup_file = MarkupFile::dummy(content, MarkupType::Html);
        le.find_links_and_anchors(&markup_file, &conf)
            .map(|(links, _anchors)| links)
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
}
