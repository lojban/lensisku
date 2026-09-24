//! Criterion benches for pure search helpers (dictionary / wiki / mail prep).
//!
//! Run: `cargo bench --bench search`
//! Filtered: `cargo bench --bench search -- escape_like`

use std::hint::black_box;
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};

#[path = "../src/search_helpers.rs"]
mod search_helpers;

use search_helpers::{
    escape_like, like_contains_pattern, lojbanize_search_term, looks_like_possible_lujvo_query,
    mail_word_like_patterns, pipe_list_contains, space_list_contains, wiki_relevance_score,
    word_boundary_pattern, DictionarySearchPatterns,
};

fn bench_escape_like(c: &mut Criterion) {
    let long_a = "a".repeat(64);
    let long_loj = "lojban-coi-do'u-%_\\".repeat(8);
    let samples = [
        ("simple", "simple"),
        ("wildcards", "has%wild_cards\\here"),
        ("long_a", long_a.as_str()),
        ("long_loj", long_loj.as_str()),
    ];
    let mut group = c.benchmark_group("escape_like");
    for (name, s) in samples {
        group.throughput(Throughput::Bytes(s.len() as u64));
        group.bench_with_input(BenchmarkId::from_parameter(name), s, |b, s| {
            b.iter(|| escape_like(black_box(s)))
        });
    }
    group.finish();
}

fn bench_dictionary_patterns(c: &mut Criterion) {
    let long = "a".repeat(32);
    let terms = ["mlatu", "brodahe", "coi", "gerku", long.as_str()];
    let mut group = c.benchmark_group("dictionary_patterns");
    for term in terms {
        group.bench_with_input(
            BenchmarkId::new("from_search_term", term),
            term,
            |b, t| b.iter(|| DictionarySearchPatterns::from_search_term(black_box(t))),
        );
    }
    group.finish();
}

fn bench_lojbanize_and_gates(c: &mut Criterion) {
    c.bench_function("lojbanize_brodahe", |b| {
        b.iter(|| lojbanize_search_term(black_box("brodahe")))
    });
    c.bench_function("like_contains", |b| {
        b.iter(|| like_contains_pattern(black_box("mlatu")))
    });
    c.bench_function("word_boundary", |b| {
        b.iter(|| word_boundary_pattern(black_box("mlatu")))
    });
    c.bench_function("lujvo_gate_true", |b| {
        b.iter(|| looks_like_possible_lujvo_query(black_box("brivla")))
    });
    c.bench_function("lujvo_gate_false_phrase", |b| {
        b.iter(|| looks_like_possible_lujvo_query(black_box("hello world")))
    });
}

fn bench_token_membership(c: &mut Criterion) {
    let gloss = "cat|dog|bird|fish|house|tree|water|fire|earth|air";
    let rafsi = "bro bya byu bla bli blo blu cma cme cmi";
    c.bench_function("pipe_list_hit", |b| {
        b.iter(|| pipe_list_contains(black_box(gloss), black_box("water")))
    });
    c.bench_function("pipe_list_miss", |b| {
        b.iter(|| pipe_list_contains(black_box(gloss), black_box("zzzz")))
    });
    c.bench_function("space_list_hit", |b| {
        b.iter(|| space_list_contains(black_box(rafsi), black_box("bli")))
    });
}

fn bench_wiki_relevance(c: &mut Criterion) {
    let titles: Vec<String> = (0..500)
        .map(|i| format!("Lojban page title number {i} with extra words"))
        .collect();
    c.bench_function("wiki_relevance_500", |b| {
        b.iter(|| {
            let term = black_box("lojban");
            let mut scores = 0i32;
            for t in &titles {
                scores += wiki_relevance_score(term, &t.to_lowercase());
            }
            scores
        })
    });
}

fn bench_mail_word_patterns(c: &mut Criterion) {
    let queries = [
        "lojban",
        "lojban grammar",
        "the quick brown fox jumps over the lazy dog",
    ];
    let mut group = c.benchmark_group("mail_word_patterns");
    for q in queries {
        group.bench_with_input(
            BenchmarkId::new("words", q.split_whitespace().count()),
            q,
            |b, q| b.iter(|| mail_word_like_patterns(black_box(q))),
        );
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_escape_like,
    bench_dictionary_patterns,
    bench_lojbanize_and_gates,
    bench_token_membership,
    bench_wiki_relevance,
    bench_mail_word_patterns,
);
criterion_main!(benches);
