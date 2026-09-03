// Copyright The Domain. All Rights Reserved.
using UnrealBuildTool;
using System.Collections.Generic;

public class TheDomainTarget : TargetRules
{
    public TheDomainTarget(TargetInfo Target) : base(Target)
    {
        Type = TargetType.Game;
        DefaultBuildSettings = BuildSettingsVersion.V5;
        IncludeOrderVersion = EngineIncludeOrderVersion.Latest;
        ExtraModuleNames.Add("TheDomain");
    }
}
