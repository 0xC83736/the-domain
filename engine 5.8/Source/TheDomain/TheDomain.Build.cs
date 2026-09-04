using UnrealBuildTool;

public class TheDomain : ModuleRules
{
    public TheDomain(ReadOnlyTargetRules Target) : base(Target)
    {
        PCHUsage = PCHUsageMode.UseExplicitOrSharedPCHs;

        PublicDependencyModuleNames.AddRange(new string[]
        {
            "Core", "CoreUObject", "Engine", "InputCore",
            "EnhancedInput",
        });

        // NexusCore is intentionally NOT listed here.
        // UGameInstanceSubsystem subclasses are discovered and instantiated
        // automatically by UE at runtime — no hard module dependency needed.
        // Adding it as a dependency caused all Blueprint Event Graphs to be
        // locked when the plugin had any stale compile state.
        // Access NexusWorldSubsystem from Blueprints via Get Subsystem node,
        // or from C++ via UGameInstance::GetSubsystem<UNexusWorldSubsystem>().
    }
}
