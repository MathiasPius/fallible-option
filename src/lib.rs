//! [`Fallible`] is an [`Option`] with inverted [`Try`]-semantics.
//!
//! What this means is that using the `?` operator on a [`Fallible<E>`] will exit early
//! if an error `E` is contained within, or instead act as a no-op, if the value is [`Intact`].
//!
//! It fills the gap left by the [`Result`] and [`Option`] types:
//!
//! | `Result<T`  |     `, E>`    |
//! |-------------|---------------|
//! | `Option<T>` | `Fallible<E>` |
//!
//!
//!
//! ```
//! # use fallible::Fallible::{self, Fail, Intact};
//! # fn test_chained_failures() {
//! // Check many numbers, returning early if a tested
//! // number is equal to zero.
//! fn check_many_numbers() -> Fallible<&'static str> {
//!     let fails_if_number_is_zero = |n: u32| {
//!         if n == 0 {
//!             Fail("number is zero")
//!         } else {
//!             Intact
//!         }
//!     };
//!
//!     fails_if_number_is_zero(3)?;
//!     fails_if_number_is_zero(0)?; // <--- Will cause early exit
//!
//!     // Following lines are never reached
//!     fails_if_number_is_zero(10)?;
//!     
//!     Intact
//! }
//!
//! assert_eq!(check_many_numbers(), Fallible::Fail("number is zero"));
//! # }
//! ```
//! # Purpose
//! [`Fallible`] is similar in usage to `Result<(), E>`, but without introducing a
//! unit value `Ok(())` to cover the [`Intact`] case.
//!
//! For example, the following function could be better expressed using [`Fallible`], since
//! the happy path does not produce a value:
//! ```
//! fn validate_number(x: u32) -> Result<(), &'static str> {
//!     match x {
//!         0 ..= 9 => Err("number is too small"),
//!         10..=30 => Ok(()),
//!         31..    => Err("number is too large")
//!     }
//! }
//! ```
//! Using [`Fallible`]:
//!
//! ```
//! # use fallible::Fallible::{self, Fail, Intact};
//! fn validate_number(x: u32) -> Fallible<&'static str> {
//!     match x {
//!         0 ..= 9 => Fail("number is too small"),
//!         10..=30 => Intact,
//!         31..    => Fail("number is too large")
//!     }
//! }
//! ```
//! # Compatibility
//!
//! [`Fallible`] contains utility functions for mapping to and from [`Result`] and [`Option`],
//! as well as [`FromResidual`] implementations for automatically performing these conversions
//! when used with the `?` operator.
//! ```
//! # use fallible::Fallible::{self, Fail, Intact};
//! fn fails_if_true(should_fail: bool) -> Fallible<&'static str> {
//!     if should_fail {
//!         Fail("Darn it!")
//!     } else {
//!         Intact
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
//!
//!

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

/// Describes the outcome of an operation which does not produce a value
/// if the operation succeeds. Similar in usage to [`Result<(), E>`].
#[must_use]
#[derive(Copy, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
pub enum Fallible<E> {
    /// No error was produced.
    Intact,
    /// An error was produced.
    Fail(E),
}

use Fallible::{Fail, Intact};

impl<E> Fallible<E> {
    /// Converts from `Fallible<E>` (or `&Fallible<E>`) to `Fallible<&E::Target>`.
    ///
    /// Leaves the original `Fallible` in-place, creating a new one with a reference
    /// to the original one, additionally coercing the contents via [`Deref`].
    #[inline]
    pub const fn as_deref(&self) -> Fallible<&<E as Deref>::Target>
    where
        E: ~const Deref,
    {
        match self {
            Intact => Intact,
            Fail(e) => Fail(e.deref()),
        }
    }

    /// Converts from `Fallible<E>` (or `&mut Fallible<E>`) to `Fallible<&mut E::Target>`
    ///
    /// Leaves the original `Fallible` in-place, creating a new one containing a mutable reference to
    /// the inner type's [`Deref::Target`] type.
    #[inline]
    pub const fn as_deref_mut(&mut self) -> Fallible<&mut <E as Deref>::Target>
    where
        E: ~const DerefMut,
    {
        match self {
            Intact => Intact,
            Fail(e) => Fail(e.deref_mut()),
        }
    }

