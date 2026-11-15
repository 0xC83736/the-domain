//! The ECS World — the single source of truth for all game entities.
//!
//! The world owns the entity allocator, all archetype tables, and the
//! system scheduler.  External code interacts with it exclusively through
//! the typed API; raw pointer access is confined to the archetype layer.

use parking_lot::RwLock;
use std::collections::HashMap;

use crate::archetype::Archetype;
use crate::entity::{Entity, EntityAllocator, EntityId};

/// Location of an entity within an archetype table.
///
/// Not yet read — Phase 1 Step 1 prompt wires archetype storage into
/// `World::spawn`/`despawn`, at which point `entity_map` starts populating
/// this with real locations instead of leaving it as a placeholder.
#[derive(Clone, Copy)]
#[allow(dead_code)]
struct EntityLocation {
    archetype_index: usize,
    row: usize,
}

/// The central ECS world.  Cheap to clone as an `Arc<World>`.
pub struct World {
    allocator: EntityAllocator,
    #[allow(dead_code)]
    archetypes: RwLock<Vec<Archetype>>,
    entity_map: RwLock<HashMap<EntityId, EntityLocation>>,
}

impl World {
    /// Create an empty world.
    pub fn new() -> Self {
        Self {
            allocator: EntityAllocator::new(),
            archetypes: RwLock::new(Vec::new()),
            entity_map: RwLock::new(HashMap::new()),
        }
    }

    /// Spawn a new entity.  Returns its [`Entity`] handle.
    ///
    /// Components are added via builder pattern after spawn (Phase 1 Step 1
    /// prompt implements `world.spawn().insert(comp)` syntax).
    pub fn spawn(&self) -> Entity {
        self.allocator.allocate()
    }

    /// Despawn an entity, freeing its slot for future reuse.
    ///
    /// Any stale [`Entity`] handles pointing at this slot become invalid;
    /// subsequent accesses return `None`.
    pub fn despawn(&self, entity: Entity) {
        let mut map = self.entity_map.write();
        map.remove(&entity);
        self.allocator.free(entity);
    }

    /// Returns `true` if the entity is currently alive.
    pub fn is_alive(&self, entity: Entity) -> bool {
        self.entity_map.read().contains_key(&entity)
    }

    /// Total number of live entities.
    pub fn entity_count(&self) -> usize {
        self.entity_map.read().len()
    }
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}
