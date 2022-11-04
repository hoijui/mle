// SPDX-FileCopyrightText: 2022 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::io::Write;

use serde::ser::SerializeSeq;
use serde::ser::SerializeStruct;
use serde::Serialize;

use crate::anchor::Anchor;
use crate::config::Config;
use crate::group::{self, Grouping};
use crate::BoxError;

use super::Writer;

pub struct Sink();

#[derive(Debug, Clone)]
pub struct RootSer<'a> {
    pub grouping: &'a Grouping<'a>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RootSerAnchors<'a> {
    pub anchors: &'a Vec<Anchor>,
}

#[derive(Debug, Clone)]
pub struct GroupSer<'a> {
    pub id: &'a group::Id<'a>,
    pub items: &'a group::Items<'a>,
}

impl<'a> ::serde::Serialize for RootSer<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut groups_state = serializer.serialize_seq(Some(self.grouping.len()))?;
        for group in self.grouping {
            let group_ser = GroupSer {
                id: &group.0,
                items: &group.1,
            };
            groups_state.serialize_element(&group_ser)?;
        }
        groups_state.end()
    }
}

impl<'a> ::serde::Serialize for GroupSer<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("LinkGroup", 2)?;
        state.serialize_field("id", &self.id.to_string())?;
        state.serialize_field("items", &self.items)?;
        state.end()
    }
}

impl super::Sink for Sink {
    fn write_results(
        &self,
        _config: &Config,
        links_stream: Writer,
        anchors_stream: Writer,
        links: &Grouping,
        anchors: &[Anchor],
        errors: &[BoxError],
    ) -> std::io::Result<()> {
        if let Some(mut links_writer) = links_stream {
            let content = RootSer { grouping: links };
            let json = serde_json::to_string_pretty(&content)?;
            write!(links_writer, "{}", json)?;
        }
        if let Some(mut anchors_writer) = anchors_stream {
            let content = RootSerAnchors {
                anchors: &anchors.into(),
            };
            let json = serde_json::to_string_pretty(&content)?;
            write!(anchors_writer, "{}", json)?;
        }

        let str_errors = errors.iter().map(ToString::to_string).collect::<String>();

        Ok(())
    }
}
