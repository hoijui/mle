#!/bin/bash
# SPDX-FileCopyrightText: 2022 - 2025 Robin Vobruba <hoijui.quaero@gmail.com>
# SPDX-FileCopyrightText: 2020 Armin Becher <becherarmin@gmail.com>
#
# SPDX-License-Identifier: Unlicense

if [ "$#" -gt 1 ]
then
    mle "$@"
else
    mle "$(git ls-files ./**.{html,md})"
fi
