// BEGIN - Embark standard lints v0.3
// do not change or add/remove here, but one can add exceptions after this section
// for more info see: <https://github.com/EmbarkStudios/rust-ecosystem/issues/59>
#![deny(unsafe_code)]
#![warn(
    clippy::all,
    clippy::await_holding_lock,
    clippy::dbg_macro,
    clippy::debug_assert_with_mut_call,
    clippy::doc_markdown,
    clippy::empty_enum,
    clippy::enum_glob_use,
    clippy::exit,
    clippy::explicit_into_iter_loop,
    clippy::filter_map_next,
    clippy::fn_params_excessive_bools,
    clippy::if_let_mutex,
    clippy::imprecise_flops,
    clippy::inefficient_to_string,
    clippy::large_types_passed_by_value,
    clippy::let_unit_value,
    clippy::linkedlist,
    clippy::lossy_float_literal,
    clippy::macro_use_imports,
    clippy::map_err_ignore,
    clippy::map_flatten,
    clippy::map_unwrap_or,
    clippy::match_on_vec_items,
    clippy::match_same_arms,
    clippy::match_wildcard_for_single_variants,
    clippy::mem_forget,
    clippy::mismatched_target_os,
    clippy::needless_borrow,
    clippy::needless_continue,
    clippy::option_option,
    clippy::pub_enum_variant_names,
    clippy::ref_option_ref,
    clippy::rest_pat_in_fully_bound_structs,
    clippy::string_add_assign,
    clippy::string_add,
    clippy::string_to_string,
    clippy::suboptimal_flops,
    clippy::todo,
    clippy::unimplemented,
    clippy::unnested_or_patterns,
    clippy::unused_self,
    clippy::verbose_file_reads,
    future_incompatible,
    nonstandard_style,
    rust_2018_idioms
)]
// END - Embark standard lints v0.3

mod config;
mod data;

use std::path::PathBuf;

use octocrab::Octocrab;
use structopt::StructOpt;

use config::timeframe::Timeframe;

#[derive(StructOpt)]
/// Generate release notes for your project.
struct Cli {
    /// Path to the configuration file. (Default: `None`)
    #[structopt(short, long, parse(from_os_str))]
    config: Option<PathBuf>,
    /// The GitHub authenication token. (Default: `None`)
    #[structopt(short, long)]
    token: Option<String>,
    /// The start of the new release timeframe. Default: `release:latest`.
    #[structopt(long)]
    from: Option<Timeframe>,
    /// The end of the new release timeframe. Default: `today`.
    #[structopt(long)]
    to: Option<Timeframe>,
    /// Skip PRs if their labels match the regular expressions.
    #[structopt(long)]
    skip_labels: Option<Vec<String>>,
    /// The repository and new version to generate release notes in the
    /// form `owner/repo@version`. `owner/repo@` is optional if provided
    /// a configuration file.
    repo_and_version: String,
}

fn initialise_github(token: Option<String>) -> eyre::Result<Octocrab> {
    let mut builder = octocrab::Octocrab::builder();
    let token = token.or_else(|| std::env::var("GITHUB_TOKEN").ok());
    if let Some(token) = token {
        builder = builder.personal_token(token);
    }
    Ok(builder.build()?)
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    );

    let cli = Cli::from_args();
    let path = cli.config.map(|path| path.canonicalize()).transpose()?;

    let (mut config, version) = if let Some(path) = path {
        log::info!("Using configuration file found at `{}`.", path.display());
        let string = tokio::fs::read_to_string(path).await?;
        (
            toml::from_str::<config::Config>(&string)?,
            cli.repo_and_version,
        )
    } else {
        let regex = regex::Regex::new(r"(?P<owner>\S+)/(?P<repo>\S+)@(?P<version>\S+)").unwrap();
        let cap = regex.captures(&cli.repo_and_version).ok_or_else(|| {
            eyre::eyre!("<repo_and_version> must be in `owner/repo@version` format.")
        })?;
        let owner = cap.name("owner").unwrap().as_str().to_owned();
        let repo = cap.name("repo").unwrap().as_str().to_owned();
        let version = cap.name("version").unwrap().as_str().to_owned();

        (config::Config::new(owner, repo), version)
    };

    config.from = cli.from.unwrap_or(config.from);
    config.to = cli.to.unwrap_or(config.to);
    config.skip_labels = cli
        .skip_labels
        .map(regex::RegexSet::new)
        .transpose()?
        .unwrap_or(config.skip_labels);

    log::info!("Using `{}` as version number.", version);
    let octocrab = initialise_github(cli.token)?;
    let data = data::Data::from_config(&octocrab, version, &config).await?;
    println!(
        "{}",
        tera::Tera::one_off(
            &config.template,
            &tera::Context::from_serialize(data)?,
            false
        )?
    );

    Ok(())
}
