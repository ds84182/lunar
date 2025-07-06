use crate::*;

unsafe extern "C-unwind" fn error(mut S: *mut LoadState, mut why: *const std::ffi::c_char) -> ! {
    luaO_pushfstring(
        (*S).L,
        c"%s: bad binary format (%s)".as_ptr(),
        (*S).name,
        why,
    );
    luaD_throw((*S).L, 3 as i32);
}
unsafe extern "C-unwind" fn loadBlock(mut S: *mut LoadState, mut b: *mut c_void, mut size: size_t) {
    if luaZ_read((*S).Z, b, size) != 0 as size_t {
        error(S, c"truncated chunk".as_ptr());
    }
}
unsafe extern "C-unwind" fn loadByte(mut S: *mut LoadState) -> lu_byte {
    let fresh1 = (*(*S).Z).n;
    (*(*S).Z).n = ((*(*S).Z).n).wrapping_sub(1);
    let mut b: i32 = if fresh1 > 0 as size_t {
        let fresh2 = (*(*S).Z).p;
        (*(*S).Z).p = ((*(*S).Z).p).offset(1);
        *fresh2 as u8 as i32
    } else {
        luaZ_fill((*S).Z)
    };
    if b == -(1 as i32) {
        error(S, c"truncated chunk".as_ptr());
    }
    return b as lu_byte;
}
unsafe extern "C-unwind" fn loadUnsigned(mut S: *mut LoadState, mut limit: size_t) -> size_t {
    let mut x: size_t = 0 as size_t;
    let mut b: i32 = 0;
    limit >>= 7 as i32;
    loop {
        b = loadByte(S) as i32;
        if x >= limit {
            error(S, c"integer overflow".as_ptr());
        }
        x = x << 7 as i32 | (b & 0x7f as i32) as size_t;
        if !(b & 0x80 == 0) {
            break;
        }
    }
    return x;
}
unsafe extern "C-unwind" fn loadSize(mut S: *mut LoadState) -> size_t {
    return loadUnsigned(S, !(0 as size_t));
}
unsafe extern "C-unwind" fn loadInt(mut S: *mut LoadState) -> i32 {
    return loadUnsigned(S, 2147483647 as i32 as size_t) as i32;
}
unsafe extern "C-unwind" fn loadNumber(mut S: *mut LoadState) -> lua_Number {
    let mut x: lua_Number = 0.;
    loadBlock(
        S,
        &mut x as *mut lua_Number as *mut c_void,
        (1usize).wrapping_mul(size_of::<lua_Number>() as usize),
    );
    return x;
}
unsafe extern "C-unwind" fn loadInteger(mut S: *mut LoadState) -> lua_Integer {
    let mut x: lua_Integer = 0;
    loadBlock(
        S,
        &mut x as *mut lua_Integer as *mut c_void,
        (1usize).wrapping_mul(size_of::<lua_Integer>() as usize),
    );
    return x;
}
unsafe extern "C-unwind" fn loadStringN(mut S: *mut LoadState, mut p: *mut Proto) -> *mut TString {
    let mut L: *mut lua_State = (*S).L;
    let mut ts: *mut TString = 0 as *mut TString;
    let mut size: size_t = loadSize(S);
    if size == 0 as size_t {
        return 0 as *mut TString;
    } else {
        size = size.wrapping_sub(1);
        if size <= 40 as size_t {
            let mut buff: [std::ffi::c_char; 40] = [0; 40];
            loadBlock(
                S,
                buff.as_mut_ptr() as *mut c_void,
                size.wrapping_mul(size_of::<std::ffi::c_char>() as usize),
            );
            ts = luaS_newlstr(L, buff.as_mut_ptr(), size);
        } else {
            ts = luaS_createlngstrobj(L, size);
            let mut io: *mut TValue = &mut (*(*L).top.p).val;
            let mut x_: *mut TString = ts;
            (*io).value_.gc = &mut (*(x_ as *mut GCUnion)).gc;
            (*io).tt_ = ((*x_).tt as i32 | (1 as i32) << 6 as i32) as lu_byte;
            if (*io).tt_ as i32 & (1 as i32) << 6 as i32 == 0
                || (*io).tt_ as i32 & 0x3f as i32 == (*(*io).value_.gc).tt as i32
                    && (L.is_null()
                        || (*(*io).value_.gc).marked as i32
                            & ((*(*L).l_G).currentwhite as i32
                                ^ ((1 as i32) << 3 as i32 | (1 as i32) << 4 as i32))
                            == 0)
            {
            } else {
            };
            luaD_inctop(L);
            loadBlock(
                S,
                ((*ts).contents).as_mut_ptr() as *mut c_void,
                size.wrapping_mul(size_of::<std::ffi::c_char>() as usize),
            );
            (*L).top.p = ((*L).top.p).offset(-1);
            (*L).top.p;
        }
    }
    if (*p).marked as i32 & (1 as i32) << 5 as i32 != 0
        && (*ts).marked as i32 & ((1 as i32) << 3 as i32 | (1 as i32) << 4 as i32) != 0
    {
        luaC_barrier_(
            L,
            &mut (*(p as *mut GCUnion)).gc,
            &mut (*(ts as *mut GCUnion)).gc,
        );
    } else {
    };
    return ts;
}
unsafe extern "C-unwind" fn loadString(mut S: *mut LoadState, mut p: *mut Proto) -> *mut TString {
    let mut st: *mut TString = loadStringN(S, p);
    if st.is_null() {
        error(S, c"bad format for constant string".as_ptr());
    }
    return st;
}
unsafe extern "C-unwind" fn loadCode(mut S: *mut LoadState, mut f: *mut Proto) {
    let mut n: i32 = loadInt(S);
    if size_of::<i32>() as usize >= size_of::<size_t>() as usize
        && (n as size_t).wrapping_add(1 as i32 as size_t)
            > (!(0 as size_t)).wrapping_div(size_of::<Instruction>() as usize)
    {
        luaM_toobig((*S).L);
    } else {
    };
    (*f).code = luaM_malloc_(
        (*S).L,
        (n as usize).wrapping_mul(size_of::<Instruction>() as usize),
        0,
    ) as *mut Instruction;
    (*f).sizecode = n;
    loadBlock(
        S,
        (*f).code as *mut c_void,
        (n as usize).wrapping_mul(size_of::<Instruction>() as usize),
    );
}
unsafe extern "C-unwind" fn loadConstants(mut S: *mut LoadState, mut f: *mut Proto) {
    let mut i: i32 = 0;
    let mut n: i32 = loadInt(S);
    if size_of::<i32>() as usize >= size_of::<size_t>() as usize
        && (n as size_t).wrapping_add(1 as i32 as size_t)
            > (!(0 as size_t)).wrapping_div(size_of::<TValue>() as usize)
    {
        luaM_toobig((*S).L);
    } else {
    };
    (*f).k = luaM_malloc_(
        (*S).L,
        (n as usize).wrapping_mul(size_of::<TValue>() as usize),
        0,
    ) as *mut TValue;
    (*f).sizek = n;
    i = 0;
    while i < n {
        (*((*f).k).offset(i as isize)).tt_ = (0 | (0) << 4 as i32) as lu_byte;
        i += 1;
        i;
    }
    i = 0;
    while i < n {
        let mut o: *mut TValue = &mut *((*f).k).offset(i as isize) as *mut TValue;
        let mut t: i32 = loadByte(S) as i32;
        match t {
            0 => {
                (*o).tt_ = (0 | (0) << 4 as i32) as lu_byte;
            }
            1 => {
                (*o).tt_ = (1 as i32 | (0) << 4 as i32) as lu_byte;
            }
            17 => {
                (*o).tt_ = (1 as i32 | (1 as i32) << 4 as i32) as lu_byte;
            }
            19 => {
                let mut io: *mut TValue = o;
                (*io).value_.n = loadNumber(S);
                (*io).tt_ = (3 as i32 | (1 as i32) << 4 as i32) as lu_byte;
            }
            3 => {
                let mut io_0: *mut TValue = o;
                (*io_0).value_.i = loadInteger(S);
                (*io_0).tt_ = (3 as i32 | (0) << 4 as i32) as lu_byte;
            }
            4 | 20 => {
                let mut io_1: *mut TValue = o;
                let mut x_: *mut TString = loadString(S, f);
                (*io_1).value_.gc = &mut (*(x_ as *mut GCUnion)).gc;
                (*io_1).tt_ = ((*x_).tt as i32 | (1 as i32) << 6 as i32) as lu_byte;
                if (*io_1).tt_ as i32 & (1 as i32) << 6 as i32 == 0
                    || (*io_1).tt_ as i32 & 0x3f as i32 == (*(*io_1).value_.gc).tt as i32
                        && (((*S).L).is_null()
                            || (*(*io_1).value_.gc).marked as i32
                                & ((*(*(*S).L).l_G).currentwhite as i32
                                    ^ ((1 as i32) << 3 as i32 | (1 as i32) << 4 as i32))
                                == 0)
                {
                } else {
                };
            }
            _ => {}
        }
        i += 1;
        i;
    }
}
unsafe extern "C-unwind" fn loadProtos(mut S: *mut LoadState, mut f: *mut Proto) {
    let mut i: i32 = 0;
    let mut n: i32 = loadInt(S);
    if size_of::<i32>() as usize >= size_of::<size_t>() as usize
        && (n as size_t).wrapping_add(1 as i32 as size_t)
            > (!(0 as size_t)).wrapping_div(size_of::<*mut Proto>() as usize)
    {
        luaM_toobig((*S).L);
    } else {
    };
    (*f).p = luaM_malloc_(
        (*S).L,
        (n as usize).wrapping_mul(size_of::<*mut Proto>() as usize),
        0,
    ) as *mut *mut Proto;
    (*f).sizep = n;
    i = 0;
    while i < n {
        let ref mut fresh3 = *((*f).p).offset(i as isize);
        *fresh3 = 0 as *mut Proto;
        i += 1;
        i;
    }
    i = 0;
    while i < n {
        let ref mut fresh4 = *((*f).p).offset(i as isize);
        *fresh4 = luaF_newproto((*S).L);
        if (*f).marked as i32 & (1 as i32) << 5 as i32 != 0
            && (**((*f).p).offset(i as isize)).marked as i32
                & ((1 as i32) << 3 as i32 | (1 as i32) << 4 as i32)
                != 0
        {
            luaC_barrier_(
                (*S).L,
                &mut (*(f as *mut GCUnion)).gc,
                &mut (*(*((*f).p).offset(i as isize) as *mut GCUnion)).gc,
            );
        } else {
        };
        loadFunction(S, *((*f).p).offset(i as isize), (*f).source);
        i += 1;
        i;
    }
}
unsafe extern "C-unwind" fn loadUpvalues(mut S: *mut LoadState, mut f: *mut Proto) {
    let mut i: i32 = 0;
    let mut n: i32 = 0;
    n = loadInt(S);
    if size_of::<i32>() as usize >= size_of::<size_t>() as usize
        && (n as size_t).wrapping_add(1 as i32 as size_t)
            > (!(0 as size_t)).wrapping_div(size_of::<Upvaldesc>() as usize)
    {
        luaM_toobig((*S).L);
    } else {
    };
    (*f).upvalues = luaM_malloc_(
        (*S).L,
        (n as usize).wrapping_mul(size_of::<Upvaldesc>() as usize),
        0,
    ) as *mut Upvaldesc;
    (*f).sizeupvalues = n;
    i = 0;
    while i < n {
        let ref mut fresh5 = (*((*f).upvalues).offset(i as isize)).name;
        *fresh5 = 0 as *mut TString;
        i += 1;
        i;
    }
    i = 0;
    while i < n {
        (*((*f).upvalues).offset(i as isize)).instack = loadByte(S);
        (*((*f).upvalues).offset(i as isize)).idx = loadByte(S);
        (*((*f).upvalues).offset(i as isize)).kind = loadByte(S);
        i += 1;
        i;
    }
}
unsafe extern "C-unwind" fn loadDebug(mut S: *mut LoadState, mut f: *mut Proto) {
    let mut i: i32 = 0;
    let mut n: i32 = 0;
    n = loadInt(S);
    if size_of::<i32>() as usize >= size_of::<size_t>() as usize
        && (n as size_t).wrapping_add(1 as i32 as size_t)
            > (!(0 as size_t)).wrapping_div(size_of::<ls_byte>() as usize)
    {
        luaM_toobig((*S).L);
    } else {
    };
    (*f).lineinfo = luaM_malloc_(
        (*S).L,
        (n as usize).wrapping_mul(size_of::<ls_byte>() as usize),
        0,
    ) as *mut ls_byte;
    (*f).sizelineinfo = n;
    loadBlock(
        S,
        (*f).lineinfo as *mut c_void,
        (n as usize).wrapping_mul(size_of::<ls_byte>() as usize),
    );
    n = loadInt(S);
    if size_of::<i32>() as usize >= size_of::<size_t>() as usize
        && (n as size_t).wrapping_add(1 as i32 as size_t)
            > (!(0 as size_t)).wrapping_div(size_of::<AbsLineInfo>() as usize)
    {
        luaM_toobig((*S).L);
    } else {
    };
    (*f).abslineinfo = luaM_malloc_(
        (*S).L,
        (n as usize).wrapping_mul(size_of::<AbsLineInfo>() as usize),
        0,
    ) as *mut AbsLineInfo;
    (*f).sizeabslineinfo = n;
    i = 0;
    while i < n {
        (*((*f).abslineinfo).offset(i as isize)).pc = loadInt(S);
        (*((*f).abslineinfo).offset(i as isize)).line = loadInt(S);
        i += 1;
        i;
    }
    n = loadInt(S);
    if size_of::<i32>() as usize >= size_of::<size_t>() as usize
        && (n as size_t).wrapping_add(1 as i32 as size_t)
            > (!(0 as size_t)).wrapping_div(size_of::<LocVar>() as usize)
    {
        luaM_toobig((*S).L);
    } else {
    };
    (*f).locvars = luaM_malloc_(
        (*S).L,
        (n as usize).wrapping_mul(size_of::<LocVar>() as usize),
        0,
    ) as *mut LocVar;
    (*f).sizelocvars = n;
    i = 0;
    while i < n {
        let ref mut fresh6 = (*((*f).locvars).offset(i as isize)).varname;
        *fresh6 = 0 as *mut TString;
        i += 1;
        i;
    }
    i = 0;
    while i < n {
        let ref mut fresh7 = (*((*f).locvars).offset(i as isize)).varname;
        *fresh7 = loadStringN(S, f);
        (*((*f).locvars).offset(i as isize)).startpc = loadInt(S);
        (*((*f).locvars).offset(i as isize)).endpc = loadInt(S);
        i += 1;
        i;
    }
    n = loadInt(S);
    if n != 0 {
        n = (*f).sizeupvalues;
    }
    i = 0;
    while i < n {
        let ref mut fresh8 = (*((*f).upvalues).offset(i as isize)).name;
        *fresh8 = loadStringN(S, f);
        i += 1;
        i;
    }
}
unsafe extern "C-unwind" fn loadFunction(
    mut S: *mut LoadState,
    mut f: *mut Proto,
    mut psource: *mut TString,
) {
    (*f).source = loadStringN(S, f);
    if ((*f).source).is_null() {
        (*f).source = psource;
    }
    (*f).linedefined = loadInt(S);
    (*f).lastlinedefined = loadInt(S);
    (*f).numparams = loadByte(S);
    (*f).is_vararg = loadByte(S);
    (*f).maxstacksize = loadByte(S);
    loadCode(S, f);
    loadConstants(S, f);
    loadUpvalues(S, f);
    loadProtos(S, f);
    loadDebug(S, f);
}
unsafe extern "C-unwind" fn checkliteral(
    mut S: *mut LoadState,
    mut s: *const std::ffi::c_char,
    mut msg: *const std::ffi::c_char,
) {
    let mut buff: [std::ffi::c_char; 12] = [0; 12];
    let mut len: size_t = strlen(s);
    loadBlock(
        S,
        buff.as_mut_ptr() as *mut c_void,
        len.wrapping_mul(size_of::<std::ffi::c_char>() as usize),
    );
    if memcmp(s as *const c_void, buff.as_mut_ptr() as *const c_void, len) != 0 {
        error(S, msg);
    }
}
unsafe extern "C-unwind" fn fchecksize(
    mut S: *mut LoadState,
    mut size: size_t,
    mut tname: *const std::ffi::c_char,
) {
    if loadByte(S) as size_t != size {
        error(
            S,
            luaO_pushfstring((*S).L, c"%s size mismatch".as_ptr(), tname),
        );
    }
}
unsafe extern "C-unwind" fn checkHeader(mut S: *mut LoadState) {
    checkliteral(
        S,
        &*(c"\x1BLua".as_ptr()).offset(1),
        c"not a binary chunk".as_ptr(),
    );
    if loadByte(S) as i32 != 504 as i32 / 100 * 16 as i32 + 504 as i32 % 100 {
        error(S, c"version mismatch".as_ptr());
    }
    if loadByte(S) as i32 != 0 {
        error(S, c"format mismatch".as_ptr());
    }
    checkliteral(
        S,
        c"\x19\x93\r\n\x1A\n".as_ptr(),
        c"corrupted chunk".as_ptr(),
    );
    fchecksize(
        S,
        size_of::<Instruction>() as usize,
        c"Instruction".as_ptr(),
    );
    fchecksize(
        S,
        size_of::<lua_Integer>() as usize,
        c"lua_Integer".as_ptr(),
    );
    fchecksize(S, size_of::<lua_Number>() as usize, c"lua_Number".as_ptr());
    if loadInteger(S) != 0x5678 as i32 as lua_Integer {
        error(S, c"integer format mismatch".as_ptr());
    }
    if loadNumber(S) != 370.5f64 {
        error(S, c"float format mismatch".as_ptr());
    }
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaU_undump(
    mut L: *mut lua_State,
    mut Z: *mut ZIO,
    mut name: *const std::ffi::c_char,
) -> *mut LClosure {
    let mut S: LoadState = LoadState {
        L: 0 as *mut lua_State,
        Z: 0 as *mut ZIO,
        name: 0 as *const std::ffi::c_char,
    };
    let mut cl: *mut LClosure = 0 as *mut LClosure;
    if *name as i32 == '@' as i32 || *name as i32 == '=' as i32 {
        S.name = name.offset(1);
    } else if *name as i32
        == (*::core::mem::transmute::<&[u8; 5], &[std::ffi::c_char; 5]>(b"\x1BLua\0"))[0 as usize]
            as i32
    {
        S.name = c"binary string".as_ptr();
    } else {
        S.name = name;
    }
    S.L = L;
    S.Z = Z;
    checkHeader(&mut S);
    cl = luaF_newLclosure(L, loadByte(&mut S) as i32);
    let mut io: *mut TValue = &mut (*(*L).top.p).val;
    let mut x_: *mut LClosure = cl;
    (*io).value_.gc = &mut (*(x_ as *mut GCUnion)).gc;
    (*io).tt_ = (6 as i32 | (0) << 4 as i32 | (1 as i32) << 6 as i32) as lu_byte;
    if (*io).tt_ as i32 & (1 as i32) << 6 as i32 == 0
        || (*io).tt_ as i32 & 0x3f as i32 == (*(*io).value_.gc).tt as i32
            && (L.is_null()
                || (*(*io).value_.gc).marked as i32
                    & ((*(*L).l_G).currentwhite as i32
                        ^ ((1 as i32) << 3 as i32 | (1 as i32) << 4 as i32))
                    == 0)
    {
    } else {
    };
    luaD_inctop(L);
    (*cl).p = luaF_newproto(L);
    if (*cl).marked as i32 & (1 as i32) << 5 as i32 != 0
        && (*(*cl).p).marked as i32 & ((1 as i32) << 3 as i32 | (1 as i32) << 4 as i32) != 0
    {
        luaC_barrier_(
            L,
            &mut (*(cl as *mut GCUnion)).gc,
            &mut (*((*cl).p as *mut GCUnion)).gc,
        );
    } else {
    };
    loadFunction(&mut S, (*cl).p, 0 as *mut TString);
    return cl;
}
