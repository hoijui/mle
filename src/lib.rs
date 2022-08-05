// SPDX-FileCopyrightText: 2022 Robin Vobruba <hoijui.quaero@gmail.com>
// SPDX-FileCopyrightText: 2020 Armin Becher <becherarmin@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

#[macro_use]
extern crate log;
#[macro_use]
extern crate clap;
#[macro_use]
extern crate clap_derive;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate const_format;

pub mod anchor;
pub mod cli;
pub mod config;
pub mod extractors;
pub mod file_traversal;
pub mod ignore_link;
pub mod ignore_path;
pub mod link;
pub mod logger;
pub mod markup;
pub mod result;
pub mod state;

use crate::anchor::Anchor;
use crate::link::Link;
use crate::markup::File;
pub use colored::*;
use config::Config;
use state::State;
pub use wildmatch::WildMatch;

fn find_all_links(conf: &Config) -> (Vec<Link>, Vec<Anchor>, Vec<std::io::Error>) {
    let mut files: Vec<File> = Vec::new();
    file_traversal::find(conf, &mut files);
    let mut links = vec![];
    let mut anchor_targets = vec![];
    let mut errors = vec![];
    for file in files {
        match extractors::find_links(&file, conf) {
            Ok((mut file_links, mut file_anchor_targets)) => {
                links.append(&mut file_links);
                anchor_targets.append(&mut file_anchor_targets);
            }
            Err(err) => {
                errors.push(err);
            }
        }
    }
    (links, anchor_targets, errors)
}

fn print_helper(link: &Link, status_code: &colored::ColoredString, msg: &str, error_channel: bool) {
    let link_str = format!("[{:^4}] {:#?} - {}", status_code, link, msg);
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

/// Runs the markup link extractor.
/// This is the main entry point of this library.
///
/// # Errors
///
/// If reading of any input or writing of the log or result-file failed.
pub fn run(state: &mut State) -> Result<(), Box<dyn std::error::Error>> {
    let (links, anchors, errors) = find_all_links(&state.config);
    // let mut secondary_anchors = find_all_anchor_targets(&state.config, &links);
    // primary_anchors.append(&mut secondary_anchors);

    // // <target, (links, requires_anchors)>
    // let mut link_target_groups: HashMap<Target, (Vec<Link>, bool)> = HashMap::new();

    // let mut skipped = 0;

    // for link in &links {
    //     if state
    //         .config
    //         .ignore_links
    //         .iter()
    //         .any(|m| m.matches(&link.target1))
    //     {
    //         print_helper(
    //             link,
    //             &"Skip".green(),
    //             "Ignore link because of ignore-links option.",
    //             false,
    //         );
    //         skipped += 1;
    //         continue;
    //     }
    //     let target = resolve_target_link(link, &link.target.r#type, &state.config).await;
    //     let t = Target::new(target, link_type);
    //     match link_target_groups.get_mut(&t) {
    //         Some(v) => {
    //             v.0.push(link.clone());
    //             v.1 = v.1 || link.target.anchor.is_some();
    //         }
    //         None => {
    //             link_target_groups.insert(t, (vec![link.clone()], link.target.anchor.is_some()));
    //         }
    //     }
    // }

    // for (target, (links, _)) in link_target_groups {
    //     for link in links {
    //         // println!("{}#{}", target, link);
    //         println!("{:?}", link);
    //     }
    // }

    println!("Links ...");
    for link in links {
        println!("{}", link);
    }

    println!("\nAnchors ...");
    for anchor in anchors {
        println!("{}", anchor);
    }

    println!("\nErrors ...");
    for error in errors {
        println!("{:#?}", error);
    }

    Ok(())
}
