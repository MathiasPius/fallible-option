//! [`Errable`](crate::Errable) is an [`Option`](::core::option::Option) with inverted [`Try`](https://doc.rust-lang.org/stable/core/ops/trait.Try.html#)-semantics.
//!
//! What this means is that using the `?` operator on a `Errable<E>` will exit early
//! if an error `E` is contained within, or instead act as a no-op, if the value is `Success`.
//!
//! This is in contrast to `Option` where using `?` on a `None`-value will exit early.
//!
//! `Errable` fills the gap left by the [`Result`](::core::result::Result) and [`Option`](::core::option::Option) types:
//!
//! |   Potential Success | Potential Failure |
//! |---------------------|-------------------|
//! |          `Result<T` | `, E>`            |
//! |     `Option<T>`     | **`Errable<E>`**  |
//!
//! # Example
//! This code illustrates how `Errable` can be used to write succint
//! validation code which exits early in case of failure.
//!
//! ```rust
//! use errable::Errable::{self, Fail, Success};
//!
//! # fn test_chained_failures() {
//! // Validates the input number `n`, returning a `Fail`
//! // if the input number is zero, or `Success` otherwise.
//! fn fails_if_number_is_zero(n: u32) -> Errable<&'static str> {
//!     if n == 0 {
//!         Fail("number is zero")
//!     } else {
//!         Success
//!     }
//! };
//!
//! // Check many numbers, returning early if a tested
//! // number is equal to zero.
//! fn check_many_numbers() -> Errable<&'static str> {
//!     fails_if_number_is_zero(1)?;
//!     fails_if_number_is_zero(3)?;
//!     fails_if_number_is_zero(0)?; // <--- Will cause early exit
//!
//!     // Following lines are never reached
//!     fails_if_number_is_zero(10)?;
//!     
//!     Success
//! }
//!
//! assert_eq!(check_many_numbers(), Errable::Fail("number is zero"));
//! # }
//! ```
//!
//! # Motivation
//! `Errable` fills the gap left by `Option` and `Result` and clearly conveys intent and potential outcomes of a function.
//!
//! ### Why not `Result`?
//! Because `Result` implies output. Take `std::fs::rename` for instance:
//!
//! If I told you that the return type of `rename` was a `Result<T, E>`, what would you guess `T` and `E` to be?
//!
//! You might rightly assume that `E` was `std::io::Error`, but what about `T`? It could reasonably return any number of things:
//! * The canonical path of the destination of the renamed file.
//! * The size of the moved file.
//! * The size of the file (if any) replaced by the renamed file.
//! * Or perhaps even a handle to the overwritten file.
//!
//! Of course none of these are true, as the `T` value of `rename` is the unit value `()`. `rename` never
//! produces any output, it can only signal errors. So why not signal that clearly to the user?
//!
//! I would argue that using a type which signals the potential for failure, but no output upon success would
//! more clearly express the intent and potential outcomes when using this function.
//!
//! ### Why not `Option`?
//! Potential failure *could* be expressed using an `Option<E>`, but as stated above, the `Try`-semantics
//! of `Option` makes it unergonomic to work with:
//!
//! ```rust
//! type Error = &'static str;
//!
//! fn fails_if_number_is_zero(n: u32) -> Option<Error> {
//!     if n == 0 {
//!         Some("number is zero")
//!     } else {
//!         None
//!     }
//! };
//!
//! fn check_many_numbers() -> Option<Error> {
//!     // We have to explicitly check, since using `?` here would result in an early exit,
//!     // if the call returned None, which is the opposite of what we intend.
//!     if let Some(err) = fails_if_number_is_zero(1) {
//!         return Some(err)
//!     }
//!
//!     // .. Repeating the above three lines for each check is tedious compared to
//!     // just using the `?` operator, as in the example.
//!
//!     None
//! }
//! ```
//!
//! # Conversion from `Result`
//! Switching from using `Result` to `Errable` is very simple, as illustrated with this before/after example:
//!
//! ```rust
//! fn validate_number(x: u32) -> Result<(), &'static str> {
//!     match x {
//!         0 ..= 9 => Err("number is too small"),
//!         10..=30 => Ok(()),
//!         31..    => Err("number is too large")
//!     }
//! }
//! ```
//! Using `Errable`:
//!
//! ```rust
//! # use errable::Errable::{self, Fail, Success};
//! fn validate_number(x: u32) -> Errable<&'static str> {
//!     match x {
//!         0 ..= 9 => Fail("number is too small"),
//!         10..=30 => Success,
//!         31..    => Fail("number is too large")
//!     }
//! }
//! ```
//! # Compatibility
//!
//! `Errable` contains utility functions for mapping to and from [`Result`] and [`Option`],
//! as well as [`FromResidual`] implementations for automatically performing these conversions
//! when used with the `?` operator.
//! ```rust
//! # use errable::Errable::{self, Fail, Success};
//! fn fails_if_true(should_fail: bool) -> Errable<&'static str> {
//!     if should_fail {
//!         Fail("Darn it!")
//!     } else {
//!         Success
//!     }
//! }
//!
//! fn try_producing_value() -> Result<u32, &'static str> {
//!     fails_if_true(false)?;
//!     fails_if_true(true)?;
//!
//!     Ok(10)
//! }
//! ```

