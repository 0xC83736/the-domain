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
        // Required in UE 5.8: V5 build settings override warning levels that
        // conflict with the shared UnrealEditor build environment. This flag
        // permits those overrides without requiring a unique build environment.
        bOverrideBuildEnvironment = true;
        ExtraModuleNames.Add("TheDomain");
    }
}
