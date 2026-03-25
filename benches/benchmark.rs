use criterion::{black_box, criterion_group, criterion_main, Criterion};
use jsonrepair_rs::jsonrepair;

fn bench_valid_small(c: &mut Criterion) {
    let input = r#"{"name": "John", "age": 30, "items": [1, 2, 3]}"#;
    c.bench_function("valid_small", |b| {
        b.iter(|| jsonrepair(black_box(input)).unwrap())
    });
}

fn bench_broken_small(c: &mut Criterion) {
    let input = "{'name': 'John', 'age': 30, 'items': [1, 2, 3,]}";
    c.bench_function("broken_small", |b| {
        b.iter(|| jsonrepair(black_box(input)).unwrap())
    });
}

fn bench_valid_large(c: &mut Criterion) {
    let mut input = String::from("[");
    for i in 0..1000 {
        if i > 0 {
            input.push(',');
        }
        input.push_str(&format!(
            r#"{{"id": {}, "name": "item_{}", "value": {}}}"#,
            i,
            i,
            i * 10
        ));
    }
    input.push(']');
    c.bench_function("valid_large_1k", |b| {
        b.iter(|| jsonrepair(black_box(&input)).unwrap())
    });
}

fn bench_broken_large(c: &mut Criterion) {
    let mut input = String::from("[");
    for i in 0..1000 {
        if i > 0 {
            input.push(',');
        }
        input.push_str(&format!(
            "{{'id': {}, 'name': 'item_{}', 'value': {},}}",
            i,
            i,
            i * 10
        ));
    }
    input.push(']');
    c.bench_function("broken_large_1k", |b| {
        b.iter(|| jsonrepair(black_box(&input)).unwrap())
    });
}

fn bench_deeply_nested(c: &mut Criterion) {
    let depth = 100;
    let input = "[".repeat(depth) + &"]".repeat(depth);
    c.bench_function("nested_100", |b| {
        b.iter(|| jsonrepair(black_box(&input)).unwrap())
    });
}

fn bench_comments(c: &mut Criterion) {
    let mut input = String::from("{\n");
    for i in 0..100 {
        input.push_str(&format!(
            "  // comment {}\n  \"key_{}\": {},\n",
            i, i, i
        ));
    }
    input.push_str("  \"last\": true\n}");
    c.bench_function("comments_100", |b| {
        b.iter(|| jsonrepair(black_box(&input)).unwrap())
    });
}

fn bench_string_escapes(c: &mut Criterion) {
    let mut input = String::from("[");
    for i in 0..200 {
        if i > 0 {
            input.push(',');
        }
        input.push_str(r#""hello\nworld\t\"quoted\"""#);
    }
    input.push(']');
    c.bench_function("string_escapes_200", |b| {
        b.iter(|| jsonrepair(black_box(&input)).unwrap())
    });
}

criterion_group!(
    benches,
    bench_valid_small,
    bench_broken_small,
    bench_valid_large,
    bench_broken_large,
    bench_deeply_nested,
    bench_comments,
    bench_string_escapes,
);
criterion_main!(benches);
