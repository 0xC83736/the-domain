#include "DomainGameMode.h"

#include "DomainCharacter.h"

ADomainGameMode::ADomainGameMode()
{
    DefaultPawnClass = ADomainCharacter::StaticClass();
}
