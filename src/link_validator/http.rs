use crate::link_extractors;
use crate::link_validator::LinkCheckResult;
use crate::markup::Content;
use crate::markup_type;
use crate::AnchorTargets;
use crate::MarkupAnchorTarget;
use crate::MarkupFile;
use crate::State;
use std::collections::HashMap;

use reqwest::header::ACCEPT;
use reqwest::header::USER_AGENT;
use reqwest::Client;
use reqwest::Method;
use reqwest::Request;
use reqwest::Response;
use reqwest::StatusCode;

pub async fn check_http(
    state: &mut State,
    target: &str,
    anchors_required: bool,
) -> (LinkCheckResult, AnchorTargets) {
    debug!("Checking http link target '{:?}' ...", target);
    let url = reqwest::Url::parse(target).expect("URL of unknown type");

    match http_request(state, &url, anchors_required).await {
        Ok(res) => res,
        // Ok((response, parsed_anchors)) => {
        //     if parsed_anchors.is_some() {
        //         let rem_anchs = if remote_anchors.contains_key(&url) {
        //             TODO
        //         }
        //         remote_anchors.get_mut(&url).unwrap_or_else(|| {
        //             let urls_cache: Vec<MarkupAnchorTarget> = vec![];
        //             let opt = Ok(urls_cache);
        //             remote_anchors[&url] = opt;
        //             // &mut opt
        //             remote_anchors.get_mut(&url).unwrap()
        //         }).unwrap().append(&mut parsed_anchors.unwrap());
        //     }
        //     response
        // },
        Err(error_msg) => (
            LinkCheckResult::Failed(format!("Http(s) request failed: {}", error_msg)),
            None,
        ),
    }
}

fn new_request(method: Method, url: &reqwest::Url) -> Request {
    let mut req = Request::new(method, url.clone());
    let headers = req.headers_mut();
    headers.insert(ACCEPT, "text/html, text/markdown".parse().unwrap());
    headers.insert(USER_AGENT, "mlc (github.com/becheran/mlc)".parse().unwrap());
    req
}

fn extract_file_name(url: &reqwest::Url) -> &str {
    url.path()
}

async fn http_request(
    state: &mut State,
    url: &reqwest::Url,
    anchors_required: bool,
) -> reqwest::Result<(LinkCheckResult, AnchorTargets)> {
    lazy_static! {
        static ref CLIENT: Client = Client::new();
    }

    fn status_to_string(status: StatusCode) -> String {
        format!(
            "{} - {}",
            status.as_str(),
            status.canonical_reason().unwrap_or("Unknown reason")
        )
    }

    let primary_request = if anchors_required {
        new_request(Method::GET, url)
    } else {
        new_request(Method::HEAD, url)
    };

    let response = match CLIENT.execute(primary_request).await {
        Ok(r) => r,
        Err(e) => {
            if !anchors_required {
                println!("Head request error: {}. Retry with get-request.", e);
                let secondary_request = new_request(Method::GET, url);
                CLIENT.execute(secondary_request).await?
            } else {
                return Err(e);
            }
        }
    };

    let status = response.status();
    if status.is_success() {
        let anchor_targets = if anchors_required {
            //response;

            // 1. write response content (== file content(?)) to a temporary file (possible in RAM?))
            // TODO
            //response.text()

            // 3. delete it
            // 4. return parse results

            // 2. parse it
            let file_name = extract_file_name(&url);
            let markup_type = markup_type(&file_name, &state.config.markup_types);
            if markup_type.is_none() {
                None
            } else {
                let content = response.text().await?;
                let a_m_file = MarkupFile {
                    markup_type: markup_type.unwrap(),
                    locator: &url.to_string(),
                    // content: Content::InMemory(&response.text().await.expect("Failed to get remote markup file content")?),
                    // content: Content::InMemory(&response.text().await?),
                    content: { Content::InMemory(&content) },
                };
                let (_, mut anch_targets) =
                    link_extractors::link_extractor::find_links(&a_m_file, true);
                //Some(anchors.append(&mut anch_targets));

                // 3. delete it
                // TODO

                // 4. return parse results
                Some(anch_targets)
            }
        } else {
            None
        };
        Ok((LinkCheckResult::Ok, anchor_targets))
    } else if status.is_redirection() {
        Ok((LinkCheckResult::Warning(status_to_string(status)), None))
    } else {
        if anchors_required {
            Err(response.error_for_status().unwrap_err())
        } else {
            debug!("Got the status code {:?}. Retry with get-request.", status);
            let get_request = Request::new(Method::GET, url.clone());
            let response = CLIENT.execute(get_request).await?;
            let status = response.status();
            if status.is_success() {
                Ok((LinkCheckResult::Ok, None))
            } else {
                Ok((LinkCheckResult::Failed(status_to_string(status)), None))
            }
        }
    }
}

// #[cfg(test)]
// mod test {
//     use super::*;

//     #[tokio::test]
//     async fn check_http_is_available() {
//         let mut anchors_cache = HashMap::new();
//         let result = check_http("http://gitlab.com/becheran/mlc", None, &mut anchors_cache).await;
//         assert_eq!(result, LinkCheckResult::Ok);
//     }

//     #[tokio::test]
//     async fn check_link_available_issue_28() {
//         let result = check_http("https://deps.rs/repo/github/stanislav-tkach/os_info").await;
//         assert_eq!(result, LinkCheckResult::Ok);
//     }

//     #[tokio::test]
//     async fn check_https_crates_io_available() {
//         let result = check_http("https://crates.io").await;
//         assert_eq!(result, LinkCheckResult::Ok);
//     }

//     #[tokio::test]
//     async fn check_http_request_with_hash() {
//         let result = check_http("http://gitlab.com/becheran/mlc#bla").await;
//         assert_eq!(result, LinkCheckResult::Ok);
//     }

//     #[tokio::test]
//     async fn check_wrong_http_request() {
//         let result = check_http("https://doesNotExist.me/even/less/likelly").await;
//         assert!(result != LinkCheckResult::Ok);
//     }
// }
