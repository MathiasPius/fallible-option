# Errable

<!-- cargo-rdme start -->

[`Errable`](https://docs.rs/errable/latest/errable/enum.Errable.html) is an [`Option`](https://doc.rust-lang.org/stable/core/option/enum.Option.html) with inverted [`Try`](https://doc.rust-lang.org/stable/core/ops/trait.Try.html#)-semantics.

What this means is that using the `?` operator on a `Errable<E>` will exit early
if an error `E` is contained within, or instead act as a no-op, if the value is `Success`.

This is in contrast to `Option` where using `?` on a `None`-value will exit early.

`Errable` fills the gap left by the [`Result`](https://doc.rust-lang.org/stable/core/result/enum.Result.html) and [`Option`](https://doc.rust-lang.org/stable/core/option/enum.Option.html) types:

|   Potential Success | Potential Failure |
|---------------------|-------------------|
|          `Result<T` | `, E>`            |
|     `Option<T>`     | **`Errable<E>`**  |

### Example
This code illustrates how `Errable` can be used to write succint
validation code which exits early in case of failure.

```rust
use errable::Errable::{self, Fail, Success};

// Validates the input number `n`, returning a `Fail`
// if the input number is zero, or `Success` otherwise.
fn fails_if_number_is_zero(n: u32) -> Errable<&'static str> {
    if n == 0 {
        Fail("number is zero")
    } else {
        Success
    }
};

// Check many numbers, returning early if a tested
// number is equal to zero.
fn check_many_numbers() -> Errable<&'static str> {
    fails_if_number_is_zero(1)?;
    fails_if_number_is_zero(3)?;
    fails_if_number_is_zero(0)?; // <--- Will cause early exit

    // Following lines are never reached
    fails_if_number_is_zero(10)?;
    
    Success
}

assert_eq!(check_many_numbers(), Errable::Fail("number is zero"));
```

### Motivation
`Errable` fills the gap left by `Option` and `Result` and clearly conveys intent and potential outcomes of a function.

A function which returns `Errable` has only two potential outcomes, it can fail with an error `E`, or it can succeed.

#### Why not `Result`?
Because `Result` implies output. Take `std::fs::rename` for instance:

If I told you that the return type of `rename` was a `Result<T, E>`, what would you guess `T` and `E` to be?

You might rightly assume that `E` was `std::io::Error`, but what about `T`? It could reasonably return any number of things:
* The canonical path of the destination of the renamed file.
* The size of the moved file.
* The size of the file (if any) replaced by the renamed file.
* Or perhaps even a handle to the overwritten file.

Of course none of these are true, as the `T` value of `rename` is the unit value `()`. `rename` never
produces any output, it can only signal errors. So why not signal that clearly to the user?

I would argue that using a type which signals the potential for failure, but no output upon success would
more clearly express the intent and potential outcomes when using this function.

#### Why not `Option`?
Potential failure *could* be expressed using an `Option<E>`, but as stated above, the `Try`-semantics
of `Option` makes it unergonomic to work with:

```rust
type Error = &'static str;

fn fails_if_number_is_zero(n: u32) -> Option<Error> {
    if n == 0 {
        Some("number is zero")
    } else {
        None
    }
};

fn check_many_numbers() -> Option<Error> {
    // We have to explicitly check, since using `?` here would result in an early exit,
    // if the call returned None, which is the opposite of what we intend.
    if let Some(err) = fails_if_number_is_zero(1) {
        return Some(err)
    }

    // .. Repeating the above three lines for each check is tedious compared to
    // just using the `?` operator, as in the example.

    None
}
```

### Conversion from `Result`
Switching from using `Result` to `Errable` is very simple, as illustrated with this before/after example:

```rust
fn validate_number(x: u32) -> Result<(), &'static str> {
    match x {
        0 ..= 9 => Err("number is too small"),
        10..=30 => Ok(()),
        31..    => Err("number is too large")
    }
}
```
Using `Errable`:

```rust
fn validate_number(x: u32) -> Errable<&'static str> {
    match x {
        0 ..= 9 => Fail("number is too small"),
        10..=30 => Success,
        31..    => Fail("number is too large")
    }
}
```
### Compatibility

`Errable` contains utility functions for mapping to and from [`Result`] and [`Option`],
as well as [`FromResidual`] implementations for automatically performing these conversions
when used with the `?` operator.
```rust
fn fails_if_true(should_fail: bool) -> Errable<&'static str> {
    if should_fail {
        Fail("Darn it!")
    } else {
        Success
    }
}

fn try_producing_value() -> Result<u32, &'static str> {
    fails_if_true(false)?;
    fails_if_true(true)?;

    Ok(10)
}
```

<!-- cargo-rdme end -->
