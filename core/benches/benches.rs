mod types;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rand::{rngs::StdRng, SeedableRng};
use serde_intermediate::{Change, DiffOptimizationHint, DiffOptions, ReflectIntermediate};
use types::*;

fn randomizer() -> StdRng {
    StdRng::seed_from_u64(0)
}

fn serialize(c: &mut Criterion) {
    let mut rng = randomizer();
    let input = &Account::generate(&mut rng);

    c.bench_function("Serialize JSON", |b| {
        b.iter(|| {
            let _ = serde_json::to_string(black_box(input)).unwrap();
        })
    });
    c.bench_function("Serialize JSON value", |b| {
        b.iter(|| {
            let _ = serde_json::to_value(black_box(input)).unwrap();
        })
    });
    c.bench_function("Serialize YAML", |b| {
        b.iter(|| {
            let _ = serde_yaml::to_string(black_box(input)).unwrap();
        })
    });
    c.bench_function("Serialize YAML value", |b| {
        b.iter(|| {
            let _ = serde_yaml::to_value(black_box(input)).unwrap();
        })
    });
    c.bench_function("Serialize RON", |b| {
        b.iter(|| {
            let _ = ron::to_string(black_box(input)).unwrap();
        })
    });
    c.bench_function("Serialize Bincode", |b| {
        b.iter(|| {
            let _ = bincode::serialize(black_box(input)).unwrap();
        })
    });
    c.bench_function("Serialize Intermediate", |b| {
        b.iter(|| {
            let _ = serde_intermediate::to_intermediate(black_box(input)).unwrap();
        })
    });
}

fn deserialize(c: &mut Criterion) {
    let mut rng = randomizer();
    let input = &Account::generate(&mut rng);

    c.bench_function("Deserialize JSON", |b| {
        let input = &serde_json::to_string(input).unwrap();
        b.iter(|| {
            let _ = serde_json::from_str::<Account>(black_box(input)).unwrap();
        })
    });
    c.bench_function("Deserialize JSON value", |b| {
        let input = &serde_json::to_value(input).unwrap();
        b.iter(|| {
            let _ = serde_json::from_value::<Account>(black_box(input.to_owned())).unwrap();
        })
    });
    c.bench_function("Deserialize YAML", |b| {
        let input = &serde_yaml::to_string(input).unwrap();
        b.iter(|| {
            let _ = serde_yaml::from_str::<Account>(black_box(input)).unwrap();
        })
    });
    c.bench_function("Deserialize YAML value", |b| {
        let input = &serde_yaml::to_value(input).unwrap();
        b.iter(|| {
            let _ = serde_yaml::from_value::<Account>(black_box(input.to_owned())).unwrap();
        })
    });
    c.bench_function("Deserialize RON", |b| {
        let input = &ron::to_string(input).unwrap();
        b.iter(|| {
            let _ = ron::from_str::<Account>(black_box(input)).unwrap();
        })
    });
    c.bench_function("Deserialize Bincode", |b| {
        let input = &bincode::serialize(input).unwrap();
        b.iter(|| {
            let _ = bincode::deserialize::<Account>(black_box(input)).unwrap();
        })
    });
    c.bench_function("Deserialize Intermediate", |b| {
        let input = &serde_intermediate::to_intermediate(input).unwrap();
        b.iter(|| {
            let _ = serde_intermediate::from_intermediate::<Account>(black_box(input)).unwrap();
        })
    });
}

fn patching(c: &mut Criterion) {
    let mut rng = randomizer();
    let input_a = &Account::generate(&mut rng);
    let input_b = &Account::generate(&mut rng);
    let options = &DiffOptions::default();

    c.bench_function("Calculate change", |b| {
        let input_a = &serde_intermediate::to_intermediate(input_a).unwrap();
        let input_b = &serde_intermediate::to_intermediate(input_b).unwrap();
        b.iter(|| {
            let _ = Change::difference(input_a, input_b, options);
        })
    });

    c.bench_function("Patch change indirectly", |b| {
        let input_a = &serde_intermediate::to_intermediate(input_a).unwrap();
        let input_b = &serde_intermediate::to_intermediate(input_b).unwrap();
        let change = &Change::difference(input_a, input_b, options);
        b.iter(|| {
            let _ = black_box(change).patch(input_a).unwrap();
        })
    });

    c.bench_function("Patch change directly", |b| {
        let change = &{
            let input_a = &serde_intermediate::to_intermediate(input_a).unwrap();
            let input_b = &serde_intermediate::to_intermediate(input_b).unwrap();
            Change::difference(input_a, input_b, options)
        };
        b.iter(|| {
            let mut target = input_a.to_owned();
            let _ = target.patch_change(black_box(change));
        })
    });
}

fn patching_optimized(c: &mut Criterion) {
    let mut rng = randomizer();
    let input_a = &Account::generate(&mut rng);
    let input_b = &Account::generate(&mut rng);
    let options = &DiffOptions::default().optimization_hint(DiffOptimizationHint::SizeTarget);

    c.bench_function("Calculate optimized change", |b| {
        let input_a = &serde_intermediate::to_intermediate(input_a).unwrap();
        let input_b = &serde_intermediate::to_intermediate(input_b).unwrap();
        b.iter(|| {
            let _ = Change::difference(input_a, input_b, options);
        })
    });

    c.bench_function("Patch optimized change indirectly", |b| {
        let input_a = &serde_intermediate::to_intermediate(input_a).unwrap();
        let input_b = &serde_intermediate::to_intermediate(input_b).unwrap();
        let change = &Change::difference(input_a, input_b, options);
        b.iter(|| {
            let _ = black_box(change).patch(input_a).unwrap();
        })
    });

    c.bench_function("Patch optimized change directly", |b| {
        let change = &{
            let input_a = &serde_intermediate::to_intermediate(input_a).unwrap();
            let input_b = &serde_intermediate::to_intermediate(input_b).unwrap();
            Change::difference(input_a, input_b, options)
        };
        b.iter(|| {
            let mut target = input_a.to_owned();
            let _ = target.patch_change(black_box(change));
        })
    });
}

fn dlcs(c: &mut Criterion) {
    let mut rng = randomizer();
    let base = &Account::generate(&mut rng);
    let patch_a = &Account::generate(&mut rng);
    let patch_b = &Account::generate(&mut rng);
    let options = &DiffOptions::default().optimization_hint(DiffOptimizationHint::SizeTarget);
    let change_a = &Change::data_difference(base, patch_a, options).unwrap();
    let change_b = &Change::data_difference(base, patch_b, options).unwrap();

    c.bench_function("Apply DLCs", |b| {
        b.iter(|| {
            let patched = change_a.data_patch(black_box(base)).unwrap().unwrap();
            let _ = change_b.data_patch(black_box(&patched)).unwrap().unwrap();
        })
    });
}

criterion_group!(
    benches,
    serialize,
    deserialize,
    patching,
    patching_optimized,
    dlcs,
);
criterion_main!(benches);
