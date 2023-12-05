<!--
SPDX-FileCopyrightText: 2022-2023 Robin Vobruba <hoijui.quaero@gmail.com>
SPDX-FileCopyrightText: 2020 Armin Becher <becherarmin@gmail.com>

SPDX-License-Identifier: CC0-1.0
-->

# Markup Link Extractor

[![License: AGPL-3.0-or-later](
    https://img.shields.io/badge/License-AGPL%203.0+-blue.svg)](
    LICENSE.txt)
[![REUSE status](
    https://api.reuse.software/badge/github.com/hoijui/mle)](
    https://api.reuse.software/info/github.com/hoijui/mle)
[![Repo](
    https://img.shields.io/badge/Repo-GitHub-555555&logo=github.svg)](
    https://github.com/hoijui/mle)
[![Package Releases](
    https://img.shields.io/crates/v/mle.svg?color=orange)](
    https://crates.io/crates/mle)
[![Documentation Releases](
    https://img.shields.io/badge/docs.rs-codify_hoijui-66c2a5?labelColor=555555&logo=docs.rs)](
    https://docs.rs/codify_hoijui)
[![downloads](
    https://badgen.net/crates/d/mle?color=blue)](
    https://crates.io/crates/mle)
[![Dependency Status](
    https://deps.rs/repo/github/hoijui/mle/status.svg)](
    https://deps.rs/repo/github/hoijui/mle)
[![Build Status](
    https://github.com/hoijui/mle/workflows/build/badge.svg)](
    https://github.com/hoijui/mle/actions)

[![In cooperation with FabCity Hamburg](
    https://raw.githubusercontent.com/osegermany/tiny-files/master/res/media/img/badge-fchh.svg)](
    https://fabcity.hamburg)
[![In cooperation with Open Source Ecology Germany](
    https://raw.githubusercontent.com/osegermany/tiny-files/master/res/media/img/badge-oseg.svg)](
    https://opensourceecology.de)

<!--
[![asciicast](
    https://asciinema.org/a/299100.svg)](
    https://asciinema.org/a/299100)
-->

Extracts links in markup files.
Currently `html` and `markdown` files are supported.
The main intended purpose of the Markup Link Extractor,
is to extract links fro ma set of files,
and then check them for validity using a separate tool,
e.g. the [Markdown Link *Checker*](https://github.com/becheran/mlc).
Together, two such tools could be integrated in your CI pipeline
to warn about broken links in your markup docs.

## Features

* Find links in `markdown` and `html` files
* Support HTML links and plain URLs in `markdown` files
* User friendly command line interface
* Easy [CI pipeline integration](#ci-pipeline-integration)
* Very fast execution using [async](https://rust-lang.github.io/async-book/)

<!--
* Throttle option to prevent *429 Too Many Requests* errors
-->

## Install Locally

There are different ways to install and use *mle*.

### Cargo

Use rust's package manager [cargo](https://doc.rust-lang.org/cargo/)
to install *mle* from [crates.io](https://crates.io/crates/mle):

``` bash
cargo install mle
```

### Download Binaries

To download a compiled binary version of *mle*
go to [github releases](https://github.com/hoijui/mle/releases)
and download the binaries compiled for `x86_64-unknown-linux-gnu`
or `x86_64-apple-darwin`.

## CI Pipeline Integration

### GitHub Actions

Use *mle* in GitHub using the *GitHub-Action*
from the [Marketplace](https://github.com/marketplace/actions/markup-link-checker-mle).

``` yaml
- name: Markup Link Extractor (mle)
  uses: hoijui/mle@v0.14.3
```

Use *mle* [command line arguments](docs/reference.md) using the `with` argument:

``` yaml
- name: Markup Link Extractor (mle)
  uses: hoijui/mle@v0.14.3
  with:
    args: ./README.md
```

### Binary

To integrate *mle* in your CI pipeline running in a *linux x86_64 environment*,
you can add the following commands to download the tool:

``` bash
curl -L https://github.com/hoijui/mle/releases/download/v0.14.3/mle -o mle
chmod +x mle
```

For example take a look at the [ntest repo](https://github.com/becheran/ntest/blob/master/.gitlab-ci.yml)
which uses *mle* in the CI pipeline.

### Docker

Use the *mle* docker image from the [docker hub](https://hub.docker.com/repository/docker/hoijui/mle),
which includes *mle*.

## Usage

Once you have *mle* installed,
it can be called from the command line.
The following call will extract all links in markup files found in the current folder
and all subdirectories:

``` bash
mle
```

Another example is to call *mle* on a certain directory or file:

``` bash
mle ./docs
```

Call *mle* with the `--help` flag to display all available cli arguments:

``` bash
mle -h
```

See the [reference](docs/reference.md) for all available command line arguments.

## Funding

This project was funded by the European Regional Development Fund (ERDF)
in the context of the [INTERFACER Project](https://www.interfacerproject.eu/),
from July 2022 (fork from [`mlc`](https://github.com/becheran/mlc)/project start)
until March 2023.

![Logo of the EU ERDF program](
    https://cloud.fabcity.hamburg/s/TopenKEHkWJ8j5P/download/logo-eu-erdf.png)
