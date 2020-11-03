use std::collections::HashMap;

use octocrab::{Octocrab, models::pulls::PullRequest};

#[derive(Debug, serde::Serialize)]
pub struct Data {
    categories: HashMap<String, Vec<PullRequest>>,
    date: String,
    includes: Vec<Data>,
    owner: String,
    prs: Vec<PullRequest>,
    repo: String,
    title: String,
    version: String,
}

impl Data {
    #[async_recursion::async_recursion]
    pub async fn from_config(
        octocrab: &Octocrab,
        version: String,
        config: &crate::config::Config,
    ) -> eyre::Result<Self> {
        const DATE_FORMAT: &str = "%Y-%m-%d";
        log::debug!("Config: {:#?}", &config);

        let from_date = config.from.date_from_timeframe(&octocrab, &config).await?;
        let to_date = config.to.date_from_timeframe(&octocrab, &config).await?;

        if from_date > to_date {
            panic!(
                "`to` ({}) date is earlier than `from` ({}) date.",
                from_date, to_date
            );
        }

        log::info!(
            "Getting PRs from `{owner}/{repo}` {from} to {to}...",
            owner = config.owner,
            repo = config.repo,
            from = from_date.format(DATE_FORMAT),
            to = to_date.format(DATE_FORMAT),
        );

        let repo = format!("{}/{}", config.owner, config.repo);
        let date_range = format!(
            "{}..{}",
            from_date.format(DATE_FORMAT),
            to_date.format(DATE_FORMAT)
        );
        let query_string = format!("repo:{} is:pr is:merged merged:{}", repo, date_range);
        let page = octocrab
            .search()
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
        let mut categories: HashMap<_, Vec<_>> = HashMap::new();

        'issues: for issue in issues {
            if issue
                .labels
                .iter()
                .any(|l| config.skip_labels.is_match(&l.name))
            {
                continue;
            }

            for category in &config.categories {
                if issue
                    .labels
                    .iter()
                    .any(|l| category.labels.is_match(&l.name))
                {
                    let body = octocrab
                        ._get(issue.pull_request.unwrap().url.clone(), None::<&()>)
                        .await?
                        .text()
                        .await?;
                    categories
                        .entry(category.title.clone())
                        .or_default()
                        .push(serde_json::from_str(&body)?);
                    continue 'issues;
                }
            }

            let body = octocrab
                ._get(issue.pull_request.unwrap().url.clone(), None::<&()>)
                .await?
                .text()
                .await?;
            pulls.push(serde_json::from_str(&body)?);
        }

        let mut includes = Vec::new();
        for include in config.includes() {
            includes.push(Self::from_config(&octocrab, version.clone(), &include).await?);
        }

        Ok(Self {
            version: version,
            owner: config.owner.clone(),
            repo: config.repo.clone(),
            title: config.title.clone().unwrap_or_else(|| config.repo.clone()),
            date: to_date.format(&config.date_format).to_string(),
            categories,
            includes,
            prs: pulls,
        })
    }
}
