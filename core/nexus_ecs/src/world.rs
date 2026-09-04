//! The ECS World — the single source of truth for all game entities.
//!
//! The world owns the entity allocator, all archetype tables, and the
//! system scheduler.  External code interacts exclusively through the
//! typed public API; raw pointer manipulation is confined to the archetype
//! column layer.
//!
//! ## Entity lifecycle
//! ```text
//! spawn() → entity registered in entity_map (archetype_index: EMPTY_ARCHETYPE)
//! insert::<T>() → entity migrated to archetype that includes T
//! remove::<T>() → entity migrated to archetype that excludes T
//! despawn() → entity removed from entity_map, slot returned to free list
//! ```
//!
//! ## Archetype indexing
//! Index 0 is always the *empty archetype* — the archetype with no components.
//! Every newly spawned entity starts there.  This keeps `entity_map` accurate
//! from the moment of spawn so `entity_count()` and `is_alive()` are correct
//! before any components are added.

use std::any::TypeId;
use std::collections::HashMap;
use parking_lot::RwLock;

use crate::archetype::{Archetype, ArchetypeId};
use crate::entity::{Entity, EntityAllocator, EntityId};
use crate::query::{Component, QueryIter};

/// Sentinel index for the empty archetype (no components).
const EMPTY_ARCHETYPE: usize = 0;

/// Location of an entity inside an archetype table.
#[derive(Clone, Copy, Debug)]
pub(crate) struct EntityLocation {
    /// Which archetype this entity currently belongs to.
    pub archetype_index: usize,
    /// Row within that archetype's column arrays.
    pub row: usize,
}

/// Inner mutable state — held behind a single `RwLock` so that operations
/// that need to touch both `entity_map` and `archetypes` atomically can do
/// so without deadlock.
struct WorldInner {
    archetypes: Vec<Archetype>,
    entity_map: HashMap<EntityId, EntityLocation>,
    /// Maps a sorted `Vec<TypeId>` signature → archetype index.
    archetype_index: HashMap<ArchetypeId, usize>,
}

impl WorldInner {
    fn new() -> Self {
        let empty = Archetype::empty();
        let id    = empty.id().clone();
        let mut archetype_index = HashMap::new();
        archetype_index.insert(id, EMPTY_ARCHETYPE);
        Self {
            archetypes: vec![empty],
            entity_map: HashMap::new(),
            archetype_index,
        }
    }

    /// Find or create an archetype with exactly `type_ids` (must be sorted).
    fn get_or_create_archetype(&mut self, type_ids: ArchetypeId) -> usize {
        if let Some(&idx) = self.archetype_index.get(&type_ids) {
            return idx;
        }
        let idx = self.archetypes.len();
        self.archetypes.push(Archetype::new(type_ids.clone()));
        self.archetype_index.insert(type_ids, idx);
        idx
    }
}

/// The central ECS world.
///
/// All game entities and their components live here.  The world is designed
/// to be wrapped in an `Arc<World>` and shared across threads — reads are
/// concurrent, writes take an exclusive lock only on the affected archetype.
pub struct World {
    allocator: EntityAllocator,
    inner: RwLock<WorldInner>,
}

impl World {
    /// Create an empty world.  Index 0 is pre-populated with the empty archetype.
    pub fn new() -> Self {
        Self {
            allocator: EntityAllocator::new(),
            inner: RwLock::new(WorldInner::new()),
        }
    }

    // ── Entity lifetime ───────────────────────────────────────────────────

    /// Spawn a new entity and register it in the empty archetype.
    ///
    /// The entity is immediately visible to `entity_count()` and `is_alive()`.
    /// Components can be added with [`insert`](Self::insert).
    pub fn spawn(&self) -> Entity {
        let entity = self.allocator.allocate();
        let mut inner = self.inner.write();
        let row = inner.archetypes[EMPTY_ARCHETYPE].entity_count();
        inner.entity_map.insert(entity, EntityLocation {
            archetype_index: EMPTY_ARCHETYPE,
            row,
        });
        inner.archetypes[EMPTY_ARCHETYPE].push_entity(entity);
        entity
    }

