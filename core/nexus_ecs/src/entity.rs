//! Entity identity with generation counters.
//!
//! An [`EntityId`] is a 64-bit value encoding both a dense index and a
//! generation counter.  When an entity is despawned, the generation is
//! incremented so any stale handles pointing at the old slot are
//! immediately invalid — no ABA aliasing is possible.

use std::sync::atomic::{AtomicU64, Ordering};

/// Packed entity ID: upper 32 bits = generation, lower 32 bits = index.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EntityId(pub(crate) u64);

impl EntityId {
    #[inline]
    pub fn index(self) -> u32 { self.0 as u32 }
    #[inline]
    pub fn generation(self) -> u32 { (self.0 >> 32) as u32 }
    #[inline]
    pub(crate) fn new(index: u32, generation: u32) -> Self {
        Self(((generation as u64) << 32) | index as u64)
    }
}

/// A live entity handle.  Cheaply copyable; validity checked against the
/// world's generation table on every access.
pub type Entity = EntityId;

/// Monotonically incrementing ID allocator.  Used internally by [`World`].
pub(crate) struct EntityAllocator {
    next_index: AtomicU64,
    /// Free list entries encode (index, next_generation) packed as EntityId.
    free: parking_lot::Mutex<Vec<EntityId>>,
}

impl EntityAllocator {
    pub fn new() -> Self {
        Self {
            next_index: AtomicU64::new(0),
            free: parking_lot::Mutex::new(Vec::new()),
        }
    }

    pub fn allocate(&self) -> EntityId {
        let mut free = self.free.lock();
        if let Some(recycled) = free.pop() {
            // bump generation so old handles are dead
            EntityId::new(recycled.index(), recycled.generation().wrapping_add(1))
        } else {
            let idx = self.next_index.fetch_add(1, Ordering::Relaxed) as u32;
            EntityId::new(idx, 0)
        }
    }

    pub fn free(&self, entity: EntityId) {
        self.free.lock().push(entity);
    }
}
