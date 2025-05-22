A contiguous, growable, first-fit freelist. 

Freelist has *O*(1) indexing and amortized *O*(1) push (to the first available free slot) and *O*(1) removal.

# Examples

You can initialize an empty list with [`Freelist::new`] or populate one from an existing vector, array, or iterator using [`From`].
```
use fffl::Freelist;

let fl1: Freelist<i32> = Freelist::new();
let fl2: Freelist<i32> = Freelist::from(vec![0, 1, 2]);

```

You can use [`push`] to insert items into the freelist, while [`remove`] allows you to get them back.
```
use fffl::Freelist;

let mut fl: Freelist<i32> = Freelist::from([1, 2, 3, 4]);

let removed: Option<i32> = fl.remove(2);
assert_eq!(removed, Some(3));

let idx_1: usize = fl.push(8);
let idx_2: usize = fl.push(9);

assert_eq!(idx_1, 2);
assert_eq!(idx_2, 4);

```

Freelists support indexing (through the [`Index`] and [`IndexMut`] traits) but will panic if indexed at empty slots.  You can use [`get`] and [`get_mut`] to safely retrieve [`Option<T>`] instead.

You may also use [`iter`], [`iter_mut`], and [`into_iter`], which will skip over empty slots.  Do note that these functions are *not* provided via their respective traits.


[`Option`]: std::option::Option
['Freelist::new`]: Freelist::new
[`Freelist`]: Freelist
[`new`]: Freelist::new
[`push`]: Freelist::push
[`next_available`]: Freelist::next_available
[`remove`]: Freelist::remove
[`len`]: Freelist::len
[`size`]: Freelist::size
[`free`]: Freelist::free
[`clear`]: Freelist::clear
[`reserve`]: Freelist::reserve
[`get`]: Freelist::get
[`get_mut`]: Freelist::get_mut
[`get_unchecked`]: Freelist::get_unchecked
[`get_unchecked_mut`]: Freelist::get_unchecked_mut
[`iter`]: Freelist::iter
[`iter_mut`]: Freelist::iter_mut
[`into_iter`]: Freelist::into_iter