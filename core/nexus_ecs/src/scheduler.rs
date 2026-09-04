//! System scheduler with topological ordering and rayon parallel dispatch.
//!
//! ## Model
//! Each system declares which other systems it depends on by name.  The
//! scheduler performs a topological sort (Kahn's algorithm) to derive a
//! sequence of *stages* — groups of systems with no mutual dependency.
//! Systems within the same stage are dispatched in parallel via rayon;
//! stages execute sequentially.
//!
//! ## Example
//! ```rust
//! use nexus_ecs::scheduler::{Scheduler, SystemDescriptor};
//!
//! let mut sched = Scheduler::new();
//! sched.add_system(SystemDescriptor {
//!     name: "physics",
//!     dependencies: vec![],
//!     run: Box::new(|| println!("physics tick")),
//! });
//! sched.add_system(SystemDescriptor {
//!     name: "render",
//!     dependencies: vec!["physics"],
//!     run: Box::new(|| println!("render tick")),
//! });
//! sched.build().expect("no cycles");
//! sched.run_tick();
//! ```

use std::collections::HashMap;

/// A registered system.
pub struct SystemDescriptor {
    /// Unique name used for dependency resolution.
    pub name: &'static str,
    /// Names of systems that must complete before this one runs.
    pub dependencies: Vec<&'static str>,
    /// The system function.  Must be `Send + Sync` for rayon dispatch.
    pub run: Box<dyn Fn() + Send + Sync>,
}

/// Error returned when the dependency graph contains a cycle.
#[derive(Debug)]
pub struct CycleError(pub Vec<&'static str>);

impl std::fmt::Display for CycleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "scheduler dependency cycle involving: {:?}", self.0)
    }
}

/// Topological-sort-based parallel scheduler.
pub struct Scheduler {
    systems: Vec<SystemDescriptor>,
    /// Stages computed by [`build`].  Each stage is a list of indices into
    /// `systems` that can run concurrently.
    stages: Option<Vec<Vec<usize>>>,
}

impl Scheduler {
    pub fn new() -> Self {
        Self { systems: Vec::new(), stages: None }
    }

    /// Register a system.  Must call [`build`] after all systems are added.
    pub fn add_system(&mut self, desc: SystemDescriptor) {
        self.stages = None; // invalidate compiled stages
        self.systems.push(desc);
    }

    /// Compile the dependency graph into parallel stages (Kahn's algorithm).
    ///
    /// Returns `Err` if a cycle is detected.  Must be called before
    /// [`run_tick`].
    pub fn build(&mut self) -> Result<(), CycleError> {
        let n = self.systems.len();
        let name_to_idx: HashMap<&str, usize> = self.systems
            .iter()
            .enumerate()
            .map(|(i, s)| (s.name, i))
            .collect();

        // Build adjacency: dep_count[i] = number of systems i waits for.
        // dependents[i] = systems that wait for i.
        let mut dep_count = vec![0usize; n];
        let mut dependents: Vec<Vec<usize>> = vec![Vec::new(); n];

        for (i, sys) in self.systems.iter().enumerate() {
            for &dep_name in &sys.dependencies {
                let j = name_to_idx.get(dep_name).copied().unwrap_or_else(|| {
                    panic!("system '{}' depends on unknown system '{}'", sys.name, dep_name)
                });
                dep_count[i] += 1;
                dependents[j].push(i);
            }
        }

        // Kahn's BFS — build stages level by level.
        let mut stages: Vec<Vec<usize>> = Vec::new();
        let mut ready: Vec<usize> = dep_count
            .iter()
            .enumerate()
            .filter(|(_, &c)| c == 0)
            .map(|(i, _)| i)
            .collect();

        let mut processed = 0;

        while !ready.is_empty() {
            stages.push(ready.clone());
            processed += ready.len();
            let mut next_ready = Vec::new();
            for idx in &ready {
                for &dep in &dependents[*idx] {
                    dep_count[dep] -= 1;
                    if dep_count[dep] == 0 {
                        next_ready.push(dep);
                    }
                }
            }
            ready = next_ready;
        }

        if processed < n {
            // Some nodes were never drained — cycle exists.
            let cycle_nodes: Vec<&'static str> = dep_count
                .iter()
                .enumerate()
                .filter(|(_, &c)| c > 0)
                .map(|(i, _)| self.systems[i].name)
                .collect();
            return Err(CycleError(cycle_nodes));
        }

        self.stages = Some(stages);
        Ok(())
    }

