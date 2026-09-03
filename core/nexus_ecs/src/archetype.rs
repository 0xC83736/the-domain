//! Archetype storage — contiguous component arrays grouped by type signature.
//!
//! Each unique set of component types forms one archetype.  Within an
//! archetype every component type has its own `Vec<u8>` column stored as
//! raw bytes; type safety is recovered at the query layer via generic
//! accessors.

use std::any::TypeId;
use std::collections::HashMap;

/// Opaque per-column byte storage.
pub(crate) struct Column {
    pub item_size: usize,
    pub data: Vec<u8>,
}

impl Column {
    pub fn new(item_size: usize) -> Self {
        Self { item_size, data: Vec::new() }
    }

    pub fn push_raw(&mut self, bytes: &[u8]) {
        assert_eq!(bytes.len(), self.item_size);
        self.data.extend_from_slice(bytes);
    }

    pub fn len(&self) -> usize {
        if self.item_size == 0 { 0 } else { self.data.len() / self.item_size }
    }

    /// # Safety
    /// Caller must ensure `T` matches the type this column was created for.
    pub unsafe fn get<T: 'static>(&self, row: usize) -> &T {
        let offset = row * self.item_size;
        &*(self.data.as_ptr().add(offset) as *const T)
    }

    /// # Safety
    /// Same constraint as [`get`].
    pub unsafe fn get_mut<T: 'static>(&mut self, row: usize) -> &mut T {
        let offset = row * self.item_size;
        &mut *(self.data.as_mut_ptr().add(offset) as *mut T)
    }
}

/// One archetype: a table where each row is an entity and each column is a component.
pub(crate) struct Archetype {
    pub type_ids: Vec<TypeId>,
    pub columns: HashMap<TypeId, Column>,
    /// Entity IDs stored at each row for reverse lookup.
    pub entities: Vec<crate::EntityId>,
}

impl Archetype {
    pub fn new(type_ids: Vec<TypeId>, item_sizes: &[(TypeId, usize)]) -> Self {
        let mut columns = HashMap::new();
        for &(tid, size) in item_sizes {
            columns.insert(tid, Column::new(size));
        }
        Self { type_ids, columns, entities: Vec::new() }
    }

    pub fn row_count(&self) -> usize {
        self.entities.len()
    }
}