#![no_std]
#![deny(
    bad_style,
    dead_code,
    improper_ctypes,
    non_shorthand_field_patterns,
    no_mangle_generic_items,
    overflowing_literals,
    path_statements,
    patterns_in_fns_without_body,
    private_in_public,
    unconditional_recursion,
    unused,
    unused_allocation,
    unused_comparisons,
    unused_parens,
    while_true,
    missing_debug_implementations,
    missing_docs,
    trivial_casts,
    trivial_numeric_casts,
    unused_extern_crates,
    unused_import_braces,
    unused_qualifications,
    unused_results
)]
#![forbid(unsafe_code)]
#![feature(try_trait_v2)]
#![feature(const_trait_impl)]
#![feature(const_mut_refs)]
#![feature(const_replace)]
use core::convert::Infallible;
use core::fmt::Debug;
use core::marker::Destruct;
use core::mem;
use core::ops::{ControlFlow, Deref, DerefMut, FromResidual, Try};

/// Outcome of an operation that does not produce a value on success.
#[must_use]
#[derive(Copy, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
pub enum Errable<E> {
    /// No error was produced.
    Success,
    /// An error was produced.
    Fail(E),
}

use Errable::{Fail, Success};

impl<E> Errable<E> {
    /// Converts from `Errable<E>` (or `&Errable<E>`) to `Errable<&E::Target>`.
    ///
    /// Leaves the original `Errable` in-place, creating a new one with a reference
    /// to the original one, additionally coercing the contents via [`Deref`].
    #[inline]
    pub const fn as_deref(&self) -> Errable<&<E as Deref>::Target>
    where
        E: ~const Deref,
    {
        match self {
            Success => Success,
            Fail(e) => Fail(e.deref()),
        }
    }

    /// Converts from `Errable<E>` (or `&mut Errable<E>`) to `Errable<&mut E::Target>`
    ///
    /// Leaves the original `Errable` in-place, creating a new one containing a mutable reference to
    /// the inner type's [`Deref::Target`] type.
    #[inline]
    pub const fn as_deref_mut(&mut self) -> Errable<&mut <E as Deref>::Target>
    where
        E: ~const DerefMut,
    {
        match self {
            Success => Success,
            Fail(e) => Fail(e.deref_mut()),
        }
    }

    /// Converts from `&mut Errable<E>` to `Errable<&mut E>`
    ///
    #[inline]
    pub const fn as_mut(&mut self) -> Errable<&mut E> {
        match self {
            Success => Success,
            Fail(ref mut e) => Fail(e),
        }
    }

    /// Converts from `&Errable<E>` to `Errable<&E>`
    #[inline]
    pub const fn as_ref(&self) -> Errable<&E> {
        match self {
            Success => Success,
            Fail(ref e) => Fail(e),
        }
    }

