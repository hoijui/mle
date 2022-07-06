#[macro_use]
extern crate log;
#[macro_use]
extern crate clap;
#[macro_use]
extern crate lazy_static;

// use crate::file_traversal::markup_type;
use crate::types::MarkupAnchorTarget;
use crate::types::MarkupLink;
use crate::link_resolver::resolve_target_link;
use crate::link_type::get_link_type;
// use crate::markup::Content;
use crate::markup::MarkupFile;
use std::collections::HashMap;
use std::fmt::Write;
use std::path::PathBuf;
use std::sync::Arc;
use link_type::LinkType;
use tokio::sync::Mutex;
use tokio::time::{sleep_until, Duration, Instant};
pub mod cli;
pub mod file_traversal;
pub mod ignore_path;
pub mod link_extractors;
pub mod types;
pub mod link_type;
pub mod link_resolver;
pub mod logger;
pub mod markup;
pub use colored::*;
pub use wildmatch::WildMatch;

use futures::{stream, StreamExt};
use ignore_path::IgnorePath;
use url::Url;

const PARALLEL_REQUESTS: usize = 20;

/// If a URL is not stored in the map (the URL does not appear as a key),
/// it means that URL has not yet been checked.
/// If the Result is Err, it means the URL has been checked,
/// but was not available, or anchor parsing has failed.
/// If the Option is None, it means the URL was checked and evaluated as for available,
/// but no parsing of anchors was tried.
/// If the Vec is empty, it means that the document was parsed, but no anchors were found.
//type CheckResult = Option<Vec<MarkupAnchorTarget>>;

/// If a URL is not stored in the map (the URL does not appear as a key),
/// it means that URL has not yet been checked.
/// If the Result is Err, it means the URL has been checked,
/// but was not available, or anchor parsing has failed.
/// If the Option is None, it means the URL was checked and evaluated as for available,
/// but no parsing of anchors was tried.
/// If the Vec is empty, it means that the document was parsed, but no anchors were found.
//pub type RemoteCache = HashMap<reqwest::Url, LinkCheckResult>;
//type AnchorsCache = HashMap<reqwest::Url, Option<reqwest::Result<Vec<MarkupAnchorTarget>>>>;
//type AnchorsCache = HashMap<reqwest::Url, reqwest::Result<Vec<MarkupAnchorTarget>>>;

/// If a URL is not stored in the map (the URL does not appear as a key),
/// it means that URL has not yet been checked.
/// If the Result is Err, it means the URL has been checked,
/// but was not available, or anchor parsing has failed.
/// If the Option is None, it means the URL was checked and evaluated as for available,
/// but no parsing of anchors was tried.
/// If the Vec is empty, it means that the document was parsed, but no anchors were found.
pub type AnchorTargets = Option<Vec<MarkupAnchorTarget>>;

/// If a URL is not stored in the map (the URL does not appear as a key),
/// it means that URL has not yet been checked.
/// If the Result is Err, it means the URL has been checked,
/// but was not available, or anchor parsing has failed.
/// If the Option is None, it means the URL was checked and evaluated as for available,
/// but no parsing of anchors was tried.
/// If the Vec is empty, it means that the document was parsed, but no anchors were found.
pub type RemoteCache = HashMap<reqwest::Url, reqwest::Result<AnchorTargets>>;
//type AnchorsCache = HashMap<reqwest::Url, LinkCheckResult>;
//type AnchorsCache = HashMap<reqwest::Url, Option<reqwest::Result<Vec<MarkupAnchorTarget>>>>;
//type AnchorsCache = HashMap<reqwest::Url, reqwest::Result<Vec<MarkupAnchorTarget>>>;

#[derive(Default, Debug, Clone)]
pub struct Config {
    pub log_level: logger::LogLevel,
    pub folder: PathBuf,
    pub markup_types: Vec<markup::MarkupType>,
    pub no_web_links: bool,
    pub no_web_anchors: bool,
    pub match_file_extension: bool,
    pub ignore_links: Vec<WildMatch>,
    pub ignore_paths: Vec<IgnorePath>,
    pub root_dir: Option<PathBuf>,
    pub throttle: u32,
}

