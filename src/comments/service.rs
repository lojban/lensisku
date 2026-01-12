use crate::utils::remove_html_tags;
use std::{
    collections::{HashMap, HashSet},
    env,
};

use crate::comments::models::Comment;
use chrono::Utc;
use deadpool_postgres::Pool;
use tokio_postgres::Row;

use super::{
    dto::{
        CommentStats, ContentPart, CreateOpinionRequest, NewCommentParams, OpinionVoteRequest,
        PaginatedCommentsResponse, PaginatedReactions, ReactionResponse, ReactionSummary,
        SearchCommentsParams, ThreadParams, TrendingHashtag,
    },
    errors::ReactionError,
    models::{CommentOpinion, FreeThread, TrendingTimespan},
};

pub fn sanitize_html(html: &str) -> String {
    remove_html_tags(html)
}

pub async fn get_thread_comments(
    pool: &Pool,
    params: ThreadParams,
) -> Result<PaginatedCommentsResponse, Box<dyn std::error::Error>> {
    let mut client = pool.get().await?;
    let transaction = client.transaction().await?;

    let per_page = params.per_page.unwrap_or(20);
    let mut page = params.page.unwrap_or(1);
    let mut offset = (page - 1) * per_page;

    // Determine thread ID using sequential fallback approach
    let thread_id: Option<i32> = if let Some(thread_id) = params.thread_id.filter(|&id| id > 0) {
        // 1. Use thread_id if provided and valid
        Some(thread_id)
    } else if let Some(comment_id) = params.comment_id.filter(|&id| id > 0) {
        // 2. Get thread_id from comment if comment_id is provided and valid
        transaction
            .query_opt(
                "SELECT threadid FROM comments WHERE commentid = $1 AND $1 > 0",
                &[&comment_id],
            )
            .await?
            .map(|row| row.get("threadid"))
    } else {
        get_thread_id_by_context(
            &transaction,
            params.valsi_id,
            params.natlang_word_id,
            params.definition_id,
            params.target_user_id,
        )
        .await?
    };

    if thread_id.is_none() {
        return Err("Could not determine thread. Please provide a valid thread_id, comment_id, or a context (e.g., valsi_id, definition_id, target_user_id) that identifies an existing thread.".into());
    }

    let (comments, total) = if let Some(thread_id) = thread_id {
        // Get all comments in the thread in hierarchical order
        let all_comments = transaction
            .query(
                "WITH RECURSIVE comment_tree AS (
                    SELECT
                        c.commentid,
                        c.threadid,
                        c.parentid,
                        c.userid,
                        c.commentnum,
                        c.time,
                        c.subject,
                        c.username,
                        c.realname,
                        c.total_reactions,
                        c.total_replies,
                        c.valsiid,
                        c.definitionid,
                        c.content::text as content,
                        cc.total_reactions as counter_reactions,
                        cc.total_replies as counter_replies,
                        CASE WHEN cl.user_id IS NOT NULL THEN true ELSE false END as is_liked,
                        CASE WHEN cb.user_id IS NOT NULL THEN true ELSE false END as is_bookmarked,
                        ARRAY[c.commentnum] as path,
                        0 as depth,
                        pc.content::text as parent_content
                    FROM convenientcomments c
                    LEFT JOIN comment_counters cc ON c.commentid = cc.comment_id
                    LEFT JOIN comment_likes cl ON c.commentid = cl.comment_id AND cl.user_id = $1
                    LEFT JOIN comment_bookmarks cb ON c.commentid = cb.comment_id AND cb.user_id = $1
                    LEFT JOIN convenientcomments pc ON c.parentid = pc.commentid
                    WHERE c.threadid = $2 AND c.parentid is null

                    UNION ALL

                    SELECT
                        c.commentid,
                        c.threadid,
                        c.parentid,
                        c.userid,
                        c.commentnum,
                        c.time,
                        c.subject,
                        c.username,
                        c.realname,
                        c.total_reactions,
                        c.total_replies,
                        c.valsiid,
                        c.definitionid,
                        c.content::text as content,
                        cc.total_reactions as counter_reactions,
                        cc.total_replies as counter_replies,
                        CASE WHEN cl.user_id IS NOT NULL THEN true ELSE false END as is_liked,
                        CASE WHEN cb.user_id IS NOT NULL THEN true ELSE false END as is_bookmarked,
                        ct.path || c.commentnum as path,
                        ct.depth + 1 as depth,
                        ct.content as parent_content
                    FROM convenientcomments c
                    LEFT JOIN comment_counters cc ON c.commentid = cc.comment_id
                    LEFT JOIN comment_likes cl ON c.commentid = cl.comment_id AND cl.user_id = $1
                    LEFT JOIN comment_bookmarks cb ON c.commentid = cb.comment_id AND cb.user_id = $1
                    JOIN comment_tree ct ON c.parentid = ct.commentid
                )
                SELECT *, ROW_NUMBER() OVER (ORDER BY path) - 1 as position
                FROM comment_tree",
                &[&params.current_user_id, &thread_id],
            )
            .await?;

        let total = all_comments.len() as i64;

        // If a specific comment ID was requested, find its page
        if let Some(target_comment_id) = params.scroll_to {
            if let Some((position, _)) = all_comments
                .iter()
                .enumerate()
                .find(|(_, row)| row.get::<_, i32>("commentid") == target_comment_id)
            {
                // Only set page if it wasn't specified in params
                if params.page.is_none() {
                    page = (position as i64 / per_page) + 1;
                    offset = (page - 1) * per_page;
                }
            }
        }

        // Apply pagination
        let comments: Vec<Row> = all_comments
            .into_iter()
            .skip(offset as usize)
            .take(per_page as usize)
            .collect();

        (comments, total)
    } else {
        (Vec::new(), 0)
    };

    // Get comment IDs and fetch reactions
    let comment_ids: Vec<i32> = comments.iter().map(|row| row.get("commentid")).collect();
    let reactions_map = fetch_reactions(&transaction, &comment_ids, params.current_user_id).await?;

    let mapped_comments = comments
        .iter()
        .map(|row| {
            let comment_id = row.get::<_, i32>("commentid");
            Ok(Comment {
                comment_id,
                thread_id: row.get("threadid"),
                parent_id: row.get("parentid"),
                user_id: row.get("userid"),
                comment_num: row.get("commentnum"),
                time: row.get("time"),
                subject: row.get("subject"),
                content: serde_json::from_str(&row.get::<_, String>("content")).unwrap_or_default(),
                username: row.get("username"),
                realname: row.get("realname"),
                total_reactions: row.get::<_, Option<i64>>("total_reactions").unwrap_or(0),
                total_replies: row.get::<_, Option<i64>>("total_replies").unwrap_or(0),
                is_liked: row.get("is_liked"),
                is_bookmarked: row.get("is_bookmarked"),
                valsi_id: None,
                definition_id: None,
                reactions: reactions_map.get(&comment_id).cloned().unwrap_or_default(),
                parent_content: row
                    .get::<_, Option<String>>("parent_content")
                    .and_then(|s| serde_json::from_str(&s).ok()),
                valsi_word: None,
                definition: None,
                first_comment_subject: None,
                first_comment_content: None,
                last_comment_username: None,
            })
        })
        .collect::<Result<Vec<_>, Box<dyn std::error::Error>>>()?;

    transaction.commit().await?;

    Ok(PaginatedCommentsResponse {
        comments: mapped_comments,
        total,
        page,
        per_page,
    })
}