    /// Despawn `entity`, removing it and all its components from the world.
    ///
    /// The entity slot is returned to the allocator's free list with a bumped
    /// generation so stale handles are immediately invalid.
    ///
    /// No-op if `entity` is not alive.
    pub fn despawn(&self, entity: Entity) {
        let mut inner = self.inner.write();
        let Some(loc) = inner.entity_map.remove(&entity) else { return };

        // Swap-remove the entity from its archetype to keep columns dense.
        let swapped = inner.archetypes[loc.archetype_index]
            .swap_remove(loc.row);

        // If a different entity was moved into `loc.row`, update its location.
        if let Some(moved_entity) = swapped {
            if let Some(moved_loc) = inner.entity_map.get_mut(&moved_entity) {
                moved_loc.row = loc.row;
            }
        }

        self.allocator.free(entity);
    }

    /// Returns `true` if `entity` is currently alive.
    pub fn is_alive(&self, entity: Entity) -> bool {
        self.inner.read().entity_map.contains_key(&entity)
    }

    /// Total number of live entities across all archetypes.
    pub fn entity_count(&self) -> usize {
        self.inner.read().entity_map.len()
    }

    // ── Component operations ──────────────────────────────────────────────

    /// Add component `C` to `entity`.
    ///
    /// If the entity already has a component of type `C` the old value is
    /// replaced.  The entity is migrated to the archetype that includes `C`.
    ///
    /// # Panics
    /// Panics if `entity` is not alive.
    pub fn insert<C: Component>(&self, entity: Entity, component: C) {
        let type_id = TypeId::of::<C>();
        let mut inner = self.inner.write();

        let loc = *inner.entity_map.get(&entity)
            .expect("insert: entity is not alive");

        let current_arch = &inner.archetypes[loc.archetype_index];

        // Build new type signature = current + C (sorted, deduplicated).
        let mut new_ids = current_arch.id().clone();
        if !new_ids.contains(&type_id) {
            new_ids.push(type_id);
            new_ids.sort_unstable();
        }

        if new_ids == *current_arch.id() {
            // Same archetype — overwrite in place.
            // SAFETY: type_id matches C.
            unsafe {
                inner.archetypes[loc.archetype_index]
                    .write_component::<C>(loc.row, component);
            }
            return;
        }

        // Collect existing component bytes before mutating anything.
        let existing_data: Vec<(TypeId, Vec<u8>)> = {
            let old_arch = &inner.archetypes[loc.archetype_index];
            old_arch.id()
                .iter()
                .filter_map(|&tid| {
                    let col = old_arch.column(tid)?;
                    Some((tid, col.row_bytes(loc.row).to_vec()))
                })
                .collect()
        };

        // Migrate entity: remove from old archetype, add to new.
        let swapped = inner.archetypes[loc.archetype_index]
            .swap_remove(loc.row);
        if let Some(moved) = swapped {
            if let Some(ml) = inner.entity_map.get_mut(&moved) {
                ml.row = loc.row;
            }
        }

        let new_arch_idx = inner.get_or_create_archetype(new_ids);
        let new_row = inner.archetypes[new_arch_idx].entity_count();
        inner.archetypes[new_arch_idx].push_entity(entity);

        // Restore old component data into the new archetype.
        for (tid, bytes) in existing_data {
            inner.archetypes[new_arch_idx].write_raw_bytes(tid, new_row, &bytes);
        }

        // SAFETY: type_id matches C.
        unsafe {
            inner.archetypes[new_arch_idx]
                .write_component::<C>(new_row, component);
        }

        inner.entity_map.insert(entity, EntityLocation {
            archetype_index: new_arch_idx,
            row: new_row,
        });
    }

