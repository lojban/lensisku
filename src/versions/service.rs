use super::{
    models::{Change, ChangeType, Version, VersionContent, VersionDiff},
    VersionHistoryResponse,
};
use crate::{auth::permissions::PermissionCache, jbovlaste::KeywordMapping};
use deadpool_postgres::Pool;

pub async fn get_definition_history(
    pool: &Pool,
    definition_id: i32,
    page: i64,
    per_page: i64,
) -> Result<VersionHistoryResponse, Box<dyn std::error::Error>> {
    let mut client = pool.get().await?;
    let transaction = client.transaction().await?;
    let offset = (page - 1) * per_page;

    let versions = transaction
        .query(
            "SELECT v.*, u.username,
             v.definition, v.notes, v.selmaho, v.jargon,
             v.gloss_keywords::text as gloss_json,
             v.place_keywords::text as place_json
             FROM definition_versions v
             JOIN users u ON v.user_id = u.userid
             WHERE v.definition_id = $1
             ORDER BY v.created_at DESC
             LIMIT $2 OFFSET $3",
            &[&definition_id, &per_page, &offset],
        )
        .await?
        .iter()
        .map(|row| {
            let gloss_keywords: Option<Vec<KeywordMapping>> =
                if let Ok(json) = row.try_get::<_, String>("gloss_json") {
                    serde_json::from_str(&json).ok()
                } else {
                    None
                };

            let place_keywords: Option<Vec<KeywordMapping>> =
                if let Ok(json) = row.try_get::<_, String>("place_json") {
                    serde_json::from_str(&json).ok()
                } else {
                    None
                };

            let content = VersionContent {
                definition: row.get("definition"),
                notes: row.get("notes"),
                selmaho: row.get("selmaho"),
                jargon: row.get("jargon"),
                gloss_keywords,
                place_keywords,
            };

            Version {
                version_id: row.get("version_id"),
                definition_id: row.get("definition_id"),
                user_id: row.get("user_id"),
                username: row.get("username"),
                created_at: row.get("created_at"),
                content,
                commit_message: row.get("message"),
            }
        })
        .collect();

    let total: i64 = transaction
        .query_one(
            "SELECT COUNT(*) FROM definition_versions WHERE definition_id = $1",
            &[&definition_id],
        )
        .await?
        .get(0);

    transaction.commit().await?;

    Ok(VersionHistoryResponse {
        versions,
        total,
        page,
        per_page,
    })
}

pub async fn get_version(
    pool: &Pool,
    version_id: i32,
) -> Result<Version, Box<dyn std::error::Error>> {
    let mut client = pool.get().await?;
    let transaction = client.transaction().await?;

    let version = get_version_with_transaction(&transaction, version_id).await?;

    transaction.commit().await?;

    Ok(version)
}

pub async fn get_version_with_transaction(
    transaction: &tokio_postgres::Transaction<'_>,
    version_id: i32,
) -> Result<Version, Box<dyn std::error::Error>> {
    let row = transaction
        .query_one(
            "SELECT v.*, u.username,
             v.definition, v.notes, v.selmaho, v.jargon,
             v.gloss_keywords::text as gloss_json,
             v.place_keywords::text as place_json
             FROM definition_versions v
             JOIN users u ON v.user_id = u.userid
             WHERE v.version_id = $1",
            &[&version_id],
        )
        .await?;

    let gloss_keywords: Option<Vec<KeywordMapping>> =
        if let Ok(json) = row.try_get::<_, String>("gloss_json") {
            serde_json::from_str(&json).ok()
        } else {
            None
        };

    let place_keywords: Option<Vec<KeywordMapping>> =
        if let Ok(json) = row.try_get::<_, String>("place_json") {
            serde_json::from_str(&json).ok()
        } else {
            None
        };

    let content = VersionContent {
        definition: row.get("definition"),
        notes: row.get("notes"),
        selmaho: row.get("selmaho"),
        jargon: row.get("jargon"),
        gloss_keywords,
        place_keywords,
    };

    Ok(Version {
        version_id: row.get("version_id"),
        definition_id: row.get("definition_id"),
        user_id: row.get("user_id"),
        username: row.get("username"),
        created_at: row.get("created_at"),
        content,
        commit_message: row.get("message"),
    })
}

