use crate::*;

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaM_growaux_(
    mut L: *mut lua_State,
    mut block_0: *mut c_void,
    mut nelems: i32,
    mut psize: *mut i32,
    mut size_elems: i32,
    mut limit: i32,
    mut what: *const std::ffi::c_char,
) -> *mut c_void {
    let mut newblock: *mut c_void = 0 as *mut c_void;
    let mut size: i32 = *psize;
    if nelems + 1 as i32 <= size {
        return block_0;
    }
    if size >= limit / 2 as i32 {
        if ((size >= limit) as i32 != 0) as i32 as std::ffi::c_long != 0 {
            luaG_runerror(L, c"too many %s (limit is %d)".as_ptr(), what, limit);
        }
        size = limit;
    } else {
        size *= 2 as i32;
        if size < 4 as i32 {
            size = 4 as i32;
        }
    }
    newblock = luaM_saferealloc_(
        L,
        block_0,
        *psize as size_t * size_elems as size_t,
        size as size_t * size_elems as size_t,
    );
    *psize = size;
    return newblock;
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaM_shrinkvector_(
    mut L: *mut lua_State,
    mut block_0: *mut c_void,
    mut size: *mut i32,
    mut final_n: i32,
    mut size_elem: i32,
) -> *mut c_void {
    let mut newblock: *mut c_void = 0 as *mut c_void;
    let mut oldsize: size_t = (*size * size_elem) as size_t;
    let mut newsize: size_t = (final_n * size_elem) as size_t;
    newblock = luaM_saferealloc_(L, block_0, oldsize, newsize);
    *size = final_n;
    return newblock;
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaM_toobig(mut L: *mut lua_State) -> ! {
    luaG_runerror(L, c"memory allocation error: block too big".as_ptr());
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaM_free_(
    mut L: *mut lua_State,
    mut block_0: *mut c_void,
    mut osize: size_t,
) {
    let mut g: *mut global_State = (*L).l_G;
    (Some(((*g).frealloc).expect("non-null function pointer"))).expect("non-null function pointer")(
        (*g).ud,
        block_0,
        osize,
        0 as size_t,
    );
    (*g).GCdebt = ((*g).GCdebt as size_t).wrapping_sub(osize) as l_mem as l_mem;
}
unsafe extern "C-unwind" fn tryagain(
    mut L: *mut lua_State,
    mut block_0: *mut c_void,
    mut osize: size_t,
    mut nsize: size_t,
) -> *mut c_void {
    let mut g: *mut global_State = (*L).l_G;
    if (*g).nilvalue.tt_ as i32 & 0xf as i32 == 0 && (*g).gcstopem == 0 {
        luaC_fullgc(L, 1 as i32);
        return (Some(((*g).frealloc).expect("non-null function pointer")))
            .expect("non-null function pointer")((*g).ud, block_0, osize, nsize);
    } else {
        return 0 as *mut c_void;
    };
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaM_realloc_(
    mut L: *mut lua_State,
    mut block_0: *mut c_void,
    mut osize: size_t,
    mut nsize: size_t,
) -> *mut c_void {
    let mut newblock: *mut c_void = 0 as *mut c_void;
    let mut g: *mut global_State = (*L).l_G;
    newblock = (Some(((*g).frealloc).expect("non-null function pointer")))
        .expect("non-null function pointer")((*g).ud, block_0, osize, nsize);
    if ((newblock.is_null() && nsize > 0 as size_t) as i32 != 0) as i32 as std::ffi::c_long != 0 {
        newblock = tryagain(L, block_0, osize, nsize);
        if newblock.is_null() {
            return 0 as *mut c_void;
        }
    }
    (*g).GCdebt = ((*g).GCdebt as size_t)
        .wrapping_add(nsize)
        .wrapping_sub(osize) as l_mem;
    return newblock;
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaM_saferealloc_(
    mut L: *mut lua_State,
    mut block_0: *mut c_void,
    mut osize: size_t,
    mut nsize: size_t,
) -> *mut c_void {
    let mut newblock: *mut c_void = luaM_realloc_(L, block_0, osize, nsize);
    if ((newblock.is_null() && nsize > 0 as size_t) as i32 != 0) as i32 as std::ffi::c_long != 0 {
        luaD_throw(L, 4 as i32);
    }
    return newblock;
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaM_malloc_(
    mut L: *mut lua_State,
    mut size: size_t,
    mut tag: i32,
) -> *mut c_void {
    if size == 0 as size_t {
        return 0 as *mut c_void;
    } else {
        let mut g: *mut global_State = (*L).l_G;
        let mut newblock: *mut c_void = (Some(((*g).frealloc).expect("non-null function pointer")))
            .expect("non-null function pointer")(
            (*g).ud, 0 as *mut c_void, tag as size_t, size
        );
        if ((newblock == 0 as *mut c_void) as i32 != 0) as i32 as std::ffi::c_long != 0 {
            newblock = tryagain(L, 0 as *mut c_void, tag as size_t, size);
            if newblock.is_null() {
                luaD_throw(L, 4 as i32);
            }
        }
        (*g).GCdebt = ((*g).GCdebt as size_t).wrapping_add(size) as l_mem as l_mem;
        return newblock;
    };
}
