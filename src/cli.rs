// SPDX-FileCopyrightText: 2022 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::ignore_path::IgnorePath;
use crate::result;
use crate::Config;
use crate::{ignore_link, ignore_path};
use crate::{markup, BoxResult};
use clap::builder::ValueParser;
use clap::{Arg, ArgAction, ArgMatches, Command, ValueHint};
use std::collections::HashSet;
use std::path::PathBuf;
use std::str::FromStr;
use std::{env, io};
use wildmatch::WildMatch;

const A_N_FILES: &str = "files";
const A_L_VERSION: &str = "version";
const A_S_VERSION: char = 'V';
const A_S_QUIET: char = 'q';
const A_L_QUIET: &str = "quiet";
const A_L_NON_RECURSIVE: &str = "non-recursive";
const A_S_NON_RECURSIVE: char = 'N';
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
const A_L_LINKS_FILE: &str = "links-file";
const A_S_LINKS_FILE: char = 'P';
const A_L_RESULT_FORMAT: &str = "result-format";
const A_S_RESULT_FORMAT: char = 'F';
const A_L_RESULT_EXTENDED: &str = "result-extended";
const A_S_RESULT_EXTENDED: char = 'E';
const A_L_RESULT_FLUSH: &str = "result-flush";
const A_S_RESULT_FLUSH: char = 'f';

lazy_static! {
    static ref STDOUT_PATH: PathBuf = PathBuf::from_str("-").unwrap();
}

fn arg_version() -> Arg {
    Arg::new(A_L_VERSION)
        .help(formatcp!(
            "Print version information and exit. \
May be combined with -{A_S_QUIET},--{A_L_QUIET}, \
to really only output the version string."
        ))
        .short(A_S_VERSION)
        .long(A_L_VERSION)
        .action(ArgAction::SetTrue)
}

fn arg_quiet() -> Arg {
    Arg::new(A_L_QUIET)
        .help("Minimize or suppress output to stdout")
        .long_help("Minimize or suppress output to stdout, and only shows log output on stderr.")
        .action(ArgAction::SetTrue)
        .short(A_S_QUIET)
        .long(A_L_QUIET)
}

fn arg_files() -> Arg {
    Arg::new(A_N_FILES)
        .help("The markup files and dirs to scann for markup files")
        .long_help(formatcp!(
            "The markup files and root directories to scann for markup files. \
            See also --{A_L_NON_RECURSIVE}."
        ))
        .num_args(1..)
        .value_parser(value_parser!(std::path::PathBuf))
        .value_name("FILE")
        .value_hint(ValueHint::DirPath)
        .action(ArgAction::Append)
        .default_value(".")
}

fn arg_non_recursive() -> Arg {
    Arg::new(A_L_NON_RECURSIVE)
        .help("Do not scan for files recursively")
        .short(A_S_NON_RECURSIVE)
        .long(A_L_NON_RECURSIVE)
        .action(ArgAction::SetTrue)
}

fn arg_no_links() -> Arg {
    Arg::new(A_L_NO_LINKS)
        .help("Do not extract links")
        .long_help(
            "Do not extract links. \
            See -{A_S_ANCHORS},--{A_L_ANCHORS}.",
        )
        .short(A_S_NO_LINKS)
        .long(A_L_NO_LINKS)
        .requires(A_L_ANCHORS)
        .action(ArgAction::SetTrue)
}

fn arg_anchors() -> Arg {
    Arg::new(A_L_ANCHORS)
        .help("Extract anchors")
        .num_args(0..1)
        .value_name("FILE")
        .short(A_S_ANCHORS)
        .long(A_L_ANCHORS)
        .default_missing_value("-")
        .value_parser(value_parser!(std::path::PathBuf))
        .action(ArgAction::Set)
}

/*
fn arg_match_file_extension() -> Arg {
    Arg::new(A_L_MATCH_FILE_EXTENSION)
        .help("Do check for the exact file extension when searching for a file")
        .short(A_S_MATCH_FILE_EXTENSION)
        .long(A_L_MATCH_FILE_EXTENSION)
        .action(ArgAction::SetTrue)
}
*/

fn arg_ignore_paths() -> Arg {
    Arg::new(A_L_IGNORE_PATHS)
        .help("List of files and directories which will not be scanned; space separated")
        .long_help(
            "One or more files or directories which will not be scanned, \
            separated by white-space.",
        )
        .num_args(1..)
        .value_name("PATH/GLOB")
        .value_hint(ValueHint::FilePath)
        .short(A_S_IGNORE_PATHS)
        .long(A_L_IGNORE_PATHS)
        .action(ArgAction::Append)
        .value_parser(ValueParser::new(ignore_path::parse))
}

fn arg_ignore_links() -> Arg {
    Arg::new(A_L_IGNORE_LINKS)
        .help("List of links which will not be extracted; space separated")
        .long_help(
            "One or more wildcard-patterns/globs, matching links \
            which will not be extracted, separated by white-space.",
        )
        .num_args(1..)
        .value_parser(ValueParser::new(ignore_link::parse))
        .value_name("GLOB")
        .short(A_S_IGNORE_LINKS)
        .long(A_L_IGNORE_LINKS)
        .action(ArgAction::Append)
}