pub async fn create_version(
    transaction: &tokio_postgres::Transaction<'_>,
    definition_id: i32,
    user_id: i32,
    content: &VersionContent,
    commit_message: &str,
) -> Result<Version, Box<dyn std::error::Error>> {
    // Get valsi and lang IDs from the definition
    let def_info = transaction
        .query_one(
            "SELECT valsiid, langid FROM definitions WHERE definitionid = $1",
            &[&definition_id],
        )
        .await?;

    let valsi_id: i32 = def_info.get("valsiid");
    let lang_id: i32 = def_info.get("langid");

    // Convert keywords to JSONB
    let gloss_json = serde_json::to_value(&content.gloss_keywords).map(postgres_types::Json)?;
    let place_json = serde_json::to_value(&content.place_keywords).map(postgres_types::Json)?;

    let row = transaction
        .query_one(
            "INSERT INTO definition_versions 
             (definition_id, langid, valsiid, definition, notes, selmaho, jargon, 
              gloss_keywords, place_keywords, user_id, message)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
             RETURNING version_id, created_at",
            &[
                &definition_id,
                &lang_id,
                &valsi_id,
                &content.definition,
                &content.notes,
                &content.selmaho,
                &content.jargon,
                &gloss_json,
                &place_json,
                &user_id,
                &commit_message,
            ],
        )
        .await?;

    let username: String = transaction
        .query_one("SELECT username FROM users WHERE userid = $1", &[&user_id])
        .await?
        .get("username");

    Ok(Version {
        version_id: row.get("version_id"),
        definition_id,
        user_id,
        username,
        created_at: row.get("created_at"),
        content: content.clone(),
        commit_message: commit_message.to_string(),
    })
}

pub async fn revert_to_version(
    pool: &Pool,
    version_id: i32,
    user_id: i32,
    user_role: &str,
    perm_cache: &PermissionCache,
) -> Result<Version, Box<dyn std::error::Error>> {
    let mut client = pool.get().await?;
    let transaction = client.transaction().await?;

    let old_version: Version = get_version_with_transaction(&transaction, version_id).await?;

    // Check permissions
    let has_permission = perm_cache
        .has_permission(user_role.to_string(), "revert_entry_version")
        .await;
    if !has_permission {
        // Check if user is the original author of the definition
        let definition_author: i32 = transaction
            .query_one(
                "SELECT user_id FROM definitions WHERE definitionid = $1",
                &[&old_version.definition_id],
            )
            .await?
            .get("user_id");

        if definition_author != user_id {
            return Err(Box::new(actix_web::error::ErrorForbidden(
                "You must be the definition author or have revert permissions",
            )));
        }
    }

    // Create a new version with the old content
    let new_version = create_version(
        &transaction,
        old_version.definition_id,
        user_id,
        &old_version.content,
        &format!("Reverted to version {}", version_id),
    )
    .await?;

    // Update the definition with the old content
    transaction
        .execute(
            "UPDATE definitions 
             SET definition = $1, notes = $2, selmaho = $3, jargon = $4
             WHERE definitionid = $5",
            &[
                &old_version.content.definition,
                &old_version.content.notes,
                &old_version.content.selmaho,
                &old_version.content.jargon,
                &old_version.definition_id,
            ],
        )
        .await?;

    // Update keywords if they exist
    if let Some(gloss_keywords) = &old_version.content.gloss_keywords {
        // Clear existing keywords
        transaction
            .execute(
                "DELETE FROM keywordmapping WHERE definitionid = $1",
                &[&old_version.definition_id],
            )
            .await?;

        // Add gloss keywords
        for keyword in gloss_keywords {
            transaction
                .execute(
                    "INSERT INTO keywordmapping (definitionid, place, natlangwordid)
                     SELECT $1, 0, wordid
                     FROM natlangwords
                     WHERE word = $2 AND (meaning = $3 OR ($3 IS NULL AND meaning IS NULL))
                     LIMIT 1",
                    &[&old_version.definition_id, &keyword.word, &keyword.meaning],
                )
                .await?;
        }

        // Add place keywords
        if let Some(place_keywords) = &old_version.content.place_keywords {
            for (i, keyword) in place_keywords.iter().enumerate() {
                transaction
                    .execute(
                        "INSERT INTO keywordmapping (definitionid, place, natlangwordid)
                         SELECT $1, $2, wordid
                         FROM natlangwords
                         WHERE word = $3 AND (meaning = $4 OR ($4 IS NULL AND meaning IS NULL))
                         LIMIT 1",
                        &[
                            &old_version.definition_id,
                            &((i + 1) as i32),
                            &keyword.word,
                            &keyword.meaning,
                        ],
                    )
                    .await?;
            }
        }
    }

    transaction.commit().await?;

    Ok(new_version)
}

