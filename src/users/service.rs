use crate::comments::models::Comment;
use crate::middleware::image::ImageProcessor;

use super::dto::{
    ContributionsResponse, Definition, ProfileImageRequest, PublicUserProfile, UserInfo,
    UserListQuery, UserListResponse, Vote,
};
use crate::comments::dto::ReactionResponse;
use deadpool_postgres::Pool;

pub async fn list_users(
    pool: &Pool,
    query: UserListQuery,
) -> Result<UserListResponse, Box<dyn std::error::Error>> {
    let mut client = pool.get().await?;
    let transaction = client.transaction().await?;

    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(20);
    let offset = query.offset.unwrap_or((page - 1) * per_page);

    // Build the base query
    let mut query_conditions = vec!["TRUE"];
    let mut params = Vec::new();

    // Handle search
    let mut search_pattern = String::new();
    if let Some(search) = query.search {
        search_pattern = format!("%{}%", search);
        query_conditions.push("(username ILIKE $1 OR COALESCE(realname, '') ILIKE $1 OR COALESCE(email, '') ILIKE $1)");
        params.push(&search_pattern as &(dyn tokio_postgres::types::ToSql + Sync));
    }

    // Handle role filter
    let role_pattern = query.role.as_ref().map(|role| format!("%{}%", role));
    if let Some(pattern) = &role_pattern {
        query_conditions.push("role::text ILIKE $2");
        params.push(pattern as &(dyn tokio_postgres::types::ToSql + Sync));
    }

    // Build sort clause
    let sort_by = match query.sort_by.as_deref().unwrap_or("created_at") {
        "realname" => "realname",
        "username" => "username",
        _ => "created_at",
    };

    let sort_order = match query
        .sort_order
        .as_deref()
        .unwrap_or("desc")
        .to_uppercase()
        .as_str()
    {
        "ASC" => "ASC",
        _ => "DESC",
    };

    // Add limit and offset to params
    params.push(&per_page as &(dyn tokio_postgres::types::ToSql + Sync));
    params.push(&offset as &(dyn tokio_postgres::types::ToSql + Sync));

    // Build the final query
    let param_offset = match (search_pattern.is_empty(), query.role.is_some()) {
        (true, false) => 0,
        (false, false) => 1,
        (_, true) => 2,
    };
    let query_string = format!(
        "SELECT userid, username, realname, email, personal, url, role, password != 'DISABLED' as is_enabled
         FROM users
         WHERE {}
         ORDER BY {} {}
         LIMIT ${} OFFSET ${}",
        query_conditions.join(" AND "),
        sort_by,
        sort_order,
        param_offset + 1,
        param_offset + 2
    );

    // Execute query
    let rows = transaction.query(&query_string, &params).await?;

    let users = rows
        .iter()
        .map(|row| {
            let email: Option<String> = row.get("email");
            let obfuscated_email = email.map(|e| {
                e.replace(
                    '@',
                    "[this would be an amphora symbol but we don't like spambots]",
                )
            });

            UserInfo {
                user_id: row.get("userid"),
                username: row.get("username"),
                realname: row.get("realname"),
                email: obfuscated_email,
                is_enabled: row.get("is_enabled"),
                personal: row.get("personal"),
                url: row.get("url"),
                role: row.get("role"),
            }
        })
        .collect();

    // Count total with the same conditions
    let count_query = format!(
        "SELECT COUNT(*) FROM users WHERE {}",
        query_conditions.join(" AND ")
    );

    // Create count params by removing limit and offset
    let count_params = if params.len() > 2 {
        params[0..params.len() - 2].to_vec()
    } else {
        vec![]
    };

    let total: i64 = transaction
        .query_one(&count_query, &count_params)
        .await?
        .get(0);

    transaction.commit().await?;

    Ok(UserListResponse {
        users,
        total,
        page,
        per_page,
    })
}

