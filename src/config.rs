use std::path::PathBuf;

use wildmatch::WildMatch;

use crate::{ignore_path::IgnorePath, logger::LogLevel, markup::Type};

const PARALLEL_REQUESTS: usize = 20;

#[derive(Default, Debug, Clone)]
pub struct Config {
    pub log_level: LogLevel,
    pub log_file: Option<PathBuf>,
    pub scan_root: PathBuf,
    pub recursive: bool,
    pub links: bool,
    pub anchors: bool,
    // pub match_file_extension: bool,
    pub ignore_paths: Vec<IgnorePath>,
    pub ignore_links: Vec<WildMatch>,
    pub markup_types: Vec<Type>,
    pub resolve_root: Option<PathBuf>,
    // pub dry: bool,
    pub result_file: Option<PathBuf>,
    pub result_format: String, // TODO Make this an enum
}
