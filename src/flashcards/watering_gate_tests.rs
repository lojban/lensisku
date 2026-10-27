//! `apply_watering_gate`: completed-but-weak earlier levels hold back unstarted next levels.
#![allow(clippy::unwrap_used)]

use super::dto::{ChronoDateTime, LevelProgress, LevelResponse, PrerequisiteLevel};
use super::service::apply_watering_gate;

fn lvl(
    id: i32,
    pos: i32,
    prereq: Option<i32>,
    completed: bool,
    strength: Option<i32>,
    due: i32,
    started: bool,
) -> LevelResponse {
    let now = ChronoDateTime(chrono::Utc::now());
    let prereq_completed = true;
    LevelResponse {
        level_id: id,
        name: format!("L{id}"),
        description: None,
        min_cards: 1,
        min_success_rate: 0.8,
        position: pos,
        prerequisites: prereq
            .map(|p| {
                vec![PrerequisiteLevel {
                    level_id: p,
                    name: format!("L{p}"),
                    is_completed: prereq_completed,
                }]
            })
            .unwrap_or_default(),
        progress: (completed || started).then_some(LevelProgress {
            cards_completed: 1,
            correct_answers: 1,
            total_answers: 1,
            success_rate: 1.0,
            is_unlocked: true,
            is_completed: completed,
            unlocked_at: None,
            completed_at: None,
            last_activity_at: ChronoDateTime(now.0),
        }),
        card_count: 5,
        is_locked: false,
        is_started: started || completed,
        strength,
        due_count: due,
        needs_watering: strength.is_some_and(|s| due > 0 || s < 70),
        watering_locked: false,
        water_first_level_id: None,
        created_at: now,
    }
}

#[test]
fn strong_prerequisite_unlocks_next() {
    let mut v = vec![
        lvl(1, 0, None, true, Some(95), 0, true),
        lvl(2, 1, Some(1), false, None, 0, false),
    ];
    apply_watering_gate(&mut v);
    assert!(!v[1].is_locked && !v[1].watering_locked && v[1].water_first_level_id.is_none());
}

#[test]
fn weak_direct_prerequisite_gates_unstarted_level() {
    let mut v = vec![
        lvl(1, 0, None, true, Some(80), 3, true),
        lvl(2, 1, Some(1), false, None, 0, false),
    ];
    apply_watering_gate(&mut v);
    assert!(v[1].is_locked && v[1].watering_locked);
    assert_eq!(v[1].water_first_level_id, Some(1));
}

#[test]
fn started_level_is_never_relocked_only_hinted() {
    let mut v = vec![
        lvl(1, 0, None, true, Some(20), 3, true),
        lvl(2, 1, Some(1), false, Some(90), 0, true),
    ];
    apply_watering_gate(&mut v);
    assert!(!v[1].is_locked && !v[1].watering_locked);
    assert_eq!(v[1].water_first_level_id, Some(1));
}

#[test]
fn wilted_earlier_level_gates_and_weakest_is_pointed_to() {
    // L1 wilted (30), L2 slightly weak direct prereq (60), L3 next: weakest = L1.
    let mut v = vec![
        lvl(1, 0, None, true, Some(30), 0, true),
        lvl(2, 1, Some(1), true, Some(60), 0, true),
        lvl(3, 2, Some(2), false, None, 0, false),
    ];
    apply_watering_gate(&mut v);
    assert!(v[2].watering_locked);
    assert_eq!(v[2].water_first_level_id, Some(1));
    // Mildly weak (50..70) non-direct earlier levels do not gate.
    let mut v = vec![
        lvl(1, 0, None, true, Some(55), 1, true),
        lvl(2, 1, Some(1), true, Some(95), 0, true),
        lvl(3, 2, Some(2), false, None, 0, false),
    ];
    apply_watering_gate(&mut v);
    assert!(!v[2].is_locked);
}

#[test]
fn prerequisite_locked_levels_untouched() {
    let mut v = vec![
        lvl(1, 0, None, false, Some(10), 3, true),
        lvl(2, 1, Some(1), false, None, 0, false),
    ];
    v[1].is_locked = true;
    apply_watering_gate(&mut v);
    assert!(!v[1].watering_locked && v[1].water_first_level_id.is_none());
}
