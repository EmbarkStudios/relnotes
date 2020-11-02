use chrono::{DateTime, Utc};

use super::Config;

#[derive(Clone, Debug, serde::Deserialize)]
pub enum DateKind {
    Absolute(DateTime<Utc>),
    Today,
}

#[derive(Clone, Debug, serde::Deserialize)]
pub enum ReleaseKind {
    Latest,
    Absolute(String),
    RelativeFromLast(u8),
}

#[derive(Clone, Debug)]
pub enum Timeframe {
    Release(ReleaseKind),
    Date(DateKind),
}

impl Timeframe {
    pub async fn date_from_timeframe(
        &self,
        octocrab: &octocrab::Octocrab,
        config: &Config,
    ) -> Result<chrono::DateTime<chrono::Utc>, Box<dyn std::error::Error>> {
        let (owner, repo) = config
            .parent
            .clone()
            .unwrap_or_else(|| (config.owner.clone(), config.repo.clone()));
        Ok(match self {
            Timeframe::Release(ReleaseKind::Latest) => {
                octocrab
                    .repos(&owner, &repo)
                    .releases()
                    .get_latest()
                    .await?
                    .published_at
            }
            Timeframe::Release(ReleaseKind::RelativeFromLast(number)) => {
                let page = octocrab
                    .repos(&owner, &repo)
                    .releases()
                    .list()
                    .per_page(100)
                    .send()
                    .await?;

                let mut next = page.next;
                let mut releases = page.items;
                while let Some(mut page) = octocrab.get_page(&next).await? {
                    releases.append(&mut page.items);
                    next = page.next;
                }

                releases.sort_by(|a, b| b.created_at.cmp(&a.created_at));

                releases
                    .iter()
                    .nth(*number as usize)
                    .expect(&format!(
                        "Expected at least {} releases, but only {} found.",
                        number,
                        releases.len()
                    ))
                    .created_at
            }
            Timeframe::Release(ReleaseKind::Absolute(tag)) => {
                octocrab
                    .repos(&owner, &repo)
                    .releases()
                    .get_by_tag(tag)
                    .await?
                    .published_at
            }
            Timeframe::Date(DateKind::Today) => chrono::Utc::now(),
            Timeframe::Date(DateKind::Absolute(time)) => time.clone(),
        })
    }
}

impl<'de> serde::Deserialize<'de> for Timeframe {
    fn deserialize<D>(de: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        static REGEX: once_cell::sync::Lazy<regex::Regex> = once_cell::sync::Lazy::new(|| {
            regex::Regex::new(r"release:(?:(?:latest(-\d+)?)|(\S+))").unwrap()
        });

        if let Ok(s) = String::deserialize(de) {
            if let Ok(d) = s.parse() {
                Ok(Timeframe::Date(DateKind::Absolute(d)))
            } else if let Some(c) = REGEX.captures(&s) {
                Ok(if s.starts_with("release:latest") {
                    if let Some(number) = c
                        .get(1)
                        .and_then(|c| c.as_str().parse::<isize>().ok())
                        .map(|n| n.abs() as u8)
                    {
                        Timeframe::Release(ReleaseKind::RelativeFromLast(number))
                    } else {
                        Timeframe::Release(ReleaseKind::Latest)
                    }
                } else if let Some(tag) = c.get(1).map(|c| c.as_str().to_owned()) {
                    Timeframe::Release(ReleaseKind::Absolute(tag))
                } else {
                    unreachable!()
                })
            } else if s == "today" {
                Ok(Timeframe::Date(DateKind::Today))
            } else {
                Err(serde::de::Error::custom(
                    "Timeframe must be a date or relative to the last release.",
                ))
            }
        } else {
            Err(serde::de::Error::custom("Timeframe must be a string type."))
        }
    }
}