pub async fn add_comment(params: NewCommentParams) -> Result<Comment, Box<dyn std::error::Error>> {
    let mut client = params.pool.get().await?;
    let transaction = client.transaction().await?;

    let thread_id: i32 = if let Some(parent_id) = params.parent_id.filter(|&id| id > 0) {
        // If only parent_id is provided, get thread_id from parent comment
        transaction
            .query_one(
                "SELECT threadid FROM comments WHERE commentid = $1",
                &[&parent_id],
            )
            .await?
            .get("threadid")
    } else {
        get_or_create_thread_id(
            &transaction,
            params.valsi_id,
            params.natlang_word_id,
            params.definition_id,
            params.target_user_id,
        )
        .await?
    };

    let comment_num: i32 = transaction
        .query_one(
            "SELECT COALESCE(MAX(commentnum), 0) + 1 as next_num
             FROM comments
             WHERE threadid = $1",
            &[&thread_id],
        )
        .await?
        .get::<_, i32>("next_num");

    // Store media items

    let mut content_parts: Vec<ContentPart> = serde_json::from_str(&params.content)
        .map_err(|e| format!("Failed to parse content: {}", e))?;

    // Sanitize each part's content
    for part in content_parts.iter_mut() {
        if part.r#type == "text" || part.r#type == "header" {
            part.data = sanitize_html(&part.data);
        }
    }

    // Remove empty text parts at the end
    while let Some(last) = content_parts.last() {
        if last.r#type == "text" && last.data.is_empty() {
            content_parts.pop();
        } else {
            break;
        }
    }

    // Validate total content size (approximate, actual JSON size might differ slightly)
    let total_size: usize = content_parts.iter().map(|p| p.data.len()).sum();
    const MAX_COMMENT_SIZE: usize = 5 * 1024 * 1024; // 5MB limit
    if total_size > MAX_COMMENT_SIZE {
        return Err(format!(
            "Comment content exceeds the maximum size of {}MB",
            MAX_COMMENT_SIZE / (1024 * 1024)
        )
        .into());
    }

    let sanitized_subject = sanitize_html(&params.subject);

    // Add subject as header if present
    if !sanitized_subject.is_empty() {
        content_parts.insert(
            0,
            ContentPart {
                r#type: "header".to_string(),
                data: sanitized_subject.clone(),
            },
        );
    }

    let content_json = serde_json::Value::Array(
        content_parts
            .iter()
            .map(|p| {
                serde_json::json!({
                    "type": p.r#type,
                    "data": p.data
                })
            })
            .collect(),
    );

    let comment_id: i32 = transaction
        .query_one(
            "INSERT INTO comments
             (threadid, parentid, userid, commentnum, time, subject, content)
             VALUES ($1, $2, $3, $4, $5, $6, $7)
             RETURNING commentid",
            &[
                &thread_id,
                &params.parent_id,
                &params.user_id,
                &comment_num,
                &(Utc::now().timestamp() as i32),
                &sanitized_subject,
                &content_json,
            ],
        )
        .await?
        .get::<_, i32>("commentid");

    // Store media items
    // for part in content_parts.iter() {
    //     if part.r#type == "image" {
    //         let image_data = base64::engine::general_purpose::STANDARD
    //             .decode(&part.data)
    //             .map_err(|e| format!("Invalid base64 image data: {}", e))?;

    //         transaction
    //             .execute(
    //                 "INSERT INTO comment_media
    //             (media_id, comment_id, media_type, media_data, text_content)
    //             VALUES (nextval('comment_media_media_id_seq'), $1, $2, $3, $4)",
    //                 &[&comment_id, &part.r#type, &image_data, &None::<String>],
    //             )
    //             .await?;
    //     } else {
    //         transaction
    //             .execute(
    //                 "INSERT INTO comment_media
    //             (media_id, comment_id, media_type, media_data, text_content)
    //             VALUES (nextval('comment_media_media_id_seq'), $1, $2, $3, $4)",
    //                 &[&comment_id, &part.r#type, &None::<Vec<u8>>, &part.data],
    //             )
    //             .await?;
    //     }
    // }

    // Extract hashtags from all text parts
    let content_parts: Vec<ContentPart> = serde_json::from_str(&params.content)
        .map_err(|e| format!("Failed to parse content: {}", e))?;

    let hashtags = content_parts
        .iter()
        .filter(|p| p.r#type == "text")
        .map(|p| Comment::extract_hashtags(&p.data))
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .flatten()
        .collect::<HashSet<_>>();

    for tag in hashtags {
        // Insert hashtag if it doesn't exist and get its ID
        let hashtag_id = transaction
            .query_one(
                "INSERT INTO hashtags (tag)
                 VALUES ($1)
                 ON CONFLICT (tag) DO UPDATE
                 SET tag = EXCLUDED.tag
                 RETURNING id",
                &[&tag],
            )
            .await?
            .get::<_, i32>("id");

        // Link hashtag to comment
        transaction
            .execute(
                "INSERT INTO post_hashtags (post_id, hashtag_id)
                 VALUES ($1, $2)
                 ON CONFLICT (post_id, hashtag_id) DO NOTHING",
                &[&comment_id, &hashtag_id],
            )
            .await?;
    }

    transaction
        .execute(
            "INSERT INTO comment_counters (comment_id, total_reactions, total_replies)
             VALUES ($1, 0, 0)",
            &[&comment_id],
        )
        .await?;
    if let Some(parent_id) = params.parent_id {
        transaction
            .execute(
                "INSERT INTO comment_counters (comment_id, total_reactions, total_replies)
                 VALUES ($1, 0, 1)
                 ON CONFLICT (comment_id) DO UPDATE
                 SET total_replies = comment_counters.total_replies + 1",
                &[&parent_id],
            )
            .await?;
    }

    // Get complete comment details
    let comment = get_comment_by_id(&transaction, comment_id, Some(params.user_id)).await?;

    // Notify subscribers about the new comment, if it's on a valsi thread
    let thread_info_row = transaction
        .query_opt(
            "SELECT t.valsiid, v.word, t.definitionid 
         FROM threads t 
         LEFT JOIN valsi v ON t.valsiid = v.valsiid 
         WHERE t.threadid = $1",
            &[&thread_id],
        )
        .await?;

    if let Some(row) = thread_info_row {
        if let Some(notif_valsi_id) = row.get::<_, Option<i32>>("valsiid") {
            let valsi_word: Option<String> = row.get("word");
            let notif_definition_id: Option<i32> = row.get("definitionid");
            let url = format!(
                "{}/comments?valsi_id={}&definition_id={}&thread_id={}&scroll_to={}",
                env::var("FRONTEND_URL")?,
                notif_valsi_id,
                notif_definition_id.unwrap_or(0),
                thread_id,
                comment_id
            );

            transaction
                .execute(
                    "SELECT notify_valsi_subscribers($1, 'comment', $2, $3, $4)",
                    &[
                        &notif_valsi_id,
                        &format!(
                            "New comment on thread for {}",
                            valsi_word.unwrap_or_else(|| "a valsi".to_string())
                        ),
                        &url,
                        &params.user_id,
                    ],
                )
                .await?;
        }
    }

    transaction.commit().await?;

    Ok(comment)
}

async fn get_thread_id_by_context(
    transaction: &tokio_postgres::Transaction<'_>,
    valsi_id: Option<i32>,
    natlang_word_id: Option<i32>,
    definition_id: Option<i32>,
    target_user_id: Option<i32>,
) -> Result<Option<i32>, Box<dyn std::error::Error>> {
    // Ensure only one context type is primarily active or it's a free-standing thread context
    let mut active_contexts = 0;
    if valsi_id.is_some() || natlang_word_id.is_some() || definition_id.is_some() {
        active_contexts += 1;
    }
    if target_user_id.is_some() {
        active_contexts += 1;
    }

    if active_contexts > 1 {
        // This case should ideally be prevented by frontend/controller logic
        // but as a safeguard:
        return Err("Ambiguous thread context: Multiple context IDs provided.".into());
    }

    Ok(transaction
        .query_opt(
            "SELECT threadid
             FROM threads t
            WHERE (t.valsiid = $1 OR ($1 IS NULL AND (t.valsiid IS NULL OR t.valsiid = 0)))
              AND (t.natlangwordid = $2 OR ($2 IS NULL AND (t.natlangwordid IS NULL OR t.natlangwordid = 0)))
              AND (t.definitionid = $3 OR ($3 IS NULL AND (t.definitionid IS NULL OR t.definitionid = 0)))
              AND (t.target_user_id = $4 OR ($4 IS NULL AND t.target_user_id IS NULL))
            LIMIT 1",
            &[
                &valsi_id,
                &natlang_word_id,
                &definition_id,
                &target_user_id,
            ],
        )
        .await?
        .map(|row| row.get("threadid")))
}

pub async fn toggle_like(
    pool: &Pool,
    comment_id: i32,
    user_id: i32,
    like: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut client = pool.get().await?;
    let transaction = client.transaction().await?;

    // First, ensure comment_counters record exists
    transaction
        .execute(
            "INSERT INTO comment_counters (comment_id, total_likes, total_replies)
             VALUES ($1, 0, 0)
             ON CONFLICT (comment_id) DO NOTHING",
            &[&comment_id],
        )
        .await?;

    let is_liked = transaction
        .query_opt(
            "SELECT 1 FROM comment_likes
             WHERE comment_id = $1 AND user_id = $2",
            &[&comment_id, &user_id],
        )
        .await?
        .is_some();

    if like != is_liked {
        if like {
            transaction
                .execute(
                    "INSERT INTO comment_likes (comment_id, user_id, created_at)
                     VALUES ($1, $2, $3)",
                    &[&comment_id, &user_id, &Utc::now()],
                )
                .await?;

            transaction
                .execute(
                    "UPDATE comment_counters
                     SET total_likes = total_likes + 1
                     WHERE comment_id = $1",
                    &[&comment_id],
                )
                .await?;
            transaction
                .execute(
                    "SELECT update_comment_counter($1, 'likes', true)",
                    &[&comment_id],
                )
                .await?;
        } else {
            transaction
                .execute(
                    "DELETE FROM comment_likes
                     WHERE comment_id = $1 AND user_id = $2",
                    &[&comment_id, &user_id],
                )
                .await?;

            transaction
                .execute(
                    "UPDATE comment_counters
                     SET total_likes = total_likes - 1
                     WHERE comment_id = $1",
                    &[&comment_id],
                )
                .await?;
            transaction
                .execute(
                    "SELECT update_comment_counter($1, 'likes', false)",
                    &[&comment_id],
                )
                .await?;
        }
    }

    transaction.commit().await?;
    Ok(())
}

