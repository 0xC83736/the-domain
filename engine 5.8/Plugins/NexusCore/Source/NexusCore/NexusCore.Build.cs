// Copyright The Domain. All Rights Reserved.
using UnrealBuildTool;
using System.IO;

public class NexusCore : ModuleRules
{
    public NexusCore(ReadOnlyTargetRules Target) : base(Target)
    {
        PCHUsage = ModuleRules.PCHUsageMode.UseExplicitOrSharedPCHs;

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

        // ── Rust FFI static libraries ──────────────────────────────────────
        // Run `cargo build --workspace --release` from repo root before building.
        // Plugin opens and generates project files without them (graceful skip).

        // Repo root is 5 levels up from this file:
        // TheDomain/Plugins/NexusCore/Source/NexusCore/
        string RepoRoot = Path.GetFullPath(
            Path.Combine(ModuleDirectory, "..", "..", "..", "..", ".."));

        string RustTarget = Path.Combine(RepoRoot, "core", "target");

        string LibDir;
        string LibPrefix;
        string LibExt;

        if (Target.Platform == UnrealTargetPlatform.Win64)
        {
            LibDir    = Path.Combine(RustTarget, "x86_64-pc-windows-msvc", "release");
            LibPrefix = "";
            LibExt    = ".lib";
        }
        else if (Target.Platform == UnrealTargetPlatform.Mac)
        {
            LibDir    = Path.Combine(RustTarget, "release");
            LibPrefix = "lib";
            LibExt    = ".a";
        }
        else
        {
            LibDir    = Path.Combine(RustTarget, "release");
            LibPrefix = "lib";
            LibExt    = ".a";
        }

        string[] RustCrates = { "nexus_ecs", "nexus_world", "nexus_physics", "nexus_net" };
        foreach (string Crate in RustCrates)
        {
            string LibPath = Path.Combine(LibDir, LibPrefix + Crate + LibExt);
            if (File.Exists(LibPath))
            {
                PublicAdditionalLibraries.Add(LibPath);
                System.Console.WriteLine("[NexusCore] Linking: " + LibPath);
            }
            else
            {
                System.Console.WriteLine(
                    "[NexusCore] WARNING: Missing Rust lib (run cargo build --workspace --release): "
                    + LibPath);
            }
        }
    }
}
