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
        // build.bat). The Cargo workspace root is the repo root (one level
        // above plugins/), not core/, so cargo's output lands in
        // <repo-root>/target/release, not <repo-root>/core/target/release.
        // rustc's MSVC target emits "<name>.lib" with no "lib" prefix; its
        // GNU/Unix targets emit "lib<name>.a". UBT only resolves
        // PublicAdditionalLibraries entries as actual files when they're
        // fully qualified (no ".." segments) — GetFullPath collapses the
        // relative traversal so it doesn't fall back to a slow library-path
        // search.
        string RustLibPath = Path.GetFullPath(Path.Combine(ModuleDirectory, "../../../../target/release"));
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
