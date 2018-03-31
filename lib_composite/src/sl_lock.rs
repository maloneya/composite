use std::cell::UnsafeCell;
use std::ops::{Deref, DerefMut};

use super::sl::Sl;
use super::sys::sl_lock;

pub struct Lock<T: ?Sized> {
    // We have to box the internal lock, because sl_lock cannot move
    internal_lock: sl_lock::sl_lock,
    data: UnsafeCell<T>
}

unsafe impl<T: ?Sized + Send> Send for Lock<T> {}
unsafe impl<T: ?Sized + Send> Sync for Lock<T> {}

pub struct LockGuard<'a, T: ?Sized + 'a> {
    // funny underscores due to how Deref/DerefMut currently work (they
    // disregard field privacy).
    __lock: &'a Lock<T>,
}

impl<'a, T: ?Sized> !Send for LockGuard<'a, T> { }
unsafe impl<'a, T: ?Sized + Sync> Sync for LockGuard<'a, T> { }


impl<T> Lock<T> {
    pub fn new(_: Sl, t: T) -> Self {
        let mut lock = Lock {
            internal_lock: sl_lock::sl_lock {
                holder: 0
            },
            data: UnsafeCell::new(t)
        };
        unsafe {
            sl_lock::sl_lock_init(&mut lock.internal_lock);
        }
        lock
    }
}

impl <T: ?Sized> Lock<T> {
    pub fn lock(&self) -> LockGuard<T> {
        unsafe {
            sl_lock::sl_lock_take_rs(&self.internal_lock);
            LockGuard::new(self)
        }
    }

    pub fn try_lock(&self) -> Option<LockGuard<T>> {
        unsafe {
            if sl_lock::sl_lock_try_take(&self.internal_lock) != 0 {
                Some(LockGuard::new(self))
            } else {
                None
            }
        }
    }
}

impl <'lock, T: ?Sized> LockGuard<'lock, T> {
    unsafe fn new(lock: &'lock Lock<T>) -> Self {
        LockGuard {
            __lock: lock
        }
    }
}

impl<'lock, T: ?Sized> Deref for LockGuard<'lock, T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.__lock.data.get() }
    }
}

impl<'lock, T: ?Sized> DerefMut for LockGuard<'lock, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.__lock.data.get() }
    }
}

impl<'a, T: ?Sized> Drop for LockGuard<'a, T> {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            sl_lock::sl_lock_release_rs(&self.__lock.internal_lock);
        }
    }
}