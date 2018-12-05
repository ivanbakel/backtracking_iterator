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

/// A wrapper around an existing iterator to give it a historical representation
/// with the ability to then produce copying and referencing backtracking iterators
/// on the history
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

  /// Take all items out of the history.
  /// ```
  /// extern crate backtracking_iterator;
  /// use backtracking_iterator::{BacktrackingIterator, BacktrackingRecorder};
  ///
  /// let vec_iter = vec![1_u8, 2, 3].into_iter();
  /// let mut rec = BacktrackingRecorder::new(vec_iter);
  ///
  /// {
  ///   let mut bt_ref = rec.referencing();
  ///   bt_ref.next(); // 1_u8
  /// }
  ///
  /// let mut history = rec.drain_history().into_iter();
  /// // Repeats only what was in the history
  /// assert!(history.next().unwrap() == 1_u8);
  /// assert!(history.next().is_none());
  /// ```
  pub fn drain_history(&mut self) -> Vec<Iter::Item> {
    // What happes when a `Drain` iterator is leaked is not defined
    // so to guard, we collect it into a vec before returning
    self.backtracking_vec.drain(..).collect()
  }
}

impl<Iter, Item> IntoIterator for BacktrackingRecorder<Iter> where Iter: Iterator<Item=Item> + IntoIterator<Item=Item> {
  type Item = Item;
  type IntoIter = std::iter::Chain<std::vec::IntoIter<Item>, Iter::IntoIter>;

  /// Destroy the record and return an iterator which starts from the beginning
  /// of the history and chains into the originally-given iterator
  /// ```
  /// extern crate backtracking_iterator;
  /// use backtracking_iterator::{BacktrackingIterator, BacktrackingRecorder};
  ///
  /// let vec_iter = vec![1_u8, 2, 3].into_iter();
  /// let mut rec = BacktrackingRecorder::new(vec_iter);
  ///
  /// {
  ///   let mut bt_ref = rec.referencing();
  ///   bt_ref.next(); // 1_u8
  /// }
  ///
  /// let mut rec_iter = rec.into_iter();
  /// // Repeats the value in the history
  /// assert!(rec_iter.next().unwrap() == 1_u8);
  /// // And follows up with the ones not yet recorded
  /// assert!(rec_iter.next().unwrap() == 2_u8);
  /// assert!(rec_iter.next().unwrap() == 3_u8);
  /// assert!(rec_iter.next().is_none());
  /// ```
  fn into_iter(self) -> Self::IntoIter {
    self.backtracking_vec.into_iter().chain(self.iterator)
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
