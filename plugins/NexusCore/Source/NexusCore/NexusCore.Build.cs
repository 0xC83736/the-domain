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

        // Rust FFI library path — built via cargo before UE compilation (see
        // tools/build.bat). rustc's MSVC target emits "<name>.lib" with no
        // "lib" prefix; its GNU/Unix targets emit "lib<name>.a".
        string RustLibPath = Path.Combine(ModuleDirectory, "../../../../core/target/release");
        string[] RustCrates = { "nexus_ecs", "nexus_world", "nexus_physics", "nexus_net" };
        foreach (string Crate in RustCrates)
        {
            string FileName = Target.Platform == UnrealTargetPlatform.Win64
                ? Crate + ".lib"
                : "lib" + Crate + ".a";
            PublicAdditionalLibraries.Add(Path.Combine(RustLibPath, FileName));
        }
    }
}
