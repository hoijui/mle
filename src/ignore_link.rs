// SPDX-FileCopyrightText: 2022 Robin Vobruba <hoijui.quaero@gmail.com>
// SPDX-FileCopyrightText: 2020 Armin Becher <becherarmin@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use wildmatch::WildMatch;

/// Parses the argument into a [`WildMatch`].
///
/// # Errors
///
/// If the argument is not a valid link glob.
pub fn parse(link_glob: &str) -> Result<WildMatch, String> {
    // TODO Should be moved to an other file, probably.
    Ok(WildMatch::new(link_glob))
}
