// SPDX-FileCopyrightText: 2022 Robin Vobruba <hoijui.quaero@gmail.com>
// SPDX-FileCopyrightText: 2020 Armin Becher <becherarmin@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fmt;

use crate::link::Locator;

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum Type {
    /// An anchor associated to a title, auto generated from the title
    TitleAuto,
    /// An anchor associated to a title, manually defined for the title
    TitleManual,
    /// A dedicated anchor, defined as such (`<a name="..."/>` or `<a id="..."/>`)
    Direct,
    /// An anchor associated to an HTML element (e.g. a div)
    ElementId,
}

/// Anchor target found in markup files
///
/// In HTML, these look like:
/// <a name="manual-anchor">target part</a>
/// <a id="manual-anchor">target part</a>
/// <p id="manual-anchor">target part</p>
/// <div id="manual-anchor">target part</div>
/// <... id="manual-anchor">target part</...>
///
/// In Markdown - in addition to the HTML form -
/// different MD flavors support different anchors:
/// * GFM supplies auto-generated anchors for headers,
///   using the following rules:
///   1. downcase the headline
///   2. remove anything that is not a letter, number, space or hyphen
///   3. change any space to a hyphen
///   so `# My 1. @#%^$^-cool header!!` will have the anchor `my-1--cool-header`
/// * Pandoc MD supports similar (but sadly not equal) auto-generated anchors,
///   or additionally manually set anchors for headers,
///   using the following syntax:
///   `# My header {#manual-anchor}`
///
#[derive(PartialEq, Clone)]
pub struct Anchor {
    /// Where the anchor was found
    pub source: Locator,
    /// The anchor name (the thing one links to)
    pub name: String,
    /// The anchor type
    pub r#type: Type,
}

impl fmt::Debug for Anchor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}#{} (type {:#?})", self.source, self.name, self.r#type)
    }
}

impl fmt::Display for Anchor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}#{}", self.source, self.name)
    }
}