pub async fn get_public_profile(
    pool: &Pool,
    username: &str,
) -> Result<PublicUserProfile, Box<dyn std::error::Error>> {
    let mut client = pool.get().await?;
    let transaction = client.transaction().await?;

    // Get basic user info
    let user_row = transaction
        .query_one(
            "SELECT userid, username, role::text, realname, url, personal, created_at,
                    has_profile_image(userid) as has_profile_image
             FROM users 
             WHERE username = $1",
            &[&username],
        )
        .await?;

    let user_id: i32 = user_row.get("userid");

    // Get definition count
    let definition_count: i64 = transaction
        .query_one(
            "SELECT COUNT(*) FROM definitions WHERE userid = $1",
            &[&user_id],
        )
        .await?
        .get(0);

    // Get comment count
    let comment_count: i64 = transaction
        .query_one(
            "SELECT COUNT(*) FROM comments WHERE userid = $1",
            &[&user_id],
        )
        .await?
        .get(0);

    // Get vote count
    let vote_count: i64 = transaction
        .query_one(
            "SELECT COUNT(*) FROM definitionvotes WHERE userid = $1",
            &[&user_id],
        )
        .await?
        .get(0);

    transaction.commit().await?;

    Ok(PublicUserProfile {
        user_id,
        username: user_row.get("username"),
        role: user_row.get("role"),
        realname: user_row.get("realname"),
        url: user_row.get("url"),
        personal: user_row.get("personal"),
        join_date: user_row.get("created_at"),
        definition_count,
        comment_count,
        vote_count,
        has_profile_image: user_row.get("has_profile_image"),
    })
}

pub async fn get_user_votes(
    pool: &Pool,
    user_id: i32,
    page: i64,
    per_page: i64,
) -> Result<ContributionsResponse<Vote>, Box<dyn std::error::Error>> {
    let mut client = pool.get().await?;
    let transaction = client.transaction().await?;
    let offset = (page - 1) * per_page;

    let votes = transaction
        .query(
            "SELECT 
                dv.definitionid, dv.value::integer as vote_value,
                v.word as valsi_word, d.definition, d.langid,
                l.realname as language,
                (SELECT COALESCE(SUM(value), 0)::integer FROM definitionvotes WHERE definitionid = dv.definitionid) as score,
                to_timestamp(dv.time) as voted_at
             FROM definitionvotes dv
             JOIN definitions d ON dv.definitionid = d.definitionid
             JOIN valsi v ON d.valsiid = v.valsiid
             JOIN languages l ON d.langid = l.langid
             WHERE dv.userid = $1
             ORDER BY dv.time DESC
             LIMIT $2 OFFSET $3",
            &[&user_id, &per_page, &offset],
        )
        .await?;

    let total: i64 = transaction
        .query_one(
            "SELECT COUNT(*) FROM definitionvotes WHERE userid = $1",
            &[&user_id],
        )
        .await?
        .get(0);

    let votes = votes
        .iter()
        .map(|row| Vote {
            definition_id: row.get("definitionid"),
            valsi_word: row.get("valsi_word"),
            definition: row.get("definition"),
            language: row.get("language"),
            langid: row.get("langid"),
            vote_value: row.get("vote_value"),
            score: row.get("score"),
            voted_at: row.get("voted_at"),
        })
        .collect();

    transaction.commit().await?;

    Ok(ContributionsResponse {
        items: votes,
        total,
        page,
        per_page,
    })
}

