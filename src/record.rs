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

use super::{BacktrackingState, Record, ReferencingBacktrackingIterator, CopyingBacktrackingIterator};
use self::BacktrackingState::{Progressing, Backtracking};

pub struct BacktrackingRecorder<Iter> where Iter: Iterator {
  pub(crate) iterator: Iter,
  pub(crate) backtracking_vec: Vec<Iter::Item>,
  pub(crate) state: BacktrackingState,
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

  pub fn referencing<'record>(&'record mut self) -> ReferencingBacktrackingIterator<'record, Iter> {
    ReferencingBacktrackingIterator::new(self)
  }

  pub fn copying<'record>(&'record mut self) -> CopyingBacktrackingIterator<'record, Iter> where Iter::Item: Clone {
    CopyingBacktrackingIterator::new(self)
  }
}

impl<Iter> Record for BacktrackingRecorder<Iter> where Iter: Iterator {
  type RefPoint = usize;
  
  fn get_ref_point(&self) -> usize {
    match self.state {
        Progressing => self.backtracking_vec.len(),
        Backtracking { position } => position,
    }
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
