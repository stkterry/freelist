# Freelist

A contiguous, growable, first-fit freelist. 

Freelist has *O*(1) indexing and push (to the first available free slot) and *O*(1) removal.

## Examples
```rust
use fffl::Freelist;

let mut fl = Freelist::new();
let idx1 = fl.push(1);
let idx2 = fl.push(2);
let idx3 = fl.push(3);

assert_eq!(idx2, 1);

assert_eq!(fl.filled(), 3);
assert_eq!(fl[0], 1);

assert_eq!(fl.remove(1), Some(2));
assert_eq!(fl.remove(1), None);

assert_eq!(fl.filled(), 2);
assert_eq!(fl.free(), 1);
assert_eq!(fl.size(), 3);

let first_free_idx = fl.push(7);
assert_eq!(first_free_idx, 1);

fl[1] = 5;
assert_eq!(fl[1], 5);
```
## Indexing

The `Freelist` type allows access to values by index. 

```rust
use fffl::Freelist;

let fl = Freelist::from([1, 3, 5, 7]);
println!("{}", fl[1]); // displays '3'
```
*Note:* if you try to access an index which has been previously freed or isn't in the `Freelist`, the software will panic! Example:

```rust
use fffl::Freelist;

let mut fl = Freelist::from([1, 3, 5, 7]);
fl.remove(2);
println!("{}", fl[2]); // PANIC
```
Use `get` and `get_mut` if you want to check whether the index contains a value.

## Slicing
`Freelist` cannot be sliced.  Doing so would require internally building a slice that skips freed slots, meaning the returned slice may not be of the same length.  Moreover the apparent indices would be unordered. 

You may iterate over the entire `Freelist` via `iter`, `iter_mut`, or `into_iter`, all of which will skip over empty slots.

## Guarantees
`push` and `remove` are always *O*(1), maintain index order, and offer similar performance to `Vec`


## Pages
* [Crate](https://crates.io/crates/fffl)
* [Documentation](https://docs.rs/fffl/0.9.2/fffl/struct.Freelist.html)
* [Github](https://github.com/stkterry/freelist) (You are here)