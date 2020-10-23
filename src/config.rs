use std::{collections::HashMap, path::PathBuf};

use regex::RegexSet;
use serde::Deserialize;
use chrono::{DateTime, Utc};

fn default_from() -> Timeframe {
    Timeframe::Release(ReleaseKind::Latest)
}

fn default_to() -> Timeframe {
    Timeframe::Date(DateKind::Today)
}

fn parse_timeframe<'de, D>(de: D) -> Result<Timeframe, D::Error> where D: serde::Deserializer<'de> {
    static REGEX: once_cell::sync::Lazy<regex::Regex> = once_cell::sync::Lazy::new(|| {
        regex::Regex::new(r"release:(?:(?:latest)|(\S+))").unwrap()
    });

    if let Ok(s) = String::deserialize(de) {
        if let Ok(d) = s.parse() {
            Ok(Timeframe::Date(DateKind::Absolute(d)))
        } else if let Some(c) = REGEX.captures(&s) {
            Ok(if let Some(tag) = c.get(1).map(|c| c.as_str().to_owned()) {
                Timeframe::Release(ReleaseKind::Absolute(tag))
            } else {
                Timeframe::Release(ReleaseKind::Latest)
            })
        } else if s == "today" {
            Ok(Timeframe::Date(DateKind::Today))
        } else {
            Err(serde::de::Error::custom("Timeframe must be a date or relative to the last release."))
        }
    } else {
        Err(serde::de::Error::custom("Timeframe must be a string type."))
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
    #[serde(deserialize_with = "parse_timeframe")]
    #[serde(default = "default_from")]
    pub from: Timeframe,
    #[serde(deserialize_with = "parse_timeframe")]
    #[serde(default = "default_to")]
    pub to: Timeframe,
    pub owner: String,
    pub repo: String,
    pub date_format: String,
    #[serde(deserialize_with = "from_regex_set")]
    #[serde(default = "default_regex_set")]
    pub skip_labels: RegexSet,
    #[serde(default)]
    pub categories: Vec<Category>,
    pub template: Template,
}

#[derive(Debug)]
pub struct Template(String);

impl std::ops::Deref for Template {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'de> Deserialize<'de> for Template {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "lowercase")]
        enum Field { Path, String }
        struct TemplateVisitor;

        impl<'de> serde::de::Visitor<'de> for TemplateVisitor {
            type Value = Template;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a map with either `path` pointing to a template file, or `string`")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Template, V::Error>
            where
                V: serde::de::MapAccess<'de>,
            {
                use serde::de;

                let mut path: Option<PathBuf> = None;
                let mut string = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Path => {
                            if path.is_some() {
                                return Err(de::Error::duplicate_field("path"));
                            }
                            path = Some(map.next_value()?);
                        }
                        Field::String => {
                            if string.is_some() {
                                return Err(de::Error::duplicate_field("string"));
                            }
                            string = Some(map.next_value()?);
                        }
                    }
                }

                let string = if let Some(path) = path {
                    std::fs::read_to_string(path).map_err(|e| de::Error::custom(e))?
                } else if let Some(s) = string {
                    s
                } else {
                    return Err(de::Error::custom("template file not found and no `string` key provided."))
                };

                Ok(Template(string))
            }
        }

        const FIELDS: &'static [&'static str] = &["path", "string"];
        deserializer.deserialize_struct("Duration", FIELDS, TemplateVisitor)
    }
}

#[derive(Debug, Deserialize)]
pub struct Category {
    pub title: String,
    #[serde(deserialize_with = "from_regex_set")]
    #[serde(default = "default_regex_set")]
    pub labels: RegexSet,
}

fn default_regex_set() -> RegexSet {
    // const to get around type inference issues.
    const EMPTY: &[&str] = &[];
    RegexSet::new(EMPTY).unwrap()
}

fn from_regex_set<'de, D>(de: D) -> Result<RegexSet, D::Error> where D: serde::Deserializer<'de> {
    let list: Option<Vec<String>> = <_>::deserialize(de)?;
    let list = list.unwrap_or_default();

    regex::RegexSet::new(list).map_err(|_| serde::de::Error::custom("Category labels were not valid regular expressions"))
}

#[derive(Debug, serde::Deserialize)]
pub enum Timeframe {
    Release(ReleaseKind),
    Date(DateKind),
}

impl Timeframe {
    pub async fn date_from_timeframe(&self, octocrab: &octocrab::Octocrab, config: &Config) -> Result<chrono::DateTime<chrono::Utc>, Box<dyn std::error::Error>> {
        Ok(match self {
            Timeframe::Release(ReleaseKind::Latest) => {
                octocrab.repos(&config.owner, &config.repo).releases().get_latest().await?.published_at
            }
            Timeframe::Release(ReleaseKind::Absolute(tag)) => {
                octocrab.repos(&config.owner, &config.repo).releases().get_by_tag(tag).await?.published_at
            }
            Timeframe::Release(ReleaseKind::RelativeFromLast(number)) => {
                let mut releases = octocrab.repos(&config.owner, &config.repo)
                    .releases()
                    .list()
                    .per_page(100)
                    .send()
                    .await?
                    .items;

                releases.sort_by(|a, b| b.created_at.partial_cmp(&a.created_at).unwrap());
                releases.get(*number as usize).unwrap().created_at
            }
            Timeframe::Date(DateKind::Today) => chrono::Utc::now(),
            Timeframe::Date(DateKind::Absolute(time)) => time.clone(),
        })
    }
}

#[derive(Debug, serde::Deserialize)]
pub enum ReleaseKind {
    Latest,
    Absolute(String),
    RelativeFromLast(u8),
}

#[derive(Debug, serde::Deserialize)]
pub enum DateKind {
    Absolute(DateTime<Utc>),
    Today,
}
