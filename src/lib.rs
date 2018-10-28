/*
 * Copyright (c) 2018 Isaac van Bakel
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy of
 * this software and associated documentation files (the "Software"), to deal in
 * the Software without restriction, including without limitation the rights to
 * use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of
 * the Software, and to permit persons to whom the Software is furnished to do so,
 * subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS
 * FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR
 * COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER
 * IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN
 * CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
 */

/// A wrapper around an existing iterator to extend it with backtracking functionality
pub struct BacktrackingIterator<I> where I: Iterator {
  iterator: I,
  backtracking_vec: Vec<I::Item>,
  state: BacktrackingState,
}

use crate::BacktrackingState::*;

enum BacktrackingState {
  /// There may be some values in the history, but we're taking values off the iterator
  Progressing,
  /// We've been asked to backtrack, so we've started taking values from the history instead
  /// The `position` field tracks where we are in the history, with 0 being at the start.
  ///
  /// A `BacktrackingIterator` may be in this state with `position` as an invalid index into
  /// the history - in this case, the next call to `next()` will restore it to the `Progressing`
  /// state and yield a value from the internal `Iterator`.
  Backtracking { position: usize },
}

impl<I> BacktrackingIterator<I> where I:Iterator {
  /// Create a `BacktrackingIterator` from an existing iterator.
  pub fn new(iterator: I) -> Self {
    BacktrackingIterator {
      iterator,
      backtracking_vec: vec![],
      state: Progressing,
    }
  }

  /// Start the iterator again from all the elements in the current history
  /// The iterator will repeat every element which was emitted since the last
  /// call to `forget()`.
  ///
  /// ```
  /// extern crate backtracking_iterator;
  /// use backtracking_iterator::BacktrackingIterator;
  ///
  /// let v = vec![1_u8, 2_u8];
  /// let mut bt = BacktrackingIterator::new(v.into_iter());
  /// bt.next();
  /// bt.backtrack();
  /// assert!(bt.next().unwrap() == 1_u8);
  /// ```
  pub fn backtrack(&mut self) {
    self.state = Backtracking { position: 0_usize };
  }

  /// Clear the current history.
  /// ```
  /// extern crate backtracking_iterator;
  /// use backtracking_iterator::BacktrackingIterator;
  ///
  /// let v = vec![1_u8, 2_u8];
  /// let mut bt = BacktrackingIterator::new(v.into_iter());
  /// bt.next();
  ///
  /// //Before we call this, 1_u8 is in the history
  /// bt.forget();
  ///
  /// bt.backtrack();
  /// assert!(bt.next().unwrap() == 2_u8);
  /// ```
  pub fn forget(&mut self) {
    self.backtracking_vec.clear();
  }
}

/// In order to be able to backtrack, the iterator values must be `Clone`able
/// The reason for this is simple - the value will both be owned by the caller,
/// and stored in the history to be repeated later.
impl<I> Iterator for BacktrackingIterator<I> where I: Iterator, I::Item: Clone {
  type Item = I::Item;

  fn next(&mut self) -> Option<Self::Item> {
    use crate::{Backtracking, Progressing};
    match self.state {
      Progressing => {
        if let Some(val) = self.iterator.next() {
          self.backtracking_vec.push(val.clone());
          Some(val)
        } else {
          None
        }
      },
      Backtracking { position } => {
        if position >= self.backtracking_vec.len() {
          self.state = Progressing;
          self.next()
        } else {
          let backtracked_value = self.backtracking_vec[position].clone();
          let new_position = position + 1;
          self.state = Backtracking { position: new_position };
          Some(backtracked_value)
        }
      },
    }
  }
}

#[cfg(test)]
mod tests {
  #[test]
  fn basic_test() {
    use crate::{BacktrackingIterator};

    let num_vec = vec![1_u8, 2, 3, 4, 5, 6];
    let vec_iter = num_vec.into_iter();
    let mut bt_iter = BacktrackingIterator::new(vec_iter);
    assert!(bt_iter.next().unwrap() == 1_u8);
    assert!(bt_iter.next().unwrap() == 2_u8);

    bt_iter.backtrack();
    assert!(bt_iter.next().unwrap() == 1_u8);
    assert!(bt_iter.next().unwrap() == 2_u8);

    bt_iter.forget();
    bt_iter.backtrack();
    assert!(bt_iter.next().unwrap() == 3_u8);
    assert!(bt_iter.next().unwrap() == 4_u8);
    assert!(bt_iter.next().unwrap() == 5_u8);
    assert!(bt_iter.next().unwrap() == 6_u8);
    assert!(!bt_iter.next().is_some());

    bt_iter.backtrack();
    assert!(bt_iter.next().unwrap() == 3_u8);
  }
}
