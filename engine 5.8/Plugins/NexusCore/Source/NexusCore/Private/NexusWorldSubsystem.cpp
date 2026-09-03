#include "NexusWorldSubsystem.h"

// Forward declarations of Rust FFI functions.
// Implementations live in nexus_ecs (compiled to static lib by cargo).
extern "C"
{
    void* nexus_world_create();
    void  nexus_world_destroy(void* world);
    uint64_t nexus_world_spawn(void* world);
    void  nexus_world_despawn(void* world, uint64_t entity_id);
    uint32_t nexus_world_entity_count(void* world);
}

void UNexusWorldSubsystem::Initialize(FSubsystemCollectionBase& Collection)
{
    Super::Initialize(Collection);
    RustWorldPtr = nexus_world_create();
    UE_LOG(LogTemp, Log, TEXT("NexusWorldSubsystem: Rust world created @ %p"), RustWorldPtr);
}

void UNexusWorldSubsystem::Deinitialize()
{
    if (RustWorldPtr)
    {
        nexus_world_destroy(RustWorldPtr);
        RustWorldPtr = nullptr;
    }
    Super::Deinitialize();
}

int64 UNexusWorldSubsystem::SpawnEntity()
{
    return static_cast<int64>(nexus_world_spawn(RustWorldPtr));
}

void UNexusWorldSubsystem::DespawnEntity(int64 EntityId)
{
    nexus_world_despawn(RustWorldPtr, static_cast<uint64_t>(EntityId));
}

int32 UNexusWorldSubsystem::GetEntityCount() const
{
    return static_cast<int32>(nexus_world_entity_count(RustWorldPtr));
}
