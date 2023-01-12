//! [`Fallible`](crate::Fallible) is an [`Option`](::core::option::Option) with inverted [`Try`](https://doc.rust-lang.org/stable/core/ops/trait.Try.html#)-semantics.
//!
//! What this means is that using the `?` operator on a `Fallible<E>` will exit early
//! if an error `E` is contained within, or instead act as a no-op, if the value is `Success`.
//!
//! This is in contrast to `Option` where using `?` on a `None`-value will exit early.
//!
//! `Fallible` fills the gap left by the [`Result`](::core::result::Result) and [`Option`](::core::option::Option) types:
//!
//! |   Potential Success | Potential Failure |
//! |---------------------|-------------------|
//! |          `Result<T` | `, E>`            |
//! |     `Option<T>`     | **`Fallible<E>`**  |
//!
//! ## Example
//! This code illustrates how `Fallible` can be used to write succint
//! validation code which exits early in case of failure.
//!
//! ```rust
//! use fallible_option::Fallible::{self, Fail, Success};
//!
//! # fn test_chained_failures() {
//! // Validates the input number `n`, returning a `Fail`
//! // if the input number is zero, or `Success` otherwise.
//! fn fails_if_number_is_zero(n: u32) -> Fallible<&'static str> {
//!     if n == 0 {
//!         Fail("number is zero")
//!     } else {
//!         Success
//!     }
//! };
//!
//! // Check many numbers, returning early if a tested
//! // number is equal to zero.
//! fn check_many_numbers() -> Fallible<&'static str> {
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
//! assert_eq!(check_many_numbers(), Fallible::Fail("number is zero"));
//! # }
//! ```
//!
//! ## Motivation
//! `Fallible` fills the gap left by `Option` and `Result` and clearly conveys intent and potential outcomes of a function.
//!
//! A function which returns `Fallible` has only two potential outcomes, it can fail with an error `E`, or it can succeed.
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
//! ## Conversion from `Result`
//! Switching from using `Result` to `Fallible` is very simple, as illustrated with this before/after example:
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
//! Using `Fallible`:
//!
//! ```rust
//! # use fallible_option::Fallible::{self, Fail, Success};
//! fn validate_number(x: u32) -> Fallible<&'static str> {
//!     match x {
//!         0 ..= 9 => Fail("number is too small"),
//!         10..=30 => Success,
//!         31..    => Fail("number is too large")
//!     }
//! }
//! ```
//! ## Compatibility
//!
//! `Fallible` contains utility functions for mapping to and from [`Result`] and [`Option`],
//! as well as [`FromResidual`] implementations for automatically performing these conversions
//! when used with the `?` operator.
//! ```rust
//! # use fallible_option::Fallible::{self, Fail, Success};
//! fn fails_if_true(should_fail: bool) -> Fallible<&'static str> {
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
pub enum Fallible<E> {
    /// No error was produced.
    Success,
    /// An error was produced.
    Fail(E),
}

use Fallible::{Fail, Success};

impl<E> Fallible<E> {
    /// Converts from `Fallible<E>` (or `&Fallible<E>`) to `Fallible<&E::Target>`.
    ///
    /// Leaves the original `Fallible` in-place, creating a new one with a reference
    /// to the original one, additionally coercing the contents via [`Deref`].
    ///
    /// ```rust
    /// # use fallible_option::Fallible::{self, Fail};
    /// let fail: Fallible<String> = Fail("something went wrong".to_owned());
    /// assert_eq!(fail.as_deref(), Fail("something went wrong"))
    /// ```
    #[inline]
    pub const fn as_deref(&self) -> Fallible<&<E as Deref>::Target>
    where
        E: ~const Deref,
    {
        match self {
            Success => Success,
            Fail(e) => Fail(e.deref()),
        }
    }

