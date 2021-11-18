mod file_system;
mod http;
mod mail;

pub mod link_type;

use crate::link_extractors::link_extractor::MarkupLink;
use crate::link_validator::file_system::check_filesystem;
use crate::link_validator::http::check_http;
use crate::Config;
use colored::ColoredString;
use colored::Colorize;
use mail::check_mail;

pub use link_type::get_link_type;
pub use link_type::LinkType;

#[derive(Debug, PartialEq, Clone)]
pub enum LinkCheckResult {
    Ok,
    Failed(String),
    Warning(String),
    Ignored(String),
    NotImplemented(String),
}

impl LinkCheckResult {
    #[must_use]
    pub fn msg(&self) -> &'_ str {
        match self {
            Self::Ok => "",
            Self::Failed(msg)
            | Self::Warning(msg)
            | Self::Ignored(msg)
            | Self::NotImplemented(msg) => msg,
        }
    }

    #[must_use]
    pub fn status_code(&self) -> &'static ColoredString {
        lazy_static! {
            static ref CODE_OK: ColoredString = "OK".green();
            static ref CODE_WARN: ColoredString = "Warn".yellow();
            static ref CODE_SKIP: ColoredString = "Skip".green();
            static ref CODE_ERR: ColoredString = "Err".red();
        }
        match self {
            LinkCheckResult::Ok => &CODE_OK,
            LinkCheckResult::NotImplemented(_) | LinkCheckResult::Warning(_) => &CODE_WARN,
            LinkCheckResult::Ignored(_) => &CODE_SKIP,
            LinkCheckResult::Failed(_) => &CODE_ERR,
        }
    }

    #[must_use]
    pub fn is_err(&self) -> bool {
        matches!(self, LinkCheckResult::Failed(_))
    }
}

pub async fn resolve_target_link(
    link: &MarkupLink,
    link_type: &LinkType,
    config: &Config,
) -> String {
    if link_type == &LinkType::FileSystem {
        file_system::resolve_target_link(&link.source, &link.target, config).await
    } else {
        link.target.to_string()
    }
}

pub async fn check(link_target: &str, link_type: &LinkType, config: &Config) -> LinkCheckResult {
    info!("Check link {}.", &link_target);
    match link_type {
        LinkType::Ftp | LinkType::UnknownUrlSchema => LinkCheckResult::NotImplemented(format!(
            "Checking of link type '{:?}' is not implemented (yet).",
            &link_type
        )),
        LinkType::Mail => check_mail(link_target),
        LinkType::Http => {
            if config.no_web_links {
                LinkCheckResult::Ignored(
                    "Ignore web link because of the no-web-link flag.".to_string(),
                )
            } else {
                check_http(link_target).await
            }
        }
        LinkType::FileSystem => check_filesystem(link_target, config).await,
    }
}