async fn get_comment_by_id(
    transaction: &tokio_postgres::Transaction<'_>,
    comment_id: i32,
    user_id: Option<i32>,
) -> Result<Comment, Box<dyn std::error::Error>> {
    let row = transaction
        .query_one(
            "SELECT
                    c.commentid,
                    c.threadid,
                    c.parentid,
                    c.userid,
                    c.commentnum,
                    c.time,
                    c.subject,
                    c.username,
                    c.realname,
                    c.total_reactions,
                    c.total_replies,
                    c.valsiid,
                    c.definitionid,
                    c.content::text as content,
                    cc.total_reactions,
                    cc.total_replies,
                    CASE WHEN cl.user_id IS NOT NULL THEN true ELSE false END as is_liked,
                    CASE WHEN cb.user_id IS NOT NULL THEN true ELSE false END as is_bookmarked
             FROM convenientcomments c
             LEFT JOIN comment_counters cc ON c.commentid = cc.comment_id
             LEFT JOIN comment_likes cl ON c.commentid = cl.comment_id AND cl.user_id = $2
             LEFT JOIN comment_bookmarks cb ON c.commentid = cb.comment_id AND cb.user_id = $2
             WHERE c.commentid = $1",
            &[&comment_id, &user_id],
        )
        .await?;

    // Fetch reactions using the helper function
    let reactions_map = fetch_reactions(transaction, &[comment_id], user_id).await?;
    let reactions = reactions_map.get(&comment_id).cloned().unwrap_or_default();

    Ok(Comment {
        parent_content: None,
        comment_id: row.get("commentid"),
        thread_id: row.get("threadid"),
        parent_id: row.get("parentid"),
        user_id: row.get("userid"),
        comment_num: row.get("commentnum"),
        time: row.get("time"),
        subject: row.get("subject"),
        content: serde_json::from_str(&row.get::<_, String>("content")).unwrap_or_default(),
        username: row.get("username"),
        realname: row.get("realname"),
        total_reactions: row.get::<_, Option<i64>>("total_reactions").unwrap_or(0),
        total_replies: row.get::<_, Option<i64>>("total_replies").unwrap_or(0),
        is_liked: row.get("is_liked"),
        is_bookmarked: row.get("is_bookmarked"),
        valsi_id: None,
        definition_id: None,
        reactions,
        valsi_word: None,
        definition: None,
        first_comment_subject: None,
        first_comment_content: None,
        last_comment_username: None,
    })
}

pub async fn get_bookmarked_comments(
    pool: &Pool,
    user_id: i32,
    page: i64,
    per_page: i64,
) -> Result<PaginatedCommentsResponse, Box<dyn std::error::Error>> {
    let mut client = pool.get().await?;
    let transaction = client.transaction().await?;

    let offset = (page - 1) * per_page;

    // Get total count
    let total: i64 = transaction
        .query_one(
            "SELECT COUNT(*) FROM comment_bookmarks WHERE user_id = $1",
            &[&user_id],
        )
        .await?
        .get(0);

    let comments = transaction
        .query(
            "SELECT
                    c.commentid,
                    c.threadid,
                    c.parentid,
                    c.userid,
                    c.commentnum,
                    c.time,
                    c.subject,
                    c.username,
                    c.realname,
                    c.total_reactions,
                    c.total_replies,
                    c.valsiid,
                    c.definitionid,
                    c.content::text as content,
                    t.*,
                    COALESCE(cc.total_reactions, 0) as total_reactions,
                    COALESCE(cc.total_replies, 0) as total_replies,
                    CASE WHEN cl.user_id IS NOT NULL THEN true ELSE false END as is_liked,
                    CASE WHEN cb.user_id IS NOT NULL THEN true ELSE false END as is_bookmarked,
                    pc.content::text as parent_content
             FROM convenientcomments c
             LEFT JOIN comment_counters cc ON c.commentid = cc.comment_id
             LEFT JOIN comment_likes cl ON c.commentid = cl.comment_id AND cl.user_id = $1::int4
             JOIN comment_bookmarks cb ON c.commentid = cb.comment_id
             JOIN threads t ON t.threadid = c.threadid
             LEFT JOIN convenientcomments pc ON c.parentid = pc.commentid
             WHERE cb.user_id = $1::int4
             ORDER BY cb.created_at DESC
             LIMIT $2 OFFSET $3",
            &[&user_id, &per_page, &offset],
        )
        .await?;

    let comment_ids: Vec<i32> = comments.iter().map(|row| row.get("commentid")).collect();
    let reactions_map = fetch_reactions(&transaction, &comment_ids, Some(user_id)).await?;

    let mapped_comments = comments
        .iter()
        .map(|row| {
            let comment_id = row.get::<_, i32>("commentid");
            Ok(Comment {
                parent_content: row
                    .get::<_, Option<String>>("parent_content")
                    .and_then(|s| serde_json::from_str(&s).ok()),
                comment_id,
                thread_id: row.get("threadid"),
                parent_id: row.get("parentid"),
                user_id: row.get("userid"),
                comment_num: row.get("commentnum"),
                time: row.get("time"),
                subject: row.get("subject"),
                content: serde_json::from_str(&row.get::<_, String>("content")).unwrap_or_default(),
                username: row.get("username"),
                realname: row.get("realname"),
                total_reactions: row.get::<_, Option<i64>>("total_reactions").unwrap_or(0),
                total_replies: row.get::<_, Option<i64>>("total_replies").unwrap_or(0),
                is_liked: row.get("is_liked"),
                is_bookmarked: row.get("is_bookmarked"),
                valsi_id: row.get("valsiid"),
                definition_id: row.get("definitionid"),
                reactions: reactions_map.get(&comment_id).cloned().unwrap_or_default(),
                valsi_word: None,
                definition: None,
                first_comment_subject: None,
                first_comment_content: None,
                last_comment_username: None,
            })
        })
        .collect::<Result<Vec<_>, Box<dyn std::error::Error>>>()?;

    transaction.commit().await?;

    Ok(PaginatedCommentsResponse {
        comments: mapped_comments,
        total,
        page,
        per_page,
    })
}

pub async fn get_liked_comments(
    pool: &Pool,
    user_id: i32,
    page: i64,
    per_page: i64,
) -> Result<PaginatedCommentsResponse, Box<dyn std::error::Error>> {
    let mut client = pool.get().await?;
    let transaction = client.transaction().await?;

    let offset = (page - 1) * per_page;

    // Get total count
    let total: i64 = transaction
        .query_one(
            "SELECT COUNT(*) FROM comment_likes WHERE user_id = $1",
            &[&user_id],
        )
        .await?
        .get(0);

    let comments = transaction
        .query(
            "SELECT
                        c.commentid,
                        c.threadid,
                        c.parentid,
                        c.userid,
                        c.commentnum,
                        c.time,
                        c.subject,
                        c.username,
                        c.realname,
                        c.total_reactions,
                        c.total_replies,
                        c.valsiid,
                        c.definitionid,
                        c.content::text as content,
                        t.*,
                    COALESCE(cc.total_reactions, 0) as total_reactions,
                    COALESCE(cc.total_replies, 0) as total_replies,
                    CASE WHEN cl.user_id IS NOT NULL THEN true ELSE false END as is_liked,
                    CASE WHEN cb.user_id IS NOT NULL THEN true ELSE false END as is_bookmarked
             FROM convenientcomments c
             LEFT JOIN comment_counters cc ON c.commentid = cc.comment_id
             LEFT JOIN comment_bookmarks cb ON c.commentid = cb.comment_id AND cb.user_id = $1
             JOIN comment_likes cl ON c.commentid = cl.comment_id
             JOIN threads t ON t.threadid = c.threadid
             WHERE cl.user_id = $1
             ORDER BY cl.created_at DESC
             LIMIT $2 OFFSET $3",
            &[&user_id, &per_page, &offset],
        )
        .await?;

    let comment_ids: Vec<i32> = comments.iter().map(|row| row.get("commentid")).collect();
    let reactions_map = fetch_reactions(&transaction, &comment_ids, Some(user_id)).await?;

    let mapped_comments = comments
        .iter()
        .map(|row| {
            let comment_id = row.get::<_, i32>("commentid");
            Ok(Comment {
                parent_content: None,
                comment_id,
                thread_id: row.get("threadid"),
                parent_id: row.get("parentid"),
                user_id: row.get("userid"),
                comment_num: row.get("commentnum"),
                time: row.get("time"),
                subject: row.get("subject"),
                content: serde_json::from_str(&row.get::<_, String>("content")).unwrap_or_default(),
                username: row.get("username"),
                realname: row.get("realname"),
                total_reactions: row.get::<_, Option<i64>>("total_reactions").unwrap_or(0),
                total_replies: row.get::<_, Option<i64>>("total_replies").unwrap_or(0),
                is_liked: row.get("is_liked"),
                is_bookmarked: row.get("is_bookmarked"),
                valsi_id: row.get("valsiid"),
                definition_id: row.get("definitionid"),
                reactions: reactions_map.get(&comment_id).cloned().unwrap_or_default(),
                valsi_word: None,
                definition: None,
                first_comment_subject: None,
                first_comment_content: None,
                last_comment_username: None,
            })
        })
        .collect::<Result<Vec<_>, Box<dyn std::error::Error>>>()?;

    transaction.commit().await?;

    Ok(PaginatedCommentsResponse {
        comments: mapped_comments,
        total,
        page,
        per_page,
    })
}

pub async fn toggle_bookmark(
    pool: &Pool,
    comment_id: i32,
    user_id: i32,
    bookmark: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut client = pool.get().await?;
    let transaction = client.transaction().await?;

    let is_bookmarked = transaction
        .query_opt(
            "SELECT 1 FROM comment_bookmarks
             WHERE comment_id = $1 AND user_id = $2",
            &[&comment_id, &user_id],
        )
        .await?
        .is_some();

    if bookmark != is_bookmarked {
        if bookmark {
            transaction
                .execute(
                    "INSERT INTO comment_bookmarks (comment_id, user_id, created_at)
                     VALUES ($1, $2, $3)",
                    &[&comment_id, &user_id, &Utc::now()],
                )
                .await?;
            transaction
                .execute(
                    "SELECT update_comment_counter($1, 'bookmarks', true)",
                    &[&comment_id],
                )
                .await?;
        } else {
            transaction
                .execute(
                    "DELETE FROM comment_bookmarks
                     WHERE comment_id = $1 AND user_id = $2",
                    &[&comment_id, &user_id],
                )
                .await?;
            transaction
                .execute(
                    "SELECT update_comment_counter($1, 'bookmarks', false)",
                    &[&comment_id],
                )
                .await?;
        }
    }

    transaction.commit().await?;
    Ok(())
}

