# The Domain

**An Oasis-scale open world sandbox — Phase 1: Core engine foundation**

> Enhanced fidelity tier · UE5 5.8 · RTX 2060 min spec · Rust hot paths via FFI

---

## Repository structure

```
the-domain/
├── core/                   Rust workspace
│   ├── nexus_ecs/          Archetype-based ECS world
│   ├── nexus_world/        SVO + marching cubes (sandbox zones)
│   ├── nexus_physics/      Server-authoritative physics
│   ├── nexus_net/          QUIC transport layer
│   ├── nexus_worldgen/     Biome atlas + procedural generation
│   └── nexus_assets/       Async asset streaming manager
├── plugins/
│   └── NexusCore/          UE5 plugin — Rust FFI bridge
├── TheDomain/              UE5 project (TheDomain.uproject)
│   ├── Config/             Scalability tiers, engine config
│   ├── Content/            Game assets (Nanite meshes, textures)
│   └── Source/TheDomain/   Game C++ module
├── tools/
│   ├── benchmark/          ECS and frame time CI assertion scripts
│   ├── asset_validation/   Nanite + texture compliance checker
│   └── ci/                 DLSS SSIM quality gate
├── tests/
│   ├── phase1/             Phase 1 regression suite
│   └── integration/        Cross-system integration tests
└── docs/adr/               Architecture decision records
```

## Prerequisites

| Tool              | Version   | Purpose                        |
|-------------------|-----------|--------------------------------|
| Rust              | stable    | Core crate compilation         |
| cargo             | latest    | Workspace build + bench        |
| Unreal Engine     | 5.8       | Game engine (install separately)|
| Python            | 3.12+     | CI tooling                     |
| critcmp           | latest    | Benchmark regression comparison |

## Getting started

```bash
# 1. Clone
git clone https://github.com/the-domain/the-domain.git
cd the-domain

# 2. Build and test the Rust workspace
cargo test --workspace --release

# 3. Run ECS benchmarks
cargo bench --bench ecs_bench

# 4. Validate assets (empty in Phase 1 — exits 0)
python tools/asset_validation/validate.py \
    --assert-nanite \
    --assert-texture-compression \
    --asset-root TheDomain/Content

# 5. Open UE5 project (requires UE5 5.8 installed)
# TheDomain/TheDomain.uproject → right-click → Generate project files → open .sln
```

## Architecture decisions

See [docs/adr/](docs/adr/) for full rationale on:
- ADR-001: Engine choice (UE5 over Godot 4)
- ADR-002: Graphics fidelity tier and min spec
- ADR-003: Hybrid world model (Nanite static + SVO sandbox)

## CI

Every PR runs:
1. `cargo fmt --check` + `cargo clippy -D warnings`
2. `cargo test --workspace --release`
3. Benchmark regression gate via `critcmp` (>10% regression = block)
4. Asset validation (Nanite + texture compliance)
5. ECS performance assertion (spawn 1M < 50ms, query 100k < 2ms)

See [.github/workflows/ci.yml](.github/workflows/ci.yml).

## Phase roadmap

| Phase | Scope                        | Status      |
|-------|------------------------------|-------------|
| 1     | Core engine foundation       | In progress |
| 2     | Hybrid world engine          | Planned     |
| 3     | Multiplayer infrastructure   | Planned     |
| 4     | Economy and creator layer    | Planned     |
| 5     | Global scale and live ops    | Planned     |