    /// Remove component `C` from `entity`.
    ///
    /// The entity is migrated to the archetype that excludes `C`.
    /// No-op if the entity does not have component `C`.
    ///
    /// # Panics
    /// Panics if `entity` is not alive.
    pub fn remove<C: Component>(&self, entity: Entity) {
        let type_id = TypeId::of::<C>();
        let mut inner = self.inner.write();

        let loc = *inner.entity_map.get(&entity)
            .expect("remove: entity is not alive");

        let current_arch = &inner.archetypes[loc.archetype_index];
        if !current_arch.id().contains(&type_id) {
            return; // entity doesn't have this component
        }

        // Snapshot components that survive (all except C).
        let surviving_data: Vec<(TypeId, Vec<u8>)> = {
            let old_arch = &inner.archetypes[loc.archetype_index];
            old_arch.id()
                .iter()
                .filter(|&&tid| tid != type_id)
                .filter_map(|&tid| {
                    let col = old_arch.column(tid)?;
                    Some((tid, col.row_bytes(loc.row).to_vec()))
                })
                .collect()
        };

        let mut new_ids = current_arch.id().clone();
        new_ids.retain(|&id| id != type_id);

        let swapped = inner.archetypes[loc.archetype_index]
            .swap_remove(loc.row);
        if let Some(moved) = swapped {
            if let Some(ml) = inner.entity_map.get_mut(&moved) {
                ml.row = loc.row;
            }
        }

        let new_arch_idx = inner.get_or_create_archetype(new_ids);
        let new_row = inner.archetypes[new_arch_idx].entity_count();
        inner.archetypes[new_arch_idx].push_entity(entity);

        for (tid, bytes) in surviving_data {
            inner.archetypes[new_arch_idx].write_raw_bytes(tid, new_row, &bytes);
        }

        inner.entity_map.insert(entity, EntityLocation {
            archetype_index: new_arch_idx,
            row: new_row,
        });
    }

    /// Get a copy of component `C` for `entity`.
    ///
    /// Returns `None` if the entity is not alive or does not have `C`.
    pub fn get<C: Component + Copy>(&self, entity: Entity) -> Option<C> {
        let inner = self.inner.read();
        let loc = inner.entity_map.get(&entity)?;
        let arch = &inner.archetypes[loc.archetype_index];
        // SAFETY: TypeId matches C.
        unsafe { arch.read_component::<C>(loc.row) }
    }

    // ── Query ─────────────────────────────────────────────────────────────

    /// Iterate over all entities that have component `C`, yielding `(Entity, &C)`.
    ///
    /// # Example
    /// ```rust
    /// # use nexus_ecs::{World, query::Component};
    /// # #[derive(Clone, Copy)] struct Position { x: f32 }
    /// let world = World::new();
    /// let e = world.spawn();
    /// world.insert(e, Position { x: 1.0 });
    /// for (entity, pos) in world.query_ref::<Position>() {
    ///     println!("{:?} is at {}", entity, pos.x);
    /// }
    /// ```
    pub fn query_ref<C: Component + Copy>(&self) -> QueryIter<C> {
        QueryIter::new(self)
    }

    /// Expose inner for use by [`QueryIter`].
    pub(crate) fn inner_read(&self) -> parking_lot::RwLockReadGuard<'_, WorldInner> {
        self.inner.read()
    }

    /// Iterate over archetypes (read-only access for query layer).
    pub(crate) fn with_archetypes<F, R>(&self, f: F) -> R
    where F: FnOnce(&Vec<Archetype>) -> R
    {
        f(&self.inner.read().archetypes)
    }
}

