//! Optional values.
//!
//! This trait mirrors the stable API of [`std::option::Option`].
//! It facilitates creating user-defined optional values that can take advantage
//! of the representation of `T` to encode the presence or absence of a value.
//! This is particularly useful when `T` is a `#[repr(transparent)]` newtype
//! with fewer valid values than the type it's wrapping.
//!
//! Nevertheless, [`std::option::Option`] also implements this trait so that
//! generic code will also work with it.

use core::marker::PhantomData;
use core::mem;
use core::ops::{Deref, DerefMut};
use core::option::Option as StdOption;

/// Trait for optional values. See [the module's documentation](self) for more.
#[allow(unused_parens)]
pub unsafe trait Option<T>: Sized {
  /// The empty `Option`.
  const NONE: Self;

  /// An `Option` with a value.
  fn some(x: T) -> Self;

  fn is_some(&self) -> bool {
    !self.is_none()
  }

  fn is_none(&self) -> bool;

  fn as_ref(&self) -> StdOption<&T>;

  fn as_mut(&mut self) -> StdOption<&mut T>;

  fn expect(self, msg: &str) -> T {
    unsafe { (if self.is_some() { self.unwrap_unchecked() } else { panic!("{}", msg) }) }
  }

  fn unwrap(self) -> T {
    self.expect("called `Option::unwrap()` on a `none()` value")
  }

  fn unwrap_or(self, default: T) -> T {
    unsafe { (if self.is_some() { self.unwrap_unchecked() } else { default }) }
  }

  fn unwrap_or_else<F>(self, f: impl FnOnce() -> T) -> T {
    unsafe { (if self.is_some() { self.unwrap_unchecked() } else { f() }) }
  }

  fn unwrap_or_default(self) -> T
  where
    T: Default,
  {
    unsafe { (if self.is_some() { self.unwrap_unchecked() } else { T::default() }) }
  }

  unsafe fn unwrap_unchecked(self) -> T;

  fn map<O, U>(self, f: impl FnOnce(T) -> U) -> O
  where
    O: Option<U>,
  {
    unsafe { (if self.is_some() { O::some(f(self.unwrap_unchecked())) } else { O::NONE }) }
  }

  fn map_or<U>(self, default: U, f: impl FnOnce(T) -> U) -> U {
    unsafe { (if self.is_some() { f(self.unwrap_unchecked()) } else { default }) }
  }

  /// Map or else
  fn map_or_else<U>(self, default: impl FnOnce() -> U, f: impl FnOnce(T) -> U) -> U {
    unsafe { (if self.is_some() { f(self.unwrap_unchecked()) } else { default() }) }
  }

  fn ok_or<E>(self, err: E) -> Result<T, E> {
    unsafe { (if self.is_some() { Ok(self.unwrap_unchecked()) } else { Err(err) }) }
  }

  fn ok_or_else<E>(self, err: impl FnOnce() -> E) -> Result<T, E> {
    unsafe { (if self.is_some() { Ok(self.unwrap_unchecked()) } else { Err(err()) }) }
  }