pub async fn get_user_comments(
    pool: &Pool,
    user_id: i32,
    page: i64,
    per_page: i64,
    current_user_id: Option<i32>,
) -> Result<PaginatedCommentsResponse, Box<dyn std::error::Error>> {
    let mut client = pool.get().await?;
    let transaction = client.transaction().await?;

    let offset = (page - 1) * per_page;

    // Get total count
    let total: i64 = transaction
        .query_one(
            "SELECT COUNT(*) FROM comments WHERE userid = $1",
            &[&user_id],
        )
        .await?
        .get(0);

    let comment_rows = transaction
        .query(
            "SELECT
                    c.commentid,
                    c.threadid,
                    c.parentid,
                    c.userid,
                    c.commentnum,
                    c.time,
                    c.subject,
                    c.username,
                    c.realname,
                    c.total_reactions,
                    c.total_replies,
                    c.valsiid,
                    c.definitionid,
                    c.content::text as content,
                    COALESCE(cc.total_reactions, 0) as total_reactions,
                    COALESCE(cc.total_replies, 0) as total_replies,
                    CASE WHEN cl.user_id IS NOT NULL THEN true ELSE false END as is_liked,
                    CASE WHEN cb.user_id IS NOT NULL THEN true ELSE false END as is_bookmarked
             FROM convenientcomments c
             LEFT JOIN comment_counters cc ON c.commentid = cc.comment_id
             LEFT JOIN comment_likes cl ON c.commentid = cl.comment_id AND cl.user_id = $2
             LEFT JOIN comment_bookmarks cb ON c.commentid = cb.comment_id AND cb.user_id = $2
             WHERE c.userid = $1
             ORDER BY c.time DESC
             LIMIT $3 OFFSET $4",
            &[&user_id, &current_user_id, &per_page, &offset],
        )
        .await?;

    // Get comment IDs and fetch reactions
    let comment_ids: Vec<i32> = comment_rows
        .iter()
        .map(|row| row.get("commentid"))
        .collect();
    let reactions_map = fetch_reactions(&transaction, &comment_ids, current_user_id).await?;

    let comments = comment_rows
        .iter()
        .map(|row| {
            let comment_id = row.get::<_, i32>("commentid");
            Ok(Comment {
                parent_content: None,
                comment_id,
                thread_id: row.get("threadid"),
                parent_id: row.get("parentid"),
                user_id: row.get("userid"),
                comment_num: row.get("commentnum"),
                time: row.get("time"),
                subject: row.get("subject"),
                content: serde_json::from_str(&row.get::<_, String>("content")).unwrap_or_default(),
                username: row.get("username"),
                realname: row.get("realname"),
                total_reactions: row.get::<_, Option<i64>>("total_reactions").unwrap_or(0),
                total_replies: row.get::<_, Option<i64>>("total_replies").unwrap_or(0),
                is_liked: row.get("is_liked"),
                is_bookmarked: row.get("is_bookmarked"),
                valsi_id: None,
                definition_id: None,
                reactions: reactions_map.get(&comment_id).cloned().unwrap_or_default(),
                valsi_word: None,
                definition: None,
                first_comment_subject: None,
                first_comment_content: None,
                last_comment_username: None,
            })
        })
        .collect::<Result<Vec<_>, Box<dyn std::error::Error>>>()?;

    transaction.commit().await?;

    Ok(PaginatedCommentsResponse {
        comments,
        total,
        page,
        per_page,
    })
}

pub async fn create_opinion(
    pool: &Pool,
    user_id: i32,
    request: &CreateOpinionRequest,
) -> Result<CommentOpinion, Box<dyn std::error::Error>> {
    let content = CommentOpinion::parse(&request.opinion)
        .ok_or_else(|| "Invalid opinion content".to_string())?;

    let mut client = pool.get().await?;
    let transaction = client.transaction().await?;

    // Insert the opinion
    let opinion = transaction
        .query_one(
            "INSERT INTO comment_opinions
            (opinion, comment_id, user_id, votes)
            VALUES ($1, $2, $3, 1)
            RETURNING id, opinion, comment_id, user_id, votes, created_at",
            &[&content, &request.comment_id, &user_id],
        )
        .await?;

    // The person who created the opinion gets an automatic vote
    transaction
        .execute(
            "INSERT INTO comment_opinion_votes
            (opinion_id, comment_id, user_id)
            VALUES ($1, $2, $3)",
            &[&opinion.get::<_, i64>("id"), &request.comment_id, &user_id],
        )
        .await?;

    let result = CommentOpinion {
        id: opinion.get("id"),
        opinion: opinion.get("opinion"),
        comment_id: opinion.get("comment_id"),
        user_id: opinion.get("user_id"),
        votes: opinion.get("votes"),
        voted: true,
        created_at: opinion.get("created_at"),
    };

    transaction.commit().await?;
    Ok(result)
}

pub async fn set_opinion_vote(
    pool: &Pool,
    user_id: i32,
    request: &OpinionVoteRequest,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut client = pool.get().await?;
    let transaction = client.transaction().await?;

    let has_voted = transaction
        .query_opt(
            "SELECT 1 FROM comment_opinion_votes
            WHERE opinion_id = $1 AND user_id = $2 AND comment_id = $3",
            &[&request.opinion_id, &user_id, &request.comment_id],
        )
        .await?
        .is_some();

    if request.vote != has_voted {
        if request.vote {
            // Add vote
            transaction
                .execute(
                    "INSERT INTO comment_opinion_votes
                    (opinion_id, comment_id, user_id)
                    VALUES ($1, $2, $3)",
                    &[&request.opinion_id, &request.comment_id, &user_id],
                )
                .await?;

            transaction
                .execute(
                    "UPDATE comment_opinions
                    SET votes = votes + 1
                    WHERE id = $1",
                    &[&request.opinion_id],
                )
                .await?;
        } else {
            // Remove vote
            transaction
                .execute(
                    "DELETE FROM comment_opinion_votes
                    WHERE opinion_id = $1 AND user_id = $2 AND comment_id = $3",
                    &[&request.opinion_id, &user_id, &request.comment_id],
                )
                .await?;

            transaction
                .execute(
                    "UPDATE comment_opinions
                    SET votes = votes - 1
                    WHERE id = $1",
                    &[&request.opinion_id],
                )
                .await?;
        }
    }

    transaction.commit().await?;
    Ok(())
}

pub async fn get_comment_opinions(
    pool: &Pool,
    comment_id: i32,
    user_id: Option<i32>,
) -> Result<Vec<CommentOpinion>, Box<dyn std::error::Error>> {
    let client = pool.get().await?;

    let opinions = client
        .query(
            "SELECT
                co.id,
                co.opinion,
                co.comment_id,
                co.user_id,
                co.votes,
                co.created_at,
                CASE WHEN cov.user_id IS NOT NULL THEN true ELSE false END as voted
            FROM comment_opinions co
            LEFT JOIN comment_opinion_votes cov
                ON co.id = cov.opinion_id
                AND cov.user_id = $2
            WHERE co.comment_id = $1
            ORDER BY co.votes DESC
            LIMIT 5",
            &[&comment_id, &user_id],
        )
        .await?;

    let result = opinions
        .iter()
        .map(|row| CommentOpinion {
            id: row.get("id"),
            opinion: row.get("opinion"),
            comment_id: row.get("comment_id"),
            user_id: row.get("user_id"),
            votes: row.get("votes"),
            voted: row.get("voted"),
            created_at: row.get("created_at"),
        })
        .collect();

    Ok(result)
}