    /// Returns true if the value is a `Success`, otherwise false.
    #[inline]
    pub const fn is_successful(&self) -> bool {
        matches!(self, Success)
    }

    /// Returns true if the value is a `Fail`, otherwise false.
    #[inline]
    pub const fn is_fail(&self) -> bool {
        matches!(self, Fail(_))
    }

    /// Unwrap the contained error or panics if no error has occurred.
    #[inline]
    pub fn unwrap_fail(self) {
        match self {
            Success => panic!("called `Errable::unwrap_fail()` on a `Errable::Success` value"),
            Fail(_) => (),
        }
    }

    /// Returns `true` if the Errable is a `Fail` value containing an error
    /// equivalent to `f`
    #[inline]
    pub const fn contains<F: ~const PartialEq<E>>(&self, f: &F) -> bool {
        match self {
            Success => false,
            Fail(e) => f.eq(e),
        }
    }

    /// Maps a `Errable<E>` to `Errable<F>` by applying a function
    /// to the contained error.
    #[inline]
    pub const fn map<F, O>(self, op: O) -> Errable<F>
    where
        O: ~const FnOnce(E) -> F,
        O: ~const Destruct,
        E: ~const Destruct,
    {
        match self {
            Success => Success,
            Fail(e) => Fail(op(e)),
        }
    }

    /// Transforms the `Errable<E>` into a [`Result<(), E>`], where `Fail(e)`
    /// becomes `Err(e)` and `Success` becomes `Ok(())`
    #[inline]
    pub const fn result(self) -> Result<(), E>
    where
        E: ~const Destruct,
    {
        match self {
            Success => Ok(()),
            Fail(e) => Err(e),
        }
    }

    /// Borrows the `Errable<E>` as an [`Option<E>`], yielding none
    /// if no error occurred.
    #[inline]
    pub const fn err(&self) -> Option<&E> {
        match self {
            Success => None,
            Fail(err) => Some(err),
        }
    }

    /// Constructs a [`Result<T, E>`] from self.
    ///
    /// `Fail(e)` becomes `Err(e)` and `Success` becomes `Ok(value)`
    #[inline]
    pub const fn err_or<T>(self, value: T) -> Result<T, E>
    where
        E: ~const Destruct,
        T: ~const Destruct,
    {
        match self {
            Success => Ok(value),
            Fail(e) => Err(e),
        }
    }

    /// Replaces the contained error (if any) with None,
    /// and returns an [`Option<E>`] with the contained error,
    /// if the outcome was `Fail`.
    #[inline]
    pub const fn take(&mut self) -> Option<E>
    where
        E: ~const Destruct,
    {
        match mem::replace(self, Success) {
            Success => None,
            Fail(e) => Some(e),
        }
    }
}

impl<E> Errable<&E>
where
    E: ~const Clone,
{
    /// Maps a `Errable<&E>` to a [`Errable<E>`] by cloning the contents of the
    /// error.
    #[inline]
    #[must_use = "`self` will be dropped if the result is not used"]
    pub const fn cloned(self) -> Errable<E> {
        match self {
            Success => Success,
            Fail(e) => Fail(e.clone()),
        }
    }

    /// Maps an `Errable<&E>` to an `Errable<E>` by copying the contents of the
    /// error.
    #[inline]
    #[must_use = "`self` will be dropped if the result is not used"]
    pub const fn copied(self) -> Errable<E>
    where
        E: Copy,
    {
        match self {
            Success => Success,
            Fail(&e) => Fail(e),
        }
    }
}

impl<E> Errable<&mut E>
where
    E: ~const Clone,
{
    /// Maps an `Errable<&E>` to an `Errable<E>` by cloning the contents of the
    /// error.
    #[inline]
    #[must_use = "`self` will be dropped if the result is not used"]
    pub const fn cloned(self) -> Errable<E> {
        match self {
            Success => Success,
            Fail(e) => Fail(e.clone()),
        }
    }

    /// Maps an `Errable<&E>` to an `Errable<E>` by copying the contents of the
    /// error.
    #[inline]
    #[must_use = "`self` will be dropped if the result is not used"]
    pub const fn copied(self) -> Errable<E>
    where
        E: Copy,
    {
        match self {
            Success => Success,
            Fail(&mut e) => Fail(e),
        }
    }
}

