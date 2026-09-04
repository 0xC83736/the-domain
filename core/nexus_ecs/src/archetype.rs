//! Archetype storage — contiguous component arrays grouped by type signature.
//!
//! An *archetype* is a table where:
//! - Each **row** is one entity.
//! - Each **column** is one component type, stored as a flat `Vec<u8>` of
//!   tightly-packed values (SoA layout for cache efficiency).
//!
//! Entities with the same set of component types share an archetype.
//! Adding or removing a component migrates the entity to a different archetype.
//!
//! ## Safety model
//! Column reads and writes use raw pointer casts.  Every public method that
//! performs a cast is `unsafe` and requires the caller to guarantee the
//! `TypeId` matches the concrete type `T`.  Within this crate that invariant
//! is always satisfied because `TypeId::of::<T>()` is used as the key.

use std::any::TypeId;
use std::collections::HashMap;
use std::mem;

use crate::entity::EntityId;

/// A sorted `Vec<TypeId>` identifying an archetype uniquely.
pub(crate) type ArchetypeId = Vec<TypeId>;

// ── Column ───────────────────────────────────────────────────────────────────

/// Opaque contiguous byte storage for one component type.
pub(crate) struct Column {
    /// Size in bytes of one element.
    item_size: usize,
    /// Raw bytes; length is always a multiple of `item_size`.
    data: Vec<u8>,
}

impl Column {
    fn new(item_size: usize) -> Self {
        Self { item_size, data: Vec::new() }
    }

    /// Number of elements stored.
    #[inline]
    pub fn len(&self) -> usize {
        if self.item_size == 0 { 0 } else { self.data.len() / self.item_size }
    }

    /// Append a zero-initialised slot (used when migrating entities that do
    /// not yet have this component).
    fn push_zeroed(&mut self) {
        self.data.extend(std::iter::repeat(0u8).take(self.item_size));
    }

    /// Write `value` at `row`, extending the column if needed.
    ///
    /// # Safety
    /// `T` must be the type this column was created for.
    pub unsafe fn write<T: 'static>(&mut self, row: usize, value: T) {
        debug_assert_eq!(mem::size_of::<T>(), self.item_size);
        let needed = (row + 1) * self.item_size;
        if self.data.len() < needed {
            self.data.resize(needed, 0);
        }
        let offset = row * self.item_size;
        let dst = self.data.as_mut_ptr().add(offset) as *mut T;
        dst.write(value);
    }

    /// Read a copy of the value at `row`.
    ///
    /// # Safety
    /// `T` must be the type this column was created for, and `T: Copy`.
    pub unsafe fn read<T: Copy + 'static>(&self, row: usize) -> T {
        debug_assert_eq!(mem::size_of::<T>(), self.item_size);
        let offset = row * self.item_size;
        let src = self.data.as_ptr().add(offset) as *const T;
        src.read()
    }

    /// Immutable byte slice for the element at `row` (used for migrations).
    pub(crate) fn row_bytes(&self, row: usize) -> &[u8] {
        let start = row * self.item_size;
        &self.data[start..start + self.item_size]
    }

    /// Write raw bytes at `row` directly (used for cross-archetype migrations).
    pub(crate) fn write_raw(&mut self, row: usize, bytes: &[u8]) {
        debug_assert_eq!(bytes.len(), self.item_size);
        let needed = (row + 1) * self.item_size;
        if self.data.len() < needed {
            self.data.resize(needed, 0);
        }
        let start = row * self.item_size;
        self.data[start..start + self.item_size].copy_from_slice(bytes);
    }

    /// Swap-remove row `row`, returning the raw bytes of the element that was
    /// moved (i.e. the former last element, now at `row`), or `None` if the
    /// column is now empty.
    fn swap_remove_row(&mut self, row: usize) -> Option<()> {
        let last_row = self.len().checked_sub(1)?;
        if row != last_row {
            let (start, last_start) = (row * self.item_size, last_row * self.item_size);
            // copy last element into `row`
            self.data.copy_within(last_start..last_start + self.item_size, start);
        }
        self.data.truncate(last_row * self.item_size);
        Some(())
    }
}

// ── Archetype ────────────────────────────────────────────────────────────────

/// One archetype: a table of entities sharing the same component types.
pub(crate) struct Archetype {
    /// Sorted, deduplicated type IDs — the archetype's unique identity.
    id: ArchetypeId,
    /// Component columns keyed by `TypeId`.
    columns: HashMap<TypeId, Column>,
    /// Entity IDs in row order (parallel to columns).
    entities: Vec<EntityId>,
}

impl Archetype {
    /// Create the *empty archetype* (no component columns).
    pub fn empty() -> Self {
        Self {
            id: Vec::new(),
            columns: HashMap::new(),
            entities: Vec::new(),
        }
    }

