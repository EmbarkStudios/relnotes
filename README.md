# 📓 Relnotes: Automatic GitHub Release Notes

[![Build Status](https://github.com/EmbarkStudios/relnotes/workflows/CI/badge.svg)](https://github.com/EmbarkStudios/relnotes/actions?workflow=CI) <!-- 
[![Crates.io](https://img.shields.io/crates/v/relnotes.svg)](https://crates.io/crates/tame-oauth)
[![Docs](https://docs.rs/tame-oauth/badge.svg)](https://docs.rs/tame-oauth)
-->
[![Contributor Covenant](https://img.shields.io/badge/contributor%20covenant-v1.4%20adopted-ff69b4.svg)](CODE_OF_CONDUCT.md)
[![Embark](https://img.shields.io/badge/embark-open%20source-blueviolet.svg)](https://embark.dev)

Relnotes is a tool to automatically generate a file containing every merged pull request for a GitHub repository. It comes with a simple configuration file and powerful templating allowing you to easily create release notes in your preferred format and organisation.

## How it works
## Features

- Automatically gets all merged PRs since the last release, and can be configured for different release schedules.
- Automatic list of contributors for the release.
- Label filtering and categorisation using regular expressions.
- Powerful configuration file for advanced layouts.
- Supports collecting changes from multiple repositories.
- Uses [Tera] templates for release notes format.

[tera]: https://tera.netlify.app

## How It Works
The basic usage of `relnotes` works by providing the repository and version of new the release. For example if you wanted the release notes of a potential new `0.3.0` release of [rust-gpu], you would run the following.

```
relnotes EmbarkStudios/rust-gpu@0.3.0
```

This will generate the following markdown:

```markdown
# rust-gpu 0.3.0 (2020-12-29)

- [Update .cargo/config Shader Compilation Setup](https://github.com/EmbarkStudios/rust-gpu/pull/356)
- [Upgrade winit v0.23 -> v0.24](https://github.com/EmbarkStudios/rust-gpu/pull/353)
- [Update spirv-tools](https://github.com/EmbarkStudios/rust-gpu/pull/351)
- [Renamed spirv-attrib to spirv-std-macros](https://github.com/EmbarkStudios/rust-gpu/pull/347)
<!-- Shortened for brevity -->

## Contributors

- [DGriffin91](https://github.com/DGriffin91)
- [Hentropy](https://github.com/Hentropy)
- [Jake-Shadle](https://github.com/Jake-Shadle)
- [VZout](https://github.com/VZout)
<!-- Shortened for brevity -->
```


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

# Additional repositories to include in the release notes. It has all
# of the same properties as root (except `includes`), and inherits root's
# configuration if omitted.
[[includes]]
owner = "owner"
repo = "repo"
# Gets the timeframe from the root repository rather than the `includes`
# repository.
uses-parent-for-timeframe = false
# from = "release:latest"
# to = "today"
# date-format = "%Y-%m-%d"
# skip-labels = []
# [[includes.categories]]

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
