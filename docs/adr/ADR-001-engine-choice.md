# ADR-001: Engine — Unreal Engine 5.4

**Status:** Accepted  
**Date:** 2026-09-03  
**Deciders:** Core team

## Context
Project NEXUS requires photorealistic rendering at Cyberpunk-adjacent fidelity levels on an RTX 2060 minimum spec. The previous plan targeted Godot 4 with a GTX 960 floor; the enhanced fidelity tier revision raised both the visual target and the hardware floor.

## Decision
Use **Unreal Engine 5.4** as the primary engine.

## Rationale
- **Lumen** provides dynamic global illumination without pre-baked lightmaps — essential for a destructible, player-modifiable world
- **Nanite** eliminates manual LOD authoring for the static main world
- **DLSS 3 / FSR 3** built-in upscaler interface allows 60fps at native-equivalent quality on RTX 2060
- **Hardware RT** (reflections + contact shadows) available from RTX 2060 upward
- Years of engine R&D that would otherwise need to be built in-house

## Trade-offs
- 5% royalty above $1M revenue
- Less control over engine internals vs a forked open-source engine
- C++ build times longer than GDScript iteration cycles

## Consequences
- All rendering architecture in Phase 1 targets UE5 subsystems
- Rust hot-path code bridges via FFI to a UE5 plugin (NexusCore)
- WebGPU browser client deferred to Phase 5 (UE5 web export still maturing)
