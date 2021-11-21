mod file_system;
mod http;
mod mail;

pub mod link_type;

use std::sync::Arc;

use crate::link_extractors::link_extractor::MarkupAnchorTarget;
use crate::link_extractors::link_extractor::MarkupLink;
use crate::link_validator::file_system::check_filesystem;
use crate::link_validator::http::check_http;
use crate::Config;
use crate::RemoteCache;
use crate::State;
use colored::ColoredString;
use colored::Colorize;
use mail::check_mail;
use reqwest::Url;
use tokio::sync::Mutex;

pub use link_type::get_link_type;
pub use link_type::LinkType;

#[derive(Debug, PartialEq, Clone)]
pub enum LinkCheckResult {
    Ok,
    // AnchorCheckMissing,
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

pub async fn check(
    config: &Config,
    remote_cache: Arc<Mutex<&mut RemoteCache>>,
    link_target: &str,
    link_anchor: Option<String>,
    link_type: &LinkType,
    requires_anchors: bool,
) -> LinkCheckResult {
    info!("Checking link '{}' ...", &link_target);

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
            } else
            /*if requires_anchors*/
            {
                let url = Url::parse(link_target).expect("Failed to parse link target as URL");
                match remote_cache.get(&url) {
                    Some(cached_result) => {
                        // this URL does have a cached entry
                        match cached_result {
                            Ok(anchor_targets_cont) => {
                                // URL was cached as available
                                if requires_anchors {
                                    match anchor_targets_cont {
                                        Some(anchor_targets) => {
                                            // anchor targets are cached
                                            if anchor_targets
                                                .iter()
                                                .find(|mt| {
                                                    mt.source == link_target
                                                        && mt.name == link_anchor.unwrap()
                                                })
                                                .is_some()
                                            {
                                                LinkCheckResult::Ok
                                            } else {
                                                LinkCheckResult::Failed("URL was cached as not having the requested anchor target".to_string())
                                            }
                                        }
                                        None => {
                                            // if we get here, it means the link was cached as reachable,
                                            // but anchor targets ver not fetched/scanned,
                                            // but we need them
                                            if config.no_web_links {
                                                LinkCheckResult::Ignored(
                                                    "Ignore web link with anchor because of the no-web-link flag.".to_string(), // TODO Use more similar wording for Ignore states
                                                )
                                            } else {
                                                let check_res = check_http(
                                                    &mut state,
                                                    &link_target,
                                                    requires_anchors,
                                                )
                                                .await;
                                                TODO; // TODO Cache the result from the above cache and create the return state
                                                let url = reqwest::Url::parse(&link_target)
                                                    .expect("URL of unknown type"); // TODO expect is BAD! .. here, really bad
                                                if !check_res.1.is_some()
                                                    && check_res.1.unwrap().is_empty()
                                                {
                                                    match state.remote_cache.get_mut(&url) {
                                                        Some(url_cache) => url_cache
                                                            .insert_all(check_res.1.unwrap()),
                                                        None => state.remote_cache.insert(
                                                            url,
                                                            Ok(Some(check_res.1.unwrap())),
                                                        ),
                                                    }
                                                } else {
                                                    // TODO Store that we have found nothing, or that trying to fine failed (all into the remote_cache under &url)
                                                }
                                                check_res.0
                                            }
                                        }
                                    }
                                } else {
                                    LinkCheckResult::Ok
                                }
                            }
                            Err(err) => {
                                // URL was cached as *un*available
                                LinkCheckResult::Failed(err.to_string())
                            }
                        }
                    }
                    None => {
                        // this URL does *not* have a cached entry yet
                        if config.no_web_anchors {
                            LinkCheckResult::Ignored(
                                "Non-cached, online link; not checking because of offline mode"
                                    .to_string(),
                            )
                        } else {
                            // TODO fetch and cache the results
                            (links, anchor_targets) = check_http(&mut state, &link_target, requires_anchors).await
                            links
                        }
                    }
                }
                // LinkCheckResult::Warning(format!("{} (cached)", err))

                // } else {
                //     check_http(link_target).await
            }
        }
        LinkType::FileSystem => check_filesystem(link_target, config).await,
    }
}
