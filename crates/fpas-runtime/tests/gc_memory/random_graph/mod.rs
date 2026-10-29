//! Seeded graph/root state machines checked against an independent reachability oracle.

mod values;

use crate::measurement::completed_threads;
use crate::workloads::{function, image, schedule};
use fpas_runtime::{
    CollectionLimits, CollectionSchedule, DomainMaintenance, DomainPhase, DomainSession,
    MaintenanceOutcome, MaintenancePolicy, MemoryDomain, OwnershipError, RootKind, RootValues,
    Value, WeakOwnedCell,
};
use rand::{RngExt, SeedableRng};
use rand_chacha::ChaCha8Rng;
use std::cell::Cell;
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

const STEPS: usize = 96;
const KINDS: [RootKind; 7] = [
    RootKind::Execution,
    RootKind::Host,
    RootKind::Inspection,
    RootKind::Evaluation,
    RootKind::Native,
    RootKind::Globals,
    RootKind::Selection,
];

struct Collector(AtomicUsize);

impl DomainMaintenance for Collector {
    fn maintain(&self, domain: &MemoryDomain, roots: &[Value]) -> Result<(), OwnershipError> {
        let limits = match self.0.load(Ordering::Relaxed) {
            1 => CollectionLimits {
                max_cells: 0,
                max_graph_nodes: usize::MAX,
            },
            2 => CollectionLimits {
                max_cells: usize::MAX,
                max_graph_nodes: 1,
            },
            _ => CollectionLimits::default(),
        };
        domain.reclaim_unreachable(roots, limits).map(|_| ())
    }
}

/// Prints a replay command even when a lower-level assertion fails.
struct Replay {
    seed: u64,
    step: Cell<usize>,
}

impl Drop for Replay {
    fn drop(&mut self) {
        if std::thread::panicking() {
            eprintln!(
                "graph seed={}, step={}; replay: FPAS_GC_GRAPH_SEED={} cargo test -p fpas-runtime --test gc_memory seeded_graph_roots_and_mutations_match_the_reference_model -- --nocapture",
                self.seed,
                self.step.get(),
                self.seed
            );
        }
    }
}

struct Node {
    weak: WeakOwnedCell,
    edges: Vec<usize>,
    layers: Vec<u8>,
    revision: usize,
}

struct Roots {
    kind: RootKind,
    members: Vec<usize>,
    aliases: Vec<RootValues>,
}

struct Graph<'a> {
    domain: MemoryDomain,
    session: Option<DomainSession>,
    collector: Arc<Collector>,
    nodes: Vec<Node>,
    roots: Vec<Roots>,
    shapes: &'a values::Shapes,
    rng: ChaCha8Rng,
    failed_passes: usize,
}

impl<'a> Graph<'a> {
    fn new(seed: u64, config: CollectionSchedule, shapes: &'a values::Shapes) -> Self {
        let (domain, session) = MemoryDomain::new_session();
        let collector = Arc::new(Collector(AtomicUsize::new(0)));
        domain.set_maintenance(collector.clone(), MaintenancePolicy::Scheduled(config));
        let access = domain.access();
        let mut graph = Self {
            domain: domain.clone(),
            session: Some(session),
            collector,
            nodes: Vec::new(),
            roots: Vec::new(),
            shapes,
            rng: ChaCha8Rng::seed_from_u64(seed),
            failed_passes: 0,
        };
        let cells = (0..24)
            .map(|_| domain.new_cell(Value::Unit).unwrap())
            .collect::<Vec<_>>();
        for (id, cell) in cells.iter().enumerate() {
            let base = id / 12 * 12;
            let mut edges = vec![base + (id + 1) % 12];
            edges.extend(
                (0..graph.rng.random_range(0..4)).map(|_| base + graph.rng.random_range(0..12)),
            );
            let mut layers = vec![(id % 10) as u8];
            layers.extend((0..graph.rng.random_range(0..4)).map(|_| graph.rng.random_range(0..10)));
            graph.nodes.push(Node {
                weak: cell.downgrade(),
                edges,
                layers,
                revision: 0,
            });
        }
        for (id, cell) in cells.iter().enumerate() {
            cell.replace(&domain, shapes.payload(id, &graph.nodes[id], &graph.nodes))
                .unwrap();
        }
        // One rooted ring and a separate unrooted ring guarantee both sides of
        // the oracle are exercised before any random transition.
        for kind in KINDS {
            let id = graph.rng.random_range(0..12);
            graph.add_roots(kind, vec![id, id]);
        }
        drop(cells);
        drop(access);
        graph
    }

