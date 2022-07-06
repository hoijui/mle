// mod file_system;
// mod http;
// mod mail;

// pub mod link_type;

use std::sync::Arc;

use crate::link_extractors::link_extractor::MarkupAnchorTarget;
use crate::link_extractors::link_extractor::MarkupLink;
use crate::Config;
use crate::RemoteCache;
use crate::State;
use crate::link_type::LinkType;
use colored::ColoredString;
use colored::Colorize;
use reqwest::Url;
use tokio::sync::Mutex;
use async_std::fs::canonicalize;
use async_std::path::Path;
use async_std::path::PathBuf;
use std::path::MAIN_SEPARATOR;
use walkdir::WalkDir;

pub async fn resolve_target_link(
    link: &MarkupLink,
    link_type: &LinkType,
    config: &Config,
) -> String {
    if link_type == &LinkType::FileSystem {
        resolve_target_link_fs(&link.source, &link.target, config).await
    } else {
        link.target.to_string()
    }
}

/// Converts any valid file path, pointing to a en existing,
/// local file-system entry, into its canonical, absolute form.
pub async fn resolve_target_link_fs(source: &str, target: &str, config: &Config) -> String {
    let /*mut*/ normalized_link = target
        .replace('/', &MAIN_SEPARATOR.to_string())
        .replace('\\', &MAIN_SEPARATOR.to_string());
    // if let Some(idx) = normalized_link.find('#') {
    //     warn!(
    //         "Strip everything after #. The chapter (aka anchor aka fragment) part '{}' is not checked.",
    //         &normalized_link[idx..]
    //     );
    //     normalized_link = normalized_link[..idx].to_string();
    // }
    let mut fs_link_target = Path::new(&normalized_link).to_path_buf();
    if normalized_link.starts_with(MAIN_SEPARATOR) && config.root_dir.is_some() {
        match canonicalize(&config.root_dir.as_ref().unwrap()).await {
            Ok(new_root) => fs_link_target = new_root.join(Path::new(&normalized_link[1..])),
            Err(e) => panic!(
                "Root path could not be converted to an absolute path. Does the directory exit? '{}'",
                e
            ),
        }
    }

    debug!("Checking file system link target '{:?}' ...", target);
    let abs_path = absolute_target_path(source, &fs_link_target)
        .await
        .to_str()
        .unwrap_or_else(|| panic!("Could not resolve target path '{}' ", target))
        .to_string();
    // Remove verbatim path identifier which causes trouble on windows when using ../../ in paths
    abs_path
        .strip_prefix(r#"\\?\"#)
        .unwrap_or(&abs_path)
        .to_string()
}

async fn absolute_target_path(source: &str, target: &PathBuf) -> PathBuf {
    lazy_static! {
        static ref ROOT: PathBuf = PathBuf::from(&format!("{}", MAIN_SEPARATOR));
    }
    if target.is_relative() {
        let abs_source = canonicalize(source)
            .await
            .unwrap_or_else(|_| panic!("Path '{}' does not exist.", source));
        let parent = abs_source.parent().unwrap_or(&ROOT);
        let new_target = match target.strip_prefix(format!(".{}", MAIN_SEPARATOR)) {
            Ok(t) => t,
            Err(_) => target,
        };
        parent.join(new_target)
    } else {
        target.clone()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn remove_dot() {
        let source = Path::new(file!())
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("benches")
            .join("benchmark");
        let target = Path::new("./script_and_comments.md").to_path_buf();

        let path = absolute_target_path(&source.to_str().unwrap(), &target).await;

        let path_str = path.to_str().unwrap().to_string();
        println!("{:?}", path_str);
        assert_eq!(path_str.matches('.').count(), 1);
    }
}
