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

pub trait Record {
  /// The type used to refer to positions in the history
  type RefPoint;
  
  /// Yield a reference to the current point in the history
  /// This reference must be valid for as long as the current
  /// point remains in the history
  fn get_ref_point(&self) -> Self::RefPoint;

  /// Eliminate all the values before the given reference point from the history
  fn forget_before(&mut self, point: Self::RefPoint);

  /// Forget all the values before the current position in the iterator
  /// ```
  /// extern crate backtracking_iterator;
  /// use backtracking_iterator::{Record, BacktrackingIterator};
  ///
  /// let v = vec![1_u8, 2_u8];
  /// let mut rec = backtracking_iterator::BacktrackingRecorder::new(v.into_iter());
  /// {
  ///   let mut bt = rec.copying();
  ///   bt.next();
  /// }
  ///
  /// //Before we call this, 1_u8 is in the history
  /// rec.forget();
  ///
  /// {
  ///   let mut bt = rec.copying();
  ///   assert!(bt.next().unwrap() == 2_u8);
  /// }
  /// ```
  fn forget(&mut self) {
    let now = self.get_ref_point();
    self.forget_before(now);
  }
}

/// A trait for defining backtracking behaviour over
/// This serves to generify the copying and non-copying verions and their behaviour
pub trait BacktrackingIterator: Iterator {
  /// The type used to refer to positions in the history
  type RefPoint;

  /// Yield a reference to the current point in the history
  /// This reference must be valid for as long as the current
  /// point remains in the history
  fn get_ref_point(&self) -> Self::RefPoint;

  /// Yield a reference to the oldest point in the history
  fn get_oldest_point(&self) -> Self::RefPoint;

  /// Return to a given point in the history
  /// Doesn't have to do anything if the point is invalid
  /// ```
  /// extern crate backtracking_iterator;
  /// use backtracking_iterator::{BacktrackingIterator, Walkback, Walkbackable};
  ///
  /// let v = vec![1_u8, 2_u8, 3_u8];
  /// let mut rec = backtracking_iterator::BacktrackingRecorder::new(v.into_iter());
  /// let mut bt = rec.copying();
  /// bt.next(); // 1_u8
  /// bt.next(); // 2_u8
  /// let wb_pos = {
  ///   let mut wb = bt.walk_back();
  ///   assert!(wb.next().unwrap() == 2_u8);
  ///   wb.get_ref_point()
  /// };
  /// 
  /// bt.backtrack(wb_pos);
  /// assert!(bt.next().unwrap() == 2_u8);
  fn backtrack(&mut self, point: Self::RefPoint);

  /// Start the iterator again from all the elements in the current history
  /// The iterator will repeat every element which was emitted since the last
  /// call to `forget()`.
  ///
  /// ```
  /// extern crate backtracking_iterator;
  /// use backtracking_iterator::BacktrackingIterator;
  ///
  /// let v = vec![1_u8, 2_u8];
  /// let mut rec = backtracking_iterator::BacktrackingRecorder::new(v.into_iter());
  /// let mut bt = rec.copying();
  /// bt.next();
  /// bt.start_again();
  /// assert!(bt.next().unwrap() == 1_u8);
  /// ```
  fn start_again(&mut self) {
    let oldest = self.get_oldest_point();
    self.backtrack(oldest);
  }
}

/// A trait for an iterator that can be walked back on, parameterised for a lifetime
/// This trait is a workaround for the lack of generic associated types - it is expected
/// to be implemented `for` every lifetime, for reasons of utility
pub trait Walkbackable<'history> {
  type RefPoint;
  type Item;

  /// The type of the walk-back iterator 
  type Walkback: Walkback<'history, Item=Self::Item, RefPoint=Self::RefPoint>;

  /// Produce an iterator which goes back over the current history in reverse,
  /// and yields items in the history.
  /// ```
  /// extern crate backtracking_iterator;
  /// use backtracking_iterator::{BacktrackingIterator, Walkbackable};
  ///
  /// let v = vec![1_u8, 2_u8];
  /// let mut rec = backtracking_iterator::BacktrackingRecorder::new(v.into_iter());
  /// let mut bt = rec.copying();
  /// bt.next();
  ///
  /// let mut wb = bt.walk_back();
  ///
  /// assert!(wb.next().unwrap() == 1_u8);
  /// ```
  fn walk_back(&'history self) -> Self::Walkback;
}

/// A trait for walking back over a backtracking history
pub trait Walkback<'history>: Iterator {
  /// The type used to refer to positions in the history
  type RefPoint;

  /// Yield a reference to the current point in the history
  /// This reference must be valid in the parent BacktrackingIterator,
  /// and must remain valid until the next call to `next()` on this iterator
  fn get_ref_point(&self) -> Self::RefPoint;
}