    /// This oracle sees no runtime Values and calls no production graph walker.
    fn reachable(&self) -> Vec<bool> {
        let mut live = vec![false; self.nodes.len()];
        let mut pending = self
            .roots
            .iter()
            .flat_map(|root| root.members.iter().copied())
            .collect::<Vec<_>>();
        while let Some(id) = pending.pop() {
            if !live[id] {
                live[id] = true;
                pending.extend_from_slice(&self.nodes[id].edges);
            }
        }
        live
    }

    fn root_values(&mut self, members: &[usize]) -> Vec<Value> {
        members
            .iter()
            .map(|id| self.shapes.root(*id, &self.nodes, self.rng.random()))
            .collect()
    }

    fn add_roots(&mut self, kind: RootKind, members: Vec<usize>) {
        let values = self.root_values(&members);
        let root = self.domain.root_values(kind, values);
        self.roots.push(Roots {
            kind,
            members,
            aliases: vec![root],
        });
    }

    fn birth(&mut self, rooted: bool) {
        let id = self.nodes.len();
        let cell = self.domain.new_cell(Value::Unit).unwrap();
        self.nodes.push(Node {
            weak: cell.downgrade(),
            edges: vec![id],
            layers: vec![(id % 10) as u8, self.rng.random_range(0..10)],
            revision: 0,
        });
        cell.replace(
            &self.domain,
            self.shapes.payload(id, &self.nodes[id], &self.nodes),
        )
        .unwrap();
        if rooted {
            self.add_roots(KINDS[id % KINDS.len()], vec![id]);
        }
    }

    fn check_payloads(&self, present: &[bool]) {
        for (id, expected) in present.iter().enumerate() {
            assert_eq!(
                self.nodes[id].weak.upgrade().is_some(),
                *expected,
                "node {id}"
            );
            if *expected {
                values::check(id, &self.nodes[id], &self.nodes, &self.domain);
            }
        }
    }

    fn check(&self) {
        assert!(matches!(
            self.domain.request_explicit_maintenance(),
            MaintenanceOutcome::Serviced(_)
        ));
        let live = self.reachable();
        self.check_payloads(&live);
        for root in &self.roots {
            for alias in &root.aliases {
                alias.with_values(|values| {
                    assert_eq!(values.len(), root.members.len());
                    for (value, id) in values.iter().zip(&root.members) {
                        values::check_root(value, *id, &self.nodes);
                    }
                });
            }
        }
        let stats = self.domain.collection_stats();
        let count = live.iter().filter(|live| **live).count();
        assert_eq!(stats.live_cells, count);
        assert_eq!(stats.allocated_cells - stats.released_cells, count);
        assert_eq!(
            self.domain.root_count(),
            self.roots
                .iter()
                .map(|roots| roots.aliases.len())
                .sum::<usize>()
        );
        let mut kinds = [0; 7];
        self.domain
            .trace_registered_roots(&mut |kind, _| {
                kinds[KINDS
                    .iter()
                    .position(|candidate| *candidate == kind)
                    .unwrap()] += 1;
            })
            .unwrap();
        for (index, kind) in KINDS.iter().enumerate() {
            assert_eq!(
                kinds[index],
                self.roots
                    .iter()
                    .filter(|root| root.kind == *kind)
                    .map(|root| root.members.len() * root.aliases.len())
                    .sum::<usize>()
            );
        }
    }

    fn failed_collection(&mut self) {
        // Construct guaranteed collectible garbage, while deferring every pass
        // until the failing preparation limit is installed.
        let access = self.domain.access();
        if !self.reachable().iter().any(|live| *live) {
            self.birth(true);
        }
        self.birth(false);
        self.collector
            .0
            .store(1 + self.failed_passes % 2, Ordering::Relaxed);
        self.failed_passes += 1;
        drop(access);
        let present = self
            .nodes
            .iter()
            .map(|node| node.weak.upgrade().is_some())
            .collect::<Vec<_>>();
        let before = self.domain.collection_stats();
        assert_eq!(
            self.domain.request_explicit_maintenance(),
            MaintenanceOutcome::Busy(OwnershipError::ScratchUnavailable)
        );
        assert_eq!(
            self.domain.collection_stats().cleared_cells,
            before.cleared_cells
        );
        self.check_payloads(&present);
        self.collector.0.store(0, Ordering::Relaxed);
    }

