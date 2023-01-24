use std::alloc::{alloc, dealloc, Layout};
use std::mem;
use std::ptr::NonNull;
use std::sync::atomic::{AtomicUsize, fence, Ordering};

pub struct InsertOnlyConcVec<T> {
    len: AtomicUsize,
    finished_len: AtomicUsize,
    cap: usize,
    alloc: NonNull<T>,
}

unsafe impl<T: Send + Sync> Send for InsertOnlyConcVec<T> {}
unsafe impl<T: Send + Sync> Sync for InsertOnlyConcVec<T> {}

impl<T> InsertOnlyConcVec<T> {

    pub fn new(cap: usize) -> Self {
        Self {
            len: AtomicUsize::new(0),
            finished_len: AtomicUsize::new(0),
            cap,
            alloc: unsafe { NonNull::new_unchecked(alloc(Layout::array::<T>(cap).unwrap()).cast()) },
        }
    }

    pub fn push(&self, val: T) {
        let curr = self.len.fetch_add(1, Ordering::Relaxed);
        fence(Ordering::Release);

        unsafe { self.alloc.as_ptr().offset(curr as isize).write(val); }

        self.finished_len.fetch_add(1, Ordering::Acquire);
    }

    #[inline]
    pub fn is_filled(&self) -> bool {
        self.finished_len.load(Ordering::Acquire) == self.cap
    }

    #[inline]
    pub fn is_filling(&self) -> bool {
        self.len.load(Ordering::Acquire) == self.cap
    }

    #[inline]
    pub fn to_vec_finished(self) -> Vec<T> {
        let ret = unsafe { Vec::from_raw_parts(self.alloc.as_ptr(), self.cap, self.cap) };
        mem::forget(self);
        ret
    }

}

impl<T> Drop for InsertOnlyConcVec<T> {
    fn drop(&mut self) {
        unsafe { dealloc(self.alloc.as_ptr().cast(), Layout::array::<T>(self.cap).unwrap()); }
    }
}
