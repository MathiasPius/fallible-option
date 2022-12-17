#![no_std]
#![feature(try_trait_v2)]
#![feature(const_trait_impl)]
#![feature(const_mut_refs)]
#![feature(const_replace)]
use core::convert::Infallible;
use core::fmt::Debug;
use core::marker::Destruct;
use core::mem;
use core::ops::{ControlFlow, Deref, DerefMut, FromResidual, Try};

/// [`Fallible`] is an [`Option`] with inverted [`Try`]-semantics.
///
/// What this means is that using the `?` operator on a [`Fallible<E>`] will exit early
/// if an error `E` is contained within, or instead act as a no-op, if the value is [`Fallible::Ok`].
///
/// ```
/// # use fallible::Fallible;
/// # fn test_chained_failures() {
/// // Check many numbers, returning early if a tested
/// // number is equal to zero.
/// fn check_many_numbers() -> Fallible<&'static str> {
///     let fails_if_number_is_zero = |n: u32| {
///         if n == 0 {
///             Fallible::Err("number is zero")
///         } else {
///             Fallible::Ok
///         }
///     };
///
///     fails_if_number_is_zero(3)?;
///     fails_if_number_is_zero(0)?; // <--- Will cause early exit
/// 
///     // Following lines are never reached
///     fails_if_number_is_zero(10)?;
///     Fallible::Ok
/// }
///
/// assert_eq!(check_many_numbers(), Fallible::Err("number is zero"));
/// # }
/// ```
#[must_use]
#[derive(Copy, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
pub enum Fallible<E> {
    Ok,
    Err(E),
}

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
            Fallible::Ok => Fallible::Ok,
            Fallible::Err(e) => Fallible::Err(e.deref()),
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
            Fallible::Ok => Fallible::Ok,
            Fallible::Err(e) => Fallible::Err(e.deref_mut()),
        }
    }

    /// Converts from `&mut Fallible<E>` to `Fallible<&mut E>`
    #[inline]
    pub const fn as_mut(&mut self) -> Fallible<&mut E> {
        match self {
            Fallible::Ok => Fallible::Ok,
            Fallible::Err(ref mut e) => Fallible::Err(e),
        }
    }

    /// Converts from `&Fallible<E>` to `Fallible<&E>`
    #[inline]
    pub const fn as_ref(&self) -> Fallible<&E> {
        match self {
            Fallible::Ok => Fallible::Ok,
            Fallible::Err(ref e) => Fallible::Err(e),
        }
    }

    /// Returns true if the value is a [`Fallible::Ok`], otherwise false.
    #[inline]
    pub const fn is_ok(&self) -> bool {
        matches!(self, Fallible::Ok)
    }

    /// Returns true if the value is a [`Fallible::Err`], otherwise false.
    #[inline]
    pub const fn is_err(&self) -> bool {
        matches!(self, Fallible::Err(_))
    }

    /// Unwrap the contained error or panics if no error has occurred.
    #[inline]
    pub fn unwrap_err(self) {
        match self {
            Fallible::Ok => panic!("called `Fallible::unwrap_err()` on a `Fallible::Ok` value"),
            Fallible::Err(_) => (),
        }
    }

    /// Returns `true` if the fallible is a [`Fallible::Err`] value containing an error
    /// equivalent to `f`
    #[inline]
    pub const fn contains<F: ~const PartialEq<E>>(&self, f: &F) -> bool {
        match self {
            Fallible::Ok => false,
            Fallible::Err(e) => f.eq(e),
        }
    }

    /// Maps a [`Fallible<E>`] to [`Fallible<F>`] by applying a function
    /// to the contained error.
    #[inline]
    pub const fn map<F, O>(self, op: O) -> Fallible<F>
    where
        O: ~const FnOnce(E) -> F,
        O: ~const Destruct,
        E: ~const Destruct,
    {
        match self {
            Fallible::Ok => Fallible::Ok,
            Fallible::Err(e) => Fallible::Err(op(e)),
        }
    }

    /// Transforms the [`Fallible<E>`] into a [`Result<(), E>`], where [`Fallible::Err(e)`]
    /// becomes [`Err(e)`] and [`Fallible::Ok`] becomes [`Ok(())`]
    #[inline]
    pub const fn result(self) -> Result<(), E>
    where
        E: ~const Destruct,
    {
        match self {
            Fallible::Ok => Ok(()),
            Fallible::Err(e) => Err(e),
        }
    }

    #[inline]
    pub const fn err(&self) -> Option<&E> {
        match self {
            Fallible::Ok => None,
            Fallible::Err(err) => Some(err),
        }
    }

    /// Constructs a [`Result<T, E>`] from self, [`Fallible::Err(e)`]
    /// becomes [`Err(e)`] and [`Fallible::Ok`] becomes [`Ok(value)`]
    #[inline]
    pub const fn err_or<T>(self, value: T) -> Result<T, E>
    where
        E: ~const Destruct,
        T: ~const Destruct,
    {
        match self {
            Fallible::Ok => Ok(value),
            Fallible::Err(e) => Err(e),
        }
    }

    #[inline]
    pub const fn take(&mut self) -> Option<E>
    where
        E: ~const Destruct,
    {
        match mem::replace(self, Fallible::Ok) {
            Fallible::Ok => None,
            Fallible::Err(e) => Some(e),
        }
    }
}