pub async fn get_trending_comments(
    pool: &Pool,
    timespan: TrendingTimespan,
    current_user_id: Option<i32>,
    limit: i32,
) -> Result<Vec<Comment>, Box<dyn std::error::Error>> {
    let mut client = pool.get().await?;
    let transaction = client.transaction().await?;

    let hours = match timespan {
        TrendingTimespan::LastDay => 24,
        TrendingTimespan::LastWeek => 24 * 7,
        TrendingTimespan::LastMonth => 24 * 30,
        TrendingTimespan::LastYear => 24 * 365,
        TrendingTimespan::AllTime => 24 * 365 * 100,
    };

    let comments = transaction
        .query(
            "WITH ranked_comments AS (
                SELECT
                    c.commentid,
                    c.threadid,
                    c.parentid,
                    c.userid,
                    c.commentnum,
                    c.time,
                    c.subject,
                    c.username,
                    c.realname,
                    c.total_reactions,
                    c.total_replies,
                    c.valsiid,
                    c.definitionid,
                    c.content::text as content,
                    pc.content::text as parent_content,
                    t.valsiid,
                    t.definitionid,
                    COALESCE(cc.total_reactions, 0)::bigint as comment_reactions,
                    COALESCE(cc.total_replies, 0)::bigint as comment_replies,
                    COALESCE(cc.total_bookmarks, 0)::bigint as comment_bookmarks,
                    COALESCE(cb.user_id IS NOT NULL, false) as is_bookmarked,
                    COALESCE(cc.total_reactions, 0) + COALESCE(cc.total_replies, 0) + COALESCE(cc.total_bookmarks, 0) as engagement_score
                FROM convenientcomments c
                LEFT JOIN threads t ON c.threadid = t.threadid
                LEFT JOIN comments pc ON c.parentid = pc.commentid
                LEFT JOIN comment_activity_counters cc ON c.commentid = cc.comment_id
                LEFT JOIN comment_likes cl ON c.commentid = cl.comment_id AND cl.user_id = $1
                LEFT JOIN comment_bookmarks cb ON c.commentid = cb.comment_id AND cb.user_id = $1
                WHERE c.time > extract(epoch from (now() - make_interval(hours => $2)))::int
                AND (cc.total_reactions > 0 OR cc.total_replies > 0 OR cc.total_bookmarks > 0)
            )
            SELECT *
            FROM ranked_comments
            ORDER BY engagement_score DESC
            LIMIT $3",
            &[&current_user_id, &hours, &i64::from(limit)],
        )
        .await?;

    let comment_ids: Vec<i32> = comments.iter().map(|row| row.get("commentid")).collect();
    let reactions_map = fetch_reactions(&transaction, &comment_ids, current_user_id).await?;

    let mapped_comments = comments
        .iter()
        .map(|row| {
            let comment_id = row.get::<_, i32>("commentid");

            Comment {
                parent_content: row
                    .get::<_, Option<String>>("parent_content")
                    .and_then(|s| serde_json::from_str(&s).ok()),
                comment_id,
                thread_id: row.get("threadid"),
                parent_id: row.get("parentid"),
                user_id: row.get("userid"),
                comment_num: row.get("commentnum"),
                time: row.get("time"),
                subject: row.get("subject"),
                content: serde_json::from_str(&row.get::<_, String>("content")).unwrap_or_default(),
                username: row.get("username"),
                realname: row.get("realname"),
                total_reactions: row.get::<_, Option<i64>>("total_reactions").unwrap_or(0),
                total_replies: row.get::<_, Option<i64>>("total_replies").unwrap_or(0),
                is_liked: None,
                is_bookmarked: row.get("is_bookmarked"),
                valsi_id: row.get("valsiid"),
                definition_id: row.get("definitionid"),
                reactions: reactions_map.get(&comment_id).cloned().unwrap_or_default(),
                valsi_word: None,
                definition: None,
                first_comment_subject: None,
                first_comment_content: None,
                last_comment_username: None,
            }
        })
        .collect();

    transaction.commit().await?;
    Ok(mapped_comments)
}

pub async fn get_comment_stats(
    pool: &Pool,
    comment_id: i32,
) -> Result<CommentStats, Box<dyn std::error::Error>> {
    let client = pool.get().await?;

    let row = client
        .query_one(
            "SELECT total_likes, total_bookmarks, total_replies,
                    total_opinions, total_reactions, last_activity_at
             FROM comment_activity_counters
             WHERE comment_id = $1",
            &[&comment_id],
        )
        .await?;

    Ok(CommentStats {
        total_likes: row.get("total_likes"),
        total_replies: row.get("total_replies"),
        total_bookmarks: row.get("total_bookmarks"),
        total_opinions: row.get("total_opinions"),
        total_reactions: row.get("total_reactions"),
        last_activity_at: row.get("last_activity_at"),
    })
}

pub async fn get_most_bookmarked_comments(
    pool: &Pool,
    page: i64,
    per_page: i64,
    current_user_id: Option<i32>,
) -> Result<PaginatedCommentsResponse, Box<dyn std::error::Error>> {
    let offset = (page - 1) * per_page;
    let mut client = pool.get().await?;
    let transaction = client.transaction().await?;

    // Get total count first
    let total: i64 = transaction
        .query_one("SELECT COUNT(*) FROM comments", &[])
        .await?
        .get(0);

    let comments = transaction
        .query(
            "SELECT
                c.commentid,
                c.threadid,
                c.parentid,
                c.userid,
                c.commentnum,
                c.time,
                c.subject,
                c.username,
                c.realname,
                c.total_reactions,
                c.total_replies,
                c.valsiid,
                c.definitionid,
                c.content::text as content,
                u.username, u.realname,
                ac.total_reactions, ac.total_replies,
                CASE WHEN cl.user_id IS NOT NULL THEN true ELSE false END as is_liked,
                CASE WHEN cb.user_id IS NOT NULL THEN true ELSE false END as is_bookmarked
             FROM comments c
             JOIN users u ON c.userid = u.userid
             JOIN comment_activity_counters ac ON c.commentid = ac.comment_id
             LEFT JOIN comment_likes cl ON c.commentid = cl.comment_id AND cl.user_id = $1
             LEFT JOIN comment_bookmarks cb ON c.commentid = cb.comment_id AND cb.user_id = $1
             ORDER BY ac.total_bookmarks DESC
             LIMIT $2 OFFSET $3",
            &[&current_user_id, &per_page, &offset],
        )
        .await?;

    let comment_ids: Vec<i32> = comments.iter().map(|row| row.get("commentid")).collect();
    let reactions_map = fetch_reactions(&transaction, &comment_ids, current_user_id).await?;

    let mapped_comments = comments
        .iter()
        .map(|row| {
            let comment_id = row.get::<_, i32>("commentid");
            Comment {
                parent_content: None,
                comment_id,
                thread_id: row.get("threadid"),
                parent_id: row.get("parentid"),
                user_id: row.get("userid"),
                comment_num: row.get("commentnum"),
                time: row.get("time"),
                subject: row.get("subject"),
                content: serde_json::from_str(&row.get::<_, String>("content")).unwrap_or_default(),
                username: row.get("username"),
                realname: row.get("realname"),
                total_reactions: row.get::<_, Option<i64>>("total_reactions").unwrap_or(0),
                total_replies: row.get::<_, Option<i64>>("total_replies").unwrap_or(0),
                is_liked: row.get("is_liked"),
                is_bookmarked: row.get("is_bookmarked"),
                valsi_id: None,
                definition_id: None,
                reactions: reactions_map.get(&comment_id).cloned().unwrap_or_default(),
                valsi_word: None,
                definition: None,
                first_comment_subject: None,
                first_comment_content: None,
                last_comment_username: None,
            }
        })
        .collect();

    transaction.commit().await?;

    Ok(PaginatedCommentsResponse {
        comments: mapped_comments,
        total,
        page,
        per_page,
    })
}

pub async fn get_trending_hashtags(
    pool: &Pool,
    timespan: TrendingTimespan,
    limit: i32,
) -> Result<Vec<TrendingHashtag>, Box<dyn std::error::Error>> {
    let client = pool.get().await?;

    let hours = match timespan {
        TrendingTimespan::LastDay => 24,
        TrendingTimespan::LastWeek => 24 * 7,
        TrendingTimespan::LastMonth => 24 * 30,
        TrendingTimespan::LastYear => 24 * 365,
        TrendingTimespan::AllTime => 24 * 365 * 100,
    };

    let rows = client
        .query(
            r#"
            SELECT h.tag, COUNT(*) as usage_count, MAX(CURRENT_TIMESTAMP) as last_used
            FROM post_hashtags ph
            JOIN hashtags h ON h.id = ph.hashtag_id
            JOIN comments c ON c.commentid = ph.post_id
            WHERE to_timestamp(c.time) >= NOW() - ($1 || ' hours')::INTERVAL
            GROUP BY h.tag
            ORDER BY usage_count DESC
            LIMIT $2
            "#,
            &[&hours, &limit],
        )
        .await?;

    let hashtags = rows
        .iter()
        .map(|row| TrendingHashtag {
            tag: row.get("tag"),
            usage_count: row.get("usage_count"),
            last_used: row.get("last_used"),
        })
        .collect();

    Ok(hashtags)
}

pub async fn get_comments_by_hashtag(
    pool: &Pool,
    tag: &str,
    user_id: Option<i32>,
    page: Option<i64>,
    per_page: Option<i64>,
) -> Result<PaginatedCommentsResponse, Box<dyn std::error::Error>> {
    let mut client = pool.get().await?;
    let transaction = client.transaction().await?;

    let page = page.unwrap_or(1);
    let per_page = per_page.unwrap_or(20);
    let offset = (page - 1) * per_page;

    // Get total count first
    let total: i64 = transaction
        .query_one(
            "SELECT COUNT(*) FROM comments c
             JOIN post_hashtags ph ON c.commentid = ph.post_id
             JOIN hashtags h ON ph.hashtag_id = h.id
             WHERE h.tag = $1",
            &[&tag],
        )
        .await?
        .get(0);

    let rows = transaction
        .query(
            r#"
            SELECT c.commentid, c.threadid, c.parentid, c.userid,
                   c.commentnum, c.time, c.subject, c.content::text as content,
                   u.username, u.realname,
                   cc.total_reactions, cc.total_replies,
                   CASE WHEN cl.user_id IS NOT NULL THEN true ELSE false END as is_liked,
                   CASE WHEN cb.user_id IS NOT NULL THEN true ELSE false END as is_bookmarked
            FROM comments c
            JOIN users u ON c.userid = u.userid
            JOIN comment_counters cc ON c.commentid = cc.comment_id
            JOIN post_hashtags ph ON c.commentid = ph.post_id
            JOIN hashtags h ON ph.hashtag_id = h.id
            LEFT JOIN comment_likes cl ON c.commentid = cl.comment_id AND cl.user_id = $2
            LEFT JOIN comment_bookmarks cb ON c.commentid = cb.comment_id AND cb.user_id = $2
            WHERE h.tag = $1
            ORDER BY c.time DESC
            LIMIT $3 OFFSET $4
            "#,
            &[&tag, &user_id, &per_page, &offset],
        )
        .await?;

    let comment_ids: Vec<i32> = rows.iter().map(|row| row.get("commentid")).collect();
    let reactions_map = fetch_reactions(&transaction, &comment_ids, user_id).await?;

    let comments = rows
        .iter()
        .map(|row| {
            let comment_id = row.get::<_, i32>("commentid");
            Comment {
                parent_content: None,
                comment_id,
                thread_id: row.get("threadid"),
                parent_id: row.get("parentid"),
                user_id: row.get("userid"),
                comment_num: row.get("commentnum"),
                time: row.get("time"),
                subject: row.get("subject"),
                content: serde_json::from_str(&row.get::<_, String>("content")).unwrap_or_default(),
                username: row.get("username"),
                realname: row.get("realname"),
                total_reactions: row.get::<_, Option<i64>>("total_reactions").unwrap_or(0),
                total_replies: row.get::<_, Option<i64>>("total_replies").unwrap_or(0),
                is_liked: row.get("is_liked"),
                is_bookmarked: row.get("is_bookmarked"),
                valsi_id: None,
                definition_id: None,
                reactions: reactions_map.get(&comment_id).cloned().unwrap_or_default(),
                valsi_word: None,
                definition: None,
                first_comment_subject: None,
                first_comment_content: None,
                last_comment_username: None,
            }
        })
        .collect();

    transaction.commit().await?;

    Ok(PaginatedCommentsResponse {
        comments,
        total,
        page,
        per_page,
    })
}

