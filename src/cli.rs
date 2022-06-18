use crate::ignore_path::IgnorePath;
use crate::logger;
use crate::markup::MarkupType;
use crate::Config;
use clap::{App, Arg};
use std::convert::TryFrom;
use std::path::Path;
use std::path::MAIN_SEPARATOR;
use wildmatch::WildMatch;

#[must_use]
pub fn parse_args() -> Config {
    let matches = App::new(crate_name!())
        .arg(
            Arg::with_name("directory")
                .help("Check all links in given directory and subdirectory")
                .required(false)
                .index(1)
        )
        .arg(
            Arg::with_name("debug")
                .long("debug")
                .short("d")
                .help("Print debug information to console")
                .required(false)
        )
        .arg(
            Arg::with_name("no_web_links")
                .long("no-web-links")
                .help("Do not check web links")
                .required(false)
        )
        .arg(
            Arg::with_name("match_file_extension")
                .long("match-file-extension")
                .help("Do check for the exact file extension when searching for a file")
                .required(false)
        )
        .arg(
            Arg::with_name("ignore_path")
                .long("ignore-path")
                .help("List of files and directories which will not be checked")
                .long_help("One or more files or directories which will not be checked, separated by white-space.")
                .min_values(1)
                .required(false)
                .empty_values(false)
        )
        .arg(
            Arg::with_name("ignore_links")
                .long("ignore-links")
                .short("i")
                .help("List of links which will not be checked")
                .long_help("One or more wildcard-patterns/globs, matching links which will not be checked, separated by white-space.")
                .min_values(1)
                .required(false)
                .empty_values(false)
        )
        .arg(
            Arg::with_name("markup_types")
                .long("markup-types")
                .short("t")
                .help("List of markup types which shall be checked")
                .long_help("One or more markup file types which shall be checked, separated by white-space.")
                .min_values(1)
                .possible_values(&["md", "html"])
                .required(false)
                .empty_values(false)
        )
        .arg(
            Arg::with_name("throttle")
                .long("throttle")
                .help("Wait between http request to the same host for a defined number of milliseconds")
                .required(false)
                .takes_value(true)
                .empty_values(false)
        )
        .arg(
            Arg::with_name("root_dir")
                .long("root-dir")
                .takes_value(true)
                .short("r")
                .help("Path to the root folder used to resolve all relative paths")
                .required(false)
        )
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .get_matches();
    let debug = matches.is_present("debug");

    let throttle = match matches.value_of("throttle") {
        Some(v) => v
            .parse()
            .expect("Integer expected. Throttle time in milliseconds"),
        None => 0,
    };

    let log_level = if debug {
        logger::LogLevel::Debug
    } else {
        logger::LogLevel::Warn
    };
    let directory = matches
        .value_of("directory")
        .unwrap_or("./")
        .parse()
        .unwrap();

    let mut markup_types = vec![MarkupType::Markdown, MarkupType::Html];
    if let Some(types) = matches.values_of("markup_types") {
        markup_types = types.map(|x| x.parse().unwrap()).collect();
    }

    let no_web_links = matches.is_present("no_web_links");

    let match_file_extension = matches.is_present("match_file_extension");

    let ignore_links: Vec<WildMatch> = matches
        .values_of("ignore_links")
        .unwrap_or_default()
        .map(|x| WildMatch::new(x))
        .collect();

    let ignore_paths: Vec<IgnorePath> = matches
        .values_of("ignore_path")
        .unwrap_or_default()
        .map(IgnorePath::try_from)
        .collect::<Result<Vec<IgnorePath>, _>>()
        .unwrap();

    let root_dir = if let Some(root_path) = matches.value_of("root_dir") {
        let root_path = Path::new(
            &root_path
                .replace('/', &MAIN_SEPARATOR.to_string())
                .replace('\\', &MAIN_SEPARATOR.to_string()),
        )
        .to_path_buf();
        if !root_path.is_dir() {
            eprintln!("Root path {:?} must be a directory!", root_path);
            std::process::exit(1);
        }
        Some(root_path)
    } else {
        None
    };

    Config {
        log_level,
        folder: directory,
        markup_types,
        no_web_links,
        match_file_extension,
        ignore_links,
        ignore_paths,
        root_dir,
        throttle,
    }
}