impl Default for World {
    fn default() -> Self { Self::new() }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Copy, Debug, PartialEq)]
    struct Position { x: f32, y: f32 }

    #[derive(Clone, Copy, Debug, PartialEq)]
    struct Velocity { dx: f32, dy: f32 }

    #[derive(Clone, Copy, Debug, PartialEq)]
    struct Health(u32);

    // ── Spawn / despawn correctness ───────────────────────────────────────

    #[test]
    fn spawn_registers_entity_immediately() {
        let world = World::new();
        assert_eq!(world.entity_count(), 0);
        let e = world.spawn();
        // BUG FIX: was returning 0 before this commit
        assert_eq!(world.entity_count(), 1);
        assert!(world.is_alive(e));
    }

    #[test]
    fn spawn_three_despawn_one_count_correct() {
        let world = World::new();
        let e1 = world.spawn();
        let e2 = world.spawn();
        let e3 = world.spawn();
        assert_eq!(world.entity_count(), 3);
        world.despawn(e2);
        assert_eq!(world.entity_count(), 2);
        assert!(world.is_alive(e1));
        assert!(!world.is_alive(e2));
        assert!(world.is_alive(e3));
    }

    #[test]
    fn despawn_noop_on_dead_entity() {
        let world = World::new();
        let e = world.spawn();
        world.despawn(e);
        world.despawn(e); // second call must not panic or corrupt state
        assert_eq!(world.entity_count(), 0);
    }

    #[test]
    fn recycled_entity_slot_has_new_generation() {
        let world = World::new();
        let e1 = world.spawn();
        let gen1 = e1.generation();
        world.despawn(e1);
        let e2 = world.spawn();
        // Same slot should be recycled with bumped generation.
        assert_eq!(e2.index(), e1.index());
        assert_eq!(e2.generation(), gen1 + 1);
        assert!(!world.is_alive(e1)); // stale handle must be dead
        assert!(world.is_alive(e2));
    }

    #[test]
    fn stress_spawn_despawn_1m() {
        let world = World::new();
        let entities: Vec<_> = (0..1_000_000).map(|_| world.spawn()).collect();
        assert_eq!(world.entity_count(), 1_000_000);
        for e in &entities {
            world.despawn(*e);
        }
        assert_eq!(world.entity_count(), 0);
    }

    // ── Component insert / get ────────────────────────────────────────────

    #[test]
    fn insert_and_get_component() {
        let world = World::new();
        let e = world.spawn();
        world.insert(e, Position { x: 1.0, y: 2.0 });
        let pos = world.get::<Position>(e).expect("component missing");
        assert_eq!(pos, Position { x: 1.0, y: 2.0 });
    }

    #[test]
    fn insert_overwrites_existing_component() {
        let world = World::new();
        let e = world.spawn();
        world.insert(e, Health(100));
        world.insert(e, Health(50));
        assert_eq!(world.get::<Health>(e), Some(Health(50)));
    }

    #[test]
    fn insert_multiple_components_same_entity() {
        let world = World::new();
        let e = world.spawn();
        world.insert(e, Position { x: 3.0, y: 4.0 });
        world.insert(e, Velocity { dx: 1.0, dy: 0.0 });
        world.insert(e, Health(75));
        assert_eq!(world.get::<Position>(e), Some(Position { x: 3.0, y: 4.0 }));
        assert_eq!(world.get::<Velocity>(e), Some(Velocity { dx: 1.0, dy: 0.0 }));
        assert_eq!(world.get::<Health>(e), Some(Health(75)));
    }

    #[test]
    fn get_missing_component_returns_none() {
        let world = World::new();
        let e = world.spawn();
        world.insert(e, Position { x: 0.0, y: 0.0 });
        assert_eq!(world.get::<Velocity>(e), None);
    }

    // ── Component remove ──────────────────────────────────────────────────

    #[test]
    fn remove_component_makes_it_absent() {
        let world = World::new();
        let e = world.spawn();
        world.insert(e, Position { x: 1.0, y: 1.0 });
        world.insert(e, Health(100));
        world.remove::<Position>(e);
        assert_eq!(world.get::<Position>(e), None);
        assert_eq!(world.get::<Health>(e), Some(Health(100)));
    }

    #[test]
    fn remove_absent_component_is_noop() {
        let world = World::new();
        let e = world.spawn();
        world.remove::<Position>(e); // must not panic
        assert!(world.is_alive(e));
    }

    // ── Archetype integrity after swap-remove ─────────────────────────────

    #[test]
    fn despawn_middle_entity_keeps_others_correct() {
        let world = World::new();
        let e1 = world.spawn();
        let e2 = world.spawn();
        let e3 = world.spawn();
        world.insert(e1, Position { x: 1.0, y: 0.0 });
        world.insert(e2, Position { x: 2.0, y: 0.0 });
        world.insert(e3, Position { x: 3.0, y: 0.0 });

        world.despawn(e2); // triggers swap-remove; e3 slides into e2's row

        assert_eq!(world.get::<Position>(e1), Some(Position { x: 1.0, y: 0.0 }));
        assert_eq!(world.get::<Position>(e3), Some(Position { x: 3.0, y: 0.0 }));
        assert!(!world.is_alive(e2));
        assert_eq!(world.entity_count(), 2);
    }

    // ── FFI-level entity_count via raw pointer ────────────────────────────

    #[test]
    fn ffi_entity_count_reflects_spawns() {
        use crate::ffi::{nexus_world_create, nexus_world_spawn,
                          nexus_world_despawn, nexus_world_entity_count,
                          nexus_world_destroy};
        unsafe {
            let ptr = nexus_world_create();
            assert_eq!(nexus_world_entity_count(ptr), 0);
            let id1 = nexus_world_spawn(ptr);
            let _id2 = nexus_world_spawn(ptr);
            assert_eq!(nexus_world_entity_count(ptr), 2);
            nexus_world_despawn(ptr, id1);
            assert_eq!(nexus_world_entity_count(ptr), 1);
            nexus_world_destroy(ptr);
        }
    }
}