pub async fn delete_comment(
    pool: &Pool,
    comment_id: i32,
    user_id: i32,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut client = pool.get().await?;
    let transaction = client.transaction().await?;

    // Verify comment ownership and check for replies
    let row = transaction
        .query_opt(
            "SELECT c.userid, cc.total_replies, c.threadid
             FROM comments c
             JOIN comment_counters cc ON c.commentid = cc.comment_id
             WHERE c.commentid = $1",
            &[&comment_id],
        )
        .await?;

    let row = match row {
        Some(r) => r,
        None => return Err("Comment not found".into()),
    };

    let comment_user_id: i32 = row.get("userid");
    let total_replies: i64 = row.get("total_replies");
    let thread_id: i32 = row.get("threadid");

    if comment_user_id != user_id {
        return Err("Unauthorized: You can only delete your own comments".into());
    }

    if total_replies > 0 {
        return Err("Cannot delete comment with replies".into());
    }

    // Delete related data
    transaction
        .execute(
            "DELETE FROM comment_reactions WHERE comment_id = $1",
            &[&comment_id],
        )
        .await?;

    transaction
        .execute(
            "DELETE FROM comment_opinions WHERE comment_id = $1",
            &[&comment_id],
        )
        .await?;

    transaction
        .execute(
            "DELETE FROM valsi_subscriptions WHERE source_comment_id = $1",
            &[&comment_id],
        )
        .await?;

    transaction
        .execute(
            "DELETE FROM comment_likes WHERE comment_id = $1",
            &[&comment_id],
        )
        .await?;

    transaction
        .execute(
            "DELETE FROM comment_bookmarks WHERE comment_id = $1",
            &[&comment_id],
        )
        .await?;

    // Get parent ID before deleting
    let parent_id: Option<i32> = transaction
        .query_one(
            "SELECT parentid FROM comments WHERE commentid = $1",
            &[&comment_id],
        )
        .await?
        .get("parentid");

    // Update thread stats to ensure foreign key consistency
    transaction
        .execute(
            "UPDATE threads SET
            last_comment_id = null
         WHERE threadid = $1",
            &[&thread_id],
        )
        .await?;

    // Delete the comment
    transaction
        .execute("DELETE FROM comments WHERE commentid = $1", &[&comment_id])
        .await?;

    // Delete the thread if it's empty
    transaction
        .execute(
            "DELETE FROM threads WHERE threadid = $1 and total_comments = 0",
            &[&thread_id],
        )
        .await?;

    // Update parent reply count if exists
    if let Some(parent_id) = parent_id {
        transaction
            .execute(
                "UPDATE comment_counters
                 SET total_replies = total_replies - 1
                 WHERE comment_id = $1",
                &[&parent_id],
            )
            .await?;
    }

    transaction.commit().await?;
    Ok(())
}

pub async fn toggle_reaction(
    pool: &Pool,
    comment_id: i32,
    user_id: i32,
    reaction: &str,
) -> Result<bool, Box<dyn std::error::Error>> {
    // Basic validation
    if reaction.len() > 32 {
        return Err(Box::new(ReactionError::TooLong));
    }

    let mut client = pool.get().await?;
    let transaction = client.transaction().await?;

    // Check if reaction exists
    let exists = transaction
        .query_opt(
            "SELECT 1 FROM comment_reactions
             WHERE comment_id = $1 AND user_id = $2 AND reaction = $3",
            &[&comment_id, &user_id, &reaction],
        )
        .await?
        .is_some();

    if exists {
        // Remove reaction
        transaction
            .execute(
                "DELETE FROM comment_reactions
                 WHERE comment_id = $1 AND user_id = $2 AND reaction = $3",
                &[&comment_id, &user_id, &reaction],
            )
            .await?;
        transaction.commit().await?;
        Ok(false)
    } else {
        // check how many reactions you added
        let can_add = transaction
            .query_one(
                "SELECT check_reaction_limit($1, $2)",
                &[&user_id, &comment_id],
            )
            .await?
            .get::<_, bool>(0);

        if !can_add {
            return Err(Box::new(ReactionError::LimitReached));
        }

        // Add reaction
        transaction
            .execute(
                "INSERT INTO comment_reactions (comment_id, user_id, reaction)
                 VALUES ($1, $2, $3)",
                &[&comment_id, &user_id, &reaction],
            )
            .await?;
        transaction.commit().await?;
        Ok(true)
    }
}

pub async fn search_comments(
    pool: &Pool,
    search_params: SearchCommentsParams,
    current_user_id: Option<i32>,
) -> Result<PaginatedCommentsResponse, Box<dyn std::error::Error>> {
    let mut client = pool.get().await?;
    let transaction = client.transaction().await?;

    let offset = (search_params.page - 1) * search_params.per_page;
    let search_pattern = format!("%{}%", search_params.search_term);

    // Build the base query with proper joins
    let base_query = "
        SELECT
            c.commentid,
            c.threadid,
            c.parentid,
            c.userid,
            c.commentnum,
            c.time,
            c.subject,
            c.content::text as content,
            cc.total_reactions,
            cc.total_replies,
            CASE WHEN cl.user_id IS NOT NULL THEN true ELSE false END as is_liked,
            CASE WHEN cb.user_id IS NOT NULL THEN true ELSE false END as is_bookmarked,
            t.valsiid,
            t.definitionid,
            u.username,
            t.target_user_id,
            v.word as valsi_word,
            d.definition as definition
        FROM comments c
        JOIN users u ON c.userid = u.userid
        JOIN threads t ON c.threadid = t.threadid
        LEFT JOIN valsi v ON t.valsiid = v.valsiid
        LEFT JOIN definitions d ON t.definitionid = d.definitionid
        LEFT JOIN comment_activity_counters cc ON c.commentid = cc.comment_id
        LEFT JOIN comment_likes cl ON c.commentid = cl.comment_id AND cl.user_id = $1
        LEFT JOIN comment_bookmarks cb ON c.commentid = cb.comment_id AND cb.user_id = $1
        WHERE (c.subject ILIKE $2 OR c.plain_content ILIKE $2 OR u.username ILIKE $2)";

    let mut conditions = Vec::new();
    let mut query_params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> =
        vec![&current_user_id, &search_pattern];

    let mut param_count = 3;

    // Add optional filters
    if let Some(username) = &search_params.username {
        conditions.push(format!("u.username = ${}", param_count));
        query_params.push(username);
        param_count += 1;
    }

    let valsi_id_value: i32;
    if let Some(valsi_id) = search_params.valsi_id {
        valsi_id_value = valsi_id;
        conditions.push(format!("t.valsiid = ${}", param_count));
        query_params.push(&valsi_id_value);
        param_count += 1;
    }

    let definition_id_value: i32;
    if let Some(definition_id) = search_params.definition_id {
        definition_id_value = definition_id;
        conditions.push(format!("t.definitionid = ${}", param_count));
        query_params.push(&definition_id_value);
        param_count += 1;
    }

    let target_user_id_value: i32;
    if let Some(target_user_id) = search_params.target_user_id {
        target_user_id_value = target_user_id;
        conditions.push(format!("t.target_user_id = ${}", param_count));
        query_params.push(&target_user_id_value);
        // param_count += 1; // Not needed if it's the last one before pagination params
    }

    // Construct the WHERE clause
    let where_clause = if !conditions.is_empty() {
        format!("{} AND {}", base_query, conditions.join(" AND "))
    } else {
        base_query.to_string()
    };

    // Add sorting
    let sort_field = match search_params.sort_by.as_str() {
        "reactions" => "cc.total_reactions",
        "replies" => "cc.total_replies",
        _ => "c.time",
    };
    let sort_order = if search_params.sort_order == "asc" {
        "ASC"
    } else {
        "DESC"
    };

    // Construct final queries
    let count_query = format!("SELECT COUNT(*) FROM ({}) as subquery", where_clause);

    let final_query = format!(
        "{} ORDER BY {} {} LIMIT ${} OFFSET ${}",
        where_clause,
        sort_field,
        sort_order,
        param_count,
        param_count + 1
    );

    // Add pagination parameters
    query_params.push(&search_params.per_page);
    query_params.push(&offset);

    // Get total count
    let total: i64 = transaction
        .query_one(&count_query, &query_params[..query_params.len() - 2])
        .await?
        .get(0);

    // Get paginated results
    let comments = transaction.query(&final_query, &query_params).await?;

    // Get comment IDs and fetch reactions
    let comment_ids: Vec<i32> = comments.iter().map(|row| row.get("commentid")).collect();
    let reactions_map = fetch_reactions(&transaction, &comment_ids, current_user_id).await?;

    let mapped_comments = comments
        .iter()
        .map(|row| {
            let comment_id = row.get::<_, i32>("commentid");
            Comment {
                parent_content: None,
                comment_id,
                thread_id: row.get("threadid"),
                parent_id: row.get("parentid"),
                user_id: row.get("userid"),
                comment_num: row.get("commentnum"),
                time: row.get("time"),
                subject: row.get("subject"),
                content: serde_json::from_str(&row.get::<_, String>("content")).unwrap_or_default(),
                username: Some(row.get("username")),
                realname: None,
                total_reactions: row.get::<_, Option<i64>>("total_reactions").unwrap_or(0),
                total_replies: row.get::<_, Option<i64>>("total_replies").unwrap_or(0),
                is_liked: row.get("is_liked"),
                is_bookmarked: row.get("is_bookmarked"),
                valsi_id: row.get("valsiid"),
                definition_id: row.get("definitionid"),
                reactions: reactions_map.get(&comment_id).cloned().unwrap_or_default(),
                valsi_word: row.get("valsi_word"),
                definition: row.get("definition"),
                first_comment_subject: None,
                first_comment_content: None,
                last_comment_username: None,
            }
        })
        .collect();

    transaction.commit().await?;

    Ok(PaginatedCommentsResponse {
        comments: mapped_comments,
        total,
        page: search_params.page,
        per_page: search_params.per_page,
    })
}

