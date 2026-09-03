#pragma once

#include "CoreMinimal.h"
#include "Modules/ModuleManager.h"

/**
 * NexusCore — UE5 module that hosts the Rust FFI bridge.
 *
 * Loaded at game startup. Initialises the Rust allocator and exposes
 * the UNexusWorldSubsystem which downstream systems use to interact
 * with the ECS world.
 */
class FNexusCoreModule : public IModuleInterface
{
public:
    virtual void StartupModule() override;
    virtual void ShutdownModule() override;
};
