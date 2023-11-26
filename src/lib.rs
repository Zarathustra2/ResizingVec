#![doc = include_str!("../README.md")]

use std::{
    iter::repeat_with,
    ops::{Index, IndexMut},
};

#[derive(Debug, Clone)]
pub struct ResizingVec<T> {
    data: Vec<Option<T>>,
    /// The amount of positions in `vec`
    /// that have active (Some(_) values)
    active: usize,
}

impl<T> From<Vec<T>> for ResizingVec<T> {
    fn from(value: Vec<T>) -> Self {
        let data = value.into_iter().map(|e| Some(e)).collect::<Vec<_>>();
        let active = data.len();
        Self { data, active }
    }
}

impl<T> Index<usize> for ResizingVec<T> {
    type Output = Option<T>;

    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index]
    }
}

impl<T> IndexMut<usize> for ResizingVec<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.data[index]
    }
}

impl<T> Default for ResizingVec<T> {
    fn default() -> Self {
        Self {
            data: Vec::default(),
            active: 0,
        }
    }
}

impl<T> ResizingVec<T> {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Pre allocates enough space so that elements with ids < [`reserved_space()`](#method.reserved_space)
    /// fit without having to resize
    #[must_use]
    pub fn prefill(capacity: usize) -> Self {
        let vec = repeat_with(|| None).take(capacity).collect::<Vec<_>>();
        Self {
            data: vec,
            active: 0,
        }
    }

    /// Returns the amount of space length the inner vector has.
    /// Creating a fresh `ResizingVec` and then inserting elements
    /// at index 0 & 2 would result in [`reserved_space()`](#method.reserved_space) returning 3
    /// as at position 1 would be an empty value
    #[must_use]
    pub fn reserved_space(&self) -> usize {
        self.data.len()
    }

    /// Returns the amount of active values.
    /// [filled()](#method.filled) <= [`reserved_space()`](#method.reserved_space) will always hold true.
    #[must_use]
    pub fn filled(&self) -> usize {
        self.active
    }

    /// Returns the element at the given index
    #[must_use]
    pub fn get(&self, idx: usize) -> Option<&T> {
        match self.data.get(idx) {
            Some(inner) => inner.as_ref(),
            None => None,
        }
    }

    /// Returns a mutable reference to element at the given index
    #[must_use]
    pub fn get_mut(&mut self, idx: usize) -> Option<&mut T> {
        match self.data.get_mut(idx) {
            Some(inner) => inner.as_mut(),
            None => None,
        }
    }

    // TODO: Use traits
    pub fn iter(&self) -> impl Iterator<Item = (usize, &T)> + '_ {
        self.data
            .iter()
            .enumerate()
            .filter_map(|(idx, t)| t.as_ref().map(|e| (idx, e)))
    }

    /// Removes the element at the given index and returns the
    /// remove element. If the given index is out of bounds
    /// than None is being returned
    pub fn remove(&mut self, idx: usize) -> Option<T> {
        if self.data.len() > idx {
            let prev = self.data[idx].take();
            if prev.is_some() {
                self.active -= 1;
            }

            prev
        } else {
            None
        }
    }

    /// Inserts the element at the given index.
    /// IMPORTANT: The time complexity of this operation
    /// depends on whether it has to resize or not.
    ///
    /// if [`reserved_space()`](#method.reserved_space) < idx then `insert`
    /// will first insert (idx - [`reserved_space()`](#method.reserved_space)) elements before
    /// inserting the given element at the index.
    pub fn insert(&mut self, idx: usize, t: T) -> Option<T> {
        while self.data.len() <= idx {
            self.data.push(None);
        }

        let prev = std::mem::replace(&mut self.data[idx], Some(t));

        if prev.is_none() {
            self.active += 1;
        }

        prev
    }

    /// Clears the vector, removing all values
    pub fn clear(&mut self) {
        *self = Self::default();
    }

    /// Resizes the vector shrinks it so that every reserved space is being occupied by an element.
    ///
    /// # Examples
    /// ```
    /// use resizing_vec::{ResizingVec, Position};
    ///
    /// let mut r_vec = ResizingVec::new();
    /// r_vec.insert(5, "6th elem".to_string());
    /// // ResizingVec { vec: [None, None, None, None, None, Some("5th elem")], active: 1 }
    /// assert_eq!(6, r_vec.reserved_space());
    /// assert_eq!(1, r_vec.filled());
    ///
    /// let new_positions = r_vec.resize();
    /// // ResizingVec { vec: [Some("5th elem")], active: 1 }
    /// assert_eq!(new_positions, [Position {prev_idx: 5, new_idx: 0}]);
    /// assert_eq!(1, r_vec.reserved_space());
    /// assert_eq!(1, r_vec.filled());
    /// ```
    #[must_use]
    pub fn resize(&mut self) -> Vec<Position> {
        let vec = Vec::with_capacity(self.active);
        let mut positions = Vec::with_capacity(self.active);

        let prev = std::mem::replace(&mut self.data, vec);

        for (idx, elem) in prev.into_iter().enumerate() {
            if elem.is_some() {
                self.data.push(elem);
                positions.push(Position {
                    prev_idx: idx,
                    new_idx: self.data.len() - 1,
                });
            }
        }

        self.active = self.data.len();

        positions
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Position {
    /// The previous index of the element before resizing
    pub prev_idx: usize,
    /// The new index of the element
    pub new_idx: usize,
}

impl Position {
    #[must_use]
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
