# ADR-002: Graphics fidelity tier and min spec

**Status:** Accepted  
**Date:** 2026-09-03

## Decision
Raise the minimum hardware spec from GTX 960 to **RTX 2060 / Ryzen 5 3600 / 16GB RAM**.

## Fidelity tier mapping

| Preset   | GPU          | RT scope                        | Lumen mode         | Upscaling        |
|----------|--------------|---------------------------------|--------------------|------------------|
| Min spec | RTX 2060     | Reflections + contact shadows   | Software GI        | DLSS Quality     |
| High     | RTX 3070+    | Full hardware Lumen reflections | Hardware Lumen GI  | DLSS Quality     |
| Ultra    | RTX 4080+    | Full path tracing               | Path Tracing       | DLSS Performance |

## Rationale
RTX 2060 is the lowest NVIDIA card with DXR support sufficient for selective RT features. Raising the floor unlocks hardware RT reflections (the single highest visual-impact feature per GPU cost), DLSS 3 frame generation, and hardware Lumen reflections on mid-range hardware. The player base sacrifice is accepted in exchange for the fidelity uplift.

## Consequences
- CI benchmark profile updated to RTX 2060 emulation
- VRAM budget raised to 6GB (min spec), 8GB (high), 12GB (ultra)
- GTX 900-series players cannot run the game — this is a deliberate trade-off
