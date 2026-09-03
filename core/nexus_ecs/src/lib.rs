//! # nexus_ecs
//!
//! Archetype-based Entity Component System for The Domain.
//!
//! ## Design principles
//! - Data-oriented: components packed contiguously by archetype for cache efficiency
//! - Thread-safe reads: multiple systems may query concurrently; writes are exclusive
//! - No runtime reflection: component types resolved at compile time via generics
//! - Generation counters on entity IDs prevent ABA aliasing after despawn

pub mod archetype;
pub mod entity;
pub mod query;
pub mod scheduler;
pub mod world;

pub use entity::{Entity, EntityId};
pub use world::World;

/// Re-export the query derive macro (implementation pending Phase 1 Step 1 prompt).
pub use query::Query;
