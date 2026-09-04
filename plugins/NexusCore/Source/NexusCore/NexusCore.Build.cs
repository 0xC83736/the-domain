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
        // Built by: cargo build --workspace --release  (run from repo root)
        // Output lands in: <repo_root>/target/release/  (workspace-level target)
        //
        // Plugin sits at:
        //   TheDomain/Plugins/NexusCore/Source/NexusCore/   (4 levels up = TheDomain/)
        //   plugins/NexusCore/Source/NexusCore/             (4 levels up = repo root)
        // Repo root is one more level up from TheDomain/:

        string PluginDir = Path.GetFullPath(Path.Combine(ModuleDirectory, "..", "..", "..", ".."));
        // PluginDir is now either TheDomain/ or plugins/ depending on which copy UE loaded.
        // Repo root is the parent of whichever folder we're in.
        string RepoRoot  = Path.GetFullPath(Path.Combine(PluginDir, ".."));

        // Cargo workspace target — always at repo root, never inside core/
        string RustTarget = Path.Combine(RepoRoot, "target");

        string LibDir;
        string LibPrefix;
        string LibExt;

        if (Target.Platform == UnrealTargetPlatform.Win64)
        {
            // MSVC toolchain: cargo build produces .lib directly in release/
            LibDir    = Path.Combine(RustTarget, "release");
            LibPrefix = "";
            LibExt    = ".lib";
        }
        else if (Target.Platform == UnrealTargetPlatform.Mac)
        {
            LibDir    = Path.Combine(RustTarget, "release");
            LibPrefix = "lib";
            LibExt    = ".a";
        }
        else // Linux
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
                    "[NexusCore] WARNING: Rust lib not found — run `cargo build --workspace --release` from repo root: "
                    + LibPath);
            }
        }
    }
}