pub async fn get_user_definitions(
    pool: &Pool,
    username: &str,
    page: i64,
    per_page: i64,
) -> Result<ContributionsResponse<Definition>, Box<dyn std::error::Error>> {
    let mut client = pool.get().await?;
    let transaction = client.transaction().await?;
    let offset = (page - 1) * per_page;

    let user_row = transaction
        .query_opt("SELECT userid FROM users WHERE username = $1", &[&username])
        .await?;

    let user_id: i32 = match user_row {
        Some(row) => row.get("userid"),
        None => return Err("User not found".into()),
    };

    let query = r#"
        WITH all_definitions AS (
            -- Get definitions from versions
            SELECT DISTINCT ON (v.definition_id, v.version_id)
                v.version_id,
                val.word,
                v.definition as content,
                v.created_at,
                v.definition_id as definitionid
            FROM definition_versions v
            JOIN definitions d ON v.definition_id = d.definitionid
            JOIN valsi val ON d.valsiid = val.valsiid
            WHERE v.user_id = $1

            UNION ALL

            -- Get definitions not in versions
            SELECT 
                0 as version_id,
                v.word,
                d.definition as content,
                d.created_at,
                d.definitionid as definitionid
            FROM definitions d
            JOIN valsi v ON d.valsiid = v.valsiid
            LEFT JOIN definition_versions dv ON d.definitionid = dv.definition_id
            WHERE d.userid = $1 AND dv.definition_id IS NULL
        )
        SELECT * FROM all_definitions
        ORDER BY created_at DESC
        LIMIT $2 OFFSET $3
    "#;

    let rows = transaction
        .query(query, &[&user_id, &per_page, &offset])
        .await?;

    let count_query = r#"
        WITH all_definitions AS (
            -- Count versioned definitions
            SELECT DISTINCT definition_id
            FROM definition_versions
            WHERE user_id = $1

            UNION

            -- Count non-versioned definitions
            SELECT d.definitionid
            FROM definitions d
            LEFT JOIN definition_versions dv ON d.definitionid = dv.definition_id
            WHERE d.userid = $1 AND dv.definition_id IS NULL
        )
        SELECT COUNT(*) FROM all_definitions
    "#;

    let items = rows
        .iter()
        .map(|row| Definition {
            definitionid: row.get("definitionid"),
            word: row.get("word"),
            version_id: row.get("version_id"),
            created_at: row.get("created_at"),
            content: row.get("content"),
        })
        .collect();

    let total: i64 = transaction
        .query_one(count_query, &[&user_id])
        .await?
        .get(0);

    transaction.commit().await?;

    Ok(ContributionsResponse {
        items,
        total,
        page,
        per_page,
    })
}

pub async fn get_user_comments(
    pool: &Pool,
    username: &str,
    page: i64,
    per_page: i64,
) -> Result<ContributionsResponse<Comment>, Box<dyn std::error::Error>> {
    let mut client = pool.get().await?;
    let transaction = client.transaction().await?;
    let offset = (page - 1) * per_page;

    let user_row = transaction
        .query_opt("SELECT userid FROM users WHERE username = $1", &[&username])
        .await?;

    let user_id: i32 = match user_row {
        Some(row) => row.get("userid"),
        None => return Err("User not found".into()),
    };

    let rows = transaction
        .query(
            "SELECT
                c.commentid, c.threadid, c.content::text as content, c.time,
                c.subject, c.commentnum, c.userid, c.parentid,
                t.valsiid, t.definitionid, t.definition_link_id,
                v.word as valsi_word,
                d.definition,
                u.username,
                u.realname,
                cc.total_reactions,
                cc.total_replies,
                CASE WHEN cl.user_id IS NOT NULL THEN true ELSE false END as is_liked,
                CASE WHEN cb.user_id IS NOT NULL THEN true ELSE false END as is_bookmarked,
                (SELECT json_agg(json_build_object(
                    'reaction', r.reaction,
                    'count', r.count,
                    'reacted', r.reacted
                ))
                FROM (
                    SELECT
                        cr.reaction,
                        COUNT(*) as count,
                        BOOL_OR(cr.user_id = $1) as reacted
                    FROM comment_reactions cr
                    WHERE cr.comment_id = c.commentid
                    GROUP BY cr.reaction
                ) r) as reactions
             FROM comments c
             JOIN threads t ON c.threadid = t.threadid
             LEFT JOIN valsi v ON t.valsiid = v.valsiid
             LEFT JOIN definitions d ON t.definitionid = d.definitionid
             JOIN users u ON c.userid = u.userid
             LEFT JOIN comment_counters cc ON c.commentid = cc.comment_id
             LEFT JOIN comment_likes cl ON c.commentid = cl.comment_id AND cl.user_id = $1
             LEFT JOIN comment_bookmarks cb ON c.commentid = cb.comment_id AND cb.user_id = $1
             WHERE c.userid = $1
             ORDER BY c.time DESC
             LIMIT $2 OFFSET $3",
            &[&user_id, &per_page, &offset],
        )
        .await?;

    let total: i64 = transaction
        .query_one(
            "SELECT COUNT(*) FROM comments WHERE userid = $1",
            &[&user_id],
        )
        .await?
        .get(0);

    let items = rows
        .iter()
        .map(|row| {
            let reactions: Option<Vec<ReactionResponse>> = row
                .get::<_, Option<serde_json::Value>>("reactions")
                .and_then(|v| serde_json::from_value(v).ok());

            Comment {
                comment_id: row.get("commentid"),
                thread_id: row.get("threadid"),
                subject: row.get("subject"),
                content: serde_json::from_str(&row.get::<_, String>("content")).unwrap_or_default(),
                valsi_id: row.get("valsiid"),
                definition_id: row.get("definitionid"),
                definition_link_id: row.get("definition_link_id"),
                parent_id: row.get("parentid"),
                user_id: row.get("userid"),
                comment_num: row.get("commentnum"),
                time: row.get("time"),
                username: Some(row.get("username")),
                last_comment_username: None,
                realname: row.get("realname"),
                total_reactions: row.get::<_, Option<i64>>("total_reactions").unwrap_or(0),
                total_replies: row.get::<_, Option<i64>>("total_replies").unwrap_or(0),
                is_liked: row.get("is_liked"),
                is_bookmarked: row.get("is_bookmarked"),
                reactions: reactions.unwrap_or_default(),
                parent_content: None,
                valsi_word: row.get("valsi_word"),
                definition: row.get("definition"),
                first_comment_subject: None,
                first_comment_content: None,
            }
        })
        .collect();

    transaction.commit().await?;

    Ok(ContributionsResponse {
        items,
        total,
        page,
        per_page,
    })
}

