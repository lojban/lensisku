//! Optional end-to-end retrieval tests against a disposable schema on local pgvector.
use super::*;
use crate::waves::{indexer, service};

fn query(term: &str) -> WavesSearchQuery {
    WavesSearchQuery {
        search: Some(term.into()),
        page: Some(1),
        per_page: Some(100),
        sort_by: None,
        sort_order: None,
        source: None,
        collection_id: None,
    }
}

#[tokio::test]
#[ignore = "requires local PostgreSQL/pgvector, .env configuration, and the embedding model"]
async fn live_model_indexing_and_hybrid_hydration() -> Result<(), SearchError> {
    dotenvy::dotenv().ok();
    let base_pool = crate::config::create_app_config()?.db_pools.app_pool;
    let admin = base_pool.get().await?;
    let schema = format!("wave_test_{}", uuid::Uuid::new_v4().simple());
    let fixture = include_str!("../../tests/waves_fixture.sql").replace("__TEST_SCHEMA__", &schema);
    admin.batch_execute(&fixture).await?;
    admin
        .batch_execute(include_str!(
            "../../migrations/V178__wave_hybrid_search.sql"
        ))
        .await?;
    // Complete the minimal SQL fixture so real result hydration uses its normal joins.
    admin
        .batch_execute(
            "ALTER TABLE comments ADD parentid integer, ADD commentnum integer DEFAULT 1,
             ADD content jsonb DEFAULT '[]';
         ALTER TABLE threads ADD valsiid integer, ADD definitionid integer,
             ADD definition_link_id integer, ADD target_user_id integer;
         ALTER TABLE messages ADD message_id text, ADD date text, ADD cleaned_subject text,
             ADD to_address text, ADD parts_json jsonb;
         ALTER TABLE wiki_articles ADD namespace integer DEFAULT 0;
         CREATE TABLE comment_bookmarks (comment_id integer, user_id integer);
         CREATE TABLE comment_reactions (comment_id integer, user_id integer, reaction text);
         CREATE TABLE message_spam_votes (message_id integer);
         COMMIT;",
        )
        .await?;
    // Every pooled connection explicitly uses the isolated schema.
    let config = tokio_postgres::Config::new()
        .host(std::env::var("DB_HOST").unwrap_or_else(|_| "localhost".into()))
        .port(
            std::env::var("DB_PORT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(5432),
        )
        .user(&std::env::var("DB_USER")?)
        .password(std::env::var("DB_PASSWORD")?)
        .dbname(&std::env::var("DB_NAME")?)
        .options(format!("-c search_path={schema},public"))
        .clone();
    let manager = deadpool_postgres::Manager::new(config, tokio_postgres::NoTls);
    let pool = Pool::builder(manager).max_size(3).build()?;
    let result = exercise(&pool).await;
    pool.close();
    // Cleanup happens even if any exercise assertion returns an error.
    admin
        .batch_execute(&format!("DROP SCHEMA {schema} CASCADE"))
        .await?;
    result
}

async fn exercise(pool: &Pool) -> Result<(), SearchError> {
    let client = pool.get().await?;
    let lexical = service::search_waves(pool, query("scope"), None).await?;
    if lexical.total != 4 || lexical.items.len() != 4 {
        return Err(format!(
            "Expected comment, mail, mirrored wiki, native wiki hydration; got {lexical:?}"
        )
        .into());
    }
    let fallback = search_with_embedding(pool, &query("scope"), None, None).await?;
    if fallback.total != 4 || fallback.items.len() != 4 {
        return Err("Lexical fallback did not retain all source results".into());
    }
    let mut deep = query("scope");
    deep.page = Some(200);
    let empty = service::search_waves(pool, deep, None).await?;
    if empty.total != 4 || !empty.items.is_empty() {
        return Err("Deep API page lost its total or returned phantom hits".into());
    }
    let mut chronological = query("scope");
    chronological.sort_by = Some("time".into());
    chronological.sort_order = Some("asc".into());
    let oldest = service::search_waves(pool, chronological, None).await?;
    if !matches!(oldest.items[0], WaveSearchHit::Comment { .. }) {
        return Err("Explicit chronological search ordering failed".into());
    }
    let mut filtered = query("scope");
    filtered.collection_id = Some(10);
    let scoped = service::search_waves(pool, filtered, None).await?;
    if scoped.total != 1 || !matches!(scoped.items[0], WaveSearchHit::Comment { .. }) {
        return Err("Collection scope failed during live hydration".into());
    }
    let long_body = format!(
        "{}\n\nBuying an automobile requires choosing a vehicle and paying the seller.",
        "The garden flowers grow in the sunshine and need regular watering.\n\n".repeat(30)
    );
    client
        .execute(
            "UPDATE messages SET subject = 'Long message', content = $1 WHERE id = 1",
            &[&long_body],
        )
        .await?;
    drop(client);
    while indexer::index_pending(pool).await? != 0 {}
    let client = pool.get().await?;
    let pending: i64 = client
        .query_one(
            "SELECT count(*) FROM wave_search_documents WHERE embedding IS NULL",
            &[],
        )
        .await?
        .get(0);
    if pending != 0 {
        return Err("Embedding backfill did not finish".into());
    }
    let mut search = query("purchasing a car");
    search.source = Some("mail".into());
    let semantic = service::search_waves(pool, search, None).await?;
    if semantic.total != 1
        || !matches!(&semantic.items[0], WaveSearchHit::Mail {
        relevance: Some(r), .. } if r.semantic_match && !r.lexical_match && r.excerpt.contains("automobile"))
    {
        return Err(format!("Late semantic paragraph was not retrieved: {semantic:?}").into());
    }
    // Reindex an edit, proving the previous topic cannot continue matching stale vectors.
    client
        .execute(
            "UPDATE messages SET content = 'Rain nourishes the flowers in a garden.' WHERE id = 1",
            &[],
        )
        .await?;
    let chunks: i64 = client.query_one("SELECT count(*) FROM wave_search_chunks c JOIN wave_search_documents d USING(document_id) WHERE kind='mail'", &[]).await?.get(0);
    if chunks != 0 {
        return Err("Stale paragraphs remained after edit".into());
    }
    drop(client);
    while indexer::index_pending(pool).await? != 0 {}
    let mut search = query("purchasing a car");
    search.source = Some("mail".into());
    if service::search_waves(pool, search, None).await?.total != 0 {
        return Err("Previous topic still matches after reindex".into());
    }
    Ok(())
}
