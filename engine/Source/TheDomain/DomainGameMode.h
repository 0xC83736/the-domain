#pragma once

#include "CoreMinimal.h"
#include "GameFramework/GameModeBase.h"
#include "DomainGameMode.generated.h"

/** Default game mode: possesses players with ADomainCharacter on spawn. */
UCLASS(Blueprintable)
class THEDOMAIN_API ADomainGameMode : public AGameModeBase
{
    GENERATED_BODY()

public:
    ADomainGameMode();
};
