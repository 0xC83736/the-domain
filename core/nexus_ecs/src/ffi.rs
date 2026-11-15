//! C ABI bridge consumed by the NexusCore UE5 plugin
//! (`plugins/NexusCore/Source/NexusCore/Private/NexusWorldSubsystem.cpp`).
//!
//! The UE5 side holds `World` only behind an opaque pointer; it never
//! inspects its layout, so `World` itself needs no `#[repr(C)]`.

use crate::entity::EntityId;
use crate::world::World;

#[no_mangle]
pub extern "C" fn nexus_world_create() -> *mut World {
    Box::into_raw(Box::new(World::new()))
}

/// # Safety
/// `world` must be a pointer previously returned by [`nexus_world_create`]
/// and not already passed to this function.
#[no_mangle]
pub unsafe extern "C" fn nexus_world_destroy(world: *mut World) {
    if !world.is_null() {
        drop(Box::from_raw(world));
    }
}

/// # Safety
/// `world` must be a valid, non-null pointer returned by [`nexus_world_create`].
#[no_mangle]
pub unsafe extern "C" fn nexus_world_spawn(world: *mut World) -> u64 {
    (*world).spawn().to_bits()
}

/// # Safety
/// `world` must be a valid, non-null pointer returned by [`nexus_world_create`].
#[no_mangle]
pub unsafe extern "C" fn nexus_world_despawn(world: *mut World, entity_id: u64) {
    (*world).despawn(EntityId::from_bits(entity_id));
}

/// # Safety
/// `world` must be a valid, non-null pointer returned by [`nexus_world_create`].
#[no_mangle]
pub unsafe extern "C" fn nexus_world_entity_count(world: *mut World) -> u32 {
    (*world).entity_count() as u32
}