#[derive(Default, Debug)]
pub struct State {
    pub config: Config,
    pub remote_cache: RemoteCache,
}

impl State {
    pub fn new(config: Config) -> State {
        State {
            remote_cache: RemoteCache::new(),
            config,
        }
    }
}

#[derive(Hash, PartialEq, Eq, Clone, Debug)]
struct Target {
    target: String,
    link_type: LinkType,
    anchor: Option<String>,
}

impl std::fmt::Display for Target {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.target)?;
        if let Some(anchor) = &self.anchor {
            f.write_fmt(format_args!("#{}", anchor))?;
        }
        Ok(())
    }
}

impl Target {
    // TODO I think we have this elsewehere already, nicer.. search for '#'
    pub fn new(link: String, link_type: LinkType) -> Target {
        let (target, anchor) = match link.find('#') {
            Some(idx) => {
                // warn!(
                //     "Strip everything after #. The chapter part ´{}´ is not checked.",
                //     &normalized_link[idx..]
                // );
                (link[..idx].to_owned(), Some(link[idx..].to_owned()))
            }
            None => (link, None),
        };
        Target {
            target,
            link_type,
            anchor,
        }
    }
}

fn find_all_links(config: &Config) -> (Vec<MarkupLink>, Vec<MarkupAnchorTarget>) {
    let mut files: Vec<MarkupFile> = Vec::new();
    file_traversal::find(config, &mut files);
    let mut links = vec![];
    let mut anchor_targets = vec![];
    for file in files {
        let (mut file_links, mut file_anchor_targets) = link_extractors::link_extractor::find_links(&file, false);
        links.append(&mut file_links);
        anchor_targets.append(&mut file_anchor_targets);
    }
    (links, anchor_targets)
}

fn print_helper(
    link: &MarkupLink,
    status_code: &colored::ColoredString,
    msg: &str,
    error_channel: bool,
) {
    let link_str = format!(
        "[{:^4}] {} ({}, {}) => {} - {}",
        status_code, link.source, link.line, link.column, link.target, msg
    );
    if error_channel {
        eprintln!("{}", link_str);
    } else {
        println!("{}", link_str);
    }
}

// fn print_result(result: &FinalResult, map: &HashMap<Target, Vec<MarkupLink>>) {
//     for link in &map[&result.target] {
//         let code = &result.result_code;
//         print_helper(link, code.status_code(), code.msg(), code.is_err());
//     }
// }

