#![allow(unused)]

use anyhow::Result;

pub trait TryTap: Sized {
    #[inline(always)]
    fn try_tap<F, E>(self, f: F) -> Result<Self, E>
    where
        F: FnOnce(&Self) -> Result<(), E>,
    {
        f(&self)?;
        Ok(self)
    }

    #[inline(always)]
    fn try_tap_mut<F, E>(mut self, f: F) -> Result<Self, E>
    where
        F: FnOnce(&mut Self) -> Result<(), E>,
    {
        f(&mut self)?;
        Ok(self)
    }

    #[inline(always)]
    fn try_tap_if<F, E>(self, condition: bool, f: F) -> Result<Self, E>
    where
        F: FnOnce(&Self) -> Result<(), E>,
    {
        if condition {
            f(&self)?;
        }
        Ok(self)
    }

    #[inline(always)]
    fn try_tap_when<P, F, E>(self, predicate: P, f: F) -> Result<Self, E>
    where
        P: FnOnce(&Self) -> bool,
        F: FnOnce(&Self) -> Result<(), E>,
    {
        if predicate(&self) {
            f(&self)?;
        }
        Ok(self)
    }
}

impl<T> TryTap for T {}

pub trait TapEx: Sized {
    #[inline(always)]
    fn tap_if<F, E>(self, condition: bool, f: impl FnOnce(&Self)) -> Self {
        if condition {
            f(&self);
        }
        self
    }

    #[inline(always)]
    fn tap_when<P, F>(self, predicate: P, f: F) -> Self
    where
        P: FnOnce(&Self) -> bool,
        F: FnOnce(&Self),
    {
        if predicate(&self) {
            f(&self);
        }
        self
    }
}

impl<T> TapEx for T {}

pub trait TryPipe: Sized {
    #[inline(always)]
    fn try_pipe<R, F, E>(self, f: F) -> Result<R, E>
    where
        F: FnOnce(Self) -> Result<R, E>,
    {
        f(self)
    }
}

impl<T> TryPipe for T {}

pub trait PipeEx: Sized {
    #[inline(always)]
    fn pipe_if<F, E>(self, condition: bool, f: impl FnOnce(Self) -> Self) -> Self {
        if condition { f(self) } else { self }
    }

    #[inline(always)]
    fn pipe_when<P, F>(self, predicate: P, f: F) -> Self
    where
        P: FnOnce(&Self) -> bool,
        F: FnOnce(Self) -> Self,
    {
        if predicate(&self) { f(self) } else { self }
    }
}

impl<T> PipeEx for T {}