    /// Converts from `&mut Fallible<E>` to `Fallible<&mut E>`
    #[inline]
    pub const fn as_mut(&mut self) -> Fallible<&mut E> {
        match self {
            Intact => Intact,
            Fail(ref mut e) => Fail(e),
        }
    }

    /// Converts from `&Fallible<E>` to `Fallible<&E>`
    #[inline]
    pub const fn as_ref(&self) -> Fallible<&E> {
        match self {
            Intact => Intact,
            Fail(ref e) => Fail(e),
        }
    }

    /// Returns true if the value is a `Intact`, otherwise false.
    #[inline]
    pub const fn is_intact(&self) -> bool {
        matches!(self, Intact)
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
            Intact => panic!("called `Fallible::unwrap_fail()` on a `Fallible::Intact` value"),
            Fail(_) => (),
        }
    }

    /// Returns `true` if the fallible is a `Fail` value containing an error
    /// equivalent to `f`
    #[inline]
    pub const fn contains<F: ~const PartialEq<E>>(&self, f: &F) -> bool {
        match self {
            Intact => false,
            Fail(e) => f.eq(e),
        }
    }

    /// Maps a `Fallible<E>` to `Fallible<F>` by applying a function
    /// to the contained error.
    #[inline]
    pub const fn map<F, O>(self, op: O) -> Fallible<F>
    where
        O: ~const FnOnce(E) -> F,
        O: ~const Destruct,
        E: ~const Destruct,
    {
        match self {
            Intact => Intact,
            Fail(e) => Fail(op(e)),
        }
    }

    /// Transforms the `Fallible<E>` into a [`Result<(), E>`], where `Fail(e)`
    /// becomes `Err(e)` and `Intact` becomes `Ok(())`
    #[inline]
    pub const fn result(self) -> Result<(), E>
    where
        E: ~const Destruct,
    {
        match self {
            Intact => Ok(()),
            Fail(e) => Err(e),
        }
    }

    /// Borrows the `Fallible<E>` as an [`Option<E>`], yielding none
    /// if no error occurred.
    #[inline]
    pub const fn err(&self) -> Option<&E> {
        match self {
            Intact => None,
            Fail(err) => Some(err),
        }
    }

    /// Constructs a [`Result<T, E>`] from self.
    ///
    /// `Fail(e)` becomes `Err(e)` and `Intact` becomes `Ok(value)`
    #[inline]
    pub const fn err_or<T>(self, value: T) -> Result<T, E>
    where
        E: ~const Destruct,
        T: ~const Destruct,
    {
        match self {
            Intact => Ok(value),
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
        match mem::replace(self, Intact) {
            Intact => None,
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
            Intact => Intact,
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
            Intact => Intact,
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
            Intact => Intact,
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
            Intact => Intact,
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
    #[inline]
    pub fn unwrap(self) {
        match self {
            Intact => (),
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
            Intact => Intact,
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
            Ok(_) => Intact,
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
        Intact
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
            Intact => Intact,
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
        Intact
    }

    #[inline]
    fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
        match self {
            Intact => ControlFlow::Continue(()),
            Fail(e) => ControlFlow::Break(Fail(e)),
        }
    }
}

impl<E> FromResidual<Fallible<E>> for Fallible<E> {
    #[inline]
    fn from_residual(residual: Fallible<E>) -> Self {
        residual
    }
}

impl<T, E> FromResidual<Fallible<E>> for Result<T, E> {
    #[inline]
    fn from_residual(residual: Fallible<E>) -> Self {
        match residual {
            Intact => unreachable!(),
            Fail(e) => Err(e),
        }
    }
}

impl<E> FromResidual<Result<(), E>> for Fallible<E> {
    #[inline]
    fn from_residual(residual: Result<(), E>) -> Self {
        match residual {
            Ok(()) => Intact,
            Err(e) => Fail(e),
        }
    }
}

impl<E> FromResidual<Result<Infallible, E>> for Fallible<E> {
    #[inline]
    fn from_residual(residual: Result<Infallible, E>) -> Self {
        match residual {
            Ok(_) => Intact,
            Err(e) => Fail(e),
        }
    }
}
