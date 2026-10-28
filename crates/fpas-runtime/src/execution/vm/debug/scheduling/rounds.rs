//! Fixed-width seeded fair-round state. Only shuffles consume random words.

use std::sync::Arc;

use super::ScheduleGuard;

pub(in crate::execution::vm::debug) struct SeededRounds {
    pub seed: u64,
    state: u64,
    order: Vec<u64>,
    next: usize,
    pub rounds: u64,
    pub dispatches: u64,
    pub round_dispatched: bool,
    pub guard: Arc<ScheduleGuard>,
}

impl SeededRounds {
    pub fn new(seed: u64) -> Self {
        Self {
            seed,
            state: seed,
            order: Vec::new(),
            next: 0,
            rounds: 0,
            dispatches: 0,
            round_dispatched: false,
            guard: Arc::default(),
        }
    }

    fn word(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9e3779b97f4a7c15);
        let mut word = self.state;
        word = (word ^ (word >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        word = (word ^ (word >> 27)).wrapping_mul(0x94d049bb133111eb);
        word ^ (word >> 31)
    }

    pub fn begin(&mut self, ids: Vec<u64>) {
        self.order = ids;
        self.next = 0;
        self.round_dispatched = false;
        if self.order.is_empty() {
            return;
        }
        self.rounds = self.rounds.saturating_add(1);
        for index in (1..self.order.len()).rev() {
            let Ok(bound) = u64::try_from(index + 1) else {
                unreachable!("allocated task vector length fits u64");
            };
            let threshold = bound.wrapping_neg() % bound;
            let selected = loop {
                let word = self.word();
                if word >= threshold {
                    break word % bound;
                }
            };
            let Ok(selected) = usize::try_from(selected) else {
                unreachable!("selected task index is bounded by its usize source");
            };
            self.order.swap(index, selected);
        }
    }

    pub fn next(&mut self) -> Option<u64> {
        let task = self.order.get(self.next).copied()?;
        self.next += 1;
        Some(task)
    }

    pub fn dispatched(&mut self) {
        self.dispatches = self.dispatches.saturating_add(1);
        self.round_dispatched = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splitmix64_reference_vectors_and_empty_rounds() {
        for (seed, first) in [
            (0, 0xe220a8397b1dcdaf),
            (42, 0xbdd732262feb6e95),
            (u64::MAX, 0xe4d971771b652c20),
        ] {
            let mut rounds = SeededRounds::new(seed);
            rounds.begin(vec![]);
            rounds.begin(vec![0]);
            assert_eq!(rounds.word(), first);
        }
    }

    #[test]
    fn shuffled_rounds_are_repeatable_permutations_with_one_turn_per_id() {
        let mut a = SeededRounds::new(42);
        let mut b = SeededRounds::new(42);
        for _ in 0..20 {
            a.begin((0..10).collect());
            b.begin((0..10).collect());
            let order: Vec<_> = std::iter::from_fn(|| a.next()).collect();
            assert_eq!(order, std::iter::from_fn(|| b.next()).collect::<Vec<_>>());
            let mut sorted = order;
            sorted.sort_unstable();
            assert_eq!(sorted, (0..10).collect::<Vec<_>>());
        }
    }

    #[test]
    fn version_one_round_vectors_match_independent_splitmix_fisher_yates_oracle() {
        for (seed, first, second) in [
            (0, [4, 2, 5, 3, 0, 1], [1, 4, 2, 5, 3, 0]),
            (42, [4, 3, 0, 2, 5, 1], [2, 3, 1, 4, 5, 0]),
            (u64::MAX, [3, 5, 0, 1, 4, 2], [5, 2, 3, 4, 0, 1]),
        ] {
            let mut rounds = SeededRounds::new(seed);
            rounds.begin((0..6).collect());
            assert_eq!(
                std::iter::from_fn(|| rounds.next()).collect::<Vec<_>>(),
                first
            );
            rounds.begin(vec![]);
            rounds.begin(vec![10]);
            rounds.begin((0..6).collect());
            assert_eq!(
                std::iter::from_fn(|| rounds.next()).collect::<Vec<_>>(),
                second
            );
        }
    }
}
