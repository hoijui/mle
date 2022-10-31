// SPDX-FileCopyrightText: 2022 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::ignore_path::IgnorePath;
use crate::{group, Config};
use crate::{ignore_link, ignore_path};
use crate::{logger, result};
use crate::{markup, BoxResult};
use clap::builder::ValueParser;
use clap::Arg;
use clap::{ArgAction, Command, ValueHint};
use std::collections::HashSet;
use std::env;
use std::path::PathBuf;
use std::str::FromStr;
use wildmatch::WildMatch;

const A_L_SCAN_ROOT: &str = "scann-root";
const A_S_SCAN_ROOT: char = 'S';
const A_L_NON_RECURSIVE: &str = "non-recursive";
const A_S_NON_RECURSIVE: char = 'N';
const A_L_DEBUG: &str = "debug";
const A_S_DEBUG: char = 'D';
const A_L_NO_LINKS: &str = "no-links";
const A_S_NO_LINKS: char = 'n';
const A_L_ANCHORS: &str = "anchors";
const A_S_ANCHORS: char = 'a';
//const A_L_MATCH_FILE_EXTENSION: &str = "match-file-extension";
//const A_S_MATCH_FILE_EXTENSION: char = 'M';
const A_L_IGNORE_PATHS: &str = "ignore-paths";
const A_S_IGNORE_PATHS: char = 'I';
const A_L_IGNORE_LINKS: &str = "ignore-links";
const A_S_IGNORE_LINKS: char = 'i';
const A_L_MARKUP_TYPES: &str = "markup-types";
const A_S_MARKUP_TYPES: char = 'm';
const A_L_RESOLVE_ROOT: &str = "resolve-root";
const A_S_RESOLVE_ROOT: char = 'R';
//const A_L_DRY: &str = "dry";
//const A_S_DRY: char = 'd';
const A_L_LOG_FILE: &str = "log-file";
const A_S_LOG_FILE: char = 'l';
const A_L_LINKS_FILE: &str = "links-file";
const A_S_LINKS_FILE: char = 'P';
const A_L_RESULT_FORMAT: &str = "result-format";
const A_S_RESULT_FORMAT: char = 'F';
const A_L_GROUP_BY: &str = "group-by";
const A_S_GROUP_BY: char = 'G';

lazy_static! {
    static ref STDOUT_PATH: PathBuf = PathBuf::from_str("-").unwrap();
}

#[allow(clippy::useless_transmute)]
fn arg_scan_root() -> Arg<'static> {
    Arg::new(A_L_SCAN_ROOT)
        .help("The root dir to scann for markup files")
        .long_help(formatcp!(
            "The root directory to scann for markup files. \
            See also --{A_L_NON_RECURSIVE}."
        ))
        .takes_value(true)
        .value_parser(value_parser!(std::path::PathBuf))
        .value_name("DIR")
        .value_hint(ValueHint::DirPath)
        .short(A_S_SCAN_ROOT)
        .long(A_L_SCAN_ROOT)
        .action(ArgAction::Set)
        .required(false)
        .default_value(".")
}

fn arg_non_recursive() -> Arg<'static> {
    Arg::new(A_L_NON_RECURSIVE)
        .help("Do not scan for files recursively")
        .takes_value(false)
        .short(A_S_NON_RECURSIVE)
        .long(A_L_NON_RECURSIVE)
        .required(false)
}

fn arg_debug() -> Arg<'static> {
    Arg::new(A_L_DEBUG)
        .help("Print debug information to the console")
        .takes_value(false)
        .short(A_S_DEBUG)
        .long(A_L_DEBUG)
        // .multiple_occurrences(false)
        .required(false)
}

fn arg_no_links() -> Arg<'static> {
    Arg::new(A_L_NO_LINKS)
        .help("Do not extract links")
        .long_help(
            "Do not extract links. \
            See -{A_S_ANCHORS},--{A_L_ANCHORS}.",
        )
        .takes_value(false)
        .short(A_S_NO_LINKS)
        .long(A_L_NO_LINKS)
        .requires(A_L_ANCHORS)
        // .multiple_occurrences(false)
        .required(false)
}

fn arg_anchors() -> Arg<'static> {
    Arg::new(A_L_ANCHORS)
        .help("Extract anchors")
        .takes_value(true)
        .value_name("FILE")
        .short(A_S_ANCHORS)
        .long(A_L_ANCHORS)
        .default_missing_value("-")
        .value_parser(value_parser!(std::path::PathBuf))
        .action(ArgAction::Set)
        // .multiple_occurrences(false)
        .required(false)
}

/*
fn arg_match_file_extension() -> Arg<'static> {
    Arg::new(A_L_MATCH_FILE_EXTENSION)
        .help("Do check for the exact file extension when searching for a file")
        .takes_value(false)
        .short(A_S_MATCH_FILE_EXTENSION)
        .long(A_L_MATCH_FILE_EXTENSION)
        .required(false)
}
*/

