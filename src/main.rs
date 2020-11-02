#![deny(clippy::unimplemented, clippy::ok_expect, clippy::mem_forget)]
// Our standard Clippy lints that we use in Embark projects, we opt out of a few that are not appropriate for the specific crate (yet)
#![warn(
    clippy::all,
    clippy::doc_markdown,
    clippy::dbg_macro,
    clippy::todo,
    clippy::empty_enum,
    clippy::enum_glob_use,
    clippy::pub_enum_variant_names,
    clippy::mem_forget,
    clippy::use_self,
    clippy::filter_map_next,
    clippy::needless_continue,
    clippy::needless_borrow,
    clippy::match_wildcard_for_single_variants,
    clippy::if_let_mutex,
    clippy::mismatched_target_os,
    clippy::await_holding_lock,
    clippy::match_on_vec_items,
    clippy::imprecise_flops,
    clippy::suboptimal_flops,
    clippy::lossy_float_literal,
    clippy::rest_pat_in_fully_bound_structs,
    clippy::fn_params_excessive_bools,
    clippy::exit,
    clippy::inefficient_to_string,
    clippy::linkedlist,
    clippy::macro_use_imports,
    clippy::option_option,
    clippy::verbose_file_reads,
    clippy::unnested_or_patterns,
    rust_2018_idioms,
    future_incompatible,
    nonstandard_style
)]

mod config;
mod data;

use std::path::PathBuf;
use std::sync::Arc;

use structopt::StructOpt;
use octocrab::Octocrab;

#[derive(StructOpt)]
/// Generate release notes for your project.
struct Cli {
    /// Path to the configuration file. (Default: `./relnotes.toml`)
    #[structopt(short, long, parse(from_os_str))]
    config: Option<PathBuf>,
    /// The GitHub authenication token. (Default: `None`)
    #[structopt(short, long)]
    token: Option<String>,
    /// The version number for the release.
    version: String,
}

fn initialise_github(token: Option<String>) -> Result<Arc<Octocrab>, Box<dyn std::error::Error>> {
    let mut builder = octocrab::Octocrab::builder();
    let token = token.or_else(|| std::env::var("GITHUB_TOKEN").ok());
    if token.is_some() {
        builder = builder.personal_token(token.unwrap());
    }
    Ok(octocrab::initialise(builder)?)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    );

    let cli = Cli::from_args();
    let path = cli
        .config
        .unwrap_or_else(|| PathBuf::from("./relnotes.toml"));

    let config = {
        let string = tokio::fs::read_to_string(path.canonicalize()?).await?;
        toml::from_str::<config::Config>(&string).unwrap()
    };

    let octocrab = initialise_github(cli.token)?;
    let data = data::Data::from_config(octocrab, cli.version, &config).await?;
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
