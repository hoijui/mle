#!/bin/bash
# SPDX-FileCopyrightText: 2022 - 2025 Robin Vobruba <hoijui.quaero@gmail.com>
# SPDX-FileCopyrightText: 2020 Armin Becher <becherarmin@gmail.com>
#
# SPDX-License-Identifier: Unlicense

# slightly adapted version of <https://stackoverflow.com/a/47541882/586229>
containsElementStartingWith () {
    local prefix="$1"
    shift
    printf '%s\0' "$@" | grep -z -- '^'"$prefix"
}

# List project files if an input-listing file is not provided
if containsElementStartingWith '-I' "$@" || containsElementStartingWith '--markup-files-list' "$@"
then
    echo mle "$@"
else
    echo mle "$@" "$(git ls-files ./**.{html,md} | tr '\n' ' ')"
fi