/// The following functions are only available if the generic parameter `E` implements [`Debug`]
impl<E> Errable<E>
where
    E: Debug,
{
    /// Returns a unit value if the `Errable` is not `Fail`.
    ///
    /// # Panics
    /// Panics if the value is a `Fail`, with a panic message including
    /// the content of the `Fail`.
    #[inline]
    pub fn unwrap(self) {
        match self {
            Success => (),
            Fail(e) => {
                panic!("called `Errable::unwrap()` on a `Errable::Fail` value: {e:?}")
            }
        }
    }
}

impl<E> Errable<Errable<E>> {
    /// Flattens a `Errable<Errable<E>>` into a `Errable<E>`
    #[inline]
    pub const fn flatten(self) -> Errable<E>
    where
        E: ~const Destruct,
    {
        match self {
            Success => Success,
            Fail(inner) => inner,
        }
    }
}

impl<E> const From<E> for Errable<E> {
    #[inline]
    fn from(value: E) -> Self {
        Fail(value)
    }
}

impl<T, E> const From<Result<T, E>> for Errable<E>
where
    E: ~const Destruct,
    T: ~const Destruct,
{
    #[inline]
    fn from(value: Result<T, E>) -> Self {
        match value {
            Ok(_) => Success,
            Err(e) => Fail(e),
        }
    }
}

impl<'a, E> const From<&'a Errable<E>> for Errable<&'a E> {
    #[inline]
    fn from(value: &'a Errable<E>) -> Self {
        value.as_ref()
    }
}

impl<'a, E> const From<&'a mut Errable<E>> for Errable<&'a mut E> {
    #[inline]
    fn from(value: &'a mut Errable<E>) -> Self {
        value.as_mut()
    }
}

impl<E> const Default for Errable<E> {
    #[inline]
    fn default() -> Self {
        Success
    }
}

impl<E> const Clone for Errable<E>
where
    E: ~const Clone + ~const Destruct,
{
    #[inline]
    fn clone(&self) -> Self {
        match self {
            Fail(x) => Fail(x.clone()),
            Success => Success,
        }
    }

    #[inline]
    fn clone_from(&mut self, source: &Self) {
        match (self, source) {
            (Fail(to), Fail(from)) => to.clone_from(from),
            (to, from) => *to = from.clone(),
        }
    }
}

impl<E> Try for Errable<E> {
    type Output = ();
    type Residual = Errable<E>;

    #[inline]
    fn from_output(_: Self::Output) -> Self {
        Success
    }

    #[inline]
    fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
        match self {
            Success => ControlFlow::Continue(()),
            Fail(e) => ControlFlow::Break(Fail(e)),
        }
    }
}

impl<E> FromResidual<Errable<E>> for Errable<E> {
    #[inline]
    fn from_residual(residual: Errable<E>) -> Self {
        residual
    }
}

impl<T, E> FromResidual<Errable<E>> for Result<T, E> {
    #[inline]
    fn from_residual(residual: Errable<E>) -> Self {
        match residual {
            Success => unreachable!(),
            Fail(e) => Err(e),
        }
    }
}

impl<E> FromResidual<Result<(), E>> for Errable<E> {
    #[inline]
    fn from_residual(residual: Result<(), E>) -> Self {
        match residual {
            Ok(()) => Success,
            Err(e) => Fail(e),
        }
    }
}

impl<E> FromResidual<Result<Infallible, E>> for Errable<E> {
    #[inline]
    fn from_residual(residual: Result<Infallible, E>) -> Self {
        match residual {
            Ok(_) => Success,
            Err(e) => Fail(e),
        }
    }
}
