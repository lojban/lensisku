use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

/// Query for `GET /lingo/leaderboard`.
#[derive(Debug, Deserialize, IntoParams, ToSchema)]
pub struct LingoLeaderboardQuery {
    /// Max rows to return (1–50, default 10).
    #[param(minimum = 1, maximum = 50)]
    pub limit: Option<i64>,
}

/// One ranked learner on the Lingo leaderboard.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct LingoLeaderboardEntry {
    pub user_id: i32,
    pub username: String,
    /// Study XP: sum of flashcard `review_count` × 10 (matches client session XP).
    pub points: i64,
    pub rank: i64,
    pub has_profile_image: bool,
    /// API-relative profile image path when `has_profile_image` is true
    /// (e.g. `/users/alice/profile-image`). Omitted when the user has no avatar.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile_image_url: Option<String>,
    /// True when this row is the authenticated caller (if any).
    pub is_you: bool,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct LingoLeaderboardResponse {
    pub entries: Vec<LingoLeaderboardEntry>,
    /// Caller's standing when authenticated (even if outside the top `limit`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub me: Option<LingoLeaderboardEntry>,
}
