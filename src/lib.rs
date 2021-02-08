#![allow(incomplete_features)]
#![feature(specialization)]

mod opt_sendsync;

use opt_sendsync::{OptSend, OptSync};
use std::error::Error as StdError;
use std::marker::PhantomData;

pub struct SendSync;
pub struct NoSendSync;
pub struct NoSync;

pub struct Error<M = NoSendSync> {
    real: Box<dyn StdError>,
    impl_send: bool,
    impl_sync: bool,
    _tag: PhantomData<M>,
}

impl<M> Error<M> {
    /// # Safety
    /// must ensure tag type is correct
    #[inline]
    unsafe fn to_tag<T>(self) -> Error<T> {
        Error {
            real: self.real,
            impl_send: self.impl_send,
            impl_sync: self.impl_sync,
            _tag: PhantomData,
        }
    }

    #[inline]
    pub fn make_no_send(self) -> Error<NoSendSync> {
        // SAFETY: make NoSendSync is always safe
        unsafe { self.to_tag() }
    }

    #[inline]
    pub fn into_inner(self) -> Box<dyn StdError> {
        self.real
    }
}

impl<M> std::fmt::Display for Error<M> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.real, f)
    }
}

unsafe impl Send for Error<SendSync> {}
unsafe impl Sync for Error<SendSync> {}
unsafe impl Send for Error<NoSync> {}

impl<E: Into<Box<dyn StdError>>> From<E> for Error<NoSendSync> {
    fn from(err: E) -> Self {
        Error {
            real: err.into(),
            impl_send: E::IMPL_SEND,
            impl_sync: E::IMPL_SYNC,
            _tag: PhantomData,
        }
    }
}

impl<E: Into<Box<dyn StdError + Send>>> From<E> for Error<NoSync> {
    fn from(err: E) -> Self {
        Error {
            real: err.into() as _,
            impl_send: true,
            impl_sync: E::IMPL_SYNC,
            _tag: PhantomData,
        }
    }
}

impl<E: Into<Box<dyn StdError + Send + Sync>>> From<E> for Error<SendSync> {
    fn from(err: E) -> Self {
        Error {
            real: err.into() as _,
            impl_send: true,
            impl_sync: true,
            _tag: PhantomData,
        }
    }
}

impl Error<NoSendSync> {
    pub fn new(e: Box<dyn StdError>) -> Self {
        Self {
            real: e,
            impl_send: false,
            impl_sync: false,
            _tag: PhantomData,
        }
    }

    pub fn make_send(self) -> Option<Error<NoSync>> {
        if self.impl_send {
            // SAFETY: Ensure real implement Send in above condition
            Some(unsafe { self.to_tag() })
        } else {
            None
        }
    }

    pub fn make_sync(self) -> Option<Error<SendSync>> {
        if self.impl_send && self.impl_sync {
            // SAFETY: Ensure real implement Send + Sync in above condition
            Some(unsafe { self.to_tag() })
        } else {
            None
        }
    }
}

impl Error<NoSync> {
    pub fn new(e: Box<dyn StdError + Send>) -> Self {
        Self {
            real: e as _,
            impl_send: true,
            impl_sync: false,
            _tag: PhantomData,
        }
    }

    pub fn make_sync(self) -> Option<Error<SendSync>> {
        if self.impl_sync {
            // SAFETY: Ensure real implement Sync in above condition and Send with tag
            Some(unsafe { self.to_tag() })
        } else {
            None
        }
    }
}

impl Error<SendSync> {
    pub fn new(e: Box<dyn StdError + Send + Sync>) -> Self {
        Self {
            real: e as _,
            impl_send: true,
            impl_sync: true,
            _tag: PhantomData,
        }
    }
}

#[cfg(test)]
mod tests {
    use thiserror::Error;

    #[derive(Debug, Error)]
    #[error("Error")]
    struct SendSyncError;

    #[derive(Debug, Error)]
    #[error("Error")]
    struct NoSendError(PhantomData<*mut u8>);

    use super::*;

    #[test]
    fn it_works() {
        let erased_send_sync: Error = SendSyncError.into();
        let erased_no_send: Error = NoSendError(PhantomData).into();

        assert!(erased_send_sync.make_send().is_some());
        assert!(erased_no_send.make_send().is_none());
    }
}