impl<E> Fallible<&E>
where
    E: ~const Clone,
{
    /// Maps an `Fallible<&E>` to an `Fallible<E>` by cloning the contents of the
    /// error.
    #[inline]
    #[must_use = "`self` will be dropped if the result is not used"]
    pub const fn cloned(self) -> Fallible<E> {
        match self {
            Fallible::Ok => Fallible::Ok,
            Fallible::Err(e) => Fallible::Err(e.clone()),
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
            Fallible::Ok => Fallible::Ok,
            Fallible::Err(&e) => Fallible::Err(e),
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
            Fallible::Ok => Fallible::Ok,
            Fallible::Err(e) => Fallible::Err(e.clone()),
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
            Fallible::Ok => Fallible::Ok,
            Fallible::Err(&mut e) => Fallible::Err(e),
        }
    }
}

/// The following functions are only available if the generic parameter `E` implements [`Debug`]
impl<E> Fallible<E>
where
    E: Debug,
{
    /// Returns a unit value if the [`Fallible`] is not an [`Fallible::Err`].
    ///
    /// # Panics
    /// Panics if the value is a [`Fallible::Err`], with a panic message including
    /// the content of the [`Fallible::Err`].
    #[inline]
    pub fn unwrap(self) {
        match self {
            Fallible::Ok => (),
            Fallible::Err(e) => {
                panic!("called `Fallible::unwrap()` on a `Fallible::Err` value: {e:?}")
            }
        }
    }
}

impl<E> Fallible<Fallible<E>> {
    /// Flattens a [`Fallible<Fallible<E>>`] into a [`Fallible<E>`]
    #[inline]
    pub const fn flatten(self) -> Fallible<E>
    where
        E: ~const Destruct,
    {
        match self {
            Fallible::Ok => Fallible::Ok,
            Fallible::Err(inner) => inner,
        }
    }
}

impl<E> const From<E> for Fallible<E> {
    #[inline]
    fn from(value: E) -> Self {
        Fallible::Err(value)
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
            Ok(_) => Fallible::Ok,
            Err(e) => Fallible::Err(e),
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
        Fallible::Ok
    }
}

impl<E> const Clone for Fallible<E>
where
    E: ~const Clone + ~const Destruct,
{
    #[inline]
    fn clone(&self) -> Self {
        match self {
            Fallible::Err(x) => Fallible::Err(x.clone()),
            Fallible::Ok => Fallible::Ok,
        }
    }

    #[inline]
    fn clone_from(&mut self, source: &Self) {
        match (self, source) {
            (Fallible::Err(to), Fallible::Err(from)) => to.clone_from(from),
            (to, from) => *to = from.clone(),
        }
    }
}

impl<E> Try for Fallible<E> {
    type Output = ();
    type Residual = E;

    #[inline]
    fn from_output(_: Self::Output) -> Self {
        Fallible::Ok
    }

    #[inline]
    fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
        match self {
            Fallible::Ok => ControlFlow::Continue(()),
            Fallible::Err(e) => ControlFlow::Break(e),
        }
    }
}

impl<E> FromResidual<E> for Fallible<E> {
    #[inline]
    fn from_residual(residual: E) -> Self {
        Fallible::Err(residual)
    }
}

impl<E> FromResidual<Result<(), E>> for Fallible<E> {
    #[inline]
    fn from_residual(residual: Result<(), E>) -> Self {
        match residual {
            Ok(()) => Fallible::Ok,
            Err(e) => Fallible::Err(e),
        }
    }
}

impl<E> FromResidual<Result<Infallible, E>> for Fallible<E> {
    #[inline]
    fn from_residual(residual: Result<Infallible, E>) -> Self {
        match residual {
            Ok(_) => Fallible::Ok,
            Err(e) => Fallible::Err(e),
        }
    }
}
