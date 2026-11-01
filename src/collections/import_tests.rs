//! Pins the behaviour of the collection import / backup-restore paths
//! (`import_json`, `import_collection_from_json`, `import_full`) so their per-item
//! loops can be batched without changing results.
//!
//! Needs a disposable database with the lensisku schema plus the `languages` and
//! `valsitypes` rows (e.g. `pg_dump -s` of the dev DB + `pg_dump -a -t languages -t valsitypes`)
//! in `COLLECTIONS_TEST_DATABASE_URL`. Redis is not needed: the cache points at a closed
//! port, so invalidation just logs a warning.
#![allow(clippy::unwrap_used)]

use super::dto::{ImportFullRequest, ImportJsonRequest};
use super::service::{import_collection_from_json, import_full, import_json};
use crate::export::models::CollectionExportItem;
use crate::middleware::cache::RedisCache;
use base64::Engine;
use serde_json::json;
use std::fmt::Write;

fn stub_cache() -> RedisCache {
    RedisCache::new("redis://127.0.0.1:1/", std::time::Duration::from_secs(1)).unwrap()
}

fn png_data_url() -> String {
    let mut png = std::io::Cursor::new(Vec::new());
    image::DynamicImage::new_rgb8(2, 2)
        .write_to(&mut png, image::ImageFormat::Png)
        .unwrap();
    format!(
        "data:image/png;base64,{}",
        base64::engine::general_purpose::STANDARD.encode(png.into_inner())
    )
}

/// Order-independent textual snapshot of everything an import writes for a collection.
async fn snapshot(client: &deadpool_postgres::Client, collection_id: i32, user_id: i32) -> String {
    let mut out = String::new();
    for r in client
        .query(
            "SELECT ci.position, ci.definition_id IS NOT NULL AS has_def, ci.free_content_front,
                    ci.free_content_back, ci.notes, ci.canonical_form,
                    (SELECT string_agg(side, ',' ORDER BY side) FROM collection_item_images i WHERE i.item_id = ci.item_id) AS imgs
             FROM collection_items ci WHERE collection_id = $1 ORDER BY position, item_id",
            &[&collection_id],
        )
        .await
        .unwrap()
    {
        writeln!(
            out,
            "item pos={} def={} front={:?} back={:?} notes={:?} canon={:?} imgs={:?}",
            r.get::<_, i32>(0),
            r.get::<_, bool>(1),
            r.get::<_, Option<String>>(2),
            r.get::<_, Option<String>>(3),
            r.get::<_, Option<String>>(4),
            r.get::<_, Option<String>>(5),
            r.get::<_, Option<String>>(6)
        )
        .unwrap();
    }
    for r in client
        .query(
            "SELECT f.position, f.direction::text, ci.position,
                    (SELECT string_agg(card_side, ',' ORDER BY card_side) FROM user_flashcard_progress p
                      WHERE p.flashcard_id = f.id AND p.user_id = $2)
             FROM flashcards f JOIN collection_items ci ON ci.item_id = f.item_id
             WHERE f.collection_id = $1 ORDER BY f.position",
            &[&collection_id, &user_id],
        )
        .await
        .unwrap()
    {
        writeln!(
            out,
            "card pos={} dir={} item_pos={} progress={:?}",
            r.get::<_, i32>(0),
            r.get::<_, String>(1),
            r.get::<_, i32>(2),
            r.get::<_, Option<String>>(3)
        )
        .unwrap();
    }
    for r in client
        .query(
            "SELECT l.position, l.name, l.description, l.min_cards, l.min_success_rate,
                    (SELECT string_agg(p.position::text, ',' ORDER BY p.position) FROM level_prerequisites lp
                       JOIN flashcard_levels p ON p.level_id = lp.prerequisite_id WHERE lp.level_id = l.level_id),
                    (SELECT string_agg(f.position::text || '@' || li.position::text, ',' ORDER BY li.position, f.position)
                       FROM flashcard_level_items li JOIN flashcards f ON f.id = li.flashcard_id WHERE li.level_id = l.level_id)
             FROM flashcard_levels l WHERE collection_id = $1 ORDER BY l.position, l.name",
            &[&collection_id],
        )
        .await
        .unwrap()
    {
        writeln!(
            out,
            "level pos={} name={} desc={:?} min_cards={} rate={} prereqs={:?} cards={:?}",
            r.get::<_, i32>(0),
            r.get::<_, String>(1),
            r.get::<_, Option<String>>(2),
            r.get::<_, i32>(3),
            r.get::<_, f64>(4),
            r.get::<_, Option<String>>(5),
            r.get::<_, Option<String>>(6)
        )
        .unwrap();
    }
    out
}

