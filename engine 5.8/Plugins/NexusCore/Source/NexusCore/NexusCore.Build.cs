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
        // Built by running from repo root:
        //   cargo build --workspace --release
        //
        // Directory layout:
        //   <repo_root>/                           <- C:\UEProjects\the-domain\
        //     target/release/nexus_ecs.lib
        //     engine 5.8/
        //       Plugins/
        //         NexusCore/
        //           Source/
        //             NexusCore/
        //               NexusCore.Build.cs   <- THIS FILE
        //
        // ModuleDirectory = engine 5.8\Plugins\NexusCore\Source\NexusCore
        // Levels to repo root: NexusCore(1) Source(2) NexusCore(3) Plugins(4) engine 5.8(5) repo(6) = 6
        // BUT Path.Combine("..",x6) from ModuleDirectory walks:
        //   NexusCore → Source → NexusCore → Plugins → engine 5.8 → the-domain → UEProjects
        // That is 7 steps because ModuleDirectory itself is the starting folder.
        // Confirmed by manual test: 6× ".." from NexusCore lands at C:\UEProjects (one too high).
        // Correct depth is 5× ".." to land at repo root (the-domain\).

        string RepoRoot = Path.GetFullPath(
            Path.Combine(ModuleDirectory, "..", "..", "..", "..", ".."));

        string RustRelease = Path.Combine(RepoRoot, "target", "release");

        System.Console.WriteLine("[NexusCore] ModuleDirectory : " + ModuleDirectory);
        System.Console.WriteLine("[NexusCore] Resolved RepoRoot: " + RepoRoot);
        System.Console.WriteLine("[NexusCore] Rust release dir : " + RustRelease);

        // Try candidate library paths in priority order.
        // MSVC toolchain produces nexus_ecs.lib
        // GNU toolchain produces libnexus_ecs.a (also works on Mac/Linux)
        string[] RustCrates = { "nexus_ecs", "nexus_world", "nexus_physics", "nexus_net" };

        foreach (string Crate in RustCrates)
        {
            // Candidates in order of preference for each platform
            string[] Candidates;

            if (Target.Platform == UnrealTargetPlatform.Win64)
            {
                Candidates = new string[]
                {
                    Path.Combine(RustRelease, Crate + ".lib"),           // MSVC toolchain
                    Path.Combine(RustRelease, "lib" + Crate + ".a"),     // GNU toolchain fallback
                };
            }
            else
            {
                Candidates = new string[]
                {
                    Path.Combine(RustRelease, "lib" + Crate + ".a"),
                };
            }

            bool Found = false;
            foreach (string Candidate in Candidates)
            {
                if (File.Exists(Candidate))
                {
                    PublicAdditionalLibraries.Add(Candidate);
                    System.Console.WriteLine("[NexusCore] Linking " + Crate + ": " + Candidate);
                    Found = true;
                    break;
                }
            }

            if (!Found)
            {
                System.Console.WriteLine(
                    "[NexusCore] WARNING: Could not find lib for " + Crate +
                    ". Tried: " + string.Join(", ", Candidates) +
                    ". Run `cargo build --workspace --release` from: " + RepoRoot);
            }
        }

        // On Windows, the MSVC Rust runtime needs these system libs.
        if (Target.Platform == UnrealTargetPlatform.Win64)
        {
            PublicSystemLibraries.AddRange(new string[]
            {
                "Bcrypt.lib",   // Rust crypto primitives
                "Ntdll.lib",    // Rust std thread/process APIs
                "Userenv.lib",  // Rust std env APIs
                "Ws2_32.lib",   // Rust std net APIs
            });
        }
    }
}
