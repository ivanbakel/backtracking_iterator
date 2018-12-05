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

use super::BacktrackingRecorder;
use super::BacktrackingState::*;

/// An iterator over a historical record which produces references to historical
/// elements
pub struct ReferencingBacktrackingIterator<'record, Iter> where Iter: Iterator {
  recorder: &'record mut BacktrackingRecorder<Iter>,
}

impl<'record, Iter> ReferencingBacktrackingIterator<'record, Iter> where Iter: Iterator {
  pub(crate) fn new(recorder: &'record mut BacktrackingRecorder<Iter>) -> Self {
    ReferencingBacktrackingIterator {
      recorder,
    }
  }
}

impl<'record, Iter> Iterator for ReferencingBacktrackingIterator<'record, Iter> where Iter: Iterator, Iter::Item: 'record {
  type Item = &'record Iter::Item;

  fn next(&mut self) -> Option<&'record Iter::Item> {
    /// Produce a borrow on the vec which should be valid
    /// We borrow the vec for the lifetime of 'record, so we should
    /// be able to produce a reference for the lifetime of 'record,
    /// so long as we never remove items from the vec - which only
    /// happens with a `forget`, which requires a mutable borrow on
    /// the parent `Record`, which we already hold!
    macro_rules! unsafe_backtracking_index {
      ($index:expr) => {
        unsafe {
          &*(&self.recorder.backtracking_vec[$index] as *const Iter::Item)
        }
      };
    }

    use crate::{Backtracking, Progressing};
    match self.recorder.state {
      Progressing => {
        if let Some(val) = self.recorder.iterator.next() {
          self.recorder.backtracking_vec.push(val);
          Some(unsafe_backtracking_index!(self.recorder.backtracking_vec.len() - 1))
        } else {
          None
        }
      },
      Backtracking { position } => {
        if position >= self.recorder.backtracking_vec.len() {
          self.recorder.state = Progressing;
          self.next()
        } else {
          let new_position = position + 1;
          self.recorder.state = Backtracking { position: new_position };
          Some(unsafe_backtracking_index!(position))
        }
      },
    }
  }
}

use super::BacktrackingIterator;

impl<'record, Iter> BacktrackingIterator for ReferencingBacktrackingIterator<'record, Iter> where Iter: Iterator {
  type RefPoint = usize;

  fn get_ref_point(&self) -> usize {
    match self.recorder.state {
        Progressing => self.recorder.backtracking_vec.len(),
        Backtracking { position } => position,
    }
  }

  fn get_oldest_point(&self) -> usize {
    // Always the oldest position
    0_usize
  }

  fn backtrack(&mut self, position: usize) {
    self.recorder.state = Backtracking { position };
  }
}

use super::Walkbackable;

impl<'history, 'record, Iter: 'history> Walkbackable<'history> for ReferencingBacktrackingIterator<'record, Iter> where Iter: Iterator, 'history : 'record {
  type RefPoint = usize;
  type Item = &'record Iter::Item;
  type Walkback = ReferencingWalkback<'record, Iter>;

  fn walk_back(&'history self) -> ReferencingWalkback<'record, Iter> where {
    ReferencingWalkback::new(self)
  }
}

/// A backwalk through a `ReferencingBacktrackingIterator`'s history. Yields references to
/// items in the history, and can be used to walk back to a desired point.
pub struct ReferencingWalkback<'record, Iter> where Iter: Iterator {
  backtracker: &'record BacktrackingRecorder<Iter>,
  reverse_position: usize,
}

impl<'record, Iter> ReferencingWalkback<'record, Iter> 
  where Iter: Iterator, Iter::Item: 'record {
  fn new<'history>(backtracker: &'history ReferencingBacktrackingIterator<'record, Iter>) -> Self where 'history : 'record {
    let history_len = backtracker.recorder.backtracking_vec.len();
    ReferencingWalkback {
      backtracker: backtracker.recorder,
      reverse_position: history_len,
    }
  }
}

use super::Walkback;

impl<'history, 'record, Iter> Walkback<'history> for ReferencingWalkback<'record, Iter>
  where Iter: Iterator, 'history : 'record {
  type RefPoint = usize;

  fn get_ref_point(&self) -> usize {
    self.reverse_position
  }
}

impl<'history, 'record, Iter> Iterator for ReferencingWalkback<'record, Iter> 
  where Iter: Iterator, Iter::Item: 'record, 'history : 'record {
  type Item = &'record Iter::Item;

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