  fn as_deref<'a>(&'a self) -> StdOption<&<T as Deref>::Target>
  where
    T: Deref + 'a,
  {
    self.as_ref().map(|x| x.deref())
  }

  fn as_deref_mut<'a>(&'a mut self) -> StdOption<&mut <T as Deref>::Target>
  where
    T: DerefMut + 'a,
  {
    self.as_mut().map(|x| x.deref_mut())
  }

  fn iter(&self) -> Iter<'_, T> {
    Iter::new(self.as_ref())
  }

  fn iter_mut(&mut self) -> IterMut<'_, T> {
    IterMut::new(self.as_mut())
  }

  fn into_iter(self) -> IntoIter<Self, T> {
    IntoIter::new(self)
  }

  fn and<U, O>(self, other: O) -> O
  where
    O: Option<U>,
  {
    (if self.is_some() { other } else { O::NONE })
  }

  fn and_then<O, U>(self, f: impl FnOnce(T) -> O) -> O
  where
    O: Option<U>,
  {
    unsafe { (if self.is_some() { f(self.unwrap_unchecked()) } else { O::NONE }) }
  }

  fn filter(self, predicate: impl FnOnce(&T) -> bool) -> Self {
    if self.is_some() {
      let x = unsafe { self.unwrap_unchecked() };
      if predicate(&x) {
        return Self::some(x);
      }
    }
    Self::NONE
  }

  fn or(self, other: Self) -> Self {
    (if self.is_some() { self } else { other })
  }

  fn or_else(self, f: impl FnOnce() -> Self) -> Self {
    (if self.is_some() { self } else { f() })
  }

  fn xor(self, other: Self) -> Self {
    if self.is_some() {
      (if other.is_none() { self } else { Self::NONE })
    } else {
      (if other.is_some() { other } else { Self::NONE })
    }
  }

  fn insert(&mut self, value: T) -> &mut T {
    *self = Self::some(value);
    unsafe { self.as_mut().unwrap_unchecked() }
  }

  fn get_or_insert(&mut self, value: T) -> &mut T {
    if self.is_none() {
      *self = Self::some(value);
    }
    unsafe { self.as_mut().unwrap_unchecked() }
  }

  fn get_or_insert_with<F>(&mut self, f: impl FnOnce() -> T) -> &mut T {
    if self.is_none() {
      *self = Self::some(f())
    }
    unsafe { self.as_mut().unwrap_unchecked() }
  }

  fn take(&mut self) -> Self {
    mem::replace(self, Self::NONE)
  }

  fn replace(&mut self, value: Self) -> Self {
    mem::replace(self, value)
  }

  fn zip<O, U>(self, other: impl Option<U>) -> O
  where
    O: Option<(T, U)>,
  {
    if self.is_some() && other.is_some() {
      O::some(unsafe { (self.unwrap_unchecked(), other.unwrap_unchecked()) })
    } else {
      O::NONE
    }
  }
}

unsafe impl<T> Option<T> for StdOption<T> {
  const NONE: Self = None;

  fn some(x: T) -> Self {
    Some(x)
  }

  fn is_none(&self) -> bool {
    self.is_none()
  }

  fn as_ref(&self) -> StdOption<&T> {
    self.as_ref()
  }

  fn as_mut(&mut self) -> StdOption<&mut T> {
    self.as_mut()
  }

  unsafe fn unwrap_unchecked(self) -> T {
    self.unwrap_unchecked()
  }
}

pub struct Iter<'a, T: 'a>(StdOption<&'a T>);

impl<'a, T: 'a> Iter<'a, T> {
  pub fn new(value: StdOption<&'a T>) -> Self {
    Iter(value)
  }
}

impl<'a, T> Iterator for Iter<'a, T> {
  type Item = &'a T;

  fn next(&mut self) -> StdOption<Self::Item> {
    self.0.take()
  }
}

pub struct IterMut<'a, T: 'a>(StdOption<&'a mut T>);

impl<'a, T: 'a> IterMut<'a, T> {
  pub fn new(value: StdOption<&'a mut T>) -> Self {
    IterMut(value)
  }
}

impl<'a, T> Iterator for IterMut<'a, T> {
  type Item = &'a mut T;

  fn next(&mut self) -> std::option::Option<Self::Item> {
    self.0.take()
  }
}

pub struct IntoIter<O, T>(O, PhantomData<T>);

impl<O, T> IntoIter<O, T>
where
  O: Option<T>,
{
  pub fn new(value: O) -> Self {
    IntoIter(value, PhantomData)
  }
}

impl<O, T> Iterator for IntoIter<O, T>
where
  O: Option<T>,
{
  type Item = T;

  fn next(&mut self) -> StdOption<Self::Item> {
    self.0.take().map::<StdOption<T>, _>(|x| x)
  }
}