    /// Create an archetype for the given sorted type signature.
    ///
    /// Columns are sized using [`mem::size_of`] via a registration table
    /// populated lazily when components are first inserted (see
    /// [`write_component`](Self::write_component)).
    pub fn new(id: ArchetypeId) -> Self {
        Self { id, columns: HashMap::new(), entities: Vec::new() }
    }

    /// The archetype's sorted `TypeId` signature.
    pub fn id(&self) -> &ArchetypeId { &self.id }

    /// Number of entities currently stored.
    pub fn entity_count(&self) -> usize { self.entities.len() }

    /// Append `entity` as a new row.  Component columns are extended with
    /// zeroed slots for any types already registered in this archetype.
    pub fn push_entity(&mut self, entity: EntityId) {
        for col in self.columns.values_mut() {
            col.push_zeroed();
        }
        self.entities.push(entity);
    }

    /// Write component `C` at `row`.  Creates the column if this is the first
    /// time type `C` appears in this archetype.
    ///
    /// # Safety
    /// `TypeId::of::<C>()` must match the column being written.
    pub unsafe fn write_component<C: 'static>(&mut self, row: usize, value: C) {
        let type_id = TypeId::of::<C>();
        let col = self.columns
            .entry(type_id)
            .or_insert_with(|| Column::new(mem::size_of::<C>()));
        col.write::<C>(row, value);
    }

    /// Read component `C` at `row`, returning a copy.  Returns `None` if this
    /// archetype does not have a column for `C`.
    ///
    /// # Safety
    /// `TypeId::of::<C>()` must match the column being read.
    pub unsafe fn read_component<C: Copy + 'static>(&self, row: usize) -> Option<C> {
        let col = self.columns.get(&TypeId::of::<C>())?;
        Some(col.read::<C>(row))
    }

    /// Immutable column reference for the query layer.
    pub fn column(&self, type_id: TypeId) -> Option<&Column> {
        self.columns.get(&type_id)
    }

    /// Entity slice for the query layer.
    pub fn entities(&self) -> &[EntityId] { &self.entities }

    /// Write raw bytes for component `type_id` at `row` (used for migrations).
    /// Creates the column with `item_size = bytes.len()` if it doesn't exist.
    pub fn write_raw_bytes(&mut self, type_id: TypeId, row: usize, bytes: &[u8]) {
        let col = self.columns
            .entry(type_id)
            .or_insert_with(|| Column::new(bytes.len()));
        col.write_raw(row, bytes);
    }

    /// Swap-remove the entity at `row`.
    ///
    /// Returns the entity that was moved into `row` (formerly the last entity),
    /// or `None` if the removed entity was the only one.
    pub fn swap_remove(&mut self, row: usize) -> Option<EntityId> {
        let last = self.entities.len().checked_sub(1)?;
        for col in self.columns.values_mut() {
            col.swap_remove_row(row);
        }
        if row == last {
            self.entities.pop();
            None
        } else {
            self.entities.swap_remove(row);
            Some(self.entities[row])
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Copy, PartialEq, Debug)]
    struct Hp(u32);

    #[test]
    fn column_write_read() {
        let mut col = Column::new(mem::size_of::<Hp>());
        unsafe {
            col.write::<Hp>(0, Hp(100));
            assert_eq!(col.read::<Hp>(0), Hp(100));
        }
    }

    #[test]
    fn column_swap_remove_preserves_others() {
        let mut col = Column::new(4);
        for i in 0u32..5 {
            unsafe { col.write(i as usize, i); }
        }
        col.swap_remove_row(1); // removes index 1, moves 4 there
        assert_eq!(col.len(), 4);
        unsafe {
            assert_eq!(col.read::<u32>(0), 0);
            assert_eq!(col.read::<u32>(1), 4);
            assert_eq!(col.read::<u32>(2), 2);
            assert_eq!(col.read::<u32>(3), 3);
        }
    }

    #[test]
    fn archetype_push_and_read() {
        let mut arch = Archetype::new(vec![TypeId::of::<Hp>()]);
        let dummy_id = crate::entity::EntityId::new(0, 0);
        arch.push_entity(dummy_id);
        unsafe {
            arch.write_component::<Hp>(0, Hp(42));
            assert_eq!(arch.read_component::<Hp>(0), Some(Hp(42)));
        }
    }

    #[test]
    fn archetype_swap_remove_returns_moved_entity() {
        let mut arch = Archetype::new(vec![]);
        let e0 = crate::entity::EntityId::new(0, 0);
        let e1 = crate::entity::EntityId::new(1, 0);
        let e2 = crate::entity::EntityId::new(2, 0);
        arch.push_entity(e0);
        arch.push_entity(e1);
        arch.push_entity(e2);
        let moved = arch.swap_remove(0); // removes e0, moves e2 to row 0
        assert_eq!(moved, Some(e2));
        assert_eq!(arch.entity_count(), 2);
        assert_eq!(arch.entities()[0], e2);
        assert_eq!(arch.entities()[1], e1);
    }
}
