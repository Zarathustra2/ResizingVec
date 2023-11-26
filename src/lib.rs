use std::{
    iter::repeat_with,
    ops::{Index, IndexMut},
};

#[derive(Debug)]
pub struct ResizingVec<T> {
    vec: Vec<Option<T>>,
    /// The amount of positions in `vec`
    /// that have active (Some(_) values)
    active: usize,
}

impl<T: Clone> Clone for ResizingVec<T> {
    fn clone(&self) -> Self {
        Self {
            vec: self.vec.clone(),
            active: self.active,
        }
    }
}

impl<T> Index<usize> for ResizingVec<T> {
    type Output = Option<T>;

    fn index(&self, index: usize) -> &Self::Output {
        &self.vec[index]
    }
}

impl<T> IndexMut<usize> for ResizingVec<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.vec[index]
    }
}

impl<T> Default for ResizingVec<T> {
    fn default() -> Self {
        Self {
            vec: Vec::default(),
            active: 0,
        }
    }
}

impl<T> ResizingVec<T> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn prefill(capacity: usize) -> Self {
        let vec = repeat_with(|| None).take(capacity).collect::<Vec<_>>();
        Self { vec, active: 0 }
    }

    /// Returns the amount of space length the inner vector has.
    /// Creating a fresh `ResizingVec` and then inserting elements
    /// at index 0 & 2 would result in reserved_space() returning 3
    /// as at position 1 would be an empty value
    pub fn reserved_space(&self) -> usize {
        self.vec.len()
    }

    /// Returns the amount of active values.
    /// filled() <= reserved_space() will always hold true.
    pub fn filled(&self) -> usize {
        self.active
    }

    pub fn get(&self, idx: usize) -> Option<&T> {
        match self.vec.get(idx) {
            Some(inner) => inner.as_ref(),
            None => None,
        }
    }

    // TODO: Use traits
    pub fn iter(&self) -> impl Iterator<Item = (usize, &T)> + '_ {
        self.vec
            .iter()
            .enumerate()
            .filter(|(_, t)| t.is_some())
            .map(|(idx, t)| (idx, t.as_ref().unwrap()))
    }

    pub fn get_mut(&mut self, idx: usize) -> Option<&mut T> {
        match self.vec.get_mut(idx) {
            Some(inner) => inner.as_mut(),
            None => None,
        }
    }

    pub fn remove(&mut self, idx: usize) -> Option<T> {
        if self.vec.len() > idx {
            let prev = self.vec[idx].take();
            if prev.is_some() {
                self.active -= 1;
            }

            prev
        } else {
            None
        }
    }

    pub fn insert(&mut self, idx: usize, t: T) -> Option<T> {
        while self.vec.len() <= idx {
            self.vec.push(None);
        }

        let prev = std::mem::replace(&mut self.vec[idx], Some(t));

        if prev.is_none() {
            self.active += 1;
        }

        prev
    }

    pub fn clear(&mut self) {
        *self = Self::default();
    }

    pub fn resize(&mut self) -> Vec<Position> {
        let vec = Vec::with_capacity(self.active);
        let mut positions = Vec::with_capacity(self.active);

        let prev = std::mem::replace(&mut self.vec, vec);

        for (idx, elem) in prev.into_iter().enumerate() {
            if elem.is_some() {
                self.vec.push(elem);
                positions.push(Position {
                    prev_idx: idx,
                    new_idx: self.vec.len() - 1,
                });
            }
        }

        self.active = self.vec.len();

        positions
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Position {
    pub prev_idx: usize,
    pub new_idx: usize,
}

impl Position {
    pub fn changed(&self) -> bool {
        self.prev_idx != self.new_idx
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reserved_space() {
        let mut rv = ResizingVec::prefill(10);
        assert_eq!(rv.reserved_space(), 10);
        assert_eq!(rv.filled(), 0);

        rv.insert(0, "0");

        assert_eq!(rv.reserved_space(), 10);
        assert_eq!(rv.filled(), 1);
    }

    #[test]
    fn get_remove_iter_clear() {
        let mut rv = ResizingVec::default();
        assert_eq!(None, rv.get(0));

        rv.insert(0, "0");

        assert_eq!(Some(&"0"), rv.get(0));
        assert_eq!(None, rv.get(1));

        rv.remove(0);

        assert_eq!(None, rv.get(0));
        assert_eq!(None, rv.get(1));

        rv.insert(1, "1");
        rv.insert(2, "2");
        rv.insert(3, "3");

        assert_eq!(Some(&"1"), rv.get(1));
        assert_eq!(Some(&"2"), rv.get(2));
        assert_eq!(Some(&"3"), rv.get(3));

        assert_eq!(3, rv.iter().count());

        rv.clear();

        assert_eq!(0, rv.iter().count());

        assert_eq!(rv.reserved_space(), 0);
        assert_eq!(rv.filled(), 0);
    }

    #[test]
    fn resize() {
        let mut rv = ResizingVec::default();

        rv.insert(10, "1");
        rv.insert(22, "2");
        rv.insert(44, "3");
        rv.insert(0, "0");

        assert_eq!(rv.reserved_space(), 45);
        assert_eq!(rv.filled(), 4);

        let new_positions = rv.resize();

        assert_eq!(new_positions.len(), 4);

        assert_eq!(new_positions[0].prev_idx, 0);
        assert_eq!(new_positions[0].new_idx, 0);
        assert!(!new_positions[0].changed());

        assert_eq!(new_positions[1].prev_idx, 10);
        assert_eq!(new_positions[1].new_idx, 1);
        assert!(new_positions[1].changed());

        assert_eq!(new_positions[2].prev_idx, 22);
        assert_eq!(new_positions[2].new_idx, 2);
        assert!(new_positions[2].changed());

        assert_eq!(new_positions[3].prev_idx, 44);
        assert_eq!(new_positions[3].new_idx, 3);
        assert!(new_positions[3].changed());

        assert_eq!(rv.reserved_space(), 4);
        assert_eq!(rv.filled(), 4);
    }
}
