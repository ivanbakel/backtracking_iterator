/*
 * Copyright (c) 2019 Isaac van Bakel
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

use super::{BacktrackingState};

use std::sync::{RwLock, Mutex};

static EXPECT_RW: &'static str = "The read-write lock on the history has been poisoned by a thread panic!";
static EXPECT_MUTEX: &'static str = "The mutual exclusion lock on the iterator has been poisoned by a thread panic!"; 

pub struct ConcurrentBacktrackingRecorder<Iter> where Iter: Iterator {
  pub(crate) iterator: Mutex<Iter>,
  pub(crate) backtracking_vec: RwLock<Vec<Iter::Item>>,
}

impl<Iter> ConcurrentBacktrackingRecorder<Iter> where Iter: Iterator {
  /// Create a `ConcurrentBacktrackingRecorder` from an existing iterator.
  pub fn new(iterator: Iter) -> Self {
    ConcurrentBacktrackingRecorder {
      iterator: Mutex::new(iterator),
      backtracking_vec: RwLock::new(vec![]),
    }
  }

  pub fn referencing<'record>(&'record self) -> ConcurrentReferencingBacktrackingIterator<'record, Iter> {
    ConcurrentReferencingBacktrackingIterator::new(self)
  }

  /// Take all items out of the history.
  pub fn drain_history(&self) -> Vec<Iter::Item> {
    // What happes when a `Drain` iterator is leaked is not defined
    // so to guard, we collect it into a vec before returning
    self.backtracking_vec.write().expect(EXPECT_RW).drain(..).collect()
  }
}

use super::BacktrackingState::*;

#[derive(Clone)]
pub struct ConcurrentReferencingBacktrackingIterator<'record, Iter> where Iter: Iterator {
  recorder: &'record ConcurrentBacktrackingRecorder<Iter>,
  state: BacktrackingState,
}

impl<'record, Iter> ConcurrentReferencingBacktrackingIterator<'record, Iter> where Iter: Iterator {
  pub(crate) fn new(recorder: &'record ConcurrentBacktrackingRecorder<Iter>) -> Self {
    ConcurrentReferencingBacktrackingIterator {
      recorder,
      state: Progressing,
    }
  }
}

impl<'record, Iter> Iterator for ConcurrentReferencingBacktrackingIterator<'record, Iter> where Iter: Iterator, Iter::Item: 'record {
  type Item = &'record Iter::Item;

  fn next(&mut self) -> Option<&'record Iter::Item> {
    macro_rules! unsafe_backtracking_index {
      ($index:expr) => {
        unsafe {
          &*(&self.recorder.backtracking_vec.read().expect(EXPECT_RW)[$index] as *const Iter::Item)
        }
      };
    }

    use crate::{Backtracking, Progressing};
    match self.state {
      Progressing => {
        if let Some(val) = self.recorder.iterator.lock().expect(EXPECT_MUTEX).next() {
          self.recorder.backtracking_vec.write().expect(EXPECT_RW).push(val);
          Some(unsafe_backtracking_index!(self.recorder.backtracking_vec.read().expect(EXPECT_RW).len() - 1))
        } else {
          None
        }
      },
      Backtracking { position } => {
        if position >= self.recorder.backtracking_vec.read().expect(EXPECT_RW).len() {
          self.state = Progressing;
          self.next()
        } else {
          let new_position = position + 1;
          self.state = Backtracking { position: new_position };
          Some(unsafe_backtracking_index!(position))
        }
      },
    }
  }
}

use super::BacktrackingIterator;

impl<'record, Iter> BacktrackingIterator for ConcurrentReferencingBacktrackingIterator<'record, Iter> where Iter: Iterator {
  type RefPoint = usize;

  fn get_ref_point(&self) -> usize {
    match self.state {
        Progressing => self.recorder.backtracking_vec.read().expect(EXPECT_RW).len(),
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
}

#[cfg(test)]
mod tests {
  #[test]
  fn many_iter_test() {
    use matches::{matches};

    let bt_con_rec = crate::concurrent::ConcurrentBacktrackingRecorder::new(1..1000);
    
    for _ in 1..100 {
      let mut bt_iter = bt_con_rec.referencing();
      for expected in 1..1000 {
        matches!(bt_iter.next(), Some(&i) if i == expected);
      }
    }
  }
}

