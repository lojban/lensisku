use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use super::Version;

#[derive(Debug, Deserialize, ToSchema)]
pub struct GetVersionsQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct GetDiffQuery {
    pub from_version: i32,
    pub to_version: i32,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct VersionHistoryResponse {
    pub versions: Vec<Version>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
    pub can_revert: bool,
}
