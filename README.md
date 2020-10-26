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
# order in toml file).
[[categories]]
title = "Updated Dependencies"
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
{{ title }}
-----------

  {% for pr in prs %}
- [{{pr.title}}]({{pr.html_url}})
  {%- endfor %}
{% endfor %}

Uncategorised PRs
-----------------
{% for pr in prs -%}
- [{{pr.title}}]({{pr.html_url}})
{% endfor %}
"""
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
