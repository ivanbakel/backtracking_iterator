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

use crate::BacktrackingIterator;

use std::ops::RangeBounds;


/// A backtracking iterator which represents history in a way that makes it 
/// possible to produce slices over ranges of `RefPoint`s.
pub trait SliceableIterator: BacktrackingIterator {
  type Slice: ?Sized;

  /// Produce a slice corresponding to the given range.
  ///   * If the starting bound is `Unbounded`, behaviour must be equivalent to if it
  /// were set to the value of `get_oldest_point()`.
  ///   * If the end bound is `Unbounded`, behaviour must be that the slice contains
  /// at least up to the element corresponding last-most `RefPoint` obtainable from 
  /// the `BacktrackingIterator` - the slice may or may not contain more elements.
  fn slice(&self, range: impl RangeBounds<Self::RefPoint>) -> Option<&Self::Slice>;
}

/// A generic `Index` representation would conflict with one provided by Rust, and
/// the given function is unimplementable for trait objects, so this macro provides
/// an easy way to produce an `Index` impl that just calls `slice()` as expected.
///
/// If the slice is out of bounds, or for any other reason `slice()` returns None,
/// the implementation panics.
#[macro_export]
macro_rules! sliceable_indexing {
  (<$($parameter:tt),*>, $a_type:ty) => {
    impl<$($parameter),*, RefPoint, Range: ::std::ops::RangeBounds<RefPoint>> ::std::ops::Index<Range> for $a_type where Self: SliceableIterator<RefPoint=RefPoint> {
      type Output = <Self as SliceableIterator>::Slice;
    
      fn index(&self, range: Range) -> &Self::Output {
        if let Some(slice) = self.slice(range) {
          slice
        } else {
          panic!("Could not slice history: the range given was out of bounds!")
        }
      }
    }
  }
}

