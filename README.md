# 📓 Relnotes: Automatic GitHub Release Notes

[![Build Status](https://github.com/EmbarkStudios/relnotes/workflows/CI/badge.svg)](https://github.com/EmbarkStudios/relnotes/actions?workflow=CI) <!-- 
[![Crates.io](https://img.shields.io/crates/v/relnotes.svg)](https://crates.io/crates/tame-oauth)
[![Docs](https://docs.rs/tame-oauth/badge.svg)](https://docs.rs/tame-oauth)
-->
[![Contributor Covenant](https://img.shields.io/badge/contributor%20covenant-v1.4%20adopted-ff69b4.svg)](CODE_OF_CONDUCT.md)
[![Embark](https://img.shields.io/badge/embark-open%20source-blueviolet.svg)](https://embark.dev)

Relnotes is a tool to automatically generate a file containing every merged pull request for a GitHub repository. It comes with a simple configuration file and powerful templating allowing you to easily create release notes in your preferred format and organisation.

## How it works
The relnotes has two required arguments, the version and a path the configuration file (default: `./relnotes.toml` in current directory). E.g. `relnotes v1.0.0 [--config=./relnotes.toml]`. Based on the input from the configuration file `relnotes` will create a "timeframe" for the release (e.g. from the last release to today) and collect all PRs that were merged in that timeframe. Those PRs will then be filtered and categorised before being rendered using [Tera].

[tera]: https://tera.netlify.app

## Configuration File
```toml
# GitHub repository owner
owner = "EmbarkStudios"
# GitHub Repository
repo = "relnotes"
# Both `from` and `to` accept either any fixed timestamp, `today`, or
# `release:` followed by either a tag to use that tag's release date
# or `latest` to always select the latest release.
# Syntax: <date|(release:(latest|<tag>))>
#
# The start of the new release timeframe. Default: `release:latest`.
from = "release:latest"
# The end of the timeframe. Default: `today`.
to = "today"
# Format string for the `date` variable in `[template]`. Default: `%Y-%m-%d`
date-format = "%Y-%m-%d"
# Set of regular expressions that if any of the PR's labels match will
# be skipped and not included in the release notes. Default: `[]`
skip-labels = []

# A set of categories to populate the `categories` variable and to help
# organise the release notes, if any of the issues labels match the set
# of regexes in `labels` it will be placed in this category. (Priority matches
# order in toml file). Default: empty
[[categories]]
# The title of the category
title = "Updated Dependencies"
# Set of regexes to match against the labels.
labels = ["dependencies"]

# The template to generate the release notes. The `[template]` map accepts
# either a `string` literal or a `path` to the tera template to use. (Does
# not accept both.)
# Variables available
# - `version`: The version passed to `relnotes`
# - `date`: The `to` date formatted by `date_format`.
# - `categories`: A map of prs categorised by their `title`. `title -> prs`
# - `prs`: Any PRs that weren't filtered or categorised.
[template]
# path = "template.md"
string = """
Version {{version}} ({{date}})
============================

{% for title, prs in categories %}
## {{ title }}
{%- for pr in prs %}
- [{{pr.title}}]({{pr.html_url}})
  {%- endfor %}
{% endfor %}

## Uncategorised PRs
{% for pr in prs -%}
- [{{pr.title}}]({{pr.html_url}})
{% endfor %}
"""
```

## Example Output
Here's an example of the above config being run on `XAMPPRocky/tokei`.

```markdown
Version v13.0.0 (2020-10-26)
============================

## Updated Dependencies
- [Bump env_logger from 0.7.1 to 0.8.1](https://github.com/XAMPPRocky/tokei/pull/645)
- [Bump serde from 1.0.116 to 1.0.117](https://github.com/XAMPPRocky/tokei/pull/644)
- [Bump regex from 1.4.0 to 1.4.1](https://github.com/XAMPPRocky/tokei/pull/643)
- [Bump git2 from 0.13.11 to 0.13.12](https://github.com/XAMPPRocky/tokei/pull/642)
- [Bump serde_json from 1.0.58 to 1.0.59](https://github.com/XAMPPRocky/tokei/pull/641)
- [Bump aho-corasick from 0.7.13 to 0.7.14](https://github.com/XAMPPRocky/tokei/pull/639)
- [Bump regex from 1.3.9 to 1.4.0](https://github.com/XAMPPRocky/tokei/pull/638)
- [Bump toml from 0.5.6 to 0.5.7](https://github.com/XAMPPRocky/tokei/pull/637)
- [Bump serde_json from 1.0.57 to 1.0.58](https://github.com/XAMPPRocky/tokei/pull/636)
- [Bump rayon from 1.4.0 to 1.4.1](https://github.com/XAMPPRocky/tokei/pull/635)
- [Bump serde from 1.0.115 to 1.0.116](https://github.com/XAMPPRocky/tokei/pull/629)
- [Bump git2 from 0.13.10 to 0.13.11](https://github.com/XAMPPRocky/tokei/pull/625)
- [Bump crossbeam-channel from 0.4.2 to 0.4.4](https://github.com/XAMPPRocky/tokei/pull/623)
- [Bump rayon from 1.3.1 to 1.4.0](https://github.com/XAMPPRocky/tokei/pull/617)
- [Bump git2 from 0.13.6 to 0.13.10](https://github.com/XAMPPRocky/tokei/pull/615)
- [Bump once_cell from 1.4.0 to 1.4.1](https://github.com/XAMPPRocky/tokei/pull/613)
- [Bump clap from 2.33.2 to 2.33.3](https://github.com/XAMPPRocky/tokei/pull/610)
- [Bump tera from 1.3.1 to 1.5.0](https://github.com/XAMPPRocky/tokei/pull/609)
- [Bump serde from 1.0.114 to 1.0.115](https://github.com/XAMPPRocky/tokei/pull/608)
- [Bump clap from 2.33.1 to 2.33.2](https://github.com/XAMPPRocky/tokei/pull/606)
- [Bump dashmap from 3.11.9 to 3.11.10](https://github.com/XAMPPRocky/tokei/pull/603)
- [Bump dashmap from 3.11.7 to 3.11.9](https://github.com/XAMPPRocky/tokei/pull/600)
- [Bump serde_json from 1.0.56 to 1.0.57](https://github.com/XAMPPRocky/tokei/pull/596)
- [Bump log from 0.4.8 to 0.4.11](https://github.com/XAMPPRocky/tokei/pull/590)
- [Bump dirs from 2.0.2 to 3.0.1](https://github.com/XAMPPRocky/tokei/pull/584)
- [Bump dashmap from 3.11.4 to 3.11.7](https://github.com/XAMPPRocky/tokei/pull/583)
- [Bump serde_json from 1.0.55 to 1.0.56](https://github.com/XAMPPRocky/tokei/pull/579)
- [Bump parking_lot from 0.10.2 to 0.11.0](https://github.com/XAMPPRocky/tokei/pull/575)
- [Bump aho-corasick from 0.7.10 to 0.7.13](https://github.com/XAMPPRocky/tokei/pull/574)


## Uncategorised PRs
- [Add support for the Gleam language](https://github.com/XAMPPRocky/tokei/pull/646)
- [Add jsonnet to language list](https://github.com/XAMPPRocky/tokei/pull/634)
- [Add Stan language](https://github.com/XAMPPRocky/tokei/pull/633)
- [Unify format](https://github.com/XAMPPRocky/tokei/pull/631)
- [added beancount file format](https://github.com/XAMPPRocky/tokei/pull/630)
- [add Tera templating language](https://github.com/XAMPPRocky/tokei/pull/627)
- [Add support for TTCN-3](https://github.com/XAMPPRocky/tokei/pull/621)
- [Add definition for DAML](https://github.com/XAMPPRocky/tokei/pull/620)
- [Add Stylus language](https://github.com/XAMPPRocky/tokei/pull/619)
- [Add LiveScript to languages.json](https://github.com/XAMPPRocky/tokei/pull/607)
- [Added CodeQL language support](https://github.com/XAMPPRocky/tokei/pull/604)
- [Fix very minor typo in README](https://github.com/XAMPPRocky/tokei/pull/601)
- [feat: number formatted printing](https://github.com/XAMPPRocky/tokei/pull/591)
- [Make --no-ignore imply all other --no-ignore- flags](https://github.com/XAMPPRocky/tokei/pull/588)
- [Add summary information to output formats](https://github.com/XAMPPRocky/tokei/pull/580)
- [Add support for ABNF grammar specs](https://github.com/XAMPPRocky/tokei/pull/577)
- [Remove duplicated 'json' from --output](https://github.com/XAMPPRocky/tokei/pull/576)
```

## Contributing

We welcome community contributions to this project.

Please read our [Contributor Guide](CONTRIBUTING.md) for more information on how to get started.

## License

Licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
