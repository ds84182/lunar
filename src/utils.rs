use std::{
    alloc::Layout,
    ptr::{self, NonNull},
};

use crate::global_State;

trait LuaDrop {
    fn drop_with_state(&mut self, state: &global_State);
}

impl<T: Copy> LuaDrop for T {
    fn drop_with_state(&mut self, _state: &global_State) {}
}

impl global_State {
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

        let alloc = unsafe { self.frealloc.unwrap_unchecked() };
        let ptr = unsafe { alloc(self.ud, ptr::null_mut(), 0, size_of::<T>()) };

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

        let alloc = unsafe { self.frealloc.unwrap_unchecked() };
        let ptr: *mut std::ffi::c_void =
            unsafe { alloc(self.ud, ptr::null_mut(), 0, size as usize) };

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

        let alloc = unsafe { self.frealloc.unwrap_unchecked() };
        let ptr: *mut std::ffi::c_void =
            unsafe { alloc(self.ud, ptr.as_ptr().cast(), old_size.size(), size as usize) };

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

        let alloc = unsafe { self.frealloc.unwrap_unchecked() };
        unsafe { alloc(self.ud, ptr.as_ptr().cast(), layout.size() as usize, 0) };
    }
}