    /// Converts from `Fallible<E>` (or `&mut Fallible<E>`) to `Fallible<&mut E::Target>`
    ///
    /// Leaves the original `Fallible` in-place, creating a new one containing a mutable reference to
    /// the inner type's [`Deref::Target`] type.
    ///
    /// ```rust
    /// # use fallible_option::Fallible::{self, Fail};
    /// let mut fail = Fail("uh oh!".to_owned());
    ///
    /// fail.as_deref_mut().map(|err| {
    ///     err.make_ascii_uppercase();
    ///     err
    /// });
    ///
    /// assert_eq!(fail, Fail("UH OH!".to_owned()));
    /// ```
    ///
    #[inline]
    pub const fn as_deref_mut(&mut self) -> Fallible<&mut <E as Deref>::Target>
    where
        E: ~const DerefMut,
    {
        match self {
            Success => Success,
            Fail(e) => Fail(e.deref_mut()),
        }
    }

    /// Converts from `&mut Fallible<E>` to `Fallible<&mut E>`
    ///
    /// ```rust
    /// # use fallible_option::Fallible::{self, Fail, Success};
    /// let mut fail = Fail("uh oh!".to_owned());
    /// match fail.as_mut() {
    ///     Fail(err) => err.make_ascii_uppercase(),
    ///     Success => {}
    /// }
    ///
    /// assert_eq!(fail, Fail("UH OH!".to_owned()))
    /// ```
    #[inline]
    pub const fn as_mut(&mut self) -> Fallible<&mut E> {
        match self {
            Success => Success,
            Fail(ref mut e) => Fail(e),
        }
    }

    /// Converts from `&Fallible<E>` to `Fallible<&E>`
    ///
    /// ```rust
    /// # use fallible_option::Fallible::{self, Fail};
    /// let fail = Fail("uh oh!");
    /// let err_length = fail.as_ref().map(|err| err.len());
    ///
    /// assert_eq!(err_length, Fail(6));
    ///
    /// ```
    #[inline]
    pub const fn as_ref(&self) -> Fallible<&E> {
        match self {
            Success => Success,
            Fail(ref e) => Fail(e),
        }
    }

    /// Returns true if the value is a `Success`, otherwise false.
    ///
    /// ```rust
    /// # use fallible_option::Fallible::{self, Fail, Success};
    /// assert_eq!(Fail("some error").is_successful(), false);
    /// assert_eq!(Success::<&str>.is_successful(), true)
    /// ```
    #[inline]
    pub const fn is_successful(&self) -> bool {
        matches!(self, Success)
    }

    /// Returns true if the value is a `Fail`, otherwise false.
    ///
    /// ```rust
    /// # use fallible_option::Fallible::{self, Fail, Success};
    /// assert_eq!(Fail("some error").is_fail(), true);
    /// assert_eq!(Success::<&str>.is_fail(), false)
    /// ```
    #[inline]
    pub const fn is_fail(&self) -> bool {
        matches!(self, Fail(_))
    }

    /// Unwrap the contained error or panics if no error has occurred.
    ///
    /// ```rust
    /// # use fallible_option::Fallible::{self, Fail, Success};
    /// let fail = Fail(70);
    /// assert_eq!(fail.unwrap_fail(), 70);
    /// ```
    ///
    /// ```rust,should_panic
    /// # use fallible_option::Fallible::{self, Fail, Success};
    /// let fail: Fallible<u32> = Success;
    /// assert_eq!(fail.unwrap_fail(), 70);
    /// ```
    #[inline]
    pub fn unwrap_fail(self) -> E {
        match self {
            Success => panic!("called `Fallible::unwrap_fail()` on a `Fallible::Success` value"),
            Fail(err) => err,
        }
    }

    /// Returns `true` if the Fallible is a `Fail` value containing an error
    /// equivalent to `f`
    ///
    /// ```rust
    /// # use fallible_option::Fallible::{self, Fail};
    /// let fail = Fail("hello".to_owned());
    /// assert!(fail.contains(&"hello"))
    /// ```
    #[inline]
    pub const fn contains<U: ~const PartialEq<E>>(&self, x: &U) -> bool {
        match self {
            Success => false,
            Fail(e) => x.eq(e),
        }
    }

