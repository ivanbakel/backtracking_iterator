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

use super::BacktrackingState;
use super::BacktrackingState::*;

/// A wrapper around an existing iterator to extend it with backtracking functionality
pub struct CopyingBacktrackingIterator<I> where I: Iterator, I::Item: Clone {
  iterator: I,
  backtracking_vec: Vec<I::Item>,
  state: BacktrackingState,
}

/// In order to be able to backtrack, the iterator values must be `Clone`able
/// The reason for this is simple - the value will both be owned by the caller,
/// and stored in the history to be repeated later.
impl<I> Iterator for CopyingBacktrackingIterator<I> where I: Iterator, I::Item: Clone {
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

impl<I> CopyingBacktrackingIterator<I> where I:Iterator, I::Item: Clone {
  /// Create a `CopyingBacktrackingIterator` from an existing iterator.
  pub fn new(iterator: I) -> Self {
    CopyingBacktrackingIterator {
      iterator,
      backtracking_vec: vec![],
      state: Progressing,
    }
  }
}

use super::BacktrackingIterator;

impl<I> BacktrackingIterator for CopyingBacktrackingIterator<I> where I:Iterator, I::Item: Clone {
  type RefPoint = usize;

  fn get_ref_point(&self) -> usize {
    match self.state {
        Progressing => self.backtracking_vec.len(),
        Backtracking { position } => position,
    }
  }

  fn get_oldest_point(&self) -> usize {
    // Always the oldest position
    0_usize
  }

  fn backtrack(&mut self, position: usize) {
    self.state = Backtracking { position };
  }

  fn forget_before(&mut self, position: usize) {
    if position <= self.backtracking_vec.len() {
      //Split the history at the given point
      let kept = self.backtracking_vec.split_off(position);
      //Keep the second half
      self.backtracking_vec = kept;
    }
  }

  fn forget(&mut self) {
    self.backtracking_vec.clear();
  }
}

use super::Walkbackable;

impl<'history, I: 'history> Walkbackable<'history> for CopyingBacktrackingIterator<I> where I: Iterator, I::Item: Clone {
  type RefPoint = usize;
  type Item = I::Item;
  type Walkback = CopyingWalkback<'history, I>;

  fn walk_back(&'history self) -> CopyingWalkback<'history, I> {
    CopyingWalkback::new(self)
  }
}

/// A backwalk through a `BacktrackingIterator`'s history. Yields references
/// to items in the history, and can be used to walk back to a desired point.
/// The current position is before the most-recently-yielded element. To restart
/// a `BacktrackingIterator` at the current position of the backwalk, use the
/// `history_position()` method.
pub struct CopyingWalkback<'history, I> where I: Iterator, I::Item: Clone {
  backtracker: &'history CopyingBacktrackingIterator<I>,
  reverse_position: usize,
}

impl<'history, I> CopyingWalkback<'history, I> where I: Iterator, I::Item: Clone {
  fn new(backtracker: &'history CopyingBacktrackingIterator<I>) -> Self {
    let history_len = backtracker.backtracking_vec.len();
    CopyingWalkback {
      backtracker: &backtracker,
      reverse_position: history_len,
    }
  }
}

use super::Walkback;

impl<'history, I> Walkback<'history> for CopyingWalkback<'history, I> where I: Iterator, I::Item: Clone {
  type RefPoint = usize;

  fn get_ref_point(&self) -> usize {
    self.reverse_position
  }
}

impl<'history, I> Iterator for CopyingWalkback<'history, I> where I: Iterator, I::Item: Clone {
  type Item = I::Item;

  fn next(&mut self) -> Option<Self::Item> {
    if self.reverse_position == 0 {
      None
    } else {
      let new_position = self.reverse_position - 1_usize;
      let val = &self.backtracker.backtracking_vec[new_position];
      self.reverse_position = new_position;
      Some(val.clone())
    }
  }
}

