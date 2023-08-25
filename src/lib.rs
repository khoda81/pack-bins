use std::{collections, time};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Bin<T> {
    capacity: T,
    items: Vec<T>,
}

impl<T> Bin<T>
where
    T: Clone + std::cmp::PartialOrd + std::ops::SubAssign + std::ops::AddAssign,
{
    pub fn new(capacity: T) -> Self {
        Self {
            capacity,
            items: Vec::new(),
        }
    }

    pub fn try_push(&mut self, item: T) -> Result<&mut T, T> {
        if self.capacity >= item {
            self.capacity -= item.clone();
            self.items.push(item);

            Ok(self.items.last_mut().unwrap())
        } else {
            Err(item)
        }
    }

    pub fn pop(&mut self) -> Option<T> {
        self.items.pop().map(|item| {
            self.capacity += item.clone();
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

enum State {
    Try,
    Backtrack,
}
struct StackState<T> {
    searched_bins: collections::HashSet<T>,
    bin_idx: usize,
    state: State,
}

pub struct Fitter<T> {
    items: Vec<T>,
    bins: Vec<Bin<T>>,

    state_stack: Vec<StackState<T>>,
}

impl<T> Fitter<T>
where
    T: Ord
        + Clone
        + std::hash::Hash
        + std::ops::SubAssign
        + std::ops::AddAssign
        + for<'a> std::iter::Sum<&'a T>,
{
    pub fn new(mut items: Vec<T>, bin_capacities: impl IntoIterator<Item = T>) -> Self {
        items.sort();

        Self {
            bins: bin_capacities.into_iter().map(Bin::new).collect(),
            items,
            state_stack: vec![StackState {
                searched_bins: collections::HashSet::new(),
                bin_idx: 0,
                state: State::Try,
            }],
        }
    }

    pub fn fit(mut self) -> Option<Vec<Bin<T>>> {
        let total_weight: T = self.items.iter().sum();
        let total_size: T = self.bins.iter().map(|bin| &bin.capacity).sum();

        if total_weight <= total_size && {
            let start = time::Instant::now();
            let num_iters = self.count();
            let dur = start.elapsed();
            println!(
                "c {num_iters} iterations in {dur:?} ({:?} per iteration)",
                dur / num_iters as u32
            );

            self.items.is_empty()
        } {
            Some(self.bins.into_iter().collect())
        } else {
            None
        }
    }
}

impl<T> Iterator for &mut Fitter<T>
where
    T: Clone + Ord + std::hash::Hash + std::ops::SubAssign + std::ops::AddAssign,
{
    type Item = ();

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.state_stack.last_mut()?;

        match current.state {
            State::Backtrack => {
                current.state = State::Try;

                self.items
                    .push(self.bins[current.bin_idx - 1].pop().unwrap());
            }

            State::Try => match self.bins.get_mut(current.bin_idx) {
                None => drop(self.state_stack.pop()),

                Some(bin) => match self.items.pop() {
                    None => {
                        self.state_stack.pop();
                        return None;
                    }

                    Some(item) => match bin.try_push(item) {
                        Err(item) => {
                            current.bin_idx += 1;
                            self.items.push(item)
                        }

                        Ok(_) => match current.searched_bins.insert(bin.capacity.clone()) {
                            false => {
                                current.bin_idx += 1;
                                self.items.push(bin.pop().unwrap())
                            }

                            true => {
                                current.state = State::Backtrack;

                                let mut bin_idx = 0;
                                if let Some(next_item) = self.items.last_mut() {
                                    if bin.items.last().unwrap() <= next_item {
                                        bin_idx = current.bin_idx;
                                    }
                                }

                                current.bin_idx += 1;
                                self.state_stack.push(StackState {
                                    searched_bins: collections::HashSet::new(),
                                    bin_idx,
                                    state: State::Try,
                                })
                            }
                        },
                    },
                },
            },
        };

        Some(())
    }
}