    /// Execute one tick.
    ///
    /// Systems within each stage run in parallel via rayon.
    /// Stages themselves run sequentially (each stage waits for the prior).
    ///
    /// # Panics
    /// Panics if [`build`] has not been called successfully.
    pub fn run_tick(&self) {
        let stages = self.stages.as_ref()
            .expect("Scheduler::build() must be called before run_tick()");

        for stage in stages {
            if stage.len() == 1 {
                // Avoid rayon overhead for single-system stages.
                (self.systems[stage[0]].run)();
            } else {
                use rayon::prelude::*;
                stage.par_iter().for_each(|&idx| {
                    (self.systems[idx].run)();
                });
            }
        }
    }

    /// Number of stages compiled (for testing/debugging).
    pub fn stage_count(&self) -> usize {
        self.stages.as_ref().map_or(0, |s| s.len())
    }
}

impl Default for Scheduler {
    fn default() -> Self { Self::new() }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    fn counter_system(log: Arc<Mutex<Vec<&'static str>>>, name: &'static str)
        -> Box<dyn Fn() + Send + Sync>
    {
        Box::new(move || log.lock().unwrap().push(name))
    }

    #[test]
    fn single_system_runs() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let mut sched = Scheduler::new();
        sched.add_system(SystemDescriptor {
            name: "a",
            dependencies: vec![],
            run: counter_system(log.clone(), "a"),
        });
        sched.build().unwrap();
        sched.run_tick();
        assert_eq!(*log.lock().unwrap(), vec!["a"]);
    }

    #[test]
    fn dependency_respected_ordering() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let mut sched = Scheduler::new();
        // b depends on a — a must run first
        sched.add_system(SystemDescriptor {
            name: "b",
            dependencies: vec!["a"],
            run: counter_system(log.clone(), "b"),
        });
        sched.add_system(SystemDescriptor {
            name: "a",
            dependencies: vec![],
            run: counter_system(log.clone(), "a"),
        });
        sched.build().unwrap();
        assert_eq!(sched.stage_count(), 2);
        sched.run_tick();
        let result = log.lock().unwrap().clone();
        // a must appear before b
        let a_pos = result.iter().position(|&x| x == "a").unwrap();
        let b_pos = result.iter().position(|&x| x == "b").unwrap();
        assert!(a_pos < b_pos, "a must run before b, got: {:?}", result);
    }

    #[test]
    fn independent_systems_same_stage() {
        let mut sched = Scheduler::new();
        for name in ["x", "y", "z"] {
            sched.add_system(SystemDescriptor {
                name,
                dependencies: vec![],
                run: Box::new(|| {}),
            });
        }
        sched.build().unwrap();
        // All independent — should collapse into one stage.
        assert_eq!(sched.stage_count(), 1);
    }

    #[test]
    fn diamond_dependency() {
        // a → b, a → c, b → d, c → d
        let mut sched = Scheduler::new();
        for (name, deps) in [
            ("a", vec![]),
            ("b", vec!["a"]),
            ("c", vec!["a"]),
            ("d", vec!["b", "c"]),
        ] {
            sched.add_system(SystemDescriptor {
                name,
                dependencies: deps,
                run: Box::new(|| {}),
            });
        }
        sched.build().unwrap();
        // stage 0: [a], stage 1: [b, c], stage 2: [d]
        assert_eq!(sched.stage_count(), 3);
    }

    #[test]
    fn cycle_returns_error() {
        let mut sched = Scheduler::new();
        sched.add_system(SystemDescriptor {
            name: "p", dependencies: vec!["q"], run: Box::new(|| {}),
        });
        sched.add_system(SystemDescriptor {
            name: "q", dependencies: vec!["p"], run: Box::new(|| {}),
        });
        assert!(sched.build().is_err());
    }

    #[test]
    fn run_tick_panics_without_build() {
        let result = std::panic::catch_unwind(|| {
            let sched = Scheduler::new();
            sched.run_tick();
        });
        assert!(result.is_err());
    }
}