    fn transition(&mut self, action: usize) {
        if action == 7 {
            self.failed_collection();
            return;
        }
        let access = self.domain.access();
        let live = self
            .reachable()
            .iter()
            .enumerate()
            .filter_map(|(id, live)| live.then_some(id))
            .collect::<Vec<_>>();
        if live.is_empty() {
            self.birth(true);
            return;
        }
        let id = live[self.rng.random_range(0..live.len())];
        let group = self.rng.random_range(0..self.roots.len());
        match action {
            0 => {
                let edges = (0..self.rng.random_range(0..5))
                    .map(|_| live[self.rng.random_range(0..live.len())])
                    .collect();
                self.nodes[id].edges = edges;
                self.nodes[id].layers =
                    vec![self.rng.random_range(0..10), self.rng.random_range(0..10)];
                self.nodes[id].revision += 1;
                let value = self.shapes.payload(id, &self.nodes[id], &self.nodes);
                values::cell(&self.nodes[id].weak)
                    .replace(&self.domain, value)
                    .unwrap();
            }
            1 => {
                let kind = KINDS[self.rng.random_range(0..KINDS.len())];
                self.add_roots(kind, vec![id]);
            }
            2 => {
                if self.roots[group].aliases.len() < 8 {
                    let alias = self.roots[group].aliases[0].clone();
                    self.roots[group].aliases.push(alias);
                }
            }
            3 => {
                let members = (0..self.rng.random_range(0..4))
                    .map(|_| live[self.rng.random_range(0..live.len())])
                    .collect::<Vec<_>>();
                let values = self.root_values(&members);
                self.roots[group].aliases.last().unwrap().replace(values);
                self.roots[group].members = members;
            }
            4 => {
                let value = self.shapes.root(id, &self.nodes, self.rng.random());
                self.roots[group].aliases.last().unwrap().push(value);
                self.roots[group].members.push(id);
            }
            5 => {
                let alias = self.rng.random_range(0..self.roots[group].aliases.len());
                drop(self.roots[group].aliases.swap_remove(alias));
                if self.roots[group].aliases.is_empty() {
                    self.roots.swap_remove(group);
                }
            }
            6 => self.birth(true),
            8 => {
                let foreign = MemoryDomain::new();
                let value = Value::Cell(foreign.new_cell(Value::Integer(3)).unwrap());
                assert_eq!(
                    values::cell(&self.nodes[id].weak).replace(&self.domain, value),
                    Err(OwnershipError::ForeignDomain)
                );
            }
            9 => {
                let reference = self.shapes.root(id, &self.nodes, true);
                assert_eq!(
                    values::cell(&self.nodes[id].weak).replace(&self.domain, reference),
                    Err(OwnershipError::EscapingReference)
                );
            }
            _ => unreachable!(),
        }
        drop(access);
    }
}

pub fn generated_graphs() {
    let image = image("program GraphTarget; procedure Target(); begin end procedure; begin end.");
    let shapes = values::Shapes::new(image.function_target(function(&image, "Target")).unwrap());
    let seeds = match std::env::var("FPAS_GC_GRAPH_SEED") {
        Ok(seed) => vec![seed.parse::<u64>().expect("decimal replay seed")],
        Err(_) => (0..32)
            .map(|seed| 0xd1b5_4a32_d192_ed03_u64.wrapping_mul(seed + 1))
            .collect(),
    };
    for seed in seeds {
        for config in [CollectionSchedule::default(), schedule()] {
            completed_threads(&format!("seeded graph {seed}"), 2, || {
                let replay = Replay {
                    seed,
                    step: Cell::new(0),
                };
                let mut graph = Graph::new(seed, config, &shapes);
                graph.check();
                assert!(
                    graph.domain.collection_stats().cleared_cells >= 12,
                    "the unrooted ring must require actual cycle collection"
                );
                for step in 0..STEPS {
                    replay.step.set(step);
                    if step == STEPS / 2 {
                        drop(graph.session.take());
                        assert_eq!(graph.domain.phase(), DomainPhase::Retired);
                    }
                    let action = if step < 10 {
                        step
                    } else if step == 10 {
                        7
                    } else {
                        graph.rng.random_range(0..10)
                    };
                    graph.transition(action);
                    graph.check();
                }
                while !graph.roots.is_empty() {
                    let index = graph.rng.random_range(0..graph.roots.len());
                    drop(graph.roots.swap_remove(index));
                    graph.check();
                }
                let stats = graph.domain.collection_stats();
                assert_eq!(stats.allocated_cells, stats.released_cells);
                assert!(stats.aborted_passes > 0);
                assert!(graph.failed_passes >= 2);
            });
        }
    }
}
