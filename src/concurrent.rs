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

use super::{BacktrackingState, BacktrackingIterator};

use std::sync::{Arc, RwLock, Mutex};

static EXPECT_RW: &'static str = "The read-write lock on the history has been poisoned by a thread panic!";
static EXPECT_MUTEX: &'static str = "The mutual exclusion lock on the iterator has been poisoned by a thread panic!"; 


impl<'item, Iter> From<Iter> for ConcurrentReferencingBacktrackingIterator<'item, Iter> where Iter: Iterator, Iter: 'item {
  /// Create a `ConcurrentReferencingBacktrackingIterator` from an existing iterator.
  fn from(iterator: Iter) -> Self {
    ConcurrentReferencingBacktrackingIterator {
      item_marker: std::marker::PhantomData,
      iterator: Arc::new(Mutex::new(iterator)),
      backtracking_vec: Arc::new(RwLock::new(vec![])),
      position: 0,
    }
  }
}

pub struct ConcurrentReferencingBacktrackingIterator<'item, Iter> where Iter: Iterator, Iter: 'item {
  item_marker: std::marker::PhantomData<&'item Iter::Item>,
  iterator: Arc<Mutex<Iter>>,
  backtracking_vec: Arc<RwLock<Vec<Iter::Item>>>,
  position: usize,
}

impl<'item, Iter> Clone for ConcurrentReferencingBacktrackingIterator<'item, Iter> where Iter: Iterator, Iter: 'item {
  fn clone(&self) -> Self {
    ConcurrentReferencingBacktrackingIterator {
      item_marker: std::marker::PhantomData,
      iterator: self.iterator.clone(),
      backtracking_vec: self.backtracking_vec.clone(),
      position: self.position,
    }
  }
}

impl<'item, Iter> Iterator for ConcurrentReferencingBacktrackingIterator<'item, Iter> where Iter: Iterator, Iter: 'item {
  type Item = &'item Iter::Item;

  fn next(&mut self) -> Option<&'item Iter::Item> {
    macro_rules! unsafe_backtracking_index {
      ($index:expr) => {
        unsafe {
          &*(&self.backtracking_vec.read().expect(EXPECT_RW)[$index] as *const Iter::Item)
        }
      };
    }

    use crate::{Backtracking, Progressing};
    if self.position >= self.backtracking_vec.read().expect(EXPECT_RW).len() {
      if let Some(val) = self.iterator.lock().expect(EXPECT_MUTEX).next() {
        self.backtracking_vec.write().expect(EXPECT_RW).push(val);
        self.position = self.position + 1;
        Some(unsafe_backtracking_index!(self.backtracking_vec.read().expect(EXPECT_RW).len() - 1))
      } else {
        None
      }
    } else {
      let old_position = self.position;
      self.position = self.position + 1;
      Some(unsafe_backtracking_index!(old_position))
    }
  }
}

impl<'item, Iter> BacktrackingIterator for ConcurrentReferencingBacktrackingIterator<'item, Iter> where Iter: Iterator, Iter: 'item {
  type RefPoint = usize;

  fn get_ref_point(&self) -> usize {
    self.position
  }

  fn get_oldest_point(&self) -> usize {
    // Always the oldest position
    0_usize
  }

  fn backtrack(&mut self, position: usize) {
    self.position = position;
  }
}

//// COPYING VERSION

impl<Iter> From<Iter> for ConcurrentCopyingBacktrackingIterator<Iter> where Iter: Iterator, Iter::Item: Clone {
  /// Create a `ConcurrentCopyingBacktrackingIterator` from an existing iterator.
  fn from(iterator: Iter) -> Self {
    ConcurrentCopyingBacktrackingIterator {
      iterator: Arc::new(Mutex::new(iterator)),
      backtracking_vec: Arc::new(RwLock::new(vec![])),
      position: 0,
    }
  }
}

pub struct ConcurrentCopyingBacktrackingIterator<Iter> where Iter: Iterator, Iter::Item: Clone {
  iterator: Arc<Mutex<Iter>>,
  backtracking_vec: Arc<RwLock<Vec<Iter::Item>>>,
  position: usize,
}

impl<Iter> Clone for ConcurrentCopyingBacktrackingIterator<Iter> where Iter: Iterator, Iter::Item: Clone {
  fn clone(&self) -> Self {
    ConcurrentCopyingBacktrackingIterator {
      iterator: self.iterator.clone(),
      backtracking_vec: self.backtracking_vec.clone(),
      position: self.position,
    }
  }
}

impl<Iter> Iterator for ConcurrentCopyingBacktrackingIterator<Iter> where Iter: Iterator, Iter::Item: Clone {
  type Item = Iter::Item;

  fn next(&mut self) -> Option<Iter::Item> {
    
    use crate::{Backtracking, Progressing};
    if self.position >= self.backtracking_vec.read().expect(EXPECT_RW).len() {
      if let Some(val) = self.iterator.lock().expect(EXPECT_MUTEX).next() {
        self.backtracking_vec.write().expect(EXPECT_RW).push(val.clone());
        self.position = self.position + 1;
        Some(val)
      } else {
        None
      }
    } else {
      let old_position = self.position;
      self.position = self.position + 1;
      Some(self.backtracking_vec.read().expect(EXPECT_RW)[old_position].clone())
    }
  }
}

impl<Iter> BacktrackingIterator for ConcurrentCopyingBacktrackingIterator<Iter> where Iter: Iterator, Iter::Item: Clone {
  type RefPoint = usize;

  fn get_ref_point(&self) -> usize {
    self.position
  }

  fn get_oldest_point(&self) -> usize {
    // Always the oldest position
    0_usize
  }

  fn backtrack(&mut self, position: usize) {
    self.position = position;
  }
}


#[cfg(test)]
mod tests {
  #[test]
  fn many_iter_test() {
    use matches::{matches};

    let bt_con_iter = crate::concurrent::ConcurrentReferencingBacktrackingIterator::from(1..1000);
    
    for _ in 1..3 {
      let mut bt_iter = bt_con_iter.clone();
      for expected in 1..1000 {
        assert!(matches!(bt_iter.next(), Some(&i) if i == expected))
      }
    }
  }
  
  #[test]
  fn many_iter_clone_test() {
    use matches::{matches};

    let bt_con_iter = crate::concurrent::ConcurrentCopyingBacktrackingIterator::from(1..1000);
    
    for _ in 1..3 {
      let mut bt_iter = bt_con_iter.clone();
      for expected in 1..1000 {
        assert!(matches!(bt_iter.next(), Some(i) if i == expected))
      }
    }
  }
  
  #[test]
  fn dont_need_clone_test() {
    use matches::{matches};

    struct Uncloneable {};
    let uncloneables = vec![Uncloneable {}];
    let mut bt_con_iter = crate::concurrent::ConcurrentReferencingBacktrackingIterator::from(uncloneables.into_iter());

    assert!(matches!(bt_con_iter.next(), Some(&Uncloneable {})))
  }
}

