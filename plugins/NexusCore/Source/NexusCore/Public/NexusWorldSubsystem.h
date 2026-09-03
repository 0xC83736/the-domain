#pragma once

#include "CoreMinimal.h"
#include "Subsystems/GameInstanceSubsystem.h"
#include "NexusWorldSubsystem.generated.h"

/**
 * UNexusWorldSubsystem
 *
 * Game-instance-scoped subsystem that owns the Rust ECS World.
 * All Blueprint and C++ code interacts with the ECS through this subsystem.
 *
 * Thread model: Rust world runs tick on a dedicated worker thread.
 * UE game thread reads a double-buffered snapshot each frame.
 * Writes (spawn/despawn/component mutation) are queued and applied
 * at the start of the next Rust tick.
 */
UCLASS()
class NEXUSCORE_API UNexusWorldSubsystem : public UGameInstanceSubsystem
{
    GENERATED_BODY()

public:
    virtual void Initialize(FSubsystemCollectionBase& Collection) override;
    virtual void Deinitialize() override;

    /** Spawn a new entity. Returns its 64-bit EntityId. */
    UFUNCTION(BlueprintCallable, Category = "Nexus|ECS")
    int64 SpawnEntity();

    /** Despawn an entity by ID. */
    UFUNCTION(BlueprintCallable, Category = "Nexus|ECS")
    void DespawnEntity(int64 EntityId);

    /** Returns the number of live entities in the current world. */
    UFUNCTION(BlueprintPure, Category = "Nexus|ECS")
    int32 GetEntityCount() const;

private:
    /** Opaque pointer to the Rust World allocated by nexus_ecs. */
    void* RustWorldPtr = nullptr;
};
