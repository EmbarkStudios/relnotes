mod config;
mod data;

use std::path::PathBuf;

use structopt::StructOpt;

#[derive(StructOpt)]
/// Generate release notes for your project.
struct Cli {
    /// Path to the configuration file. Default: `./relnotes.toml`
    #[structopt(short, long, parse(from_os_str))]
    config: Option<PathBuf>,
    /// The version number of the release.
    version: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let mut builder = octocrab::Octocrab::builder();
    let token = std::env::var("GITHUB_TOKEN").ok();
    if token.is_some() {
        builder = builder.personal_token(token.unwrap());
    }
    octocrab::initialise(builder)?;

    let cli = Cli::from_args();
    let path = cli.config.unwrap_or_else(|| PathBuf::from("./relnotes.toml"));

    let config = {
        let string = tokio::fs::read_to_string(path.canonicalize()?).await?;
        toml::from_str::<config::Config>(&string).unwrap()
    };

    let data = data::Data::from_config(cli.version, &config).await?;
    println!("{}", tera::Tera::one_off(&config.template, &tera::Context::from_serialize(data)?, false)?);

    Ok(())
}

