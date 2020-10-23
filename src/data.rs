use std::collections::HashMap;

use chrono::{DateTime, Utc};
use octocrab::models::pulls::PullRequest;

#[derive(serde::Serialize)]
pub struct Data {
    pub version: String,
    pub date: String,
    pub categories: HashMap<String, Vec<PullRequest>>,
    pub prs: Vec<PullRequest>,
}
