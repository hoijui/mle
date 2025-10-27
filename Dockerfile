# SPDX-FileCopyrightText: 2022 - 2025 Robin Vobruba <hoijui.quaero@gmail.com>
# SPDX-FileCopyrightText: 2020 Armin Becher <becherarmin@gmail.com>
#
# SPDX-License-Identifier: Unlicense

FROM ubuntu:24.04

RUN apt-get update; apt-get install -y ca-certificates; update-ca-certificates
ADD ./target/release/mle /bin/mle
RUN PATH=$PATH:/bin/mle
