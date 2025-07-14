use std::{
    alloc::Layout,
    mem::MaybeUninit,
    ptr::{self, NonNull},
};

use crate::global_State;

trait LuaDrop {
    fn drop_with_state(&mut self, state: &global_State);
}

impl<T: Copy> LuaDrop for T {
    fn drop_with_state(&mut self, _state: &global_State) {}
}

pub(super) struct LVec32<T> {
    ptr: NonNull<T>,
    len: u32,
    cap: u32,
}

impl<T> LVec32<T> {
    pub(super) fn new() -> Self {
        Self {
            ptr: NonNull::dangling(),
            len: 0,
            cap: 0,
        }
    }

    fn grow(&mut self, g: GlobalState, additional: u32) -> Result<(), ()> {
        let required_cap = self.len.checked_add(additional).ok_or(())?;
        let cap = std::cmp::max(self.cap * 2, required_cap);

        let ptr = unsafe {
            g.realloc_slice(
                NonNull::new_unchecked(ptr::slice_from_raw_parts_mut(
                    self.ptr.as_ptr(),
                    self.cap as usize,
                ) as *mut [MaybeUninit<T>]),
                cap as usize,
            )
            .ok_or(())?
        };

        self.ptr = ptr.cast();
        self.cap = cap;

        Ok(())
    }

    pub(super) fn push(&mut self, g: GlobalState, item: T) -> Result<(), T> {
        if self.len >= self.cap {
            if self.grow(g, 1).is_err() {
                return Err(item);
            }
        }
        unsafe { self.ptr.add(self.len as usize).write(item) };
        self.len += 1;
        Ok(())
    }

    pub(super) fn pop(&mut self) -> Option<T> {
        if self.len > 0 {
            self.len -= 1;
            let ptr = unsafe { self.ptr.add(self.len as usize) };
            Some(unsafe { ptr.read() })
        } else {
            None
        }
    }

    pub(super) fn clear(&mut self) {
        // TODO: Drop
        self.len = 0;
    }
}

impl<T> std::ops::Deref for LVec32<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        unsafe { std::slice::from_raw_parts(self.ptr.as_ptr(), self.len as usize) }
    }
}

impl<T> std::ops::DerefMut for LVec32<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { std::slice::from_raw_parts_mut(self.ptr.as_ptr(), self.len as usize) }
    }
}

pub(crate) struct AllocGuard<T: ?Sized> {
    g: GlobalState,
    ptr: NonNull<T>,
}

impl<T: ?Sized> Drop for AllocGuard<T> {
    fn drop(&mut self) {
        unsafe { self.g.dealloc(self.ptr) };
    }
}

impl<T: ?Sized> AllocGuard<T> {
    pub(crate) fn alloc(g: GlobalState) -> Option<AllocGuard<T>>
    where
        T: Sized,
    {
        Some(AllocGuard { g, ptr: g.alloc()? })
    }

    pub(crate) fn as_ptr(&self) -> NonNull<T> {
        self.ptr
    }

    pub(crate) fn into_ptr(self) -> NonNull<T> {
        let ptr = self.ptr;
        std::mem::forget(self);
        ptr
    }
}

impl<T> AllocGuard<[T]> {
    pub(crate) fn alloc_slice(g: GlobalState, len: usize) -> Option<AllocGuard<[T]>> {
        Some(AllocGuard {
            g,
            ptr: g.alloc_slice(len)?,
        })
    }
}

#[derive(Copy, Clone)]
pub(crate) struct GlobalState(pub(crate) NonNull<global_State>);

impl GlobalState {
    pub(crate) unsafe fn new_unchecked(g: *const global_State) -> Self {
        GlobalState(NonNull::new_unchecked(g.cast_mut()))
    }

    pub(crate) fn alloc<T>(&self) -> Option<NonNull<T>> {
        let _ = const {
            assert!(
                Layout::new::<T>().align() <= align_of::<libc::max_align_t>(),
                "alignment too high to use allocator"
            );
        };

        if size_of::<T>() == 0 {
            return Some(NonNull::dangling());
        }

        let alloc = unsafe { (*self.0.as_ptr()).frealloc.unwrap_unchecked() };
        let ptr = unsafe { alloc((*self.0.as_ptr()).ud, ptr::null_mut(), 0, size_of::<T>()) };

        NonNull::new(ptr.cast())
    }

    pub(crate) fn alloc_slice<T>(&self, len: usize) -> Option<NonNull<[T]>> {
        let _ = const {
            assert!(
                Layout::new::<T>().align() <= align_of::<libc::max_align_t>(),
                "alignment too high to use allocator"
            );
        };

        let layout = Layout::new::<T>();
        let size = layout
            .size()
            .checked_mul(len)
            .and_then(|x| isize::try_from(x).ok())
            .expect("FATAL: slice allocation too large");

        if size == 0 {
            return Some(unsafe {
                NonNull::new_unchecked(ptr::slice_from_raw_parts_mut(ptr::dangling_mut(), 0))
            });
        }

        let alloc = unsafe { (*self.0.as_ptr()).frealloc.unwrap_unchecked() };
        let ptr: *mut std::ffi::c_void =
            unsafe { alloc((*self.0.as_ptr()).ud, ptr::null_mut(), 0, size as usize) };

        NonNull::new(ptr::slice_from_raw_parts_mut(ptr.cast(), len))
    }

    pub(crate) unsafe fn realloc_slice<T>(
        &self,
        ptr: NonNull<[T]>,
        new_len: usize,
    ) -> Option<NonNull<[T]>> {
        if ptr.len() == new_len {
            return Some(ptr);
        }

        let old_size = unsafe { Layout::for_value(ptr.as_ref()) };

        let layout = Layout::new::<T>();
        let size = layout
            .size()
            .checked_mul(new_len)
            .and_then(|x| isize::try_from(x).ok())
            .expect("FATAL: slice allocation too large");

        // On shrink, drop items at the end
        if ptr.len() > new_len {
            for i in new_len..ptr.len() {
                ptr::drop_in_place(ptr.cast::<T>().add(i).as_ptr());
            }
        }

        if old_size.size() == 0 && size == 0 {
            return Some(unsafe {
                NonNull::new_unchecked(ptr::slice_from_raw_parts_mut(ptr::dangling_mut(), 0))
            });
        }

        let alloc = unsafe { (*self.0.as_ptr()).frealloc.unwrap_unchecked() };
        let ptr: *mut std::ffi::c_void = unsafe {
            alloc(
                (*self.0.as_ptr()).ud,
                ptr.as_ptr().cast(),
                old_size.size(),
                size as usize,
            )
        };

        if size == 0 {
            return Some(unsafe {
                NonNull::new_unchecked(ptr::slice_from_raw_parts_mut(ptr::dangling_mut(), 0))
            });
        }

        NonNull::new(ptr::slice_from_raw_parts_mut(ptr.cast(), new_len))
    }

    pub(crate) unsafe fn dealloc<T: ?Sized>(&self, ptr: NonNull<T>) {
        // TODO: Use Layout::for_value_raw
        let layout = Layout::for_value(ptr.as_ref());

        ptr::drop_in_place(ptr.as_ptr());

        if layout.size() == 0 {
            return;
        }

        let alloc = unsafe { (*self.0.as_ptr()).frealloc.unwrap_unchecked() };
        unsafe {
            alloc(
                (*self.0.as_ptr()).ud,
                ptr.as_ptr().cast(),
                layout.size() as usize,
                0,
            )
        };
    }
}
