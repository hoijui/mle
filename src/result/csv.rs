// SPDX-FileCopyrightText: 2022 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use csv;

use crate::config::Config;
use crate::BoxError;
use crate::{anchor::Anchor, link::Link};

use super::{AnchorRec, LinkRec, Writer};

pub struct Sink();

impl super::Sink for Sink {
    fn write_results(
        &self,
        config: &Config,
        links_stream: Writer,
        anchors_stream: Writer,
        links: &[Link],
        anchors: &[Anchor],
        errors: &[BoxError],
    ) -> std::io::Result<()> {
        let extended = config.result_extended;

        if let Some(links_writer) = links_stream {
            let mut wtr = csv::Writer::from_writer(links_writer);
            for lnk in links {
                let rec = LinkRec::new(lnk, extended);
                wtr.serialize(rec)?;
            }
            wtr.flush()?;
        }

        if let Some(anchors_writer) = anchors_stream {
            let mut wtr = csv::Writer::from_writer(anchors_writer);
            for anc in anchors {
                let rec = AnchorRec::new(anc, extended);
                wtr.serialize(rec)?;
            }
            wtr.flush()?;
        }

        super::write_to_stderr(errors)
    }
}