fn arg_ignore_paths() -> Arg<'static> {
    Arg::new(A_L_IGNORE_PATHS)
        .help("List of files and directories which will not be scanned; space separated")
        .long_help(
            "One or more files or directories which will not be scanned, \
            separated by white-space.",
        )
        .takes_value(true)
        .value_name("PATH/GLOB")
        .value_hint(ValueHint::FilePath)
        .min_values(1)
        .required(false)
        .short(A_S_IGNORE_PATHS)
        .long(A_L_IGNORE_PATHS)
        .action(ArgAction::Append)
        .value_parser(ValueParser::new(ignore_path::parse))
}

fn arg_ignore_links() -> Arg<'static> {
    Arg::new(A_L_IGNORE_LINKS)
        .help("List of links which will not be extracted; space separated")
        .long_help(
            "One or more wildcard-patterns/globs, matching links \
            which will not be extracted, separated by white-space.",
        )
        .takes_value(true)
        .min_values(1)
        .value_parser(ValueParser::new(ignore_link::parse))
        .value_name("GLOB")
        .short(A_S_IGNORE_LINKS)
        .long(A_L_IGNORE_LINKS)
        .action(ArgAction::Append)
        .required(false)
}

fn arg_markup_types() -> Arg<'static> {
    Arg::new(A_L_MARKUP_TYPES)
        .help(
            "List of markup types from which links shall be extracted; \
            space separated. Possible values are found in auto-complete, \
            or when you use a wrong one",
        )
        .takes_value(true)
        .min_values(1)
        .value_parser(value_parser!(markup::Type))
        .short(A_S_MARKUP_TYPES)
        .long(A_L_MARKUP_TYPES)
        .action(ArgAction::Append)
        .required(false)
}

fn arg_resolve_root() -> Arg<'static> {
    Arg::new(A_L_RESOLVE_ROOT)
        .help("Path or URL used to resolve all relative paths to")
        .takes_value(true)
        .value_name("PATH/URL")
        .value_hint(ValueHint::FilePath)
        .short(A_S_RESOLVE_ROOT)
        .long(A_L_RESOLVE_ROOT)
        .required(false)
        .conflicts_with(A_L_NO_LINKS)
}

/*
fn arg_dry() -> Arg<'static> {
    Arg::new(A_L_DRY)
        .help("Do not write any files or set any environment variables")
        .long_help("Set Whether to skip the actual setting of environment variables.")
        .takes_value(false)
        .short(A_S_DRY)
        .long(A_L_DRY)
        .multiple_occurrences(false)
        .required(false)
}
*/

fn arg_log_file() -> Arg<'static> {
    lazy_static! {
        static ref LOG_FILE_NAME: String = format!("{}.log.txt", crate_name!());
    }
    Arg::new(A_L_LOG_FILE)
        .help("Write log output to a file")
        .long_help("Writes a detailed log to the specifed file.")
        .takes_value(true)
        .value_parser(value_parser!(std::path::PathBuf))
        .value_hint(ValueHint::FilePath)
        .short(A_S_LOG_FILE)
        .long(A_L_LOG_FILE)
        // .multiple_occurrences(false)
        .required(false)
        .default_missing_value(&LOG_FILE_NAME)
}

fn arg_links_file() -> Arg<'static> {
    Arg::new(A_L_LINKS_FILE)
        .help("Where to store the extracted links to")
        .takes_value(true)
        .value_hint(ValueHint::FilePath)
        .value_name("FILE")
        .value_parser(value_parser!(std::path::PathBuf))
        .short(A_S_LINKS_FILE)
        .long(A_L_LINKS_FILE)
        .required(false)
}

fn arg_result_format() -> Arg<'static> {
    Arg::new(A_L_RESULT_FORMAT)
        .help("In what data format to output the extracted data")
        .takes_value(true)
        .value_parser(value_parser!(result::Type))
        .value_name("FORMAT")
        .short(A_S_RESULT_FORMAT)
        .long(A_L_RESULT_FORMAT)
        .required(false)
}

fn arg_group_by() -> Arg<'static> {
    Arg::new(A_L_GROUP_BY)
        .help("What to group links by in the output (default: No grouping -> Use oder of appearance in the input)")
        .takes_value(true)
        .value_parser(value_parser!(group::Type))
        .value_name("GROUPER")
        .short(A_S_GROUP_BY)
        .long(A_L_GROUP_BY)
        .required(false)
}

lazy_static! {
    static ref ARGS: [Arg<'static>; 13] = [
        arg_scan_root(),
        arg_non_recursive(),
        arg_debug(),
        arg_no_links(),
        arg_anchors(),
        //arg_match_file_extension(),
        arg_ignore_paths(),
        arg_ignore_links(),
        arg_markup_types(),
        arg_resolve_root(),
        //arg_dry(),
        arg_log_file(),
        arg_links_file(),
        arg_result_format(),
        arg_group_by(),
    ];
}

