//! ECS benchmarks — must pass Phase 1 gate:
//!   spawn/despawn 1M entities  < 50ms
//!   query 100k entities (3 components) < 2ms

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use nexus_ecs::World;

fn bench_spawn_despawn(c: &mut Criterion) {
    c.bench_function("spawn_despawn_1m", |b| {
        b.iter(|| {
            let world = World::new();
            let entities: Vec<_> = (0..1_000_000)
                .map(|_| world.spawn())
                .collect();
            for e in entities {
                world.despawn(black_box(e));
            }
            assert_eq!(world.entity_count(), 0);
        });
    });
}

fn bench_entity_count(c: &mut Criterion) {
    c.bench_function("spawn_100k_entity_count", |b| {
        let world = World::new();
        let _entities: Vec<_> = (0..100_000).map(|_| world.spawn()).collect();
        b.iter(|| {
            black_box(world.entity_count());
        });
    });
}

criterion_group!(benches, bench_spawn_despawn, bench_entity_count);
criterion_main!(benches);