pub async fn run(state: &mut State) -> Result<(), ()> {
    let (links, mut primary_anchors) = find_all_links(&state.config); // TODO use the anchors!
    // let mut secondary_anchors = find_all_anchor_targets(&state.config, &links);
    // primary_anchors.append(&mut secondary_anchors);
    // <target, (links, requires_anchors)>
    let mut link_target_groups: HashMap<Target, (Vec<MarkupLink>, bool)> = HashMap::new();

    let mut skipped = 0;

    for link in &links {
        if state
            .config
            .ignore_links
            .iter()
            .any(|m| m.matches(&link.target))
        {
            print_helper(
                link,
                &"Skip".green(),
                "Ignore link because of ignore-links option.",
                false,
            );
            skipped += 1;
            continue;
        }
        let link_type = get_link_type(&link.target);
        let target = resolve_target_link(link, &link_type, &state.config).await;
        let t = Target::new(target, link_type);
        match link_target_groups.get_mut(&t) {
            Some(v) => {
                v.0.push(link.clone());
                v.1 = v.1 || link.anchor.is_some();
            }
            None => {
                link_target_groups.insert(t, (vec![link.clone()], link.anchor.is_some()));
            }
        }
    }

    for (target, (links, _)) in link_target_groups {
        for link in links {
            // println!("{}#{}", target, link);
            println!("{:?}", link);
        }
    }

    // let throttle = state.config.throttle > 0;
    // info!("Throttle HTTP requests to same host: {:?}", throttle);
    // let waits = Arc::new(Mutex::new(HashMap::new()));
    // let throttle_val = state.config.throttle;
    // let config = &state.config; //.clone();
    // let remote_cache = Arc::new(Mutex::new(&mut state.remote_cache));
    // // See also http://patshaughnessy.net/2020/1/20/downloading-100000-files-using-async-rust
    // let mut buffered_stream = stream::iter(link_target_groups.iter())
    //     .map(|(target, (links, requires_anchor))| {
    //         let waits = waits.clone();
    //         // TODO State is modified inside here, but this is a multi-threaded context ... :/ -> check online how to solve this, with error message given here
    //         async move {
    //             if throttle && target.link_type == LinkType::Http {
    //                 let parsed = match Url::parse(&target.target) {
    //                     Ok(parsed) => parsed,
    //                     Err(error) => {
    //                         return FinalResult {
    //                             target: target.clone(),
    //                             result_code: LinkCheckResult::Failed(format!(
    //                                 "Could not parse URL type. Err: {:?}",
    //                                 error
    //                             )),
    //                         }
    //                     }
    //                 };
    //                 let host = match parsed.host_str() {
    //                     Some(host) => host.to_string(),
    //                     None => {
    //                         return FinalResult {
    //                             target: target.clone(),
    //                             result_code: LinkCheckResult::Failed(
    //                                 "Failed to determine host".to_string(),
    //                             ),
    //                         }
    //                     }
    //                 };
    //                 let mut waits = waits.lock().await;

    //                 let mut wait_until: Option<Instant> = None;
    //                 let next_wait = match waits.get(&host) {
    //                     Some(old) => {
    //                         wait_until = Some(*old);
    //                         *old + Duration::from_millis(throttle_val.into())
    //                     }
    //                     None => Instant::now() + Duration::from_millis(throttle_val.into()),
    //                 };
    //                 waits.insert(host, next_wait);
    //                 drop(waits);

    //                 if let Some(deadline) = wait_until {
    //                     sleep_until(deadline).await;
    //                 }
    //             }

    //             let remote_cache = Arc::clone(&remote_cache);
    //             let result_code = link_validator::check(
    //                 config,
    //                 remote_cache,
    //                 &target.target,
    //                 target.anchor,
    //                 &target.link_type,
    //                 *requires_anchor,
    //             )
    //             .await;
    //             // LinkCheckResult::Ignored(
    //             //     "Ignore web link because of the no-web-link flag.".to_string(),
    //             // ); // stub for testing/debugging -> this one resolves the threadding issue -> prove that the issue is here!

    //             FinalResult {
    //                 target: target.clone(),
    //                 result_code,
    //             }
    //         }
    //     })
    //     .buffer_unordered(PARALLEL_REQUESTS);

    // let mut oks = 0;
    // let mut warnings = 0;
    // let mut errors = vec![];

    // let mut process_result = |result| {
    //     print_result(&result, &link_target_groups);
    //     match &result.result_code {
    //         LinkCheckResult::Ok => {
    //             oks += link_target_groups[&result.target].0.len();
    //         }
    //         LinkCheckResult::NotImplemented(_) | LinkCheckResult::Warning(_) => {
    //             warnings += link_target_groups[&result.target].0.len();
    //         }
    //         LinkCheckResult::Ignored(_) => {
    //             skipped += link_target_groups[&result.target].0.len();
    //         }
    //         LinkCheckResult::Failed(_) => {
    //             errors.push(result.clone());
    //         }
    //     }
    // };

    // while let Some(result) = buffered_stream.next().await {
    //     process_result(result);
    // }

    // println!();
    // // let error_sum: usize = errors
    // //     .iter()
    // //     .map(|e| link_target_groups[&e.target].0.len())
    // //     .sum();
    // // let sum = skipped + error_sum + warnings + oks;
    // println!("Result ({} links):", sum);
    // println!();
    // println!("OK       {}", oks);
    // println!("Skipped  {}", skipped);
    // println!("Warnings {}", warnings);
    // println!("Errors   {}", error_sum);
    // println!();

    // if errors.is_empty() {
    //     Ok(())
    // } else {
    //     eprintln!();
    //     eprintln!("The following links could not be resolved:");
    //     println!();
    //     for res in errors {
    //         for link in &link_target_groups[&res.target].0 {
    //             eprintln!(
    //                 "{} ({}, {}) => {}.",
    //                 link.source, link.line, link.column, link.target
    //             );
    //         }
    //     }
    //     println!();
    //     Err(())
    // }
    Ok(())
}
