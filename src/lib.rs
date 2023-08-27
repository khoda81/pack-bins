use std::{cmp, hash, iter, ops};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Bin<T> {
    pub capacity: T,
    pub items: Vec<T>,
}

impl<T> Bin<T>
where
    T: Clone + cmp::PartialOrd + for<'a> ops::AddAssign<&'a T> + for<'a> ops::SubAssign<&'a T>,
{
    pub fn new(capacity: T) -> Self {
        Self {
            capacity,
            items: Vec::new(),
        }
    }

    pub fn fits(&self, item: &T) -> bool {
        &self.capacity >= item
    }

    pub fn push(&mut self, item: T) {
        self.capacity -= &item;
        self.items.push(item);
    }

    pub fn pop(&mut self) -> Option<T> {
        self.items.pop().map(|item| {
            self.capacity += &item;
            item
        })
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn items(&self) -> &Vec<T> {
        &self.items
    }
}

// TODO: do we need both?
impl<T: std::cmp::PartialOrd> PartialOrd for Bin<T> {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        self.items.partial_cmp(&other.items)
    }
}
impl<T: std::cmp::Ord + std::cmp::Eq> Ord for Bin<T> {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.items.cmp(&other.items)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Action {
    Try,
    Backtrack,
}
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
struct State<T> {
    last_bin_capacity: Option<T>,
    next_bin_idx: usize,
    action: Action,
}

impl<T> Default for State<T> {
    fn default() -> Self {
        Self {
            last_bin_capacity: Default::default(),
            next_bin_idx: 0,
            action: Action::Try,
        }
    }
}

pub struct Fitter<T> {
    pub items: Vec<T>,
    pub bins: Vec<Bin<T>>,

    state_stack: Vec<State<T>>,
}

impl<T> Fitter<T>
where
    T: Ord + Clone + hash::Hash + for<'a> iter::Sum<&'a T>,
    T: for<'a> ops::AddAssign<&'a T> + for<'a> ops::SubAssign<&'a T>,
{
    pub fn new(mut items: Vec<T>, bin_capacities: impl IntoIterator<Item = T>) -> Self {
        items.sort();

        Self {
            bins: bin_capacities.into_iter().map(Bin::new).collect(),
            items,
            state_stack: vec![Default::default()],
        }
    }

    pub fn is_solved(&self) -> bool {
        self.items.is_empty()
    }

    pub fn step(&mut self) -> bool {
        self.step_inner().is_some()
    }

    fn step_inner(&mut self) -> Option<()> {
        let mut current = self.state_stack.pop()?;

        let mut item = match current.action {
            Action::Backtrack => self.bins[current.next_bin_idx - 1].pop().unwrap(),
            Action::Try => self.items.pop()?,
        };

        if let Some(prev_state) = self.state_stack.last() {
            let current_bin_idx = prev_state.next_bin_idx - 1;
            let prev_item = self.bins[current_bin_idx].items.last().unwrap();

            if prev_item <= &item {
                current.next_bin_idx = current.next_bin_idx.max(current_bin_idx)
            }
        }

        loop {
            let bin_idx = current.next_bin_idx;
            current.next_bin_idx += 1;

            if bin_idx >= self.bins.len() {
                self.items.push(item);
                return Some(());
            }

            if !self.bins[bin_idx].fits(&item) {
                continue;
            }

            if current.last_bin_capacity.as_ref() == Some(&self.bins[bin_idx].capacity) {
                continue;
            };

            let capacity = self.bins[bin_idx].capacity.clone();
            self.bins[bin_idx].push(item);
            if bin_idx >= 1 {
                // check that current and previous bins are in order
                if self.bins[bin_idx - 1] < self.bins[bin_idx] {
                    item = self.bins[bin_idx].pop().unwrap();
                    continue;
                }
            }

            current.last_bin_capacity = Some(capacity);

            // item was put in a bin
            current.action = Action::Backtrack;
            self.state_stack.push(current);
            self.state_stack.push(Default::default());
            break;
        }

        Some(())
    }
}
