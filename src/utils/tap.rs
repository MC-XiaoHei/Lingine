use anyhow::Result;

pub trait TryTap: Sized {
    fn try_tap<F, E>(self, f: F) -> Result<Self, E>
    where
        F: FnOnce(&Self) -> Result<(), E>,
    {
        f(&self)?;
        Ok(self)
    }

    fn try_tap_mut<F, E>(mut self, f: F) -> Result<Self, E>
    where
        F: FnOnce(&mut Self) -> Result<(), E>,
    {
        f(&mut self)?;
        Ok(self)
    }

    fn try_pipe<R, F, E>(self, f: F) -> Result<R, E>
    where
        F: FnOnce(Self) -> Result<R, E>,
    {
        f(self)
    }
}

impl<T> TryTap for T {}