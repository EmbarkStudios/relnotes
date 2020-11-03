mod template;
mod timeframe;

use regex::RegexSet;
use serde::Deserialize;

pub use template::*;
pub use timeframe::*;

fn default_from() -> Timeframe {
    Timeframe::Release(ReleaseKind::Latest)
}

fn default_to() -> Timeframe {
    Timeframe::Date(DateKind::Today)
}

fn default_date_format() -> String {
    String::from("%Y-%m-%d")
}

#[derive(Clone, Debug, Deserialize)]
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

fn from_optional_regex_set<'de, D>(de: D) -> Result<Option<RegexSet>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let list: Option<Vec<String>> = <_>::deserialize(de)?;

    if list.is_none() {
        return Ok(None);
    }

    regex::RegexSet::new(list.unwrap())
        .map(Some)
        .map_err(|_| serde::de::Error::custom("Category labels were not valid regular expressions"))
}

fn from_regex_set<'de, D>(de: D) -> Result<RegexSet, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let regex_set = from_optional_regex_set(de)?;

    if regex_set.is_none() {
        Err(serde::de::Error::custom("Label RegexSet not found."))
    } else {
        Ok(regex_set.unwrap())
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
    #[serde(default = "default_from")]
    pub from: Timeframe,
    #[serde(default = "default_to")]
    pub to: Timeframe,
    pub owner: String,
    pub repo: String,
    pub title: Option<String>,
    #[serde(default = "default_date_format")]
    pub date_format: String,
    #[serde(deserialize_with = "from_regex_set")]
    #[serde(default = "default_regex_set")]
    pub skip_labels: RegexSet,
    #[serde(default)]
    pub categories: Vec<Category>,
    pub template: Template,
    #[serde(default)]
    includes: Vec<IncludeConfig>,
    #[serde(default)]
    parent: Option<(String, String)>,
}

impl Config {
    pub fn new(owner: String, repo: String) -> Self {
        Self {
            categories: Vec::new(),
            date_format: default_date_format(),
            from: default_from(),
            includes: Vec::new(),
            owner,
            parent: None,
            repo,
            skip_labels: default_regex_set(),
            template: Template::default(),
            title: None,
            to: default_to(),
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct IncludeConfig {
    pub owner: String,
    pub repo: String,
    pub title: Option<String>,
    pub from: Option<Timeframe>,
    pub to: Option<Timeframe>,
    pub date_format: Option<String>,
    #[serde(deserialize_with = "from_optional_regex_set")]
    #[serde(default)]
    pub skip_labels: Option<RegexSet>,
    pub categories: Option<Vec<Category>>,
    #[serde(default)]
    pub use_parent_for_timeframe: bool,
}

impl Config {
    pub fn includes(&self) -> Vec<Self> {
        self.includes
            .iter()
            .cloned()
            .map(|ic| {
                let parent = if ic.use_parent_for_timeframe {
                    Some((self.owner.clone(), self.repo.clone()))
                } else {
                    None
                };

                Self {
                    owner: ic.owner,
                    repo: ic.repo,
                    title: ic.title,
                    from: ic.from.unwrap_or_else(|| self.from.clone()),
                    to: ic.to.unwrap_or_else(|| self.to.clone()),
                    date_format: ic.date_format.unwrap_or_else(|| self.date_format.clone()),
                    skip_labels: ic.skip_labels.unwrap_or_else(|| self.skip_labels.clone()),
                    categories: ic.categories.unwrap_or_else(|| self.categories.clone()),
                    template: self.template.clone(),
                    includes: Vec::new(),
                    parent,
                }
            })
            .collect()
    }
}
