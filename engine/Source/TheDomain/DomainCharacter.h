#pragma once

#include "CoreMinimal.h"
#include "GameFramework/Character.h"
#include "DomainCharacter.generated.h"

class USpringArmComponent;
class UCameraComponent;
class UInputMappingContext;
class UInputAction;
struct FInputActionValue;

/**
 * ADomainCharacter
 *
 * Default third-person controllable player character: spring-arm camera,
 * standard movement, and Enhanced Input bindings for move/look/jump.
 *
 * Input setup required in the editor (Enhanced Input assets are binary
 * .uasset files and can't be authored from source):
 *   1. Content Browser -> right-click -> Input -> Input Action. Create
 *      IA_Move, IA_Look, IA_Jump, IA_ToggleCamera (IA_Jump and
 *      IA_ToggleCamera as Digital/bool, the other two as Axis2D).
 *   2. Content Browser -> right-click -> Input -> Input Mapping Context.
 *      Create IMC_Default, map WASD/left-stick to IA_Move, mouse/right-
 *      stick delta to IA_Look, Space/gamepad-face-button to IA_Jump, and
 *      any free key (e.g. C) to IA_ToggleCamera.
 *   3. Make a Blueprint child of this class (or edit Class Defaults
 *      directly) and assign DefaultMappingContext/MoveAction/LookAction/
 *      JumpAction/ToggleCameraAction to the assets created above.
 */
UCLASS(Blueprintable)
class THEDOMAIN_API ADomainCharacter : public ACharacter
{
    GENERATED_BODY()

public:
    ADomainCharacter();

protected:
    virtual void BeginPlay() override;
    virtual void SetupPlayerInputComponent(UInputComponent* PlayerInputComponent) override;

    void Move(const FInputActionValue& Value);
    void Look(const FInputActionValue& Value);

    /** Swaps the active camera between FollowCamera (third person) and FirstPersonCamera. */
    UFUNCTION(BlueprintCallable, Category = "Camera")
    void ToggleCameraView();

    UPROPERTY(VisibleAnywhere, BlueprintReadOnly, Category = "Camera")
    TObjectPtr<USpringArmComponent> CameraBoom;

    UPROPERTY(VisibleAnywhere, BlueprintReadOnly, Category = "Camera")
    TObjectPtr<UCameraComponent> FollowCamera;

    /** Attached directly to the capsule at eye height — no spring arm, so it can't clip through walls. */
    UPROPERTY(VisibleAnywhere, BlueprintReadOnly, Category = "Camera")
    TObjectPtr<UCameraComponent> FirstPersonCamera;

    UPROPERTY(VisibleAnywhere, BlueprintReadOnly, Category = "Camera")
    bool bIsFirstPerson = false;

    /** Assign the Input Mapping Context asset created in the editor (see class comment). */
    UPROPERTY(EditDefaultsOnly, BlueprintReadOnly, Category = "Input")
    TObjectPtr<UInputMappingContext> DefaultMappingContext;

    UPROPERTY(EditDefaultsOnly, BlueprintReadOnly, Category = "Input")
    TObjectPtr<UInputAction> MoveAction;

    UPROPERTY(EditDefaultsOnly, BlueprintReadOnly, Category = "Input")
    TObjectPtr<UInputAction> LookAction;

    UPROPERTY(EditDefaultsOnly, BlueprintReadOnly, Category = "Input")
    TObjectPtr<UInputAction> JumpAction;

    UPROPERTY(EditDefaultsOnly, BlueprintReadOnly, Category = "Input")
    TObjectPtr<UInputAction> ToggleCameraAction;
};
