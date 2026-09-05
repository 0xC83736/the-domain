// Copyright The Domain. All Rights Reserved.
using UnrealBuildTool;
using System.Collections.Generic;

public class TheDomainEditorTarget : TargetRules
{
    public TheDomainEditorTarget(TargetInfo Target) : base(Target)
    {
        Type = TargetType.Editor;
        DefaultBuildSettings = BuildSettingsVersion.V5;
        IncludeOrderVersion = EngineIncludeOrderVersion.Latest;
        // Unique build environment ensures the game module compiles
        // independently from UnrealEditor, which is required for Blueprint
        // graphs to be editable. Without this, UE may lock Blueprint Event
        // Graphs with "Graph is not editable" even when there are no errors.
        BuildEnvironment = TargetBuildEnvironment.Unique;
        ExtraModuleNames.Add("TheDomain");
    }
}