pub async fn get_my_reactions(
    pool: &Pool,
    user_id: i32,
    page: i64,
    per_page: i64,
) -> Result<PaginatedCommentsResponse, Box<dyn std::error::Error>> {
    let mut client = pool.get().await?;
    let transaction = client.transaction().await?;

    let offset = (page - 1) * per_page;

    // Get total count of distinct comments where user has reactions
    let total: i64 = transaction
        .query_one(
            "SELECT COUNT(DISTINCT comment_id) FROM comment_reactions WHERE user_id = $1",
            &[&user_id],
        )
        .await?
        .get(0);

    // Get distinct comments where user has reactions
    let comments = transaction
        .query(
            "SELECT DISTINCT
            c.commentid,
            c.threadid,
            c.parentid,
            c.userid,
            c.commentnum,
            c.time,
            c.subject,
            c.username,
            c.realname,
            c.total_reactions,
            c.total_replies,
            c.valsiid,
            c.definitionid,
            c.content::text as content,
            t.*,
            COALESCE(cc.total_reactions, 0) as total_reactions,
            COALESCE(cc.total_replies, 0) as total_replies,
            CASE WHEN cl.user_id IS NOT NULL THEN true ELSE false END as is_liked,
            CASE WHEN cb.user_id IS NOT NULL THEN true ELSE false END as is_bookmarked
             FROM convenientcomments c
             LEFT JOIN comment_counters cc ON c.commentid = cc.comment_id
             LEFT JOIN comment_likes cl ON c.commentid = cl.comment_id AND cl.user_id = $1
             LEFT JOIN comment_bookmarks cb ON c.commentid = cb.comment_id AND cb.user_id = $1
             JOIN threads t ON t.threadid = c.threadid
             WHERE c.commentid IN (
                 SELECT DISTINCT comment_id
                 FROM comment_reactions
                 WHERE user_id = $1
             )
             ORDER BY c.time DESC
             LIMIT $2 OFFSET $3",
            &[&user_id, &per_page, &offset],
        )
        .await?;

    let comment_ids: Vec<i32> = comments.iter().map(|row| row.get("commentid")).collect();
    let reactions_map = fetch_reactions(&transaction, &comment_ids, Some(user_id)).await?;

    let mapped_comments = comments
        .iter()
        .map(|row| {
            let comment_id = row.get::<_, i32>("commentid");
            Comment {
                parent_content: None,
                comment_id,
                thread_id: row.get("threadid"),
                parent_id: row.get("parentid"),
                user_id: row.get("userid"),
                comment_num: row.get("commentnum"),
                time: row.get("time"),
                subject: row.get("subject"),
                content: serde_json::from_str(&row.get::<_, String>("content")).unwrap_or_default(),
                username: row.get("username"),
                realname: row.get("realname"),
                total_reactions: row.get::<_, Option<i64>>("total_reactions").unwrap_or(0),
                total_replies: row.get::<_, Option<i64>>("total_replies").unwrap_or(0),
                is_liked: row.get("is_liked"),
                is_bookmarked: row.get("is_bookmarked"),
                valsi_id: row.get("valsiid"),
                definition_id: row.get("definitionid"),
                reactions: reactions_map.get(&comment_id).cloned().unwrap_or_default(),
                valsi_word: None,
                definition: None,
                first_comment_subject: None,
                first_comment_content: None,
                last_comment_username: None,
            }
        })
        .collect();

    transaction.commit().await?;

    Ok(PaginatedCommentsResponse {
        comments: mapped_comments,
        total,
        page,
        per_page,
    })
}

pub async fn get_reactions(
    pool: &Pool,
    comment_id: i32,
    current_user_id: Option<i32>,
    page: Option<i64>,
    page_size: Option<i32>,
) -> Result<ReactionSummary, Box<dyn std::error::Error>> {
    let mut client = pool.get().await?;
    let transaction = client.transaction().await?;

    let page = page.unwrap_or(1);
    let page_size = page_size.unwrap_or(10);
    let offset = (page - 1) * page_size as i64;

    // Get total count first
    let total_count: i64 = transaction
        .query_one(
            "SELECT COUNT(*) FROM (
                SELECT DISTINCT reaction
                FROM comment_reactions
                WHERE comment_id = $1
            ) as distinct_reactions",
            &[&comment_id],
        )
        .await?
        .get(0);

    // Get paginated reactions
    let reactions = transaction
        .query(
            "SELECT
                cr.reaction,
                COUNT(*) as count,
                BOOL_OR(cr.user_id = $1) as reacted
             FROM comment_reactions cr
             WHERE cr.comment_id = $2
             GROUP BY cr.reaction
             ORDER BY count DESC, reaction
             LIMIT $3 OFFSET $4",
            &[&current_user_id, &comment_id, &page_size, &offset],
        )
        .await?;

    let total_reactions: i64 = transaction
        .query_one(
            "SELECT COUNT(*) FROM comment_reactions WHERE comment_id = $1",
            &[&comment_id],
        )
        .await?
        .get(0);

    let total_pages = (total_count as f64 / page_size as f64).ceil() as i64;

    let reaction_list = reactions
        .iter()
        .map(|row| ReactionResponse {
            reaction: row.get("reaction"),
            count: row.get("count"),
            reacted: row.get("reacted"),
        })
        .collect();

    let paginated_reactions = PaginatedReactions {
        reactions: reaction_list,
        total_reactions,
        total_pages,
        current_page: page,
        page_size,
    };

    transaction.commit().await?;

    Ok(ReactionSummary {
        reactions: paginated_reactions,
        total_distinct_reactions: total_count,
    })
}

async fn fetch_reactions(
    transaction: &tokio_postgres::Transaction<'_>,
    comment_ids: &[i32],
    current_user_id: Option<i32>,
) -> Result<HashMap<i32, Vec<ReactionResponse>>, Box<dyn std::error::Error>> {
    let reactions = transaction
        .query(
            "SELECT
                cr.comment_id,
                cr.reaction,
                COUNT(*) as count,
                COALESCE(BOOL_OR(cr.user_id = $2), false) as reacted
             FROM comment_reactions cr
             WHERE cr.comment_id = ANY($1)
             GROUP BY cr.comment_id, cr.reaction
             ORDER BY cr.comment_id, count DESC, cr.reaction",
            &[&comment_ids, &current_user_id],
        )
        .await?;

    let mut reactions_map: HashMap<i32, Vec<ReactionResponse>> = HashMap::new();

    for row in reactions.iter() {
        let comment_id: i32 = row.get("comment_id");
        let reaction = ReactionResponse {
            reaction: row.get("reaction"),
            count: row.get("count"),
            reacted: row.get("reacted"),
        };
        reactions_map.entry(comment_id).or_default().push(reaction);
    }

    for &comment_id in comment_ids {
        reactions_map.entry(comment_id).or_default();
    }

    Ok(reactions_map)
}