pub async fn get_diff_with_transaction(
    pool: &deadpool_postgres::Pool,
    from_version: i32,
    to_version: i32,
) -> Result<VersionDiff, Box<dyn std::error::Error>> {
    let mut client = pool.get().await?;
    let transaction = client.transaction().await?;

    let diff = get_diff(&transaction, from_version, to_version).await?;
    transaction.commit().await?;

    Ok(diff)
}

pub async fn get_diff(
    transaction: &tokio_postgres::Transaction<'_>,
    from_version: i32,
    to_version: i32,
) -> Result<VersionDiff, Box<dyn std::error::Error>> {
    let old_version = get_version_with_transaction(transaction, from_version).await?;
    let new_version = get_version_with_transaction(transaction, to_version).await?;

    let mut changes = Vec::new();

    // Compare basic fields
    compare_field(
        "definition",
        &old_version.content.definition,
        &new_version.content.definition,
        &mut changes,
    );

    compare_option_field(
        "notes",
        &old_version.content.notes,
        &new_version.content.notes,
        &mut changes,
    );

    compare_option_field(
        "selmaho",
        &old_version.content.selmaho,
        &new_version.content.selmaho,
        &mut changes,
    );

    compare_option_field(
        "jargon",
        &old_version.content.jargon,
        &new_version.content.jargon,
        &mut changes,
    );

    // Compare keywords
    compare_keywords(
        "gloss_keywords",
        &old_version.content.gloss_keywords,
        &new_version.content.gloss_keywords,
        &mut changes,
    );

    compare_keywords(
        "place_keywords",
        &old_version.content.place_keywords,
        &new_version.content.place_keywords,
        &mut changes,
    );

    Ok(VersionDiff {
        old_content: old_version.content,
        new_content: new_version.content,
        changes,
    })
}

fn compare_field(field: &str, old_value: &str, new_value: &str, changes: &mut Vec<Change>) {
    if old_value != new_value {
        changes.push(Change {
            field: field.to_string(),
            old_value: Some(old_value.to_string()),
            new_value: Some(new_value.to_string()),
            change_type: ChangeType::Modified,
        });
    }
}

fn compare_option_field(
    field: &str,
    old_value: &Option<String>,
    new_value: &Option<String>,
    changes: &mut Vec<Change>,
) {
    match (old_value, new_value) {
        (None, Some(new)) => changes.push(Change {
            field: field.to_string(),
            old_value: None,
            new_value: Some(new.clone()),
            change_type: ChangeType::Added,
        }),
        (Some(old), None) => changes.push(Change {
            field: field.to_string(),
            old_value: Some(old.clone()),
            new_value: None,
            change_type: ChangeType::Removed,
        }),
        (Some(old), Some(new)) if old != new => changes.push(Change {
            field: field.to_string(),
            old_value: Some(old.clone()),
            new_value: Some(new.clone()),
            change_type: ChangeType::Modified,
        }),
        _ => {}
    }
}

/// Format keyword/gloss list for display in diffs, emails, and UI (e.g. "word1 (meaning1), word2 (meaning2)").
fn format_keywords_display(keywords: &[KeywordMapping]) -> String {
    keywords
        .iter()
        .map(|k| {
            k.meaning
                .as_deref()
                .filter(|m| !m.is_empty())
                .map(|m| format!("{} ({})", k.word, m))
                .unwrap_or_else(|| k.word.clone())
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn compare_keywords(
    field: &str,
    old_keywords: &Option<Vec<crate::jbovlaste::KeywordMapping>>,
    new_keywords: &Option<Vec<crate::jbovlaste::KeywordMapping>>,
    changes: &mut Vec<Change>,
) {
    match (old_keywords, new_keywords) {
        (None, Some(new)) => changes.push(Change {
            field: field.to_string(),
            old_value: None,
            new_value: Some(format_keywords_display(new)),
            change_type: ChangeType::Added,
        }),
        (Some(old), None) => changes.push(Change {
            field: field.to_string(),
            old_value: Some(format_keywords_display(old)),
            new_value: None,
            change_type: ChangeType::Removed,
        }),
        (Some(old), Some(new)) if old != new => changes.push(Change {
            field: field.to_string(),
            old_value: Some(format_keywords_display(old)),
            new_value: Some(format_keywords_display(new)),
            change_type: ChangeType::Modified,
        }),
        _ => {}
    }
}
