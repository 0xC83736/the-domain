using UnrealBuildTool;

public class TheDomain : ModuleRules
{
    public TheDomain(ReadOnlyTargetRules Target) : base(Target)
    {
        PCHUsage = PCHUsageMode.UseExplicitOrSharedPCHs;
        PublicDependencyModuleNames.AddRange(new string[]
        {
            "Core", "CoreUObject", "Engine", "InputCore",
            "NexusCore",
            "EnhancedInput",
        });
    }
}
