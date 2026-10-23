use deadpool_postgres::Pool;

use super::dto::{LingoLeaderboardEntry, LingoLeaderboardQuery, LingoLeaderboardResponse};
use crate::{AppError, AppResult};

/// XP awarded per flashcard review on the server leaderboard (same as client session XP).
const XP_PER_REVIEW: i64 = 10;

fn clamp_limit(limit: Option<i64>) -> i64 {
    limit.unwrap_or(10).clamp(1, 50)
}

fn profile_image_url(username: &str, has_profile_image: bool) -> Option<String> {
    if !has_profile_image {
        return None;
    }
    Some(format!(
        "/users/{}/profile-image",
        urlencoding::encode(username)
    ))
}

fn entry_from_row(row: &tokio_postgres::Row, is_you: bool) -> LingoLeaderboardEntry {
    let reviews: i64 = row.get("reviews");
    let username: String = row.get("username");
    let has_profile_image: bool = row.get("has_profile_image");
    LingoLeaderboardEntry {
        user_id: row.get("user_id"),
        profile_image_url: profile_image_url(&username, has_profile_image),
        username,
        points: reviews.saturating_mul(XP_PER_REVIEW),
        rank: row.get("rank"),
        has_profile_image,
        is_you,
    }
}

/// Top learners by flashcard review activity (public).
///
/// Points = `SUM(user_flashcard_progress.review_count) * 10`. Disabled accounts
/// (`password = 'DISABLED'`) are excluded.
pub async fn list_leaderboard(
    pool: &Pool,
    query: &LingoLeaderboardQuery,
    viewer_user_id: Option<i32>,
) -> AppResult<LingoLeaderboardResponse> {
    let limit = clamp_limit(query.limit);
    let client = pool
        .get()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let rows = client
        .query(
            "WITH scored AS (
                 SELECT
                     u.userid AS user_id,
                     u.username,
                     COALESCE(SUM(ufp.review_count), 0)::bigint AS reviews,
                     has_profile_image(u.userid) AS has_profile_image
                 FROM users u
                 JOIN user_flashcard_progress ufp ON ufp.user_id = u.userid
                 WHERE u.password != 'DISABLED'
                   AND LOWER(u.role::text) <> 'guest'
                 GROUP BY u.userid, u.username
                 HAVING COALESCE(SUM(ufp.review_count), 0) > 0
             ),
             ranked AS (
                 SELECT
                     user_id,
                     username,
                     reviews,
                     has_profile_image,
                     ROW_NUMBER() OVER (ORDER BY reviews DESC, username ASC) AS rank
                 FROM scored
             )
             SELECT user_id, username, reviews, has_profile_image, rank
             FROM ranked
             ORDER BY rank
             LIMIT $1",
            &[&limit],
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let mut entries: Vec<LingoLeaderboardEntry> = rows
        .iter()
        .map(|row| {
            let uid: i32 = row.get("user_id");
            let is_you = viewer_user_id == Some(uid);
            entry_from_row(row, is_you)
        })
        .collect();

    let me = match viewer_user_id {
        Some(uid) => {
            if let Some(existing) = entries.iter().find(|e| e.user_id == uid) {
                Some(existing.clone())
            } else {
                fetch_user_standing(&client, uid).await?
            }
        }
        None => None,
    };

    if let Some(ref me_entry) = me {
        for entry in &mut entries {
            entry.is_you = entry.user_id == me_entry.user_id;
        }
    }

    Ok(LingoLeaderboardResponse { entries, me })
}

async fn fetch_user_standing(
    client: &deadpool_postgres::Client,
    user_id: i32,
) -> AppResult<Option<LingoLeaderboardEntry>> {
    let row = client
        .query_opt(
            "WITH scored AS (
                 SELECT
                     u.userid AS user_id,
                     u.username,
                     COALESCE(SUM(ufp.review_count), 0)::bigint AS reviews,
                     has_profile_image(u.userid) AS has_profile_image
                 FROM users u
                 LEFT JOIN user_flashcard_progress ufp ON ufp.user_id = u.userid
                 WHERE u.userid = $1 AND u.password != 'DISABLED'
                   AND LOWER(u.role::text) <> 'guest'
                 GROUP BY u.userid, u.username
             ),
             all_scored AS (
                 SELECT
                     u.userid AS user_id,
                     u.username,
                     COALESCE(SUM(ufp.review_count), 0)::bigint AS reviews
                 FROM users u
                 JOIN user_flashcard_progress ufp ON ufp.user_id = u.userid
                 WHERE u.password != 'DISABLED'
                   AND LOWER(u.role::text) <> 'guest'
                 GROUP BY u.userid, u.username
                 HAVING COALESCE(SUM(ufp.review_count), 0) > 0
             ),
             ranked AS (
                 SELECT
                     s.user_id,
                     s.username,
                     s.reviews,
                     s.has_profile_image,
                     CASE
                         WHEN s.reviews > 0 THEN (
                             SELECT 1 + COUNT(*)::bigint
                             FROM all_scored a
                             WHERE a.reviews > s.reviews
                                OR (a.reviews = s.reviews AND a.username < s.username)
                         )
                         ELSE NULL
                     END AS rank
                 FROM scored s
             )
             SELECT user_id, username, reviews, has_profile_image, rank
             FROM ranked",
            &[&user_id],
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let Some(row) = row else {
        return Ok(None);
    };

    let reviews: i64 = row.get("reviews");
    let rank: Option<i64> = row.get("rank");
    let username: String = row.get("username");
    let has_profile_image: bool = row.get("has_profile_image");
    Ok(Some(LingoLeaderboardEntry {
        user_id: row.get("user_id"),
        profile_image_url: profile_image_url(&username, has_profile_image),
        username,
        points: reviews.saturating_mul(XP_PER_REVIEW),
        rank: rank.unwrap_or(0),
        has_profile_image,
        is_you: true,
    }))
}

#[cfg(test)]
mod tests {
    use super::{clamp_limit, profile_image_url};

    #[test]
    fn clamp_limit_defaults_and_bounds() {
        assert_eq!(clamp_limit(None), 10);
        assert_eq!(clamp_limit(Some(0)), 1);
        assert_eq!(clamp_limit(Some(3)), 3);
        assert_eq!(clamp_limit(Some(100)), 50);
    }

    #[test]
    fn profile_image_url_only_when_present() {
        assert_eq!(profile_image_url("alice", false), None);
        assert_eq!(
            profile_image_url("alice", true).as_deref(),
            Some("/users/alice/profile-image")
        );
        assert_eq!(
            profile_image_url("foo@jbotcan", true).as_deref(),
            Some("/users/foo%40jbotcan/profile-image")
        );
    }
}
