using UnrealBuildTool;
using System.IO;

public class NexusCore : ModuleRules
{
    public NexusCore(ReadOnlyTargetRules Target) : base(Target)
    {
        PCHUsage = ModuleRules.PCHUsageMode.UseExplicitOrSharedPCHs;

        PublicIncludePaths.AddRange(new string[] { });
        PrivateIncludePaths.AddRange(new string[] { });

        PublicDependencyModuleNames.AddRange(new string[]
        {
            "Core",
            "CoreUObject",
            "Engine",
        });

        PrivateDependencyModuleNames.AddRange(new string[]
        {
            "Projects",
            "InputCore",
            "Slate",
            "SlateCore",
        });

        // Rust FFI library path — built via cargo in CI before UE compilation
        string RustLibPath = Path.Combine(ModuleDirectory, "../../../../core/target/release");
        PublicAdditionalLibraries.Add(Path.Combine(RustLibPath, "libnexus_ecs.a"));
        PublicAdditionalLibraries.Add(Path.Combine(RustLibPath, "libnexus_world.a"));
        PublicAdditionalLibraries.Add(Path.Combine(RustLibPath, "libnexus_physics.a"));
        PublicAdditionalLibraries.Add(Path.Combine(RustLibPath, "libnexus_net.a"));
    }
}