fn find_duplicate_short_options() -> Vec<char> {
    let mut short_options: Vec<char> = ARGS.iter().filter_map(clap::Arg::get_short).collect();
    short_options.push('h'); // standard option --help
    short_options.push('V'); // standard option --version
    short_options.sort_unstable();
    let mut duplicate_short_options = HashSet::new();
    let mut last_chr = '&';
    for chr in &short_options {
        if *chr == last_chr {
            duplicate_short_options.insert(*chr);
        }
        last_chr = *chr;
    }
    duplicate_short_options.iter().copied().collect()
}

fn arg_matcher() -> Command<'static> {
    let app = command!().bin_name("mle").args(ARGS.iter());
    let duplicate_short_options = find_duplicate_short_options();
    assert!(
        duplicate_short_options.is_empty(),
        "Duplicate argument short options: {:?}",
        duplicate_short_options
    );
    app
}

/// Parses CLI arguments into our own config structure.
///
/// # Errors
///
/// If fetching the CWD failed.
pub fn parse_args() -> BoxResult<Config> {
    let args = arg_matcher().get_matches();

    let scan_root = match args.get_one::<PathBuf>(A_L_SCAN_ROOT) {
        Some(dir) => dir.clone(), //PathBuf::from(dir),
        None => env::current_dir()?,
    };
    let recursive = !args.contains_id(A_L_NON_RECURSIVE);
    let debug = args.contains_id(A_L_DEBUG);
    let links = if !args.contains_id(A_L_NO_LINKS) {
        None
    } else if let Some(path) = args.get_one::<PathBuf>(A_L_LINKS_FILE) {
        if path.as_os_str().eq(STDOUT_PATH.as_os_str()) {
            Some(None)
        } else {
            Some(Some(path.clone()))
        }
    } else {
        Some(None)
    };
    let anchors = if args.contains_id(A_L_ANCHORS) {
        None
    } else if let Some(path) = args.get_one::<PathBuf>(A_L_ANCHORS) {
        if path.as_os_str().eq(STDOUT_PATH.as_os_str()) {
            Some(None)
        } else {
            Some(Some(path.clone()))
        }
    } else {
        Some(None)
    };
    //let match_file_extension = args.value_of(A_L_MATCH_FILE_EXTENSION);
    let ignore_paths: Vec<IgnorePath> = args
        .get_many::<Result<IgnorePath, String>>(A_L_IGNORE_PATHS)
        .unwrap_or_default()
        .map(ToOwned::to_owned)
        .collect::<Result<Vec<IgnorePath>, _>>()?;
    let ignore_links: Vec<WildMatch> = args
        .get_many::<WildMatch>(A_L_IGNORE_LINKS)
        .unwrap_or_default()
        .map(ToOwned::to_owned)
        .collect();
    let mut markup_types = vec![markup::Type::Markdown, markup::Type::Html];
    if let Some(types) = args.get_many::<&str>(A_L_MARKUP_TYPES) {
        markup_types = types
            .copied()
            .map(markup::Type::from_str)
            .collect::<Result<Vec<markup::Type>, _>>()?;
    }
    let resolve_root = args.get_one::<PathBuf>(A_L_RESOLVE_ROOT).map(PathBuf::from);
    /*let resolve_root = if let Some(resolve_root) = matches.value_of(A_L_RESOLVE_ROOT) {
        let resolve_root = Path::new(
            &resolve_root
                .replace('/', &MAIN_SEPARATOR.to_string())
                .replace('\\', &MAIN_SEPARATOR.to_string()),
        )
        .to_path_buf();
        if !resolve_root.is_dir() {
            eprintln!("Resolve root '{:?}' must be a directory!", resolve_root);
            std::process::exit(1);
        }
        Some(resolve_root)
    } else {
        None
    };*/
    //let dry = args.value_of(A_L_DRY);
    let log_file = args.get_one::<PathBuf>(A_L_LOG_FILE).map(PathBuf::from);
    let result_format = args
        .get_one::<result::Type>(A_L_RESULT_FORMAT)
        .copied()
        .unwrap_or_default();

    let log_level = if debug {
        logger::LogLevel::Debug
    } else {
        logger::LogLevel::Warn
    };

    let group_by = args.get_one::<group::Type>(A_L_GROUP_BY).copied();

    Ok(Config {
        log_level,
        log_file,
        scan_root,
        recursive,
        links,
        anchors,
        //match_file_extension,
        ignore_paths,
        ignore_links,
        markup_types,
        resolve_root,
        //dry,
        result_format,
        group_by,
    })
}
