# ADR-003: Hybrid world model — Nanite static + SVO sandbox

**Status:** Accepted  
**Date:** 2026-09-03

## Decision
Adopt a **hybrid world architecture**:
- Main world terrain and structures: authored Nanite static meshes
- Player-buildable sandbox zones: SVO voxels meshed with marching cubes

## Rationale
A fully destructible voxel world (original plan) is architecturally incompatible with Lumen pre-caching, Nanite virtualized geometry, and curved organic terrain. Restricting voxel editability to designated sandbox zones preserves the creative platform ambition while unlocking the full UE5 rendering stack for the main world.

## Marching cubes over greedy meshing
Marching cubes produces smooth, curved terrain surfaces. The ~3× CPU cost vs greedy meshing is acceptable because:
1. Sandbox chunks are smaller (64³ vs 256²)
2. The RTX 2060 CPU budget (Ryzen 5 3600) is larger than the original Ryzen 5 3600 was designed for at GTX 960 frame times
3. Meshing runs on worker threads, never the game thread

## Consequences
- Phase 2 world model has two distinct code paths; WorldZoneRegistry enforces the boundary
- Art pipeline requires Houdini PDG for procedural authored zone placement
- Marching cubes chunk budget: p95 < 24ms per 64³ chunk
- Sandbox zones cannot use Nanite (dynamic geometry incompatible)