fn arg_markup_types() -> Arg {
    Arg::new(A_L_MARKUP_TYPES)
        .help(
            "List of markup types from which links shall be extracted; \
            space separated. Possible values are found in auto-complete, \
            or when you use a wrong one",
        )
        .num_args(1..)
        .value_parser(value_parser!(markup::Type))
        .short(A_S_MARKUP_TYPES)
        .long(A_L_MARKUP_TYPES)
        .action(ArgAction::Append)
}

fn arg_links_file() -> Arg {
    Arg::new(A_L_LINKS_FILE)
        .help("Where to store the extracted links to")
        .num_args(1)
        .value_hint(ValueHint::FilePath)
        .value_name("FILE")
        .value_parser(value_parser!(std::path::PathBuf))
        .short(A_S_LINKS_FILE)
        .long(A_L_LINKS_FILE)
        .action(ArgAction::Set)
}

fn arg_result_format() -> Arg {
    Arg::new(A_L_RESULT_FORMAT)
        .help("In what data format to output the extracted data")
        .num_args(1)
        .value_parser(value_parser!(result::Type))
        .value_name("FORMAT")
        .short(A_S_RESULT_FORMAT)
        .long(A_L_RESULT_FORMAT)
        .action(ArgAction::Set)
}

fn arg_result_extended() -> Arg {
    Arg::new(A_L_RESULT_EXTENDED)
        .help("Output more info in result file/stream")
        .short(A_S_RESULT_EXTENDED)
        .long(A_L_RESULT_EXTENDED)
        .action(ArgAction::SetTrue)
}

fn arg_result_flush() -> Arg {
    Arg::new(A_L_RESULT_FLUSH)
        .help("Constantly flush (after each item) all the output streams, for the output formats that support it")
        .short(A_S_RESULT_FLUSH)
        .long(A_L_RESULT_FLUSH)
        .action(ArgAction::SetTrue)
}

lazy_static! {
    static ref ARGS: [Arg; 13] = [
        arg_version(),
        arg_quiet(),
        arg_files(),
        arg_non_recursive(),
        arg_no_links(),
        arg_anchors(),
        //arg_match_file_extension(),
        arg_ignore_paths(),
        arg_ignore_links(),
        arg_markup_types(),
        arg_links_file(),
        arg_result_format(),
        arg_result_extended(),
        arg_result_flush(),
    ];
}

fn find_duplicate_short_options() -> Vec<char> {
    let mut short_options: Vec<char> = ARGS.iter().filter_map(clap::Arg::get_short).collect();
    // standard option --help
    short_options.push('h');
    // standard option --version
    // NOTE This is now implemented manually
    // short_options.push('V');
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

fn arg_matcher() -> Command {
    let duplicate_short_options = find_duplicate_short_options();
    assert!(
        duplicate_short_options.is_empty(),
        "Duplicate argument short options: {duplicate_short_options:?}",
    );
    command!()
        .bin_name(clap::crate_name!())
        .help_expected(true)
        .disable_version_flag(true)
        .args(ARGS.iter())
}

fn files_and_dirs(args: &ArgMatches) -> io::Result<Vec<PathBuf>> {
    let mut files_and_dirs = vec![];
    if let Some(out_files) = args.get_many::<PathBuf>(A_N_FILES) {
        for out_file in out_files {
            files_and_dirs.push(out_file.into());
        }
    }
    if files_and_dirs.is_empty() {
        files_and_dirs.push(env::current_dir()?);
    }

    Ok(files_and_dirs)
}

fn print_version_and_exit(quiet: bool) {
    #![allow(clippy::print_stdout)]

    if !quiet {
        print!("{} ", clap::crate_name!());
    }
    println!("{}", crate::VERSION);
    std::process::exit(0);
}

/// Parses CLI arguments into our own config structure.
///
/// # Errors
///
/// If fetching the CWD failed.
pub fn parse_args() -> BoxResult<Config> {
    let args = arg_matcher().get_matches();

    let quiet = args.get_flag(A_L_QUIET);
    let version = args.get_flag(A_L_VERSION);
    if version {
        print_version_and_exit(quiet);
    }

    let files_and_dirs = files_and_dirs(&args)?;
    let recursive = !args.get_flag(A_L_NON_RECURSIVE);
    let links = if args.get_flag(A_L_NO_LINKS) {
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
    let anchors = if !args.contains_id(A_L_ANCHORS) {
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
        .get_many::<IgnorePath>(A_L_IGNORE_PATHS)
        .unwrap_or_default()
        .map(ToOwned::to_owned)
        .collect();
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
    let result_format = args
        .get_one::<result::Type>(A_L_RESULT_FORMAT)
        .copied()
        .unwrap_or_default();
    let result_extended = args.get_flag(A_L_RESULT_EXTENDED);
    let result_flush = args.get_flag(A_L_RESULT_FLUSH);

    Ok(Config {
        files_and_dirs,
        recursive,
        links,
        anchors,
        //match_file_extension,
        ignore_paths,
        ignore_links,
        markup_types,
        //dry,
        result_format,
        result_extended,
        result_flush,
    })
}
