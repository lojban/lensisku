use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};

/// Cursor for keyset pagination of recent changes: (time, type_sort_order, cursor_id).
#[derive(serde::Serialize, serde::Deserialize)]
pub(crate) struct RecentChangesCursor {
    t: i32,
    s: i32,
    i: i64,
}

pub(crate) fn encode_recent_changes_cursor(
    time: i32,
    type_sort_order: i32,
    cursor_id: i64,
) -> String {
    let c = RecentChangesCursor {
        t: time,
        s: type_sort_order,
        i: cursor_id,
    };
    let json = serde_json::to_string(&c).unwrap_or_default();
    BASE64.encode(json.as_bytes())
}

pub(crate) fn decode_recent_changes_cursor(after: &str) -> Option<(i32, i32, i64)> {
    let bytes = BASE64.decode(after.as_bytes()).ok()?;
    let c: RecentChangesCursor = serde_json::from_slice(&bytes).ok()?;
    Some((c.t, c.s, c.i))
}

/// Feed cursors deliberately cannot be mistaken for legacy cursors.
pub(crate) fn encode_feed_cursor(time: i32, sort: i32, id: i64) -> String {
    format!("feed-v1:{}", encode_recent_changes_cursor(time, sort, id))
}
pub(crate) fn decode_feed_cursor(cursor: &str) -> Option<(i32, i32, i64)> {
    decode_recent_changes_cursor(cursor.strip_prefix("feed-v1:")?)
}
pub(crate) fn is_valid_feed_cursor(cursor: &str) -> bool {
    decode_feed_cursor(cursor).is_some()
}

pub(crate) fn activity_feed_query(mut query: String, source: &str) -> String {
    let (kind, source_expr, name_expr, marker) = if query.contains("FROM comments c") {
        // Optional thread contexts must not eliminate free waves or imported comments.
        query = query.replace("JOIN valsi v ON t.valsiid = v.valsiid", "LEFT JOIN valsi v ON t.valsiid = v.valsiid")
            .replace("JOIN definitions d ON d.definitionid = t.definitionid", "LEFT JOIN definitions d ON d.definitionid = t.definitionid")
            .replace("WHERE u.username != 'officialdata'", "WHERE u.username != 'officialdata' AND (t.collection_id IS NULL OR EXISTS (SELECT 1 FROM collections col WHERE col.collection_id = t.collection_id AND col.is_public))");
        (
            "comment",
            "CASE WHEN c.import_source IS NULL THEN 'native' ELSE 'imported' END",
            "COALESCE(c.import_source, 'lensisku')",
            "WHERE u.username != 'officialdata'",
        )
    } else if query.contains("FROM messages m") {
        ("message", "'imported'", "'mail'", "WHERE NOT EXISTS")
    } else if query.contains("FROM wiki_articles wa") {
        (
            "mirrored_wiki",
            "'imported'",
            "'mw.lojban.org'",
            "WHERE NOT wa.is_redirect",
        )
    } else if query.contains("WHERE v.typeid = 16") {
        query = query.replace(
            "WHERE v.typeid = 16",
            "WHERE v.typeid = 16 AND dv.mw_revid IS NULL",
        );
        (
            "native_wiki",
            "'native'",
            "'lensisku'",
            "WHERE v.typeid = 16",
        )
    } else {
        query = query.replace(" AND v.source_langid = 1", "");
        (
            "definition",
            "'native'",
            "'lensisku'",
            "WHERE u.username != 'officialdata'",
        )
    };
    let columns = format!("SELECT '{kind}'::text AS event_kind, ({source_expr})::text AS source, ({name_expr})::text AS source_name,");
    query = query.replacen("SELECT", &columns, 1);
    if source != "all" {
        // source has been validated by the controller; never interpolate user input.
        let wanted = if source == "native" {
            "native"
        } else {
            "imported"
        };
        query = query.replace(
            marker,
            &format!(
                "WHERE ({source_expr}) = '{wanted}' AND {}",
                marker.strip_prefix("WHERE ").unwrap_or(marker)
            ),
        );
    }
    query
}