    /// Maps an `Fallible<E>` to `Fallible<U>` by applying a function
    /// to the contained error.
    ///
    /// ```rust
    /// # use fallible_option::Fallible::{self, Fail};
    /// let fail = Fail("hello");
    /// let fail = fail.map(|err| format!("{err} world!"));
    ///
    /// assert_eq!(fail, Fail("hello world!".to_owned()));
    /// ```
    #[inline]
    pub const fn map<F, U>(self, op: F) -> Fallible<U>
    where
        F: ~const FnOnce(E) -> U,
        F: ~const Destruct,
        E: ~const Destruct,
    {
        match self {
            Success => Success,
            Fail(e) => Fail(op(e)),
        }
    }

    /// Transforms the `Fallible<E>` into a `Result<(), E>`, where `Fail(e)`
    /// becomes `Err(e)` and `Success` becomes `Ok(())`
    /// ```rust
    /// # use fallible_option::Fallible::{self, Fail};
    /// let fail = Fail("error").result();
    ///
    /// assert_eq!(fail, Err("error"));
    /// ```
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

    /// Borrows the `Fallible<E>` as an `Option<E>`, yielding none
    /// if no error occurred.
    /// ```rust
    /// # use fallible_option::Fallible::{self, Fail};
    /// let fail = Fail("error occurred");
    /// let maybe_error = fail.err();
    ///
    /// assert_eq!(maybe_error, Some(&"error occurred"));
    /// ```
    #[inline]
    pub const fn err(&self) -> Option<&E> {
        match self {
            Success => None,
            Fail(err) => Some(err),
        }
    }

    /// Constructs a `Result<T, E>` from self.
    /// ```rust
    /// # use fallible_option::Fallible::{self, Fail};
    /// let fail: Result<u32, &str> = Fail("some error").err_or(10);
    ///
    /// assert_eq!(fail, Err("some error"));
    /// ```
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
    /// and returns an `Option<E>` with the contained error,
    /// if the outcome was `Fail`.
    /// ```rust
    /// # use fallible_option::Fallible::{self, Fail, Success};
    /// let mut fail = Fail("something went wrong");
    ///
    /// let err = fail.take();
    ///
    /// assert_eq!(fail, Success);
    /// assert_eq!(err, Some("something went wrong"));
    /// ```
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

impl<E> Fallible<&E>
where
    E: ~const Clone,
{
    /// Maps a `Fallible<&E>` to a [`Fallible<E>`] by cloning the contents of the
    /// error.
    #[inline]
    #[must_use = "`self` will be dropped if the result is not used"]
    pub const fn cloned(self) -> Fallible<E> {
        match self {
            Success => Success,
            Fail(e) => Fail(e.clone()),
        }
    }

    /// Maps an `Fallible<&E>` to an `Fallible<E>` by copying the contents of the
    /// error.
    #[inline]
    #[must_use = "`self` will be dropped if the result is not used"]
    pub const fn copied(self) -> Fallible<E>
    where
        E: Copy,
    {
        match self {
            Success => Success,
            Fail(&e) => Fail(e),
        }
    }
}

impl<E> Fallible<&mut E>
where
    E: ~const Clone,
{
    /// Maps an `Fallible<&E>` to an `Fallible<E>` by cloning the contents of the
    /// error.
    #[inline]
    #[must_use = "`self` will be dropped if the result is not used"]
    pub const fn cloned(self) -> Fallible<E> {
        match self {
            Success => Success,
            Fail(e) => Fail(e.clone()),
        }
    }

    /// Maps an `Fallible<&E>` to an `Fallible<E>` by copying the contents of the
    /// error.
    #[inline]
    #[must_use = "`self` will be dropped if the result is not used"]
    pub const fn copied(self) -> Fallible<E>
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
impl<E> Fallible<E>
where
    E: Debug,
{
    /// Returns a unit value if the `Fallible` is not `Fail`.
    ///
    /// # Panics
    /// Panics if the value is a `Fail`, with a panic message including
    /// the content of the `Fail`.
    /// ```rust
    /// # use fallible_option::Fallible::{self, Fail, Success};
    /// let success: Fallible<()> = Success;
    /// success.unwrap();
    /// ```
    ///
    /// ```rust,should_panic
    /// # use fallible_option::Fallible::{self, Fail, Success};
    /// let fail = Fail("hello world");
    /// fail.unwrap();
    /// ```
    #[inline]
    pub fn unwrap(self) {
        match self {
            Success => (),
            Fail(e) => {
                panic!("called `Fallible::unwrap()` on a `Fallible::Fail` value: {e:?}")
            }
        }
    }
}

impl<E> Fallible<Fallible<E>> {
    /// Flattens a `Fallible<Fallible<E>>` into a `Fallible<E>`
    #[inline]
    pub const fn flatten(self) -> Fallible<E>
    where
        E: ~const Destruct,
    {
        match self {
            Success => Success,
            Fail(inner) => inner,
        }
    }
}

impl<E> const From<E> for Fallible<E> {
    #[inline]
    fn from(value: E) -> Self {
        Fail(value)
    }
}

impl<T, E> const From<Result<T, E>> for Fallible<E>
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

impl<'a, E> const From<&'a Fallible<E>> for Fallible<&'a E> {
    #[inline]
    fn from(value: &'a Fallible<E>) -> Self {
        value.as_ref()
    }
}

impl<'a, E> const From<&'a mut Fallible<E>> for Fallible<&'a mut E> {
    #[inline]
    fn from(value: &'a mut Fallible<E>) -> Self {
        value.as_mut()
    }
}