use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;

const MAX_IMAGE_SIZE: usize = 5 * 1024 * 1024; // 5MB

pub fn validate_profile_image(image: &ProfileImageRequest) -> Result<Vec<u8>, String> {
    // Validate mime type
    if !["image/jpeg", "image/png", "image/webp"].contains(&image.mime_type.as_str()) {
        return Err("Invalid image type. Supported types: JPEG, PNG, WebP".to_string());
    }

    // Decode and validate base64 data
    let decoded = BASE64
        .decode(&image.data)
        .map_err(|_| "Invalid base64 data".to_string())?;

    if decoded.len() > MAX_IMAGE_SIZE {
        return Err("Image size exceeds 5MB limit".to_string());
    }

    Ok(decoded)
}

pub async fn update_profile_image(
    pool: &Pool,
    user_id: i32,
    req: &ProfileImageRequest,
) -> Result<(), Box<dyn std::error::Error>> {
    let image_data = validate_profile_image(req)?;

    // Compress image
    let (compressed_data, new_mime_type) =
        ImageProcessor::compress_avatar(&image_data, &req.mime_type)?;

    let mut client = pool.get().await?;
    let transaction = client.transaction().await?;

    transaction
        .execute(
            "INSERT INTO user_profile_images (user_id, image_data, mime_type, updated_at)
             VALUES ($1, $2, $3, CURRENT_TIMESTAMP)
             ON CONFLICT (user_id) 
             DO UPDATE SET 
                image_data = EXCLUDED.image_data,
                mime_type = EXCLUDED.mime_type,
                updated_at = CURRENT_TIMESTAMP",
            &[&user_id, &compressed_data, &new_mime_type],
        )
        .await?;

    transaction.commit().await?;
    Ok(())
}

pub async fn get_profile_image(
    pool: &Pool,
    username: &str,
) -> Result<Option<(Vec<u8>, String)>, Box<dyn std::error::Error>> {
    let client = pool.get().await?;

    let result = client
        .query_opt(
            "SELECT i.image_data, i.mime_type 
         FROM user_profile_images i
         JOIN users u ON u.userid = i.user_id
         WHERE u.username = $1",
            &[&username],
        )
        .await?;

    Ok(result.map(|row| (row.get("image_data"), row.get("mime_type"))))
}

pub async fn remove_profile_image(
    pool: &Pool,
    user_id: i32,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = pool.get().await?;

    client
        .execute(
            "DELETE FROM user_profile_images WHERE user_id = $1",
            &[&user_id],
        )
        .await?;

    Ok(())
}
