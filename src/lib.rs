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

/// Module where trait behaviour is defined
mod traits;
pub use self::traits::*;

/// An internal enum for representing history
pub(crate) enum BacktrackingState {
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
use crate::BacktrackingState::*;

mod record;
pub use self::record::*;

/// The copying backtracking iterator module
mod copying;
pub use self::copying::*;

mod referencing;
pub use self::referencing::*;

#[cfg(test)]
mod tests {
  #[test]
  fn basic_test() {
    use crate::{BacktrackingIterator};

    let num_vec = vec![1_u8, 2, 3, 4, 5, 6];
    let vec_iter = num_vec.into_iter();
    let mut bt_rec = crate::BacktrackingRecorder::new(vec_iter);
    let mut bt_iter = bt_rec.copying();
    assert!(bt_iter.next().unwrap() == 1_u8);
    assert!(bt_iter.next().unwrap() == 2_u8);

    bt_iter.start_again();
    assert!(bt_iter.next().unwrap() == 1_u8);
    assert!(bt_iter.next().unwrap() == 2_u8);

    bt_iter.forget();
    bt_iter.start_again();
    assert!(bt_iter.next().unwrap() == 3_u8);
    assert!(bt_iter.next().unwrap() == 4_u8);
    assert!(bt_iter.next().unwrap() == 5_u8);
    assert!(bt_iter.next().unwrap() == 6_u8);
    assert!(!bt_iter.next().is_some());

    bt_iter.start_again();
    assert!(bt_iter.next().unwrap() == 3_u8);
  }

  #[test]
  fn backwalk_test() {
    use crate::{Walkbackable};
    let num_vec = vec![1_u8, 2, 3, 4, 5, 6];
    let vec_iter = num_vec.into_iter();
    let mut bt_rec = crate::BacktrackingRecorder::new(vec_iter);
    let mut bt_iter = bt_rec.copying();

    for _ in 1..=6 {
        bt_iter.next();
    }

    let mut wb = bt_iter.walk_back();
    for i in 1_u8..=6 {
      assert!(wb.next().unwrap() == (7 - i));
    }
  }
}