async fn get_or_create_thread_id(
    transaction: &tokio_postgres::Transaction<'_>,
    valsi_id: Option<i32>,
    natlang_word_id: Option<i32>,
    definition_id: Option<i32>,
    target_user_id: Option<i32>,
) -> Result<i32, Box<dyn std::error::Error>> {
    // Validate that only one context type is primarily active or it's a free-standing thread
    let mut active_contexts = 0;
    if valsi_id.is_some() || natlang_word_id.is_some() || definition_id.is_some() {
        active_contexts += 1;
    }
    if target_user_id.is_some() {
        active_contexts += 1;
    }

    if active_contexts > 1 {
        return Err("Ambiguous thread context: Multiple context IDs (e.g., valsi_id and target_user_id) provided. Only one type of context or none (for free-standing threads) is allowed.".into());
    }

    let query_select = "
        SELECT threadid FROM threads
        WHERE (valsiid = $1 OR ($1 IS NULL AND valsiid IS NULL))
          AND (natlangwordid = $2 OR ($2 IS NULL AND natlangwordid IS NULL))
          AND (definitionid = $3 OR ($3 IS NULL AND definitionid IS NULL))
          AND (target_user_id = $4 OR ($4 IS NULL AND target_user_id IS NULL))
        LIMIT 1";

    if let Some(row) = transaction
        .query_opt(
            query_select,
            &[&valsi_id, &natlang_word_id, &definition_id, &target_user_id],
        )
        .await?
    {
        Ok(row.get("threadid"))
    } else {
        let query_insert = "
            INSERT INTO threads (valsiid, natlangwordid, definitionid, target_user_id)
            VALUES ($1, $2, $3, $4)
            RETURNING threadid";
        Ok(transaction
            .query_one(
                query_insert,
                &[&valsi_id, &natlang_word_id, &definition_id, &target_user_id],
            )
            .await?
            .get("threadid"))
    }
}

pub async fn list_threads(
    pool: &Pool,
    page: i64,
    per_page: i64,
    sort_by: &str,
    sort_order: &str,
) -> Result<PaginatedCommentsResponse, Box<dyn std::error::Error>> {
    let mut client = pool.get().await?;
    let transaction = client.transaction().await?;
    let offset = (page - 1) * per_page;

    // Get total count
    let total: i64 = transaction
        .query_one(
            "SELECT COUNT(t.threadid)
             FROM threads t
             where t.total_comments > 0
             ",
            &[],
        )
        .await?
        .get(0);

    // Validate sort parameters
    let sort_column = match sort_by {
        "comments" => "t.total_comments",
        "subject" => "t.last_comment_subject",
        _ => "t.last_comment_time", // default to time
    };
    let sort_dir = if sort_order.eq_ignore_ascii_case("asc") {
        "ASC"
    } else {
        "DESC"
    };

    // Get paginated results using new thread stats columns
    let threads = transaction
        .query(
            &format!(
                "SELECT
                    t.threadid,
                    t.valsiid,
                    t.definitionid,
                    t.target_user_id,
                    v.word as valsi_word,
                    d.definition,
                    t.first_comment_content::text as first_comment_content,
                    u.username,
                    u.realname,
                    t.total_comments,
                    t.creator_user_id as userid,
                    t.last_comment_id,
                    t.last_comment_user_id,
                    t.last_comment_time,
                    ul.username as last_comment_username,
                    coalesce(t.last_comment_subject, '') as last_comment_subject,
                    t.last_comment_content::text as last_comment_content,
                    coalesce(t.first_comment_subject, '') as first_comment_subject
                FROM threads t
                JOIN users u ON t.creator_user_id = u.userid
                JOIN users ul ON t.last_comment_user_id = ul.userid
                LEFT JOIN valsi v ON t.valsiid = v.valsiid
                LEFT JOIN definitions d ON t.definitionid = d.definitionid
                where t.total_comments > 0
                ORDER BY {} {}
                LIMIT $1 OFFSET $2",
                sort_column, sort_dir
            ),
            &[&per_page, &offset],
        )
        .await?;

    let mapped_threads: Vec<FreeThread> = threads
        .iter()
        .map(|row| FreeThread {
            thread_id: row.get("threadid"),
            valsiid: row.get("valsiid"),
            definitionid: row.get("definitionid"),
            target_user_id: row.get("target_user_id"),
            valsi_word: row.get("valsi_word"),
            definition: row.get("definition"),
            last_comment_id: row.get("last_comment_id"),
            last_comment_time: row.get("last_comment_time"),
            last_comment_subject: row.get("last_comment_subject"),
            last_comment_content: serde_json::from_str(
                &row.get::<_, String>("last_comment_content"),
            )
            .unwrap_or_default(),
            last_comment_username: row.get("last_comment_username"),
            first_comment_subject: row.get("first_comment_subject"),
            first_comment_content: serde_json::from_str(
                &row.get::<_, String>("first_comment_content"),
            )
            .unwrap_or_default(),
            total_comments: row.get::<_, i32>("total_comments") as i64,
            username: row.get("username"),
            realname: row.get("realname"),
            is_liked: None,
            is_bookmarked: None,
            total_reactions: 0,
            reactions: Vec::new(),
            user_id: row.get("userid"),
            comment_num: 0,
            parent_id: None,
        })
        .collect();

    transaction.commit().await?;

    Ok(PaginatedCommentsResponse {
        comments: mapped_threads
            .into_iter()
            .map(|ft| Comment {
                comment_id: ft.last_comment_id,
                thread_id: ft.thread_id,
                parent_id: ft.parent_id,
                user_id: ft.user_id,
                comment_num: ft.comment_num,
                time: ft.last_comment_time,
                subject: ft.last_comment_subject.clone(),
                content: ft.last_comment_content,
                last_comment_username: ft.last_comment_username,
                username: Some(ft.username),
                realname: ft.realname,
                total_reactions: 0, // Will get from joined data
                total_replies: ft.total_comments,
                is_liked: ft.is_liked,
                is_bookmarked: ft.is_bookmarked,
                valsi_id: ft.valsiid,
                definition_id: ft.definitionid,
                // target_user_id: ft.target_user_id, // Add if Comment struct is updated
                reactions: vec![],
                parent_content: None,
                valsi_word: ft.valsi_word,
                definition: ft.definition,
                first_comment_subject: Some(ft.first_comment_subject),
                first_comment_content: Some(ft.first_comment_content),
            })
            .collect(),
        total,
        page,
        per_page,
    })
}

pub async fn get_like_count(
    pool: &Pool,
    comment_id: i32,
) -> Result<i64, Box<dyn std::error::Error>> {
    let client = pool.get().await?;
    let row = client
        .query_one(
            "SELECT total_likes FROM comment_counters WHERE comment_id = $1",
            &[&comment_id],
        )
        .await?;
    Ok(row.get("total_likes"))
}

pub async fn list_comments(
    pool: &Pool,
    page: i64,
    per_page: i64,
    sort_order: &str,
    current_user_id: Option<i32>,
) -> Result<PaginatedCommentsResponse, Box<dyn std::error::Error>> {
    let mut client = pool.get().await?;
    let transaction = client.transaction().await?;
    let offset = (page - 1) * per_page;

    let sort_dir = if sort_order.eq_ignore_ascii_case("asc") {
        "ASC"
    } else {
        "DESC"
    };

    // Get total count
    let total: i64 = transaction
        .query_one("SELECT COUNT(*) FROM comments", &[])
        .await?
        .get(0);

    // Get paginated comments
    let query_string = format!(
        "SELECT
            c.commentid,
            c.threadid,
            c.parentid,
            c.userid,
            c.commentnum,
            c.time,
            c.subject,
            c.content::text as content,
            cc.total_reactions,
            cc.total_replies,
            CASE WHEN cl.user_id IS NOT NULL THEN true ELSE false END as is_liked,
            CASE WHEN cb.user_id IS NOT NULL THEN true ELSE false END as is_bookmarked,
            t.valsiid,
            t.definitionid,
            u.username,
            u.realname,
            v.word as valsi_word,
            d.definition as definition
        FROM comments c
        JOIN users u ON c.userid = u.userid
        JOIN threads t ON c.threadid = t.threadid
        LEFT JOIN valsi v ON t.valsiid = v.valsiid
        LEFT JOIN definitions d ON t.definitionid = d.definitionid
        LEFT JOIN comment_activity_counters cc ON c.commentid = cc.comment_id
        LEFT JOIN comment_likes cl ON c.commentid = cl.comment_id AND cl.user_id = $1
        LEFT JOIN comment_bookmarks cb ON c.commentid = cb.comment_id AND cb.user_id = $1
        ORDER BY c.time {}
        LIMIT $2 OFFSET $3",
        sort_dir
    );

    let rows = transaction
        .query(&query_string, &[&current_user_id, &per_page, &offset])
        .await?;

    // Get comment IDs and fetch reactions
    let comment_ids: Vec<i32> = rows.iter().map(|row| row.get("commentid")).collect();
    let reactions_map = fetch_reactions(&transaction, &comment_ids, current_user_id).await?;

    let comments = rows
        .iter()
        .map(|row| {
            let comment_id = row.get::<_, i32>("commentid");
            Comment {
                parent_content: None,
                comment_id,
                thread_id: row.get("threadid"),
                parent_id: row.get("parentid"),
                user_id: row.get("userid"),
                comment_num: row.get("commentnum"),
                time: row.get("time"),
                subject: row.get("subject"),
                content: serde_json::from_str(&row.get::<_, String>("content")).unwrap_or_default(),
                username: Some(row.get("username")),
                realname: row.get("realname"),
                total_reactions: row.get::<_, Option<i64>>("total_reactions").unwrap_or(0),
                total_replies: row.get::<_, Option<i64>>("total_replies").unwrap_or(0),
                is_liked: row.get("is_liked"),
                is_bookmarked: row.get("is_bookmarked"),
                valsi_id: row.get("valsiid"),
                definition_id: row.get("definitionid"),
                reactions: reactions_map.get(&comment_id).cloned().unwrap_or_default(),
                valsi_word: row.get("valsi_word"),
                definition: row.get("definition"),
                first_comment_subject: None,
                first_comment_content: None,
                last_comment_username: None,
            }
        })
        .collect();

    transaction.commit().await?;

    Ok(PaginatedCommentsResponse {
        comments,
        total,
        page,
        per_page,
    })
}
