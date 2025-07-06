use crate::*;

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaD_seterrorobj(
    mut L: *mut lua_State,
    mut errcode: i32,
    mut oldtop: StkId,
) {
    match errcode {
        4 => {
            let mut io: *mut TValue = &mut (*oldtop).val;
            let mut x_: *mut TString = (*(*L).l_G).memerrmsg;
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
        }
        5 => {
            let mut io_0: *mut TValue = &mut (*oldtop).val;
            let mut x__0: *mut TString = luaS_newlstr(
                L,
                c"error in error handling".as_ptr(),
                (::core::mem::size_of::<[std::ffi::c_char; 24]>() as usize)
                    .wrapping_div(::core::mem::size_of::<std::ffi::c_char>() as usize)
                    .wrapping_sub(1),
            );
            (*io_0).value_.gc = &mut (*(x__0 as *mut GCUnion)).gc;
            (*io_0).tt_ = ((*x__0).tt as i32 | (1 as i32) << 6 as i32) as lu_byte;
            if (*io_0).tt_ as i32 & (1 as i32) << 6 as i32 == 0
                || (*io_0).tt_ as i32 & 0x3f as i32 == (*(*io_0).value_.gc).tt as i32
                    && (L.is_null()
                        || (*(*io_0).value_.gc).marked as i32
                            & ((*(*L).l_G).currentwhite as i32
                                ^ ((1 as i32) << 3 as i32 | (1 as i32) << 4 as i32))
                            == 0)
            {
            } else {
            };
        }
        0 => {
            (*oldtop).val.tt_ = (0 | (0) << 4 as i32) as lu_byte;
        }
        _ => {
            let mut io1: *mut TValue = &mut (*oldtop).val;
            let mut io2: *const TValue = &mut (*((*L).top.p).offset(-(1))).val;
            (*io1).value_ = (*io2).value_;
            (*io1).tt_ = (*io2).tt_;
            if (*io1).tt_ as i32 & (1 as i32) << 6 as i32 == 0
                || (*io1).tt_ as i32 & 0x3f as i32 == (*(*io1).value_.gc).tt as i32
                    && (L.is_null()
                        || (*(*io1).value_.gc).marked as i32
                            & ((*(*L).l_G).currentwhite as i32
                                ^ ((1 as i32) << 3 as i32 | (1 as i32) << 4 as i32))
                            == 0)
            {
            } else {
            };
        }
    }
    (*L).top.p = oldtop.offset(1);
}
struct LuaThrow;
#[unsafe(no_mangle)]
pub unsafe fn luaD_throw(mut L: *mut lua_State, mut errcode: i32) -> ! {
    if !((*L).errorJmp).is_null() {
        ::core::ptr::write_volatile(&mut (*(*L).errorJmp).status as *mut i32, errcode);
        std::panic::resume_unwind(Box::new(LuaThrow));
    } else {
        let mut g: *mut global_State = (*L).l_G;
        errcode = luaE_resetthread(L, errcode);
        if !((*(*g).mainthread).errorJmp).is_null() {
            let fresh128 = (*(*g).mainthread).top.p;
            (*(*g).mainthread).top.p = ((*(*g).mainthread).top.p).offset(1);
            let mut io1: *mut TValue = &mut (*fresh128).val;
            let mut io2: *const TValue = &mut (*((*L).top.p).offset(-(1))).val;
            (*io1).value_ = (*io2).value_;
            (*io1).tt_ = (*io2).tt_;
            if (*io1).tt_ as i32 & (1 as i32) << 6 as i32 == 0
                || (*io1).tt_ as i32 & 0x3f as i32 == (*(*io1).value_.gc).tt as i32
                    && (L.is_null()
                        || (*(*io1).value_.gc).marked as i32
                            & ((*(*L).l_G).currentwhite as i32
                                ^ ((1 as i32) << 3 as i32 | (1 as i32) << 4 as i32))
                            == 0)
            {
            } else {
            };
            luaD_throw((*g).mainthread, errcode);
        } else {
            if ((*g).panic).is_some() {
                ((*g).panic).expect("non-null function pointer")(L);
            }
            abort();
        }
    };
}
#[unsafe(no_mangle)]
pub unsafe fn luaD_rawrunprotected(
    mut L: *mut lua_State,
    mut f: Pfunc,
    mut ud: *mut c_void,
) -> i32 {
    let mut oldnCcalls: l_uint32 = (*L).nCcalls;
    let mut lj: lua_longjmp = lua_longjmp {
        previous: 0 as *mut lua_longjmp,
        status: 0,
    };
    ::core::ptr::write_volatile(&mut lj.status as *mut i32, 0);
    lj.previous = (*L).errorJmp;
    (*L).errorJmp = &mut lj;

    let res = std::panic::catch_unwind(|| {
        (Some(f.expect("non-null function pointer"))).expect("non-null function pointer")(L, ud);
    });

    if res.as_ref().is_err_and(|err| !err.is::<LuaThrow>()) {
        std::panic::resume_unwind(res.unwrap_err());
    }

    (*L).errorJmp = lj.previous;
    (*L).nCcalls = oldnCcalls;
    return lj.status;
}
unsafe extern "C-unwind" fn relstack(mut L: *mut lua_State) {
    let mut ci: *mut CallInfo = 0 as *mut CallInfo;
    let mut up: *mut UpVal = 0 as *mut UpVal;
    (*L).top.offset =
        ((*L).top.p as *mut std::ffi::c_char).offset_from((*L).stack.p as *mut std::ffi::c_char);
    (*L).tbclist.offset = ((*L).tbclist.p as *mut std::ffi::c_char)
        .offset_from((*L).stack.p as *mut std::ffi::c_char);
    up = (*L).openupval;
    while !up.is_null() {
        (*up).v.offset = ((*up).v.p as StkId as *mut std::ffi::c_char)
            .offset_from((*L).stack.p as *mut std::ffi::c_char);
        up = (*up).u.open.next;
    }
    ci = (*L).ci;
    while !ci.is_null() {
        (*ci).top.offset = ((*ci).top.p as *mut std::ffi::c_char)
            .offset_from((*L).stack.p as *mut std::ffi::c_char);
        (*ci).func.offset = ((*ci).func.p as *mut std::ffi::c_char)
            .offset_from((*L).stack.p as *mut std::ffi::c_char);
        ci = (*ci).previous;
    }
}
unsafe extern "C-unwind" fn correctstack(mut L: *mut lua_State) {
    let mut ci: *mut CallInfo = 0 as *mut CallInfo;
    let mut up: *mut UpVal = 0 as *mut UpVal;
    (*L).top.p = ((*L).stack.p as *mut std::ffi::c_char).offset((*L).top.offset as isize) as StkId;
    (*L).tbclist.p =
        ((*L).stack.p as *mut std::ffi::c_char).offset((*L).tbclist.offset as isize) as StkId;
    up = (*L).openupval;
    while !up.is_null() {
        (*up).v.p = &mut (*(((*L).stack.p as *mut std::ffi::c_char).offset((*up).v.offset as isize)
            as StkId))
            .val;
        up = (*up).u.open.next;
    }
    ci = (*L).ci;
    while !ci.is_null() {
        (*ci).top.p =
            ((*L).stack.p as *mut std::ffi::c_char).offset((*ci).top.offset as isize) as StkId;
        (*ci).func.p =
            ((*L).stack.p as *mut std::ffi::c_char).offset((*ci).func.offset as isize) as StkId;
        if (*ci).callstatus as i32 & (1 as i32) << 1 as i32 == 0 {
            ::core::ptr::write_volatile(&mut (*ci).u.l.trap as *mut sig_atomic_t, 1 as i32);
        }
        ci = (*ci).previous;
    }
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaD_reallocstack(
    mut L: *mut lua_State,
    mut newsize: i32,
    mut raiseerror: i32,
) -> i32 {
    let mut oldsize: i32 = ((*L).stack_last.p).offset_from((*L).stack.p) as std::ffi::c_long as i32;
    let mut i: i32 = 0;
    let mut newstack: StkId = 0 as *mut StackValue;
    let mut oldgcstop: i32 = (*(*L).l_G).gcstopem as i32;
    relstack(L);
    (*(*L).l_G).gcstopem = 1 as i32 as lu_byte;
    newstack = luaM_realloc_(
        L,
        (*L).stack.p as *mut c_void,
        ((oldsize + 5 as i32) as size_t)
            .wrapping_mul(::core::mem::size_of::<StackValue>() as usize),
        ((newsize + 5 as i32) as size_t)
            .wrapping_mul(::core::mem::size_of::<StackValue>() as usize),
    ) as *mut StackValue;
    (*(*L).l_G).gcstopem = oldgcstop as lu_byte;
    if ((newstack == 0 as *mut c_void as StkId) as i32 != 0) as i32 as std::ffi::c_long != 0 {
        correctstack(L);
        if raiseerror != 0 {
            luaD_throw(L, 4 as i32);
        } else {
            return 0;
        }
    }
    (*L).stack.p = newstack;
    correctstack(L);
    (*L).stack_last.p = ((*L).stack.p).offset(newsize as isize);
    i = oldsize + 5 as i32;
    while i < newsize + 5 as i32 {
        (*newstack.offset(i as isize)).val.tt_ = (0 | (0) << 4 as i32) as lu_byte;
        i += 1;
        i;
    }
    return 1 as i32;
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaD_growstack(
    mut L: *mut lua_State,
    mut n: i32,
    mut raiseerror: i32,
) -> i32 {
    let mut size: i32 = ((*L).stack_last.p).offset_from((*L).stack.p) as std::ffi::c_long as i32;
    if ((size > 1000000) as i32 != 0) as i32 as std::ffi::c_long != 0 {
        if raiseerror != 0 {
            luaD_throw(L, 5 as i32);
        }
        return 0;
    } else if n < 1000000 {
        let mut newsize: i32 = 2 as i32 * size;
        let mut needed: i32 = ((*L).top.p).offset_from((*L).stack.p) as std::ffi::c_long as i32 + n;
        if newsize > 1000000 {
            newsize = 1000000;
        }
        if newsize < needed {
            newsize = needed;
        }
        if ((newsize <= 1000000) as i32 != 0) as i32 as std::ffi::c_long != 0 {
            return luaD_reallocstack(L, newsize, raiseerror);
        }
    }
    luaD_reallocstack(L, 1000000 + 200, raiseerror);
    if raiseerror != 0 {
        luaG_runerror(L, c"stack overflow".as_ptr());
    }
    return 0;
}
unsafe extern "C-unwind" fn stackinuse(mut L: *mut lua_State) -> i32 {
    let mut ci: *mut CallInfo = 0 as *mut CallInfo;
    let mut res: i32 = 0;
    let mut lim: StkId = (*L).top.p;
    ci = (*L).ci;
    while !ci.is_null() {
        if lim < (*ci).top.p {
            lim = (*ci).top.p;
        }
        ci = (*ci).previous;
    }
    res = lim.offset_from((*L).stack.p) as std::ffi::c_long as i32 + 1 as i32;
    if res < 20 {
        res = 20;
    }
    return res;
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaD_shrinkstack(mut L: *mut lua_State) {
    let mut inuse: i32 = stackinuse(L);
    let mut max: i32 = if inuse > 1000000 / 3 as i32 {
        1000000
    } else {
        inuse * 3 as i32
    };
    if inuse <= 1000000
        && ((*L).stack_last.p).offset_from((*L).stack.p) as std::ffi::c_long as i32 > max
    {
        let mut nsize: i32 = if inuse > 1000000 / 2 as i32 {
            1000000
        } else {
            inuse * 2 as i32
        };
        luaD_reallocstack(L, nsize, 0);
    }
    luaE_shrinkCI(L);
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaD_inctop(mut L: *mut lua_State) {
    if ((((*L).stack_last.p).offset_from((*L).top.p) as std::ffi::c_long
        <= 1 as i32 as std::ffi::c_long) as i32
        != 0) as i32 as std::ffi::c_long
        != 0
    {
        luaD_growstack(L, 1 as i32, 1 as i32);
    }
    (*L).top.p = ((*L).top.p).offset(1);
    (*L).top.p;
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaD_hook(
    mut L: *mut lua_State,
    mut event: i32,
    mut line: i32,
    mut ftransfer: i32,
    mut ntransfer: i32,
) {
    let mut hook: lua_Hook = (*L).hook;
    if hook.is_some() && (*L).allowhook as i32 != 0 {
        let mut mask: i32 = (1 as i32) << 3 as i32;
        let mut ci: *mut CallInfo = (*L).ci;
        let mut top: ptrdiff_t = ((*L).top.p as *mut std::ffi::c_char)
            .offset_from((*L).stack.p as *mut std::ffi::c_char);
        let mut ci_top: ptrdiff_t = ((*ci).top.p as *mut std::ffi::c_char)
            .offset_from((*L).stack.p as *mut std::ffi::c_char);
        let mut ar: lua_Debug = lua_Debug {
            event: 0,
            name: 0 as *const std::ffi::c_char,
            namewhat: 0 as *const std::ffi::c_char,
            what: 0 as *const std::ffi::c_char,
            source: 0 as *const std::ffi::c_char,
            srclen: 0,
            currentline: 0,
            linedefined: 0,
            lastlinedefined: 0,
            nups: 0,
            nparams: 0,
            isvararg: 0,
            istailcall: 0,
            ftransfer: 0,
            ntransfer: 0,
            short_src: [0; 60],
            i_ci: 0 as *mut CallInfo,
        };
        ar.event = event;
        ar.currentline = line;
        ar.i_ci = ci;
        if ntransfer != 0 {
            mask |= (1 as i32) << 8 as i32;
            (*ci).u2.transferinfo.ftransfer = ftransfer as u16;
            (*ci).u2.transferinfo.ntransfer = ntransfer as u16;
        }
        if (*ci).callstatus as i32 & (1 as i32) << 1 as i32 == 0 && (*L).top.p < (*ci).top.p {
            (*L).top.p = (*ci).top.p;
        }
        if ((((*L).stack_last.p).offset_from((*L).top.p) as std::ffi::c_long
            <= 20 as std::ffi::c_long) as i32
            != 0) as i32 as std::ffi::c_long
            != 0
        {
            luaD_growstack(L, 20, 1 as i32);
        }
        if (*ci).top.p < ((*L).top.p).offset(20 as isize) {
            (*ci).top.p = ((*L).top.p).offset(20 as isize);
        }
        (*L).allowhook = 0 as lu_byte;
        (*ci).callstatus = ((*ci).callstatus as i32 | mask) as u16;
        (Some(hook.expect("non-null function pointer"))).expect("non-null function pointer")(
            L, &mut ar,
        );
        (*L).allowhook = 1 as i32 as lu_byte;
        (*ci).top.p = ((*L).stack.p as *mut std::ffi::c_char).offset(ci_top as isize) as StkId;
        (*L).top.p = ((*L).stack.p as *mut std::ffi::c_char).offset(top as isize) as StkId;
        (*ci).callstatus = ((*ci).callstatus as i32 & !mask) as u16;
    }
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaD_hookcall(mut L: *mut lua_State, mut ci: *mut CallInfo) {
    (*L).oldpc = 0;
    if (*L).hookmask & (1 as i32) << 0 != 0 {
        let mut event: i32 = if (*ci).callstatus as i32 & (1 as i32) << 5 as i32 != 0 {
            4 as i32
        } else {
            0
        };
        let mut p: *mut Proto = (*&mut (*((*(*ci).func.p).val.value_.gc as *mut GCUnion)).cl.l).p;
        (*ci).u.l.savedpc = ((*ci).u.l.savedpc).offset(1);
        (*ci).u.l.savedpc;
        luaD_hook(L, event, -(1 as i32), 1 as i32, (*p).numparams as i32);
        (*ci).u.l.savedpc = ((*ci).u.l.savedpc).offset(-1);
        (*ci).u.l.savedpc;
    }
}
unsafe extern "C-unwind" fn rethook(mut L: *mut lua_State, mut ci: *mut CallInfo, mut nres: i32) {
    if (*L).hookmask & (1 as i32) << 1 as i32 != 0 {
        let mut firstres: StkId = ((*L).top.p).offset(-(nres as isize));
        let mut delta: i32 = 0;
        let mut ftransfer: i32 = 0;
        if (*ci).callstatus as i32 & (1 as i32) << 1 as i32 == 0 {
            let mut p: *mut Proto =
                (*&mut (*((*(*ci).func.p).val.value_.gc as *mut GCUnion)).cl.l).p;
            if (*p).is_vararg != 0 {
                delta = (*ci).u.l.nextraargs + (*p).numparams as i32 + 1 as i32;
            }
        }
        (*ci).func.p = ((*ci).func.p).offset(delta as isize);
        ftransfer =
            firstres.offset_from((*ci).func.p) as std::ffi::c_long as u16 as i32;
        luaD_hook(L, 1 as i32, -(1 as i32), ftransfer, nres);
        (*ci).func.p = ((*ci).func.p).offset(-(delta as isize));
    }
    ci = (*ci).previous;
    if (*ci).callstatus as i32 & (1 as i32) << 1 as i32 == 0 {
        (*L).oldpc = ((*ci).u.l.savedpc)
            .offset_from((*(*&mut (*((*(*ci).func.p).val.value_.gc as *mut GCUnion)).cl.l).p).code)
            as std::ffi::c_long as i32
            - 1 as i32;
    }
}
unsafe extern "C-unwind" fn tryfuncTM(mut L: *mut lua_State, mut func: StkId) -> StkId {
    let mut tm: *const TValue = 0 as *const TValue;
    let mut p: StkId = 0 as *mut StackValue;
    if ((((*L).stack_last.p).offset_from((*L).top.p) as std::ffi::c_long
        <= 1 as i32 as std::ffi::c_long) as i32
        != 0) as i32 as std::ffi::c_long
        != 0
    {
        let mut t__: ptrdiff_t =
            (func as *mut std::ffi::c_char).offset_from((*L).stack.p as *mut std::ffi::c_char);
        if (*(*L).l_G).GCdebt > 0 as l_mem {
            luaC_step(L);
        }
        luaD_growstack(L, 1 as i32, 1 as i32);
        func = ((*L).stack.p as *mut std::ffi::c_char).offset(t__ as isize) as StkId;
    }
    tm = luaT_gettmbyobj(L, &mut (*func).val, TM_CALL);
    if (((*tm).tt_ as i32 & 0xf as i32 == 0) as i32 != 0) as i32 as std::ffi::c_long != 0 {
        luaG_callerror(L, &mut (*func).val);
    }
    p = (*L).top.p;
    while p > func {
        let mut io1: *mut TValue = &mut (*p).val;
        let mut io2: *const TValue = &mut (*p.offset(-(1))).val;
        (*io1).value_ = (*io2).value_;
        (*io1).tt_ = (*io2).tt_;
        if (*io1).tt_ as i32 & (1 as i32) << 6 as i32 == 0
            || (*io1).tt_ as i32 & 0x3f as i32 == (*(*io1).value_.gc).tt as i32
                && (L.is_null()
                    || (*(*io1).value_.gc).marked as i32
                        & ((*(*L).l_G).currentwhite as i32
                            ^ ((1 as i32) << 3 as i32 | (1 as i32) << 4 as i32))
                        == 0)
        {
        } else {
        };
        p = p.offset(-1);
        p;
    }
    (*L).top.p = ((*L).top.p).offset(1);
    (*L).top.p;
    let mut io1_0: *mut TValue = &mut (*func).val;
    let mut io2_0: *const TValue = tm;
    (*io1_0).value_ = (*io2_0).value_;
    (*io1_0).tt_ = (*io2_0).tt_;
    if (*io1_0).tt_ as i32 & (1 as i32) << 6 as i32 == 0
        || (*io1_0).tt_ as i32 & 0x3f as i32 == (*(*io1_0).value_.gc).tt as i32
            && (L.is_null()
                || (*(*io1_0).value_.gc).marked as i32
                    & ((*(*L).l_G).currentwhite as i32
                        ^ ((1 as i32) << 3 as i32 | (1 as i32) << 4 as i32))
                    == 0)
    {
    } else {
    };
    return func;
}
#[inline]
unsafe extern "C-unwind" fn moveresults(
    mut L: *mut lua_State,
    mut res: StkId,
    mut nres: i32,
    mut wanted: i32,
) {
    let mut firstresult: StkId = 0 as *mut StackValue;
    let mut i: i32 = 0;
    match wanted {
        0 => {
            (*L).top.p = res;
            return;
        }
        1 => {
            if nres == 0 {
                (*res).val.tt_ = (0 | (0) << 4 as i32) as lu_byte;
            } else {
                let mut io1: *mut TValue = &mut (*res).val;
                let mut io2: *const TValue = &mut (*((*L).top.p).offset(-(nres as isize))).val;
                (*io1).value_ = (*io2).value_;
                (*io1).tt_ = (*io2).tt_;
                if (*io1).tt_ as i32 & (1 as i32) << 6 as i32 == 0
                    || (*io1).tt_ as i32 & 0x3f as i32 == (*(*io1).value_.gc).tt as i32
                        && (L.is_null()
                            || (*(*io1).value_.gc).marked as i32
                                & ((*(*L).l_G).currentwhite as i32
                                    ^ ((1 as i32) << 3 as i32 | (1 as i32) << 4 as i32))
                                == 0)
                {
                } else {
                };
            }
            (*L).top.p = res.offset(1);
            return;
        }
        -1 => {
            wanted = nres;
        }
        _ => {
            if wanted < -(1 as i32) {
                (*(*L).ci).callstatus =
                    ((*(*L).ci).callstatus as i32 | (1 as i32) << 9 as i32) as u16;
                (*(*L).ci).u2.nres = nres;
                res = luaF_close(L, res, -(1 as i32), 1 as i32);
                (*(*L).ci).callstatus = ((*(*L).ci).callstatus as i32 & !((1 as i32) << 9 as i32))
                    as u16;
                if (*L).hookmask != 0 {
                    let mut savedres: ptrdiff_t = (res as *mut std::ffi::c_char)
                        .offset_from((*L).stack.p as *mut std::ffi::c_char);
                    rethook(L, (*L).ci, nres);
                    res =
                        ((*L).stack.p as *mut std::ffi::c_char).offset(savedres as isize) as StkId;
                }
                wanted = -wanted - 3 as i32;
                if wanted == -(1 as i32) {
                    wanted = nres;
                }
            }
        }
    }
    firstresult = ((*L).top.p).offset(-(nres as isize));
    if nres > wanted {
        nres = wanted;
    }
    i = 0;
    while i < nres {
        let mut io1_0: *mut TValue = &mut (*res.offset(i as isize)).val;
        let mut io2_0: *const TValue = &mut (*firstresult.offset(i as isize)).val;
        (*io1_0).value_ = (*io2_0).value_;
        (*io1_0).tt_ = (*io2_0).tt_;
        if (*io1_0).tt_ as i32 & (1 as i32) << 6 as i32 == 0
            || (*io1_0).tt_ as i32 & 0x3f as i32 == (*(*io1_0).value_.gc).tt as i32
                && (L.is_null()
                    || (*(*io1_0).value_.gc).marked as i32
                        & ((*(*L).l_G).currentwhite as i32
                            ^ ((1 as i32) << 3 as i32 | (1 as i32) << 4 as i32))
                        == 0)
        {
        } else {
        };
        i += 1;
        i;
    }
    while i < wanted {
        (*res.offset(i as isize)).val.tt_ = (0 | (0) << 4 as i32) as lu_byte;
        i += 1;
        i;
    }
    (*L).top.p = res.offset(wanted as isize);
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaD_poscall(
    mut L: *mut lua_State,
    mut ci: *mut CallInfo,
    mut nres: i32,
) {
    let mut wanted: i32 = (*ci).nresults as i32;
    if (((*L).hookmask != 0 && !(wanted < -(1 as i32))) as i32 != 0) as i32 as std::ffi::c_long != 0
    {
        rethook(L, ci, nres);
    }
    moveresults(L, (*ci).func.p, nres, wanted);
    (*L).ci = (*ci).previous;
}
#[inline]
unsafe extern "C-unwind" fn prepCallInfo(
    mut L: *mut lua_State,
    mut func: StkId,
    mut nret: i32,
    mut mask: i32,
    mut top: StkId,
) -> *mut CallInfo {
    (*L).ci = if !((*(*L).ci).next).is_null() {
        (*(*L).ci).next
    } else {
        luaE_extendCI(L)
    };
    let mut ci: *mut CallInfo = (*L).ci;
    (*ci).func.p = func;
    (*ci).nresults = nret as i16;
    (*ci).callstatus = mask as u16;
    (*ci).top.p = top;
    return ci;
}
#[inline]
unsafe extern "C-unwind" fn precallC(
    mut L: *mut lua_State,
    mut func: StkId,
    mut nresults: i32,
    mut f: lua_CFunction,
) -> i32 {
    let mut n: i32 = 0;
    let mut ci: *mut CallInfo = 0 as *mut CallInfo;
    if ((((*L).stack_last.p).offset_from((*L).top.p) as std::ffi::c_long <= 20 as std::ffi::c_long)
        as i32
        != 0) as i32 as std::ffi::c_long
        != 0
    {
        let mut t__: ptrdiff_t =
            (func as *mut std::ffi::c_char).offset_from((*L).stack.p as *mut std::ffi::c_char);
        if (*(*L).l_G).GCdebt > 0 as l_mem {
            luaC_step(L);
        }
        luaD_growstack(L, 20, 1 as i32);
        func = ((*L).stack.p as *mut std::ffi::c_char).offset(t__ as isize) as StkId;
    }
    ci = prepCallInfo(
        L,
        func,
        nresults,
        (1 as i32) << 1 as i32,
        ((*L).top.p).offset(20 as isize),
    );
    (*L).ci = ci;
    if ((*L).hookmask & (1 as i32) << 0 != 0) as i32 as std::ffi::c_long != 0 {
        let mut narg: i32 = ((*L).top.p).offset_from(func) as std::ffi::c_long as i32 - 1 as i32;
        luaD_hook(L, 0, -(1 as i32), 1 as i32, narg);
    }
    n = (Some(f.expect("non-null function pointer"))).expect("non-null function pointer")(L);
    luaD_poscall(L, ci, n);
    return n;
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaD_pretailcall(
    mut L: *mut lua_State,
    mut ci: *mut CallInfo,
    mut func: StkId,
    mut narg1: i32,
    mut delta: i32,
) -> i32 {
    loop {
        match (*func).val.tt_ as i32 & 0x3f as i32 {
            38 => {
                return precallC(
                    L,
                    func,
                    -(1 as i32),
                    (*&mut (*((*func).val.value_.gc as *mut GCUnion)).cl.c).f,
                );
            }
            22 => return precallC(L, func, -(1 as i32), (*func).val.value_.f),
            6 => {
                let mut p: *mut Proto = (*&mut (*((*func).val.value_.gc as *mut GCUnion)).cl.l).p;
                let mut fsize: i32 = (*p).maxstacksize as i32;
                let mut nfixparams: i32 = (*p).numparams as i32;
                let mut i: i32 = 0;
                if ((((*L).stack_last.p).offset_from((*L).top.p) as std::ffi::c_long
                    <= (fsize - delta) as std::ffi::c_long) as i32
                    != 0) as i32 as std::ffi::c_long
                    != 0
                {
                    let mut t__: ptrdiff_t = (func as *mut std::ffi::c_char)
                        .offset_from((*L).stack.p as *mut std::ffi::c_char);
                    if (*(*L).l_G).GCdebt > 0 as l_mem {
                        luaC_step(L);
                    }
                    luaD_growstack(L, fsize - delta, 1 as i32);
                    func = ((*L).stack.p as *mut std::ffi::c_char).offset(t__ as isize) as StkId;
                }
                (*ci).func.p = ((*ci).func.p).offset(-(delta as isize));
                i = 0;
                while i < narg1 {
                    let mut io1: *mut TValue = &mut (*((*ci).func.p).offset(i as isize)).val;
                    let mut io2: *const TValue = &mut (*func.offset(i as isize)).val;
                    (*io1).value_ = (*io2).value_;
                    (*io1).tt_ = (*io2).tt_;
                    if (*io1).tt_ as i32 & (1 as i32) << 6 as i32 == 0
                        || (*io1).tt_ as i32 & 0x3f as i32 == (*(*io1).value_.gc).tt as i32
                            && (L.is_null()
                                || (*(*io1).value_.gc).marked as i32
                                    & ((*(*L).l_G).currentwhite as i32
                                        ^ ((1 as i32) << 3 as i32 | (1 as i32) << 4 as i32))
                                    == 0)
                    {
                    } else {
                    };
                    i += 1;
                    i;
                }
                func = (*ci).func.p;
                while narg1 <= nfixparams {
                    (*func.offset(narg1 as isize)).val.tt_ = (0 | (0) << 4 as i32) as lu_byte;
                    narg1 += 1;
                    narg1;
                }
                (*ci).top.p = func.offset(1).offset(fsize as isize);
                (*ci).u.l.savedpc = (*p).code;
                (*ci).callstatus =
                    ((*ci).callstatus as i32 | (1 as i32) << 5 as i32) as u16;
                (*L).top.p = func.offset(narg1 as isize);
                return -(1 as i32);
            }
            _ => {
                func = tryfuncTM(L, func);
                narg1 += 1;
                narg1;
            }
        }
    }
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaD_precall(
    mut L: *mut lua_State,
    mut func: StkId,
    mut nresults: i32,
) -> *mut CallInfo {
    loop {
        match (*func).val.tt_ as i32 & 0x3f as i32 {
            38 => {
                precallC(
                    L,
                    func,
                    nresults,
                    (*&mut (*((*func).val.value_.gc as *mut GCUnion)).cl.c).f,
                );
                return 0 as *mut CallInfo;
            }
            22 => {
                precallC(L, func, nresults, (*func).val.value_.f);
                return 0 as *mut CallInfo;
            }
            6 => {
                let mut ci: *mut CallInfo = 0 as *mut CallInfo;
                let mut p: *mut Proto = (*&mut (*((*func).val.value_.gc as *mut GCUnion)).cl.l).p;
                let mut narg: i32 =
                    ((*L).top.p).offset_from(func) as std::ffi::c_long as i32 - 1 as i32;
                let mut nfixparams: i32 = (*p).numparams as i32;
                let mut fsize: i32 = (*p).maxstacksize as i32;
                if ((((*L).stack_last.p).offset_from((*L).top.p) as std::ffi::c_long
                    <= fsize as std::ffi::c_long) as i32
                    != 0) as i32 as std::ffi::c_long
                    != 0
                {
                    let mut t__: ptrdiff_t = (func as *mut std::ffi::c_char)
                        .offset_from((*L).stack.p as *mut std::ffi::c_char);
                    if (*(*L).l_G).GCdebt > 0 as l_mem {
                        luaC_step(L);
                    }
                    luaD_growstack(L, fsize, 1 as i32);
                    func = ((*L).stack.p as *mut std::ffi::c_char).offset(t__ as isize) as StkId;
                }
                ci = prepCallInfo(
                    L,
                    func,
                    nresults,
                    0,
                    func.offset(1).offset(fsize as isize),
                );
                (*L).ci = ci;
                (*ci).u.l.savedpc = (*p).code;
                while narg < nfixparams {
                    let fresh129 = (*L).top.p;
                    (*L).top.p = ((*L).top.p).offset(1);
                    (*fresh129).val.tt_ = (0 | (0) << 4 as i32) as lu_byte;
                    narg += 1;
                    narg;
                }
                return ci;
            }
            _ => {
                func = tryfuncTM(L, func);
            }
        }
    }
}
#[inline]
unsafe extern "C-unwind" fn ccall(
    mut L: *mut lua_State,
    mut func: StkId,
    mut nResults: i32,
    mut inc: l_uint32,
) {
    let mut ci: *mut CallInfo = 0 as *mut CallInfo;
    (*L).nCcalls = ((*L).nCcalls).wrapping_add(inc);
    if (((*L).nCcalls & 0xffff as i32 as l_uint32 >= 200 as l_uint32) as i32 != 0) as i32
        as std::ffi::c_long
        != 0
    {
        if ((((*L).stack_last.p).offset_from((*L).top.p) as std::ffi::c_long
            <= 0 as std::ffi::c_long) as i32
            != 0) as i32 as std::ffi::c_long
            != 0
        {
            let mut t__: ptrdiff_t =
                (func as *mut std::ffi::c_char).offset_from((*L).stack.p as *mut std::ffi::c_char);
            luaD_growstack(L, 0, 1 as i32);
            func = ((*L).stack.p as *mut std::ffi::c_char).offset(t__ as isize) as StkId;
        }
        luaE_checkcstack(L);
    }
    ci = luaD_precall(L, func, nResults);
    if !ci.is_null() {
        (*ci).callstatus = ((1 as i32) << 2 as i32) as u16;
        luaV_execute(L, ci);
    }
    (*L).nCcalls = ((*L).nCcalls).wrapping_sub(inc);
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaD_call(
    mut L: *mut lua_State,
    mut func: StkId,
    mut nResults: i32,
) {
    ccall(L, func, nResults, 1 as i32 as l_uint32);
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaD_callnoyield(
    mut L: *mut lua_State,
    mut func: StkId,
    mut nResults: i32,
) {
    ccall(L, func, nResults, (0x10000 | 1 as i32) as l_uint32);
}
unsafe extern "C-unwind" fn finishpcallk(mut L: *mut lua_State, mut ci: *mut CallInfo) -> i32 {
    let mut status: i32 = (*ci).callstatus as i32 >> 10 & 7 as i32;
    if ((status == 0) as i32 != 0) as i32 as std::ffi::c_long != 0 {
        status = 1 as i32;
    } else {
        let mut func: StkId =
            ((*L).stack.p as *mut std::ffi::c_char).offset((*ci).u2.funcidx as isize) as StkId;
        (*L).allowhook = ((*ci).callstatus as i32 & (1 as i32) << 0) as lu_byte;
        func = luaF_close(L, func, status, 1 as i32);
        luaD_seterrorobj(L, status, func);
        luaD_shrinkstack(L);
        (*ci).callstatus =
            ((*ci).callstatus as i32 & !((7 as i32) << 10) | (0) << 10) as u16;
    }
    (*ci).callstatus = ((*ci).callstatus as i32 & !((1 as i32) << 4 as i32)) as u16;
    (*L).errfunc = (*ci).u.c.old_errfunc;
    return status;
}
unsafe extern "C-unwind" fn finishCcall(mut L: *mut lua_State, mut ci: *mut CallInfo) {
    let mut n: i32 = 0;
    if (*ci).callstatus as i32 & (1 as i32) << 9 as i32 != 0 {
        n = (*ci).u2.nres;
    } else {
        let mut status: i32 = 1 as i32;
        if (*ci).callstatus as i32 & (1 as i32) << 4 as i32 != 0 {
            status = finishpcallk(L, ci);
        }
        if -(1 as i32) <= -(1 as i32) && (*(*L).ci).top.p < (*L).top.p {
            (*(*L).ci).top.p = (*L).top.p;
        }
        n = (Some(((*ci).u.c.k).expect("non-null function pointer")))
            .expect("non-null function pointer")(L, status, (*ci).u.c.ctx);
    }
    luaD_poscall(L, ci, n);
}
unsafe extern "C-unwind" fn unroll(mut L: *mut lua_State, mut ud: *mut c_void) {
    let mut ci: *mut CallInfo = 0 as *mut CallInfo;
    loop {
        ci = (*L).ci;
        if !(ci != &mut (*L).base_ci as *mut CallInfo) {
            break;
        }
        if (*ci).callstatus as i32 & (1 as i32) << 1 as i32 != 0 {
            finishCcall(L, ci);
        } else {
            luaV_finishOp(L);
            luaV_execute(L, ci);
        }
    }
}
unsafe extern "C-unwind" fn findpcall(mut L: *mut lua_State) -> *mut CallInfo {
    let mut ci: *mut CallInfo = 0 as *mut CallInfo;
    ci = (*L).ci;
    while !ci.is_null() {
        if (*ci).callstatus as i32 & (1 as i32) << 4 as i32 != 0 {
            return ci;
        }
        ci = (*ci).previous;
    }
    return 0 as *mut CallInfo;
}
unsafe extern "C-unwind" fn resume_error(
    mut L: *mut lua_State,
    mut msg: *const std::ffi::c_char,
    mut narg: i32,
) -> i32 {
    (*L).top.p = ((*L).top.p).offset(-(narg as isize));
    let mut io: *mut TValue = &mut (*(*L).top.p).val;
    let mut x_: *mut TString = luaS_new(L, msg);
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
    (*L).top.p = ((*L).top.p).offset(1);
    (*L).top.p;
    return 2 as i32;
}
unsafe extern "C-unwind" fn resume(mut L: *mut lua_State, mut ud: *mut c_void) {
    let mut n: i32 = *(ud as *mut i32);
    let mut firstArg: StkId = ((*L).top.p).offset(-(n as isize));
    let mut ci: *mut CallInfo = (*L).ci;
    if (*L).status as i32 == 0 {
        ccall(
            L,
            firstArg.offset(-(1)),
            -(1 as i32),
            0 as l_uint32,
        );
    } else {
        (*L).status = 0 as lu_byte;
        if (*ci).callstatus as i32 & (1 as i32) << 1 as i32 == 0 {
            (*ci).u.l.savedpc = ((*ci).u.l.savedpc).offset(-1);
            (*ci).u.l.savedpc;
            (*L).top.p = firstArg;
            luaV_execute(L, ci);
        } else {
            if ((*ci).u.c.k).is_some() {
                n = (Some(((*ci).u.c.k).expect("non-null function pointer")))
                    .expect("non-null function pointer")(
                    L, 1 as i32, (*ci).u.c.ctx
                );
            }
            luaD_poscall(L, ci, n);
        }
        unroll(L, 0 as *mut c_void);
    };
}
unsafe extern "C-unwind" fn precover(mut L: *mut lua_State, mut status: i32) -> i32 {
    let mut ci: *mut CallInfo = 0 as *mut CallInfo;
    while status > 1 as i32 && {
        ci = findpcall(L);
        !ci.is_null()
    } {
        (*L).ci = ci;
        (*ci).callstatus =
            ((*ci).callstatus as i32 & !((7 as i32) << 10) | status << 10) as u16;
        status = luaD_rawrunprotected(
            L,
            Some(unroll as unsafe extern "C-unwind" fn(*mut lua_State, *mut c_void) -> ()),
            0 as *mut c_void,
        );
    }
    return status;
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn lua_resume(
    mut L: *mut lua_State,
    mut from: *mut lua_State,
    mut nargs: i32,
    mut nresults: *mut i32,
) -> i32 {
    let mut status: i32 = 0;
    if (*L).status as i32 == 0 {
        if (*L).ci != &mut (*L).base_ci as *mut CallInfo {
            return resume_error(L, c"cannot resume non-suspended coroutine".as_ptr(), nargs);
        } else if ((*L).top.p).offset_from(((*(*L).ci).func.p).offset(1))
            as std::ffi::c_long
            == nargs as std::ffi::c_long
        {
            return resume_error(L, c"cannot resume dead coroutine".as_ptr(), nargs);
        }
    } else if (*L).status as i32 != 1 as i32 {
        return resume_error(L, c"cannot resume dead coroutine".as_ptr(), nargs);
    }
    (*L).nCcalls = if !from.is_null() {
        (*from).nCcalls & 0xffff as i32 as l_uint32
    } else {
        0 as l_uint32
    };
    if (*L).nCcalls & 0xffff as i32 as l_uint32 >= 200 as l_uint32 {
        return resume_error(L, c"C stack overflow".as_ptr(), nargs);
    }
    (*L).nCcalls = ((*L).nCcalls).wrapping_add(1);
    (*L).nCcalls;
    status = luaD_rawrunprotected(
        L,
        Some(resume as unsafe extern "C-unwind" fn(*mut lua_State, *mut c_void) -> ()),
        &mut nargs as *mut i32 as *mut c_void,
    );
    status = precover(L, status);
    if !((!(status > 1 as i32) as i32 != 0) as i32 as std::ffi::c_long != 0) {
        (*L).status = status as lu_byte;
        luaD_seterrorobj(L, status, (*L).top.p);
        (*(*L).ci).top.p = (*L).top.p;
    }
    *nresults = if status == 1 as i32 {
        (*(*L).ci).u2.nyield
    } else {
        ((*L).top.p).offset_from(((*(*L).ci).func.p).offset(1)) as std::ffi::c_long
            as i32
    };
    return status;
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn lua_isyieldable(mut L: *mut lua_State) -> i32 {
    return ((*L).nCcalls & 0xffff0000 as u32 == 0 as u32) as i32;
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn lua_yieldk(
    mut L: *mut lua_State,
    mut nresults: i32,
    mut ctx: lua_KContext,
    mut k: lua_KFunction,
) -> i32 {
    let mut ci: *mut CallInfo = 0 as *mut CallInfo;
    ci = (*L).ci;
    if (!((*L).nCcalls & 0xffff0000 as u32 == 0 as u32) as i32 != 0) as i32 as std::ffi::c_long != 0
    {
        if L != (*(*L).l_G).mainthread {
            luaG_runerror(L, c"attempt to yield across a C-call boundary".as_ptr());
        } else {
            luaG_runerror(L, c"attempt to yield from outside a coroutine".as_ptr());
        }
    }
    (*L).status = 1 as i32 as lu_byte;
    (*ci).u2.nyield = nresults;
    if (*ci).callstatus as i32 & (1 as i32) << 1 as i32 == 0 {
    } else {
        (*ci).u.c.k = k;
        if ((*ci).u.c.k).is_some() {
            (*ci).u.c.ctx = ctx;
        }
        luaD_throw(L, 1 as i32);
    }
    return 0;
}
unsafe extern "C-unwind" fn closepaux(mut L: *mut lua_State, mut ud: *mut c_void) {
    let mut pcl: *mut CloseP = ud as *mut CloseP;
    luaF_close(L, (*pcl).level, (*pcl).status, 0);
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaD_closeprotected(
    mut L: *mut lua_State,
    mut level: ptrdiff_t,
    mut status: i32,
) -> i32 {
    let mut old_ci: *mut CallInfo = (*L).ci;
    let mut old_allowhooks: lu_byte = (*L).allowhook;
    loop {
        let mut pcl: CloseP = CloseP {
            level: 0 as *mut StackValue,
            status: 0,
        };
        pcl.level = ((*L).stack.p as *mut std::ffi::c_char).offset(level as isize) as StkId;
        pcl.status = status;
        status = luaD_rawrunprotected(
            L,
            Some(closepaux as unsafe extern "C-unwind" fn(*mut lua_State, *mut c_void) -> ()),
            &mut pcl as *mut CloseP as *mut c_void,
        );
        if ((status == 0) as i32 != 0) as i32 as std::ffi::c_long != 0 {
            return pcl.status;
        } else {
            (*L).ci = old_ci;
            (*L).allowhook = old_allowhooks;
        }
    }
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaD_pcall(
    mut L: *mut lua_State,
    mut func: Pfunc,
    mut u: *mut c_void,
    mut old_top: ptrdiff_t,
    mut ef: ptrdiff_t,
) -> i32 {
    let mut status: i32 = 0;
    let mut old_ci: *mut CallInfo = (*L).ci;
    let mut old_allowhooks: lu_byte = (*L).allowhook;
    let mut old_errfunc: ptrdiff_t = (*L).errfunc;
    (*L).errfunc = ef;
    status = luaD_rawrunprotected(L, func, u);
    if ((status != 0) as i32 != 0) as i32 as std::ffi::c_long != 0 {
        (*L).ci = old_ci;
        (*L).allowhook = old_allowhooks;
        status = luaD_closeprotected(L, old_top, status);
        luaD_seterrorobj(
            L,
            status,
            ((*L).stack.p as *mut std::ffi::c_char).offset(old_top as isize) as StkId,
        );
        luaD_shrinkstack(L);
    }
    (*L).errfunc = old_errfunc;
    return status;
}
unsafe extern "C-unwind" fn checkmode(
    mut L: *mut lua_State,
    mut mode: *const std::ffi::c_char,
    mut x: *const std::ffi::c_char,
) {
    if !mode.is_null() && (strchr(mode, *x.offset(0 as isize) as i32)).is_null() {
        luaO_pushfstring(
            L,
            c"attempt to load a %s chunk (mode is '%s')".as_ptr(),
            x,
            mode,
        );
        luaD_throw(L, 3 as i32);
    }
}
unsafe extern "C-unwind" fn f_parser(mut L: *mut lua_State, mut ud: *mut c_void) {
    let mut cl: *mut LClosure = 0 as *mut LClosure;
    let mut p: *mut SParser = ud as *mut SParser;
    let fresh130 = (*(*p).z).n;
    (*(*p).z).n = ((*(*p).z).n).wrapping_sub(1);
    let mut c: i32 = if fresh130 > 0 as size_t {
        let fresh131 = (*(*p).z).p;
        (*(*p).z).p = ((*(*p).z).p).offset(1);
        *fresh131 as u8 as i32
    } else {
        luaZ_fill((*p).z)
    };
    if c == (*::core::mem::transmute::<&[u8; 5], &[std::ffi::c_char; 5]>(b"\x1BLua\0"))[0 as usize]
        as i32
    {
        checkmode(L, (*p).mode, c"binary".as_ptr());
        cl = luaU_undump(L, (*p).z, (*p).name);
    } else {
        checkmode(L, (*p).mode, c"text".as_ptr());
        cl = luaY_parser(L, (*p).z, &mut (*p).buff, &mut (*p).dyd, (*p).name, c);
    }
    luaF_initupvals(L, cl);
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaD_protectedparser(
    mut L: *mut lua_State,
    mut z: *mut ZIO,
    mut name: *const std::ffi::c_char,
    mut mode: *const std::ffi::c_char,
) -> i32 {
    let mut p: SParser = SParser {
        z: 0 as *mut ZIO,
        buff: Mbuffer {
            buffer: 0 as *mut std::ffi::c_char,
            n: 0,
            buffsize: 0,
        },
        dyd: Dyndata {
            actvar: C2RustUnnamed_9 {
                arr: 0 as *mut Vardesc,
                n: 0,
                size: 0,
            },
            gt: Labellist {
                arr: 0 as *mut Labeldesc,
                n: 0,
                size: 0,
            },
            label: Labellist {
                arr: 0 as *mut Labeldesc,
                n: 0,
                size: 0,
            },
        },
        mode: 0 as *const std::ffi::c_char,
        name: 0 as *const std::ffi::c_char,
    };
    let mut status: i32 = 0;
    (*L).nCcalls = ((*L).nCcalls).wrapping_add(0x10000 as l_uint32);
    p.z = z;
    p.name = name;
    p.mode = mode;
    p.dyd.actvar.arr = 0 as *mut Vardesc;
    p.dyd.actvar.size = 0;
    p.dyd.gt.arr = 0 as *mut Labeldesc;
    p.dyd.gt.size = 0;
    p.dyd.label.arr = 0 as *mut Labeldesc;
    p.dyd.label.size = 0;
    p.buff.buffer = 0 as *mut std::ffi::c_char;
    p.buff.buffsize = 0 as size_t;
    status = luaD_pcall(
        L,
        Some(f_parser as unsafe extern "C-unwind" fn(*mut lua_State, *mut c_void) -> ()),
        &mut p as *mut SParser as *mut c_void,
        ((*L).top.p as *mut std::ffi::c_char).offset_from((*L).stack.p as *mut std::ffi::c_char),
        (*L).errfunc,
    );
    p.buff.buffer = luaM_saferealloc_(
        L,
        p.buff.buffer as *mut c_void,
        (p.buff.buffsize).wrapping_mul(::core::mem::size_of::<std::ffi::c_char>() as usize),
        (0 as usize).wrapping_mul(::core::mem::size_of::<std::ffi::c_char>() as usize),
    ) as *mut std::ffi::c_char;
    p.buff.buffsize = 0 as size_t;
    luaM_free_(
        L,
        p.dyd.actvar.arr as *mut c_void,
        (p.dyd.actvar.size as usize).wrapping_mul(::core::mem::size_of::<Vardesc>() as usize),
    );
    luaM_free_(
        L,
        p.dyd.gt.arr as *mut c_void,
        (p.dyd.gt.size as usize).wrapping_mul(::core::mem::size_of::<Labeldesc>() as usize),
    );
    luaM_free_(
        L,
        p.dyd.label.arr as *mut c_void,
        (p.dyd.label.size as usize).wrapping_mul(::core::mem::size_of::<Labeldesc>() as usize),
    );
    (*L).nCcalls = ((*L).nCcalls).wrapping_sub(0x10000 as l_uint32);
    return status;
}
