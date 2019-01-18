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

// A trait which generifies indexing. The trait is stable, but its methods
// are not - they are enabled by a feature attribute in the crate.
use std::slice::SliceIndex;

use crate::traits::BacktrackingIterator;

/// A back-and-forth traversal over an indexable slice. The logic assumes that the
/// slice indexing begins at 0, and increases by 1. If the index increases outside 
/// the bounds of the slice, it will not become constant.
/// ```
/// # extern crate backtracking_iterator;
/// # use backtracking_iterator::{BacktrackingIterator, BacktrackingSlice};
/// let vec = vec![true, false];
/// let slice = &vec[..];
/// let mut backtracking_slice = BacktrackingSlice::from(slice);
///
/// assert!(backtracking_slice.next().unwrap() == &true);
/// assert!(backtracking_slice.next().unwrap() == &false);
/// assert!(backtracking_slice.next().is_none());
/// backtracking_slice.start_again();
/// assert!(backtracking_slice.next().unwrap() == &true);
/// ```
pub struct BacktrackingSlice<'slice, Slice> where Slice: ?Sized {
  slice: &'slice Slice,
  current_position: usize,
}

impl<'slice, Slice: ?Sized> From<&'slice Slice> for BacktrackingSlice<'slice, Slice> {
  fn from(slice: &'slice Slice) -> Self {
    BacktrackingSlice {
      slice: slice,
      current_position: 0_usize,
    }
  }
}

impl<'slice, Slice: ?Sized> Iterator for BacktrackingSlice<'slice, Slice> where usize: SliceIndex<Slice> {
  type Item = &'slice <usize as SliceIndex<Slice>>::Output;

  fn next(&mut self) -> Option<Self::Item> {
    let value = self.current_position.get(self.slice);
    self.current_position += 1;
    value
  }
}

impl<'slice, Slice: ?Sized> BacktrackingIterator for BacktrackingSlice<'slice, Slice> where usize: SliceIndex<Slice> {
  type RefPoint = usize;

  fn get_ref_point(&self) -> usize {
    self.current_position
  }

  fn get_oldest_point(&self) -> usize {
    0_usize
  }

  fn backtrack(&mut self, point: usize) {
    self.current_position = point;
  }
}

use std::ops::{RangeBounds, Range, RangeFull, RangeFrom, RangeInclusive, RangeTo, RangeToInclusive};

use crate::sliceable::SliceableIterator;

impl<'slice, Slice: ?Sized> SliceableIterator for BacktrackingSlice<'slice, Slice> where 
  usize: SliceIndex<Slice>, 
  Range<usize>: SliceIndex<Slice, Output=Slice>, 
  RangeFull: SliceIndex<Slice, Output=Slice>,
  RangeFrom<usize>: SliceIndex<Slice, Output=Slice>,
  RangeInclusive<usize>: SliceIndex<Slice, Output=Slice>,
  RangeTo<usize>: SliceIndex<Slice, Output=Slice>,
  RangeToInclusive<usize>: SliceIndex<Slice, Output=Slice>,
{
  type Slice = Slice;

  fn slice(&self, range: impl RangeBounds<usize>) -> Option<&Slice> {
    use std::ops::Bound::*;

    match (range.start_bound(), range.end_bound()) {
      (Unbounded, Unbounded) => (..).get(self.slice),
      (Unbounded, Included(&end)) => (..=end).get(self.slice),
      (Unbounded, Excluded(&end)) => (..end).get(self.slice),
      (Included(&start), Unbounded) => (start..).get(self.slice),
      (Excluded(&start), Unbounded) => ((start+1)..).get(self.slice),
      (Included(&start), Included(&end)) => (start..=end).get(self.slice),
      (Included(&start), Excluded(&end)) => (start..end).get(self.slice),
      (Excluded(&start), Included(&end)) => (start+1..=end).get(self.slice),
      (Excluded(&start), Excluded(&end)) => (start+1..end).get(self.slice),
    }
  }
}

sliceable_indexing!(<'slice, Slice>, BacktrackingSlice<'slice, Slice>);

