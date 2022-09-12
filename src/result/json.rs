// SPDX-FileCopyrightText: 2022 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::io::Write;

use serde::ser::SerializeSeq;
use serde::ser::SerializeStruct;

use crate::anchor::Anchor;
use crate::config::Config;
use crate::group::{self, Grouping};

pub struct Sink();

#[derive(Debug, Clone)]
pub struct RootSer<'a> {
    pub grouping: &'a Grouping<'a>,
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
        out_stream: &mut Box<dyn Write + 'static>,
        links: &Grouping,
        anchors: &[Anchor],
        errors: &[Box<dyn std::error::Error>],
    ) -> std::io::Result<()> {
        let str_errors = errors.iter().map(ToString::to_string).collect::<String>();
        let content = RootSer { grouping: links };
        let json = serde_json::to_string_pretty(&content)?;
        write!(out_stream, "{}", json)?;

        Ok(())
    }
}