impl<E> const Default for Fallible<E> {
    #[inline]
    fn default() -> Self {
        Success
    }
}

impl<E> const Clone for Fallible<E>
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

impl<E> Try for Fallible<E> {
    type Output = ();
    type Residual = Fallible<E>;

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

impl<E, U> FromResidual<Fallible<U>> for Fallible<E>
where
    E: From<U>,
{
    #[inline]
    fn from_residual(residual: Fallible<U>) -> Self {
        match residual {
            Success => Success,
            Fail(u) => Fail(u.into()),
        }
    }
}

impl<T, E, U> FromResidual<Fallible<U>> for Result<T, E>
where
    E: From<U>,
{
    #[inline]
    fn from_residual(residual: Fallible<U>) -> Self {
        match residual {
            Success => unreachable!(),
            Fail(e) => Err(e.into()),
        }
    }
}

impl<E, U> FromResidual<Result<(), U>> for Fallible<E>
where
    E: From<U>,
{
    #[inline]
    fn from_residual(residual: Result<(), U>) -> Self {
        match residual {
            Ok(()) => Success,
            Err(e) => Fail(e.into()),
        }
    }
}

impl<E, U> FromResidual<Result<Infallible, U>> for Fallible<E>
where
    E: From<U>,
{
    #[inline]
    fn from_residual(residual: Result<Infallible, U>) -> Self {
        match residual {
            Ok(_) => Success,
            Err(e) => Fail(e.into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::Fallible::{self, Fail, Success};

    #[derive(Debug, PartialEq)]
    struct InnerError(pub u8);

    #[derive(Debug, PartialEq)]
    enum OuterError {
        Inner(InnerError),
    }

    impl From<InnerError> for OuterError {
        fn from(value: InnerError) -> Self {
            OuterError::Inner(value)
        }
    }

    #[test]
    fn fallible_residual_conversion() {
        fn inner_error() -> Fallible<InnerError> {
            Fail(InnerError(1))
        }

        fn outer_error() -> Fallible<OuterError> {
            inner_error()?;
            Success
        }

        assert_eq!(
            outer_error().unwrap_fail(),
            OuterError::Inner(InnerError(1))
        );
    }

    #[test]
    fn result_residual_conversion() {
        fn inner_error() -> Fallible<InnerError> {
            Fail(InnerError(1))
        }

        fn outer_error() -> Result<(), OuterError> {
            inner_error()?;
            Ok(())
        }

        assert_eq!(outer_error(), Err(OuterError::Inner(InnerError(1))));
    }
}
