//! Extra PoE1 patch installer.
//!
//! It overlays a small set of files from
//! `gameofthemonster/PathOfBuilding` onto the official PoB install,
//! to provide Chinese localization support for Path of Building.

use super::download::{download_and_extract_tarball, DownloadEvent, ExtractionRule};
use anyhow::Context;
use regex::Regex;
use std::path::Path;

const POB1_PATCH_REPO: &str = "gameofthemonster/PathOfBuilding";

/// Downloads and applies the PoE1 localization overlay files.
pub fn apply_pob1_patch<P: AsRef<Path>>(
    client: &reqwest::blocking::Client,
    target_dir: P,
    _pob_version: &str,
    on_progress: &mut impl FnMut(String),
) -> anyhow::Result<()> {
    let target_dir = target_dir.as_ref();

    on_progress("Resolving PoE1 patch release...".to_string());
    let latest_tag = fetch_latest_release_tag(client)?;
    on_progress(format!("Using PoE1 patch release: {latest_tag}"));

    let rules = vec![
        ExtractionRule::RewritePrefix {
            prefix: "trs/".into(),
            dest_dir: target_dir.join("trs"),
        },
        ExtractionRule::File {
            tarball_path: "src/Modules/Main.lua".into(),
            dest_path: target_dir.join("Modules").join("Main.lua"),
        },
        ExtractionRule::File {
            tarball_path: "src/Modules/Common.lua".into(),
            dest_path: target_dir.join("Modules").join("Common.lua"),
        },
        ExtractionRule::File {
            tarball_path: "src/Modules/Translation.lua".into(),
            dest_path: target_dir.join("Modules").join("Translation.lua"),
        },
        ExtractionRule::File {
            tarball_path: "src/Classes/EditControl.lua".into(),
            dest_path: target_dir.join("Classes").join("EditControl.lua"),
        },
    ];

    download_and_extract_tarball(
        client,
        POB1_PATCH_REPO,
        &latest_tag,
        &rules,
        5,
        &mut |event| {
            let msg = match event {
                DownloadEvent::Progress {
                    downloaded,
                    total: Some(total),
                } => {
                    let pct = (downloaded as f32 / total as f32 * 100.0) as u32;
                    format!("Applying PoE1 patch... ({pct}%)")
                }
                DownloadEvent::Progress {
                    downloaded,
                    total: None,
                } => {
                    format!("Applying PoE1 patch... ({downloaded} bytes)")
                }
                DownloadEvent::Retrying { attempt } => {
                    format!("Applying PoE1 patch... retrying (attempt {attempt})")
                }
            };
            on_progress(msg);
        },
    )?;

    Ok(())
}

fn fetch_latest_release_tag(client: &reqwest::blocking::Client) -> anyhow::Result<String> {
    let latest_url = format!("https://github.com/{POB1_PATCH_REPO}/releases/latest");
    let response = client
        .get(&latest_url)
        .send()
        .context("Failed to query PoE1 patch latest release")?
        .error_for_status()
        .context("PoE1 patch latest release request failed")?;

    let final_url = response.url().as_str();
    let tag_re = Regex::new(r"/releases/tag/([^/?#]+)").unwrap();
    let tag = tag_re
        .captures(final_url)
        .and_then(|caps| caps.get(1).map(|m| m.as_str().to_string()))
        .context("Failed to parse latest patch release tag from redirect URL")?;

    Ok(tag)
}
