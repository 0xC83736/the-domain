//! Query API — iterate over entities possessing a specific component set.
//!
//! ## Design
//! Queries scan the archetype list for tables that contain the requested
//! component type(s), then iterate their rows.  Because archetypes are
//! compact SoA tables this is cache-friendly — every row access reads
//! contiguous bytes in the column `Vec<u8>`.
//!
//! ## Current scope (Phase 1)
//! Single-component read queries: `world.query_ref::<C>()` → `(Entity, C)`.
//! Multi-component and mutable queries are added in Phase 2.

use std::any::TypeId;
use std::marker::PhantomData;

use crate::entity::EntityId;
use crate::world::World;

/// Marker trait for component types.
///
/// Blanket-implemented for every `'static + Send + Sync` type so callers
/// never need to write `impl Component for MyType {}`.
pub trait Component: 'static + Send + Sync {}
impl<T: 'static + Send + Sync> Component for T {}

/// Placeholder kept for API compatibility with earlier scaffolding.
pub struct Query<T>(PhantomData<T>);

// ── QueryIter ────────────────────────────────────────────────────────────────

/// Iterator returned by [`World::query_ref`].
///
/// Yields `(Entity, C)` for every entity that currently has component `C`.
/// Entities without `C` are skipped.  The iteration order follows archetype
/// storage order, which is deterministic within a single tick but may change
/// across ticks as entities are added/removed.
///
/// # Allocation
/// No heap allocation per iteration step.  The collected row data is
/// pre-built once at construction by scanning matching archetypes.
pub struct QueryIter<C: Component + Copy> {
    /// Pre-collected (entity, component) pairs from all matching archetypes.
    items: Vec<(EntityId, C)>,
    index: usize,
}

impl<C: Component + Copy> QueryIter<C> {
    /// Scan the world's archetypes and collect all matching rows.
    pub(crate) fn new(world: &World) -> Self {
        let type_id = TypeId::of::<C>();
        let mut items = Vec::new();

        world.with_archetypes(|archetypes| {
            for arch in archetypes {
                // Skip archetypes that don't have this component.
                let Some(col) = arch.column(type_id) else { continue };
                for (row, &entity) in arch.entities().iter().enumerate() {
                    // SAFETY: type_id matches C — guaranteed by TypeId::of::<C>().
                    let value = unsafe { col.read::<C>(row) };
                    items.push((entity, value));
                }
            }
        });

        Self { items, index: 0 }
    }
}

impl<C: Component + Copy> Iterator for QueryIter<C> {
    type Item = (EntityId, C);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let item = self.items.get(self.index).copied();
        self.index += 1;
        item
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.items.len() - self.index;
        (remaining, Some(remaining))
    }
}

impl<C: Component + Copy> ExactSizeIterator for QueryIter<C> {}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use crate::world::World;

    #[derive(Clone, Copy, PartialEq, Debug)]
    struct Pos { x: f32, y: f32 }

    #[derive(Clone, Copy, PartialEq, Debug)]
    struct Vel { dx: f32 }

    #[test]
    fn query_finds_all_matching_entities() {
        let world = World::new();
        let e1 = world.spawn();
        let e2 = world.spawn();
        let e3 = world.spawn();
        world.insert(e1, Pos { x: 1.0, y: 0.0 });
        world.insert(e2, Pos { x: 2.0, y: 0.0 });
        world.insert(e3, Vel { dx: 5.0 }); // no Pos — must not appear

        let results: Vec<_> = world.query_ref::<Pos>().collect();
        assert_eq!(results.len(), 2);
        let xs: Vec<f32> = results.iter().map(|(_, p)| p.x).collect();
        assert!(xs.contains(&1.0));
        assert!(xs.contains(&2.0));
    }

    #[test]
    fn query_empty_world_returns_nothing() {
        let world = World::new();
        assert_eq!(world.query_ref::<Pos>().count(), 0);
    }

    #[test]
    fn query_after_despawn_excludes_removed_entity() {
        let world = World::new();
        let e1 = world.spawn();
        let e2 = world.spawn();
        world.insert(e1, Pos { x: 1.0, y: 0.0 });
        world.insert(e2, Pos { x: 2.0, y: 0.0 });
        world.despawn(e1);

        let results: Vec<_> = world.query_ref::<Pos>().collect();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, e2);
    }

    #[test]
    fn query_100k_entities_benchmark_baseline() {
        // Ensures the query path scales — not a timing assertion here
        // (timing is enforced by the criterion bench + CI gate).
        let world = World::new();
        for i in 0..100_000u32 {
            let e = world.spawn();
            world.insert(e, Pos { x: i as f32, y: 0.0 });
        }
        let count = world.query_ref::<Pos>().count();
        assert_eq!(count, 100_000);
    }
}