pub(crate) fn recent_change_queries(
    types: Option<&str>,
    home: bool,
    feed: bool,
    source: &str,
    cursor_condition: Option<(i32, i32, i64)>,
    limit_val: i64,
) -> Vec<String> {
    let mut requested_types: Vec<&str> = if types == Some("news") {
        vec!["wiki", "free_wave"]
    } else if let Some(t) = types {
        t.split(',').collect()
    } else if home {
        vec!["comment", "definition", "wiki"]
    } else {
        vec!["comment", "definition", "valsi", "message", "wiki"]
    };

    if feed {
        if requested_types.contains(&"free_wave") && !requested_types.contains(&"comment") {
            requested_types.push("comment");
        }
        // The feed comment branch covers roots and replies exactly once.
        requested_types.retain(|t| *t != "free_wave" && *t != "valsi");
    }
    if home {
        requested_types.retain(|t| *t != "valsi");
    }

    let mut queries = Vec::new();

    // Comment branch: cursor_id = c.commentid. Compare (time, -type_sort_order, cursor_id) for ORDER BY time DESC, type_sort ASC, cursor_id DESC.
    if requested_types.contains(&"comment") {
        let (where_extra, order_limit) = match cursor_condition {
            Some((ct, cs, ci)) => (
                format!(" AND (c.time, -2, c.commentid) < ({}, {}, {})", ct, -cs, ci),
                format!(
                    "ORDER BY c.time DESC, type_sort_order ASC, c.commentid DESC LIMIT {}",
                    limit_val
                ),
            ),
            None => (
                String::new(),
                format!(
                    "ORDER BY c.time DESC, type_sort_order ASC, c.commentid DESC LIMIT {}",
                    limit_val
                ),
            ),
        };
        queries.push(format!(
            "(SELECT
    'comment' AS change_type,
    c.subject AS word,
    c.content AS content,
    t.valsiid,
    d.langid,
    t.natlangwordid,
    c.commentid,
    c.threadid as threadid,
    t.definitionid,
    u.username,
    c.time,
    l.realname AS language_name,
    l.englishname AS language_english_name,
    l.lojbanname AS language_lojban_name,
    NULL::integer as version_id,
    NULL::integer as prev_version_id,
    v.typeid AS valsi_typeid,
    v.word AS valsi_word,
    c.commentnum,
    c.parentid,
    2 AS type_sort_order,
    c.commentid::bigint AS cursor_id
FROM comments c
JOIN threads t ON c.threadid = t.threadid
JOIN valsi v ON t.valsiid = v.valsiid
JOIN users u ON c.userid = u.userid
JOIN definitions d ON d.definitionid = t.definitionid
LEFT JOIN languages l ON d.langid = l.langid
WHERE u.username != 'officialdata' {}
{})",
            where_extra, order_limit
        ));
    }

    // Definition branch: cursor_id = dv.version_id
    if requested_types.contains(&"definition") {
        let (where_extra, order_limit) = match cursor_condition {
            Some((ct, cs, ci)) => (
                format!(
                    " AND (EXTRACT(EPOCH FROM dv.created_at)::integer, -0, dv.version_id) < ({}, {}, {})",
                    ct, -cs, ci
                ),
                format!("ORDER BY dv.created_at DESC, type_sort_order ASC, dv.version_id DESC LIMIT {}", limit_val),
            ),
            None => (
                String::new(),
                format!("ORDER BY dv.created_at DESC, type_sort_order ASC, dv.version_id DESC LIMIT {}", limit_val),
            ),
        };
        queries.push(format!(
            "(SELECT
    'definition' AS change_type,
    v.word,
    to_jsonb(dv.message) as content,
    d.valsiid,
    dv.langid,
    0 AS natlangwordid,
    0 AS commentid,
    0 AS threadid,
    d.definitionid,
    u.username,
    EXTRACT(EPOCH FROM dv.created_at)::integer as time,
    l.realname AS language_name,
    l.englishname AS language_english_name,
    l.lojbanname AS language_lojban_name,
    dv.version_id,
    (SELECT prev_dv.version_id
     FROM definition_versions prev_dv
     WHERE prev_dv.definition_id = dv.definition_id
       AND prev_dv.created_at < dv.created_at
     ORDER BY prev_dv.created_at DESC LIMIT 1) as prev_version_id,
    v.typeid AS valsi_typeid,
    NULL::text AS valsi_word,
    NULL::integer AS commentnum,
    NULL::integer AS parentid,
    0 AS type_sort_order,
    dv.version_id::bigint AS cursor_id
FROM definition_versions dv
JOIN definitions d ON dv.definition_id = d.definitionid
JOIN valsi v ON d.valsiid = v.valsiid
JOIN users u ON dv.user_id = u.userid
LEFT JOIN languages l ON dv.langid = l.langid
WHERE u.username != 'officialdata' AND v.source_langid = 1 AND dv.mw_revid IS NULL AND v.typeid != 16 {}
{})",
            where_extra, order_limit
        ));
    }

    // Valsi branch: cursor_id = v.valsiid
    if requested_types.contains(&"valsi") {
        let (where_extra, order_limit) = match cursor_condition {
            Some((ct, cs, ci)) => (
                format!(" AND (v.time, -1, v.valsiid) < ({}, {}, {})", ct, -cs, ci),
                format!(
                    "ORDER BY v.time DESC, type_sort_order ASC, v.valsiid DESC LIMIT {}",
                    limit_val
                ),
            ),
            None => (
                String::new(),
                format!(
                    "ORDER BY v.time DESC, type_sort_order ASC, v.valsiid DESC LIMIT {}",
                    limit_val
                ),
            ),
        };
        queries.push(format!(
            "(SELECT
    'valsi' AS change_type,
    v.word,
    '{{}}'::jsonb as content,
    v.valsiid,
    0 AS langid,
    0 AS natlangwordid,
    0 AS commentid,
    0 AS threadid,
    0 AS definitionid,
    u.username,
    v.time,
    NULL AS language_name,
    NULL::text AS language_english_name,
    NULL::text AS language_lojban_name,
    NULL::integer as version_id,
    NULL::integer as prev_version_id,
    v.typeid AS valsi_typeid,
    NULL::text AS valsi_word,
    NULL::integer AS commentnum,
    NULL::integer AS parentid,
    1 AS type_sort_order,
    v.valsiid::bigint AS cursor_id
FROM valsi v
JOIN users u ON v.userid = u.userid
WHERE u.username != 'officialdata' AND v.source_langid = 1 AND v.typeid <> 16 {}
{})",
            where_extra, order_limit
        ));
    }

    // Message branch: cursor_id = m.id
    if requested_types.contains(&"message") {
        let (where_extra, order_limit) = match cursor_condition {
            Some((ct, cs, ci)) => (
                format!(
                    " AND (EXTRACT(EPOCH FROM m.sent_at)::integer, -3, m.id) < ({}, {}, {})",
                    ct, -cs, ci
                ),
                format!(
                    "ORDER BY m.sent_at DESC, type_sort_order ASC, m.id DESC LIMIT {}",
                    limit_val
                ),
            ),
            None => (
                String::new(),
                format!(
                    "ORDER BY m.sent_at DESC, type_sort_order ASC, m.id DESC LIMIT {}",
                    limit_val
                ),
            ),
        };
        queries.push(format!(
            "(SELECT
    'message' AS change_type,
    COALESCE(m.subject, '') AS word,
    to_jsonb(COALESCE(m.content, '')) as content,
    0 AS valsiid,
    0 AS langid,
    0 AS natlangwordid,
    m.id AS commentid,
    0 AS threadid,
    0 AS definitionid,
    COALESCE(m.from_address, '') AS username,
    EXTRACT(EPOCH FROM m.sent_at)::integer AS time,
    NULL AS language_name,
    NULL::text AS language_english_name,
    NULL::text AS language_lojban_name,
    NULL::integer as version_id,
    NULL::integer as prev_version_id,
    NULL::smallint AS valsi_typeid,
    NULL::text AS valsi_word,
    NULL::integer AS commentnum,
    NULL::integer AS parentid,
    3 AS type_sort_order,
    m.id::bigint AS cursor_id
FROM messages m
WHERE NOT EXISTS (
    SELECT 1
    FROM message_spam_votes msv
    WHERE msv.message_id = m.id
) {}
{})",
            where_extra, order_limit
        ));
    }

    // The first comment of each context-free thread is a news item.
    if requested_types.contains(&"free_wave") {
        let where_extra = match cursor_condition {
            Some((ct, cs, ci)) => {
                format!(" AND (c.time, -5, c.commentid) < ({}, {}, {})", ct, -cs, ci)
            }
            None => String::new(),
        };
        queries.push(format!(
            "(SELECT
    'comment' AS change_type,
    c.subject AS word,
    c.content AS content,
    0 AS valsiid,
    0 AS langid,
    0 AS natlangwordid,
    c.commentid,
    c.threadid,
    0 AS definitionid,
    u.username,
    c.time,
    NULL::text AS language_name,
    NULL::text AS language_english_name,
    NULL::text AS language_lojban_name,
    NULL::integer AS version_id,
    NULL::integer AS prev_version_id,
    NULL::smallint AS valsi_typeid,
    NULL::text AS valsi_word,
    c.commentnum,
    c.parentid,
    5 AS type_sort_order,
    c.commentid::bigint AS cursor_id
FROM comments c
JOIN threads t ON c.threadid = t.threadid
JOIN users u ON c.userid = u.userid
WHERE t.valsiid IS NULL AND t.natlangwordid IS NULL
  AND t.definitionid IS NULL AND t.definition_link_id IS NULL
  AND t.target_user_id IS NULL AND t.collection_id IS NULL
  AND c.commentnum = 1 AND c.import_source IS NULL
  AND u.username != 'officialdata' {}
ORDER BY c.time DESC, type_sort_order ASC, c.commentid DESC LIMIT {})",
            where_extra, limit_val
        ));
    }

    // Mirrored mw.lojban.org articles (wiki_articles.last_edited).
    if requested_types.contains(&"wiki") {
        let (where_extra, order_limit) = match cursor_condition {
            Some((ct, cs, ci)) => (
                format!(
                    " AND (EXTRACT(EPOCH FROM wa.last_edited)::integer, -4, wa.id) < ({}, {}, {})",
                    ct, -cs, ci
                ),
                format!(
                    "ORDER BY wa.last_edited DESC, type_sort_order ASC, wa.id DESC LIMIT {}",
                    limit_val
                ),
            ),
            None => (
                String::new(),
                format!(
                    "ORDER BY wa.last_edited DESC, type_sort_order ASC, wa.id DESC LIMIT {}",
                    limit_val
                ),
            ),
        };
        queries.push(format!(
            "(SELECT
    'wiki' AS change_type,
    wa.title AS word,
    to_jsonb(LEFT(wa.plain_text, 200)) as content,
    0 AS valsiid,
    0 AS langid,
    0 AS natlangwordid,
    0 AS commentid,
    0 AS threadid,
    0 AS definitionid,
    COALESCE(NULLIF(u.username, 'officialdata'), 'mw.lojban.org') AS username,
    EXTRACT(EPOCH FROM wa.last_edited)::integer AS time,
    NULL AS language_name,
    NULL::text AS language_english_name,
    NULL::text AS language_lojban_name,
    NULL::integer as version_id,
    NULL::integer as prev_version_id,
    NULL::smallint AS valsi_typeid,
    NULL::text AS valsi_word,
    NULL::integer AS commentnum,
    NULL::integer AS parentid,
    4 AS type_sort_order,
    wa.id::bigint AS cursor_id
FROM wiki_articles wa
LEFT JOIN definition_versions dv ON dv.mw_revid = wa.revision_id
LEFT JOIN users u ON u.userid = dv.user_id
WHERE NOT wa.is_redirect AND wa.last_edited IS NOT NULL {}
{})",
            where_extra, order_limit
        ));

        // Native (internal) wiki pages: valsi typeid 16 definition versions.
        let native_sort = if feed { 6 } else { 4 };
        let (native_where_extra, native_order_limit) = match cursor_condition {
            Some((ct, cs, ci)) => (
                format!(
                    " AND (EXTRACT(EPOCH FROM dv.created_at)::integer, -{native_sort}, dv.version_id) < ({}, {}, {})",
                    ct, -cs, ci
                ),
                format!("ORDER BY dv.created_at DESC, type_sort_order ASC, dv.version_id DESC LIMIT {}", limit_val),
            ),
            None => (
                String::new(),
                format!("ORDER BY dv.created_at DESC, type_sort_order ASC, dv.version_id DESC LIMIT {}", limit_val),
            ),
        };
        queries.push(format!(
            "(SELECT
    'wiki' AS change_type,
    v.word,
    to_jsonb(dv.message) as content,
    d.valsiid,
    dv.langid,
    0 AS natlangwordid,
    0 AS commentid,
    0 AS threadid,
    d.definitionid,
    u.username,
    EXTRACT(EPOCH FROM dv.created_at)::integer as time,
    NULL AS language_name,
    NULL::text AS language_english_name,
    NULL::text AS language_lojban_name,
    dv.version_id,
    (SELECT prev_dv.version_id
     FROM definition_versions prev_dv
     WHERE prev_dv.definition_id = dv.definition_id
       AND prev_dv.created_at < dv.created_at
     ORDER BY prev_dv.created_at DESC LIMIT 1) as prev_version_id,
    v.typeid AS valsi_typeid,
    NULL::text AS valsi_word,
    NULL::integer AS commentnum,
    NULL::integer AS parentid,
    {native_sort} AS type_sort_order,
    dv.version_id::bigint AS cursor_id
FROM definition_versions dv
JOIN definitions d ON dv.definition_id = d.definitionid
JOIN valsi v ON d.valsiid = v.valsiid
JOIN users u ON dv.user_id = u.userid
WHERE v.typeid = 16 AND u.username != 'officialdata' {}
{})",
            native_where_extra, native_order_limit
        ));
    }

    if feed {
        queries = queries
            .into_iter()
            .map(|query| activity_feed_query(query, source))
            .collect();
    }

    queries
}
