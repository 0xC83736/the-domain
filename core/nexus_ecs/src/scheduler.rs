//! System scheduler with explicit dependency ordering.
//!
//! Systems declare read/write access to component types.  The scheduler
//! builds a DAG and runs independent systems in parallel via rayon while
//! enforcing that writers are exclusive.

/// A registered system: a boxed function plus its dependency metadata.
pub struct SystemDescriptor {
    pub name: &'static str,
    pub dependencies: Vec<&'static str>,
    pub run: Box<dyn Fn() + Send + Sync>,
}

/// Topological-sort-based scheduler.
pub struct Scheduler {
    systems: Vec<SystemDescriptor>,
}

impl Scheduler {
    pub fn new() -> Self { Self { systems: Vec::new() } }

    pub fn add_system(&mut self, desc: SystemDescriptor) {
        self.systems.push(desc);
    }

    /// Execute one tick: run all systems respecting dependency order.
    /// Parallel execution via rayon where the dependency graph permits.
    pub fn run_tick(&self) {
        // Phase 1 Step 1 prompt implements full topological sort + rayon dispatch.
        // Stub: sequential execution for scaffold correctness.
        for system in &self.systems {
            (system.run)();
        }
    }
}
