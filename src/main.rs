mod config;
mod data;

use std::path::PathBuf;

use structopt::StructOpt;

use config::*;

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
    let mut builder = octocrab::Octocrab::builder();
    let token = std::env::var("GITHUB_TOKEN").ok();
    if token.is_some() {
        builder = builder.personal_token(token.unwrap());
    }
    octocrab::initialise(builder)?;

    let cli = Cli::from_args();
    let config: Option<config::Config> = cli.config.or_else(|| Some(PathBuf::from("./relnotes.toml"))).and_then(|path| {
        toml::from_str::<config::Config>(&std::fs::read_to_string(path.canonicalize().ok()?).ok()?).unwrap().into()
    });

    if config.is_none() {
        panic!("No configuration found, please ensure that you have a `relnotes.toml` file, or specify the path with `--config`.");
    }

    let config = config.unwrap();
    let octocrab = octocrab::instance();

    let from_date = config.from.date_from_timeframe(&octocrab, &config).await?;
    let to_date = config.to.date_from_timeframe(&octocrab, &config).await?;

    if from_date > to_date {
        panic!("`to` ({}) date is earlier than `from` ({}) date.", from_date, to_date);
    }

    println!("{} - {}", from_date, to_date);

    let repo = format!("{}/{}", config.owner, config.repo);
    const DATE_FORMAT: &str = "%Y-%m-%d";
    let date_range = format!("{}..{}", from_date.format(DATE_FORMAT), to_date.format(DATE_FORMAT));
    let query_string = format!("repo:{} is:pr is:merged merged:{}", repo, date_range);
    let page = octocrab.search()
        .issues_and_pull_requests(&query_string)
        .per_page(100u8)
        .send()
        .await?;


    let mut issues = page.items;
    let mut next = page.next;
    while let Ok(Some(mut page)) = octocrab.get_page(&next).await {
        issues.append(&mut page.items);
        next = page.next;
    }

    let mut pulls = Vec::new();
    let mut categories: std::collections::HashMap<String, Vec<octocrab::models::pulls::PullRequest>> = std::collections::HashMap::new();
    'issues: for issue in issues {
        if issue.labels.iter().any(|l| config.skip_labels.is_match(&l.name)) {
            continue;
        }

        for category in &config.categories {
            if issue.labels.iter().any(|l| category.labels.is_match(&l.name)) {
                categories.entry(category.title.clone())
                    .or_default()
                    .push(octocrab._get(issue.pull_request.unwrap().url.clone(), None::<&()>).await?.json().await?);
                continue 'issues;
            }
        }

        pulls.push(octocrab._get(issue.pull_request.unwrap().url.clone(), None::<&()>).await?.json().await?);
    }

    println!("{}", tera::Tera::one_off(&config.template, &tera::Context::from_serialize(data::Data {
        version: cli.version,
        date: to_date.format(&config.date_format).to_string(),
        categories,
        prs: pulls,
    })?, false)?);

    Ok(())
}

