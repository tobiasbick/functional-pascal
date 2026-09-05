# Chapter 3 - Performance Mindset

The **golden rule** of performance work:

> Don't guess, measure.

Rust code is often already pretty fast - don't "optimize" without evidence. Optimize only after finding bottlenecks.

## 3.1 Measure FPAS workloads

Use the [FPAS benchmark skill](../../fpas-bench/SKILL.md) and the configured
[suite](../../../../docs/bench/suite.toml). Save a baseline before editing, compare
the same workload in release mode, and record settled results. Attribute CPU and
allocation costs to the current implementation before choosing a substantial
runtime rewrite. A microbenchmark result does not establish whole-app throughput.

## 3.2 Avoid Redundant Cloning

> Cloning is cheap... **until it isn't**

In sections [Borrowing over Cloning](./chapter_01.md#11-borrowing-over-cloning) and [Important Clippy lints to respect](./chapter_02.md#23-important-clippy-lints-to-respect) we mentioned the impacts of cloning and the relevant clippy lint [`redundant_clone`](https://rust-lang.github.io/rust-clippy/master/#redundant_clone), so in this section we will explore a bit "when to pass ownership".

* 🚨 If you really need to clone, leave it to the last moment.

### When to pass ownership?

* Only `.clone()` if you truly need a new owned copy. A few examples:
    * Crate API Design requires owned data.
    * Have overloaded `std::ops` but still need ownership to the old data:
    ```rust
    use std::ops::Add;

    #[derive(Debug, Copy, Clone, PartialEq)]
    struct Point {
        x: i32,
        y: i32,
    }

    impl Add for Point {
        type Output = Self;

        fn add(self, other: Self) -> Self {
            Self {
                x: self.x + other.x,
                y: self.y + other.y,
            }
        }
    }

    assert_eq!(Point { x: 1, y: 0 } + Point { x: 2, y: 3 },
               Point { x: 3, y: 3 });
    ```
    * Need to do comparison snapshots or due to API you need multiple owned instances of the data.
    ```rust
    fn snapshot(a: &MyValue, b:&MyValue) -> MyValueDiff {
        a - b
    }

    impl Sub for MyValue {
        type Output = MyValueDiff;

        fn sub(self, other: Self) -> MyValue {
            ...
        }
    }

    fn main() {
        let mut a = MyValue::default();
        let b = a.clone();

        a.magical_update();
        println!("{:?}", snapshot(&a, &b));
    }
    ```
* You have reference counted pointers (`Arc, Rc`).
* You have small structs that are to big to `Copy` but as costly as `std::collections`. An example is HTTP client like `hyper_util::client::legacy::Client` that cloning allows you to share the connection pool.
* You have a chained struct modifier that needs owned mutation, some **builders** require owned mutation, but most custom builders can be done with `pub fn with_xyz(&mut self, value: Xyz) -> &mut Self`.
```rust
// Inline `HashMap` insertion extension

fn insert_owned(mut self, key: K, value: V) -> Self {
    self.insert(key, value);
    self
}
```
* Ownership can also be a good way to model business logic / state. For example:
```rust
let not_validated: String = ...;// some user source
let validated = Validate::try_from(not_validated)?;
// Technically that `try_from` maybe didn't need ownership, but taking it lets us model intent
```

### When **NOT** to pass ownership?

* Prefer API designs that take reference (`fn process(values: &[T])`), instead of ownership (`fn process(values: Vec<T>)`).
* If you only need read access to elements, prefer `.iter` or slices:
```rust
for item in &some_vec {
    ...
}
```
* You need to mutate data that is owned by another thread, use `&mut MyStruct`.

### Use `Cow` for `Maybe Owned` data

Sometimes you don't actually need owned data, but that is not clear from the API perspective, so using [`std::borrow::Cow`](https://doc.rust-lang.org/std/borrow/enum.Cow.html) is a way to efficiently address this case:

```rust
use std::borrow::Cow;

fn hello_greet(name: Cow<'_, str>) {
    println!("Hello {name}");
}

hello_greet(Cow::Borrowed("Julia"));
hello_greet(Cow::Owned("Naomi".to_string()));
```

## 3.3 Stack vs Heap: Be size-smart!

### ✅ Good Practices 

* Keep small types (`impl Copy`, `usize`, `bool`, etc) **on the stack**.
* Avoid passing huge types (`> 512 bytes`) by value or transferring ownership. Prefer pass by reference (e.g. `&T` and `&mut T`).
* Heap allocate recursive data structures:
```rust
enum OctreeNode<T> {
    Node(T),
    Children(Box<[Node<T>; 8]>),
}
```
* Return small types by value, types that implement `Copy` or a cheaply Cloned are efficient to return by value (e.g. `struct Vector2 {x: f32, y: f32}`).

### ❗ Be Mindful

* Only use `#[inline]` when benchmark proves beneficial, Rust is already pretty good at inlining **without** hints.
* Avoid massive stack allocations, box them. Example `let buffer: Box<[u8; 65536]> = Box::new(..)` would first allocate `[u8; 65536]` on the stack then box it, a non-const solution to this would be `let buffer: Box<[u8]> = vec![0; 65536].into_boxed_slice()`.
* For large `const` arrays, considering using [crate smallvec](https://docs.rs/smallvec/latest/smallvec/) as it behaves like an array, but is smart enough to allocate large arrays on the heap.

## 3.4 Iterators and Zero-Cost Abstractions

Rust iterators are lazy, but eventually compiled away into very efficient tight loops that are only called when consumed. Chaining `.filter()`, `.map()`, `.rev()`, `.skip()`, `.take()`, `.collect()` usually doesn't cost extra and the compiler can reason well enough how to optimize them.
* Prefer `iterators` over manual `for` loops when working with collections, the compiler can optimize them better than manually doing it.
* Calling `.iter()` only creates a **reference** to the original collection, this allows you to hold multiple iterators of the same collection.

#### ❗ Avoid creating intermediate collections unless it is really needed:

* Consider that `process` accepts an `iterator`.
* ❌ BAD - useless intermediate collection:
```rust
let doubled: Vec<_> = items.iter().map(|x| x * 2).collect();
process(doubled);
```
* ✅ GOOD - pass the iterator (`fn process(arg: impl Iterator<Item = T>)`):
```rust
let doubled_iter = items.iter().map(|x| x * 2);
process(doubled_iter);
```
