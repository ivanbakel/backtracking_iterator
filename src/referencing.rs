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

pub struct BacktrackingRecorder<Iter> where Iter: Iterator {
  iterator: Iter,
  backtracking_vec: Vec<Iter::Item>,
  state: BacktrackingState,
}

impl<Iter> BacktrackingRecorder<Iter> where Iter: Iterator {
  /// Create a `BacktrackingRecorder` from an existing iterator.
  pub fn new(iterator: Iter) -> Self {
    BacktrackingRecorder {
      iterator,
      backtracking_vec: vec![],
      state: Progressing,
    }
  }

  pub fn record<'backtrack>(&'backtrack mut self) -> ReferencingBacktrackingIterator<'backtrack, Iter> {
    ReferencingBacktrackingIterator {
      backtracker: self,
    }
  }
}

pub struct ReferencingBacktrackingIterator<'backtrack, Iter> where Iter: Iterator {
  backtracker : &'backtrack mut BacktrackingRecorder<Iter>,
}

impl<'backtrack, Iter> Iterator for ReferencingBacktrackingIterator<'backtrack, Iter> where Iter: Iterator {
  type Item = &'backtrack Iter::Item;

  fn next(&mut self) -> Option<&'backtrack Iter::Item> {
    /// Produce a borrow on the vec which should be valid
    /// We borrow the vec for the lifetime of 'backtrack, so we should
    /// be able to produce a reference for the lifetime of 'backtrack,
    /// so long as we never remove items from the vec
    macro_rules! unsafe_backtracking_index {
      ($index:expr) => {
        unsafe {
          &*(&self.backtracker.backtracking_vec[$index] as *const Iter::Item)
        }
      };
    }

    use crate::{Backtracking, Progressing};
    match self.backtracker.state {
      Progressing => {
        if let Some(val) = self.backtracker.iterator.next() {
          self.backtracker.backtracking_vec.push(val);
          Some(unsafe_backtracking_index!(self.backtracker.backtracking_vec.len() - 1))
        } else {
          None
        }
      },
      Backtracking { position } => {
        if position >= self.backtracker.backtracking_vec.len() {
          self.backtracker.state = Progressing;
          self.next()
        } else {
          let new_position = position + 1;
          self.backtracker.state = Backtracking { position: new_position };
          Some(unsafe_backtracking_index!(position))
        }
      },
    }
  }
}

use super::BacktrackingIterator;

impl<'backtrack, Iter> BacktrackingIterator for ReferencingBacktrackingIterator<'backtrack, Iter> where Iter: Iterator {
  type RefPoint = usize;

  fn get_ref_point(&self) -> usize {
    match self.backtracker.state {
        Progressing => self.backtracker.backtracking_vec.len(),
        Backtracking { position } => position,
    }
  }

  fn get_oldest_point(&self) -> usize {
    // Always the oldest position
    0_usize
  }

  fn backtrack(&mut self, position: usize) {
    self.backtracker.state = Backtracking { position };
  }

  fn forget_before(&mut self, position: usize) {
    if position <= self.backtracker.backtracking_vec.len() {
      //Split the history at the given point
      let kept = self.backtracker.backtracking_vec.split_off(position);
      //Keep the second half
      self.backtracker.backtracking_vec = kept;
    }
  }

  fn forget(&mut self) {
    self.backtracker.backtracking_vec.clear();
  }
}

use super::Walkbackable;

impl<'history, 'backtrack, Iter: 'history> Walkbackable<'history> for ReferencingBacktrackingIterator<'backtrack, Iter> where Iter: Iterator, Iter::Item: Clone, 'history : 'backtrack {
  type RefPoint = usize;
  type Item = &'backtrack Iter::Item;
  type Walkback = ReferencingWalkback<'backtrack, Iter>;

  fn walk_back(&'history self) -> ReferencingWalkback<'backtrack, Iter> where {
    ReferencingWalkback::new(self)
  }
}

pub struct ReferencingWalkback<'backtrack, Iter> where Iter: Iterator, Iter::Item: Clone {
  backtracker: &'backtrack BacktrackingRecorder<Iter>,
  reverse_position: usize,
}

impl<'backtrack, Iter> ReferencingWalkback<'backtrack, Iter> 
  where Iter: Iterator, Iter::Item: Clone {
  fn new<'history>(backtracker: &'history ReferencingBacktrackingIterator<'backtrack, Iter>) -> Self where 'history : 'backtrack {
    let history_len = backtracker.backtracker.backtracking_vec.len();
    ReferencingWalkback {
      backtracker: backtracker.backtracker,
      reverse_position: history_len,
    }
  }
}

use super::Walkback;

impl<'history, 'backtrack, Iter> Walkback<'history> for ReferencingWalkback<'backtrack, Iter>
  where Iter: Iterator, Iter::Item: Clone, 'history : 'backtrack {
  type RefPoint = usize;

  fn get_ref_point(&self) -> usize {
    self.reverse_position
  }
}

impl<'history, 'backtrack, Iter> Iterator for ReferencingWalkback<'backtrack, Iter> 
  where Iter: Iterator, Iter::Item: Clone, 'history : 'backtrack {
  type Item = &'backtrack Iter::Item;

  fn next(&mut self) -> Option<Self::Item> {
    if self.reverse_position == 0 {
      None
    } else {
      let new_position = self.reverse_position - 1_usize;
      let val = &self.backtracker.backtracking_vec[new_position];
      self.reverse_position = new_position;
      Some(val)
    }
  }
}
