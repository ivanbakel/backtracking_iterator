# Backtracking Iterators

![Documentation status](https://docs.rs/backtracking_iterator/badge.svg)

A wrapper around existing iterators to extend them with backtracking functionality by providing an in-memory history.

In order to create a backtracking iterator on top of an existing iterator, you first wrap it in a `BacktrackingRecord`. From there, you have two choices of `BacktrackingIterator`:
 * `Copying`, which produces memory clones of the iterator items
 * `Referencing`, which produces immutable borrows on iterator items

The behaviour comes from the `BacktrackingIterator` trait.

## Example

    use backtracking_iterator::{BacktrackingIterator, BacktrackingRecord};

    let mut backtracking_record = BacktrackingRecord::new(my_iter);
    let mut my_backtracking_iter = backtracking_record.copying();

    // Now we can call `next()`, and the result will also be copied
    let here = my_backtracking_iter.get_ref_point();
    let fresh = my_backtracking_iter.next();
    
    my_backtracking_iter.backtrack(here);
    let remembered = my_backtracking_iter.next();
    
    assert!(fresh == remembered);

