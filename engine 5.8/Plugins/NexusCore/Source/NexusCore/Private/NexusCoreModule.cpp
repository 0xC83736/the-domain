#include "NexusCoreModule.h"
#include "NexusWorldSubsystem.h"
#include "Modules/ModuleManager.h"

#define LOCTEXT_NAMESPACE "FNexusCoreModule"

void FNexusCoreModule::StartupModule()
{
    UE_LOG(LogTemp, Log, TEXT("NexusCore: Rust FFI bridge initialised."));
}

void FNexusCoreModule::ShutdownModule()
{
    UE_LOG(LogTemp, Log, TEXT("NexusCore: Rust FFI bridge shut down."));
}

#undef LOCTEXT_NAMESPACE

IMPLEMENT_MODULE(FNexusCoreModule, NexusCore)