#[tokio::test]
#[ignore = "requires COLLECTIONS_TEST_DATABASE_URL (disposable DB with the lensisku schema)"]
async fn import_paths_current_behaviour() {
    let url = crate::test_db_guard::disposable_database_url("COLLECTIONS_TEST_DATABASE_URL");
    let config = url.parse::<tokio_postgres::Config>().unwrap();
    let manager = deadpool_postgres::Manager::new(config, tokio_postgres::NoTls);
    let pool = deadpool_postgres::Pool::builder(manager)
        .max_size(4)
        .build()
        .unwrap();
    let redis = stub_cache();
    let client = pool.get().await.unwrap();
    let suffix = std::process::id();

    let user_id: i32 = client
        .query_one(
            "INSERT INTO users (username, password, email, created_at, role) VALUES ($1, 'x', $2, now(), 'user') RETURNING userid",
            &[&format!("import_t_{suffix}"), &format!("import_t_{suffix}@example.invalid")],
        )
        .await
        .unwrap()
        .get(0);
    let valsi_id: i32 = client
        .query_one(
            "INSERT INTO valsi (word, typeid, userid, time) VALUES ($1, (SELECT min(typeid) FROM valsitypes), $2, 0) RETURNING valsiid",
            &[&format!("zzimport{suffix}"), &user_id],
        )
        .await
        .unwrap()
        .get(0);
    let mut defs = Vec::new();
    for n in 1..=3 {
        defs.push(
            client
                .query_one(
                    "INSERT INTO definitions (langid, valsiid, definitionnum, definition, userid, time)
                     VALUES ((SELECT min(langid) FROM languages), $1, $2, 'd', $3, 0) RETURNING definitionid",
                    &[&valsi_id, &n, &user_id],
                )
                .await
                .unwrap()
                .get::<_, i32>(0),
        );
    }
    let missing = i32::MAX - 7;
    let (d1, d2, d3) = (defs[0], defs[1], defs[2]);

    // ---- import_json ----
    let req: ImportJsonRequest = serde_json::from_value(json!({
        "name": "imp json", "description": "desc", "is_public": false,
        "items": [
            {"word": "broda", "definition_id": d1, "collection_note": "n1"},
            {"word": "brode", "definition_id": missing},
            {"word": "brodi"},
            {"word": "brodu", "definition_id": d2, "collection_note": "n2"}
        ]
    }))
    .unwrap();
    let res = import_json(&pool, &redis, user_id, &req).await.unwrap();
    let mut got = format!(
        "json imported={} skipped={} warnings={:?} count={} public={}\n",
        res.imported_count,
        res.skipped_count,
        res.warnings
            .iter()
            .map(|w| w.replace(&missing.to_string(), "MISSING"))
            .collect::<Vec<_>>(),
        res.collection.item_count,
        res.collection.is_public
    );
    got += &snapshot(&client, res.collection.collection_id, user_id).await;

    // A definition listed twice in one import_json request.
    let dup: ImportJsonRequest = serde_json::from_value(json!({
        "name": "imp dup",
        "items": [
            {"word": "broda", "definition_id": d1},
            {"word": "brodo", "definition_id": d1}
        ]
    }))
    .unwrap();
    match import_json(&pool, &redis, user_id, &dup).await {
        Ok(r) => {
            writeln!(
                got,
                "json dup imported={} skipped={} warnings={:?}",
                r.imported_count,
                r.skipped_count,
                r.warnings
                    .iter()
                    .map(|w| w.replace(&format!("ID {d1} "), "ID D1 "))
                    .collect::<Vec<_>>()
            )
            .unwrap();
            let _ = client
                .execute(
                    "DELETE FROM collections WHERE collection_id = $1",
                    &[&r.collection.collection_id],
                )
                .await;
        }
        Err(e) => writeln!(got, "json dup -> {e}").unwrap(),
    }

    // ---- import_collection_from_json into an existing collection ----
    let target = res.collection.collection_id;
    let png = png_data_url();
    let items: Vec<CollectionExportItem> = serde_json::from_value(json!([
        {"item_id": 1, "definition_id": d1, "collection_note": "<b>dup in db</b>"},
        {"item_id": 2, "definition_id": d3, "collection_note": "<i>new</i>", "front_image_url": png},
        {"item_id": 3, "definition_id": d3},
        {"item_id": 4, "free_content_front": "<b>F</b>", "free_content_back": "B", "word": "cmene"},
        {"item_id": 5, "free_content_front": "<b>F</b>", "free_content_back": "B"},
        {"item_id": 6, "free_content_front": "F", "free_content_back": "B"},
        {"item_id": 7, "free_content_front": "only front"},
        {"item_id": 8, "free_content_front": "G", "free_content_back": "H", "back_image_url": png}
    ]))
    .unwrap();
    let res2 = import_collection_from_json(&pool, &redis, target, user_id, &items)
        .await
        .unwrap();
    writeln!(
        got,
        "into imported={} skipped={} skipped_items={:?}",
        res2.imported_count,
        res2.skipped_count,
        res2.skipped_items
            .iter()
            .map(|s| format!(
                "{}|{}",
                // Only rewrite definition ids: in a fresh DB they can collide with item ids.
                s.identifier
                    .replace(&format!("definition_id: {d1}"), "definition_id: D1")
                    .replace(&format!("definition_id: {d3}"), "definition_id: D3"),
                s.reason
            ))
            .collect::<Vec<_>>()
    )
    .unwrap();
    got += &snapshot(&client, target, user_id).await;

    // Bad image in import_collection_from_json is a hard error.
    let bad: Vec<CollectionExportItem> = serde_json::from_value(json!([
        {"item_id": 9, "free_content_front": "X", "free_content_back": "Y", "front_image_url": "nope"}
    ]))
    .unwrap();
    let err = import_collection_from_json(&pool, &redis, target, user_id, &bad)
        .await
        .unwrap_err();
    writeln!(got, "into bad image -> {err}").unwrap();

    // ---- import_full with explicit levels ----
    let full: ImportFullRequest = serde_json::from_value(json!({
        "collection": {"name": "<b>full</b>", "description": "<i>d</i>"},
        "items": [
            {"definition_id": d1, "word": "broda", "direction": "reverse"},
            {"definition_id": missing, "word": "brode"},
            {"collection_note": "nothing"},
            {"free_content_front": "<p>Q</p>", "free_content_back": "A", "direction": "BOTH", "front_image_url": png, "back_image_url": "data:image/png;base64,!!"},
            {"free_content_front": "Q2", "free_content_back": "A2", "direction": "quiz_direct", "back_image_url": "plain"},
            {"definition_id": d2, "direction": "weird"},
            {"definition_id": d2, "word": "dup"}
        ],
        "levels": [
            {"name": "L1", "min_cards": 2, "min_success_rate": 0.5, "position": 0, "item_positions": [0, 1, 3, 3, 99]},
            {"name": "L2", "description": "two", "min_cards": 3, "min_success_rate": 0.75, "position": 1,
             "prerequisite_positions": [0, 0, 1, 7], "item_positions": [5, 4, 0]}
        ]
    }))
    .unwrap();
    let res3 = import_full(&pool, &redis, user_id, &full).await.unwrap();
    writeln!(
        got,
        "full imported={} skipped={} levels={} name={} desc={:?} warnings={:?}",
        res3.imported_count,
        res3.skipped_count,
        res3.levels_created,
        res3.collection.name,
        res3.collection.description,
        res3.warnings
            .iter()
            .map(|w| {
                w.replace(&missing.to_string(), "MISSING")
                    .replace(&format!("ID {d2} "), "ID D2 ")
            })
            .collect::<Vec<_>>()
    )
    .unwrap();
    got += &snapshot(&client, res3.collection.collection_id, user_id).await;

    // ---- import_full with inferred levels ----
    let inferred: ImportFullRequest = serde_json::from_value(json!({
        "collection": {"name": "inferred", "is_public": false},
        "items": [
            {"free_content_front": "a", "free_content_back": "1", "level_index": 2, "position_in_level": 1},
            {"free_content_front": "b", "free_content_back": "2", "level_index": 0},
            {"free_content_front": "c", "free_content_back": "3", "level_index": 2, "position_in_level": 0},
            {"free_content_front": "d", "free_content_back": "4"},
            {"definition_id": missing, "level_index": 0, "position_in_level": 0},
            {"free_content_front": "e", "free_content_back": "5", "level_index": 0, "sound_url": "data:audio/mpeg;base64,SUQzBAA="},
            {"free_content_front": "f", "free_content_back": "6", "level_index": 0, "sound_url": "data:audio/wav;base64,UklGRg=="}
        ]
    }))
    .unwrap();
    let res4 = import_full(&pool, &redis, user_id, &inferred)
        .await
        .unwrap();
    writeln!(
        got,
        "inferred imported={} skipped={} levels={}",
        res4.imported_count, res4.skipped_count, res4.levels_created
    )
    .unwrap();
    got += &snapshot(&client, res4.collection.collection_id, user_id).await;
    writeln!(got, "inferred warnings={:?}", res4.warnings).unwrap();
    // Audio round trip: stored on import, emitted as a data URL by the full export.
    let exported =
        super::service::export_collection_full(&pool, res4.collection.collection_id, Some(user_id))
            .await
            .unwrap();
    let sounds: Vec<String> = exported
        .items
        .iter()
        .filter_map(|i| {
            i.sound_url
                .as_ref()
                .map(|u| format!("{}={}", i.free_content_front.clone().unwrap_or_default(), u))
        })
        .collect();
    writeln!(got, "inferred exported sounds={sounds:?}").unwrap();

    // Cleanup (best effort; the DB is disposable anyway).
    for c in [
        target,
        res3.collection.collection_id,
        res4.collection.collection_id,
    ] {
        let _ = client
            .execute("DELETE FROM collections WHERE collection_id = $1", &[&c])
            .await;
    }

    if let Ok(path) = std::env::var("IMPORT_PIN_WRITE") {
        std::fs::write(path, &got).unwrap();
    }
    assert_eq!(got, include_str!("import_tests.expected.txt"));
}

