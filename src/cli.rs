// SPDX-FileCopyrightText: 2022 - 2025 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use clap::builder::ValueParser;
use clap::command;
use clap::value_parser;
use clap::{Arg, ArgAction, ArgMatches, Command, ValueHint};
use const_format::formatcp;
use mle::BoxResult;
use mle::Config;
use mle::ignore_link;
use mle::result;
use std::collections::HashSet;
use std::str::FromStr;
use std::sync::LazyLock;
use std::{env, io};
use wildmatch::WildMatch;

// #[cfg(feature = "async")]
// use async_std::path::PathBuf;
use mle::path_buf::PathBuf;
// #[cfg(not(feature = "async"))]
// use std::path::PathBuf;

const A_N_MARKUP_FILES: &str = "markup_files";
const A_L_VERSION: &str = "version";
const A_S_VERSION: char = 'V';
const A_S_QUIET: char = 'q';
const A_L_QUIET: &str = "quiet";
const A_L_NO_LINKS: &str = "no-links";
const A_S_NO_LINKS: char = 'n';
const A_L_ANCHORS: &str = "anchors";
const A_S_ANCHORS: char = 'a';
const A_L_IGNORE_LINKS: &str = "ignore-links";
const A_S_IGNORE_LINKS: char = 'i';
const A_L_LINKS_FILE: &str = "links-file";
const A_S_LINKS_FILE: char = 'P';
const A_L_RESULT_FORMAT: &str = "result-format";
const A_S_RESULT_FORMAT: char = 'F';
const A_L_RESULT_EXTENDED: &str = "result-extended";
const A_S_RESULT_EXTENDED: char = 'E';
const A_L_RESULT_FLUSH: &str = "result-flush";
const A_S_RESULT_FLUSH: char = 'f';

static STDOUT_PATH: LazyLock<PathBuf> = LazyLock::new(|| PathBuf::from_str("-").unwrap());

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

fn arg_markup_files() -> Arg {
    Arg::new(A_N_MARKUP_FILES)
        .help("The markup files to extract links and/or anchors from")
        .num_args(1..)
        .value_parser(value_parser!(std::path::PathBuf))
        .value_name("MARKUP_FILE")
        .value_hint(ValueHint::DirPath)
        .action(ArgAction::Append)
        .required(true)
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
        .num_args(0..=1)
        .value_name("FILE")
        .short(A_S_ANCHORS)
        .long(A_L_ANCHORS)
        .value_parser(value_parser!(std::path::PathBuf))
        .action(ArgAction::Set)
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

static ARGS: LazyLock<[Arg; 10]> = LazyLock::new(|| {
    [
        arg_version(),
        arg_quiet(),
        arg_markup_files(),
        arg_no_links(),
        arg_anchors(),
        arg_ignore_links(),
        arg_links_file(),
        arg_result_format(),
        arg_result_extended(),
        arg_result_flush(),
    ]
});

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

fn markup_files(args: &ArgMatches) -> io::Result<Vec<PathBuf>> {
    let mut files = vec![];
    if let Some(arg_files) = args.get_many::<std::path::PathBuf>(A_N_MARKUP_FILES) {
        for arg_file in arg_files {
            files.push(arg_file.into());
        }
    }
    if files.is_empty() {
        return Err(io::Error::other("No markup files provided on the CLI"));
    }

    Ok(files)
}

fn print_version_and_exit(quiet: bool) {
    #![allow(clippy::print_stdout)]

    if !quiet {
        print!("{} ", clap::crate_name!());
    }
    println!("{}", mle::VERSION);
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

    let markup_files = markup_files(&args)?;
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
    let anchors = if args.get_raw(A_L_ANCHORS).is_none() {
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

    let ignore_links: Vec<WildMatch> = args
        .get_many::<WildMatch>(A_L_IGNORE_LINKS)
        .unwrap_or_default()
        .map(ToOwned::to_owned)
        .collect();

    let result_format = args
        .get_one::<result::Type>(A_L_RESULT_FORMAT)
        .copied()
        .unwrap_or_default();
    let result_extended = args.get_flag(A_L_RESULT_EXTENDED);
    let result_flush = args.get_flag(A_L_RESULT_FLUSH);

    Ok(Config {
        markup_files,
        links,
        anchors,
        ignore_links,
        result_format,
        result_extended,
        result_flush,
    })
}
