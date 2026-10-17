//! Criterion benchmarks for Rust-native TPTP parsers.
//!
//! Run with: `cargo bench -p bench-native`
//!
//! This benchmarks the `tptp` crate (nom-based, by Michael Rawson) against
//! each file in the corpus. Additional Rust parsers can be added here as
//! library dependencies.

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use mrs_tptp::TPTPIterator as MrsTPTPIterator;
use tptp::TPTPIterator;

// ── Corpus files (embedded at compile time) ─────────────────────────────────

const CNF: &[u8] = include_bytes!("../../corpus/SYN000-1.p");
const FOF: &[u8] = include_bytes!("../../corpus/SYN000+1.p");
const TFF: &[u8] = include_bytes!("../../corpus/SYN000_1.p");
const TFA: &[u8] = include_bytes!("../../corpus/SYN000=2.p");

static CORPUS: &[(&str, &[u8])] = &[
    ("SYN000-1.p (CNF)", CNF),
    ("SYN000+1.p (FOF)", FOF),
    ("SYN000_1.p (TFF)", TFF),
    ("SYN000=2.p (TFA)", TFA),
];

// ── tptp crate (nom, zero-copy) ────────────────────────────────────────────

fn parse_with_tptp_nom(bytes: &[u8]) {
    let mut parser = TPTPIterator::<()>::new(bytes);
    for input in &mut parser {
        input.expect("syntax error");
    }
    assert!(parser.remaining.is_empty());
}

fn bench_tptp_nom(c: &mut Criterion) {
    let mut group = c.benchmark_group("tptp-nom");

    for &(name, bytes) in CORPUS {
        group.throughput(Throughput::Bytes(bytes.len() as u64));
        group.bench_with_input(BenchmarkId::from_parameter(name), &bytes, |b, &data| {
            b.iter(|| parse_with_tptp_nom(data));
        });
    }

    group.finish();
}

// ── mrs-tptp crate (winnow parser combinators) ─────────────────────────────

fn parse_with_mrs_tptp(content: &str) {
    let mut iter = MrsTPTPIterator::new(content);
    for item in &mut iter {
        item.expect("parse error");
    }
    assert!(iter.remaining().is_empty());
}

fn bench_mrs_tptp(c: &mut Criterion) {
    let mut group = c.benchmark_group("mrs-tptp");

    for &(name, bytes) in CORPUS {
        let content = std::str::from_utf8(bytes).expect("corpus file is not UTF-8");
        group.throughput(Throughput::Bytes(bytes.len() as u64));
        group.bench_with_input(BenchmarkId::from_parameter(name), &content, |b, &data| {
            b.iter(|| parse_with_mrs_tptp(data));
        });
    }

    group.finish();
}

criterion_group!(benches, bench_tptp_nom, bench_mrs_tptp);
criterion_main!(benches);
