//! # FFI exports for UE5 NexusCore plugin
//!
//! Exposes the ECS [`World`] over a C ABI so that `NexusWorldSubsystem.cpp`
//! can call through from the UE5 game thread.
//!
//! ## Safety contract
//! - Every `*mut World` pointer produced by [`nexus_world_create`] must be
//!   freed by exactly one call to [`nexus_world_destroy`].
//! - All other functions must be called with a non-null pointer obtained from
//!   [`nexus_world_create`].  Passing null or a dangling pointer is UB.
//! - The caller (UE5) is responsible for ensuring these functions are not
//!   called concurrently with [`nexus_world_destroy`].

use crate::world::World;

/// Allocate a new [`World`] on the heap and return an opaque owning pointer.
///
/// Matches `void* nexus_world_create()` in `NexusWorldSubsystem.cpp`.
#[no_mangle]
pub extern "C" fn nexus_world_create() -> *mut World {
    let world = Box::new(World::new());
    Box::into_raw(world)
}

/// Destroy a [`World`] previously created by [`nexus_world_create`].
///
/// Drops the box, running all destructors and freeing memory.
/// After this call the pointer must not be used again.
///
/// Matches `void nexus_world_destroy(void*)`.
///
/// # Safety
/// `ptr` must be a valid, non-null pointer returned by [`nexus_world_create`]
/// that has not already been destroyed.
#[no_mangle]
pub unsafe extern "C" fn nexus_world_destroy(ptr: *mut World) {
    if !ptr.is_null() {
        drop(Box::from_raw(ptr));
    }
}

/// Spawn a new entity and return its packed 64-bit [`EntityId`].
///
/// Matches `uint64_t nexus_world_spawn(void*)`.
///
/// # Safety
/// `ptr` must be a valid, non-null pointer returned by [`nexus_world_create`].
#[no_mangle]
pub unsafe extern "C" fn nexus_world_spawn(ptr: *mut World) -> u64 {
    let world = &*ptr;
    let entity = world.spawn();
    // EntityId is a repr(transparent) u64 — cast is safe.
    entity.0
}

/// Despawn the entity identified by `entity_id`.
///
/// If the ID does not refer to a live entity (already despawned, or stale
/// generation) the call is a safe no-op.
///
/// Matches `void nexus_world_despawn(void*, uint64_t)`.
///
/// # Safety
/// `ptr` must be a valid, non-null pointer returned by [`nexus_world_create`].
#[no_mangle]
pub unsafe extern "C" fn nexus_world_despawn(ptr: *mut World, entity_id: u64) {
    use crate::entity::EntityId;
    let world = &*ptr;
    world.despawn(EntityId(entity_id));
}

/// Return the number of live entities in the world.
///
/// Matches `uint32_t nexus_world_entity_count(void*)`.
///
/// # Safety
/// `ptr` must be a valid, non-null pointer returned by [`nexus_world_create`].
#[no_mangle]
pub unsafe extern "C" fn nexus_world_entity_count(ptr: *mut World) -> u32 {
    let world = &*ptr;
    world.entity_count() as u32
}