/// Export → import → export must reproduce the course content exactly (ids aside),
/// including a dialogue card whose several questions are scored separately.
#[tokio::test]
#[ignore = "requires COLLECTIONS_TEST_DATABASE_URL (disposable DB with the lensisku schema)"]
async fn full_export_import_round_trip_is_lossless() {
    let url = crate::test_db_guard::disposable_database_url("COLLECTIONS_TEST_DATABASE_URL");
    let config = url.parse::<tokio_postgres::Config>().unwrap();
    let manager = deadpool_postgres::Manager::new(config, tokio_postgres::NoTls);
    let pool = deadpool_postgres::Pool::builder(manager)
        .max_size(4)
        .build()
        .unwrap();
    let redis = stub_cache();
    let client = pool.get().await.unwrap();
    let suffix = std::process::id();
    let user_id: i32 = client
        .query_one(
            "INSERT INTO users (username, password, email, created_at, role) VALUES ($1, 'x', $2, now(), 'user') RETURNING userid",
            &[&format!("roundtrip_{suffix}"), &format!("roundtrip_{suffix}@example.invalid")],
        )
        .await
        .unwrap()
        .get(0);
    let lang: i32 = client
        .query_one("SELECT min(langid) FROM languages", &[])
        .await
        .unwrap()
        .get(0);
    let ogg = format!(
        "data:audio/ogg;base64,{}",
        base64::engine::general_purpose::STANDARD.encode(b"OggS\0fake")
    );
    let script = "# At the cafe\n> .i ti zarci\nMarian(F): coi do\n~ hi you\nM: mi tavla\n~ I talk\n---\n? fill: Marian: [coi] do\n? bank: M: mi [tavla]  (chips: klama, citka)\n? choice: Who greets?  + Marian  - nobody";
    let file = json!({
        "format": "lensisku.collection",
        "format_version": 2,
        "collection": {
            "name": format!("rt {suffix}"), "description": "desc", "is_public": false,
            "challenge_mode": "fixed", "cover_image_url": png_data_url()
        },
        "items": [
            {"free_content_front": script, "free_content_back": "At the cafe (story)",
             "collection_note": "story note", "direction": "dialogue", "auto_progress": false,
             "language_id": lang, "license": "CC BY-SA 4.0", "script_code": "Latn", "is_original": false,
             "dialogue": {"voices": {"Marian": "Victoria", "M": "Martin", "narrator": "Eva"},
                          "line_sounds": [
                              {"line_index": 0, "text_hash": "h0", "voice": "Eva", "sound_url": ogg},
                              {"line_index": 2, "text_hash": "h2", "voice": "Martin", "sound_url": ogg}]}},
            {"free_content_front": "coi", "free_content_back": "hello", "direction": "quiz_direct",
             "quiz_correct_answer": "hello", "use_canonical_comparison": false, "sound_url": ogg,
             "front_image_url": png_data_url()},
            {"free_content_front": "mi tavla", "free_content_back": "I talk", "direction": "wordbank_reverse"},
            {"free_content_front": "loose", "free_content_back": "card outside levels", "direction": "auto"}
        ],
        "levels": [
            {"name": "One", "description": "first", "min_cards": 2, "min_success_rate": 0.7,
             "position": 0, "item_positions": [1, 2]},
            {"name": "Two", "description": null, "min_cards": 1, "min_success_rate": 0.5,
             "position": 1, "prerequisite_positions": [0], "item_positions": [0]}
        ]
    });
    let req: ImportFullRequest = serde_json::from_value(file).unwrap();
    let first = import_full(&pool, &redis, user_id, &req).await.unwrap();
    assert!(first.warnings.is_empty(), "{:?}", first.warnings);
    let c1 = first.collection.collection_id;
    let export1 = super::service::export_collection_full(&pool, c1, Some(user_id))
        .await
        .unwrap();
    let mut v1 = serde_json::to_value(&export1).unwrap();

    // Re-import the exported file as-is.
    let req2: ImportFullRequest = serde_json::from_value(v1.clone()).unwrap();
    let second = import_full(&pool, &redis, user_id, &req2).await.unwrap();
    assert!(second.warnings.is_empty(), "{:?}", second.warnings);
    let c2 = second.collection.collection_id;
    let export2 = super::service::export_collection_full(&pool, c2, Some(user_id))
        .await
        .unwrap();
    let mut v2 = serde_json::to_value(&export2).unwrap();

    // Content checks on the first export.
    assert_eq!(v1["format_version"], 2);
    assert_eq!(v1["collection"]["challenge_mode"], "fixed");
    assert!(v1["collection"]["cover_image_url"].is_string());
    let items = v1["items"].as_array().unwrap();
    assert_eq!(items.len(), 4);
    let d = &items[0];
    assert_eq!(d["direction"], "dialogue");
    assert_eq!(d["auto_progress"], false);
    assert_eq!(d["script_code"], "Latn");
    assert_eq!(d["dialogue"]["voices"]["Marian"], "Victoria");
    assert_eq!(d["dialogue"]["line_sounds"].as_array().unwrap().len(), 2);
    let qs = d["dialogue"]["structure"]["questions"].as_array().unwrap();
    assert_eq!(qs.len(), 3, "three separately scored questions");
    assert_eq!(qs[1]["kind"], "bank");
    assert_eq!(d["dialogue"]["structure"]["lines"][1]["gloss"], "hi you");
    assert_eq!(items[1]["quiz_correct_answer"], "hello");
    assert_eq!(items[1]["use_canonical_comparison"], false);
    assert!(items[1]["sound_url"].is_string());
    assert_eq!(items[3]["direction"], "auto");
    assert_eq!(v1["levels"][1]["prerequisite_positions"], json!([0]));

    // Equality modulo ids, positions are re-based and name is the same.
    for v in [&mut v1, &mut v2] {
        for it in v["items"].as_array_mut().unwrap() {
            it.as_object_mut().unwrap().remove("item_id");
        }
    }
    assert_eq!(v1, v2);

    for c in [c1, c2] {
        let _ = client
            .execute("DELETE FROM collections WHERE collection_id = $1", &[&c])
            .await;
    }
}
