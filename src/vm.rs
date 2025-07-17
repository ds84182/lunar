use crate::*;

#[cfg(feature = "jit")]
pub(crate) mod trace;

/// Fast track for 'gettable': if 't' is a table and 't[k]' is present,
/// return 1 with 'slot' pointing to 't[k]' (position of final result).
/// Otherwise, return 0 (meaning it will have to check metamethod)
/// with 'slot' pointing to an empty 't[k]' (if 't' is a table) or NULL
/// (otherwise). 'f' is the raw get function to use.
#[inline]
unsafe fn luaV_fastget<K>(
    t: *const TValue,
    k: K,
    slot: &mut *const TValue,
    f: impl FnOnce(*mut Table, K) -> *const TValue,
) -> bool {
    if let Some(t) = try_hvalue(t) {
        *slot = f(t.as_ptr(), k);
        !tt_is_nil(*slot)
    } else {
        *slot = ptr::null_mut();
        false
    }
}

/// Special case of 'luaV_fastget' for integers, inlining the fast case
/// of 'luaH_getint'.
#[inline]
unsafe fn luaV_fastgeti(t: *const TValue, k: lua_Integer, slot: &mut *const TValue) -> bool {
    if let Some(t) = try_hvalue(t) {
        let t = t.as_ptr();
        // If the index is within the array part of the table, return it.
        // Else take the slow path through 'luaH_getint'
        *slot = if (k as lua_Unsigned).wrapping_sub(1) < (*t).alimit as u64 {
            (*t).array.offset(k.wrapping_sub(1) as isize)
        } else {
            luaH_getint(t, k)
        };

        !tt_is_nil(*slot)
    } else {
        *slot = ptr::null_mut();
        false
    }
}

#[inline]
unsafe fn getupval(cl: *mut LClosure, idx: usize) -> *mut UpVal {
    // TODO: UB if done by ref (Reading past the end of an array)
    *(&raw mut (*cl).upvals).cast::<*mut UpVal>().add(idx)
}

#[inline]
unsafe fn luaV_finishfastset(L: *mut lua_State, t: *mut TValue, slot: *mut TValue, v: *mut TValue) {
    setobj(slot, v);
    luaC_barrierback(L, (*t).value_.gc, v);
}

unsafe extern "C-unwind" fn l_strton(mut obj: *const TValue, mut result: *mut TValue) -> i32 {
    if !((*obj).tt_ as i32 & 0xf as i32 == 4 as i32) {
        return 0;
    } else {
        let mut st: *mut TString = &mut (*((*obj).value_.gc as *mut GCUnion)).ts;
        return (luaO_str2num(((*st).contents).as_mut_ptr(), result)
            == (if (*st).shrlen as i32 != 0xff as i32 {
                (*st).shrlen as size_t
            } else {
                (*st).u.lnglen
            })
            .wrapping_add(1 as i32 as size_t)) as i32;
    };
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaV_tonumber_(
    mut obj: *const TValue,
    mut n: *mut lua_Number,
) -> i32 {
    let mut v: TValue = TValue {
        value_: Value {
            gc: 0 as *mut GCObject,
        },
        tt_: 0,
    };
    if (*obj).tt_ as i32 == 3 as i32 | (0) << 4 as i32 {
        *n = (*obj).value_.i as lua_Number;
        return 1 as i32;
    } else if l_strton(obj, &mut v) != 0 {
        *n = (if v.tt_ as i32 == 3 as i32 | (0) << 4 as i32 {
            v.value_.i as lua_Number
        } else {
            v.value_.n
        });
        return 1 as i32;
    } else {
        return 0;
    };
}

#[inline]
pub unsafe fn luaV_flttointeger<const MODE: F2Imod>(
    mut n: lua_Number,
    mut p: *mut lua_Integer,
) -> i32 {
    let mut f: lua_Number = n.floor();
    if n != f {
        if MODE == F2Ieq {
            return 0;
        } else if MODE == F2Iceil {
            f += 1 as i32 as lua_Number;
        }
    }
    return (f
        >= (-(9223372036854775807 as std::ffi::c_longlong) - 1 as std::ffi::c_longlong)
            as std::ffi::c_double
        && f < -((-(9223372036854775807 as std::ffi::c_longlong) - 1 as std::ffi::c_longlong)
            as std::ffi::c_double)
        && {
            *p = f as std::ffi::c_longlong;
            1 as i32 != 0
        }) as i32;
}

#[inline]
pub unsafe fn luaV_tointegerns<const MODE: F2Imod>(
    mut obj: *const TValue,
    mut p: *mut lua_Integer,
) -> i32 {
    if (*obj).tt_ as i32 == 3 as i32 | (1 as i32) << 4 as i32 {
        return luaV_flttointeger::<MODE>((*obj).value_.n, p);
    } else if (*obj).tt_ as i32 == 3 as i32 | (0) << 4 as i32 {
        *p = (*obj).value_.i;
        return 1 as i32;
    } else {
        return 0;
    };
}

#[inline]
pub unsafe fn luaV_tointeger<const MODE: F2Imod>(
    mut obj: *const TValue,
    mut p: *mut lua_Integer,
) -> i32 {
    let mut v: TValue = TValue {
        value_: Value {
            gc: 0 as *mut GCObject,
        },
        tt_: 0,
    };
    if l_strton(obj, &mut v) != 0 {
        obj = &mut v;
    }
    return luaV_tointegerns::<MODE>(obj, p);
}

unsafe extern "C-unwind" fn forlimit(
    mut L: *mut lua_State,
    mut init: lua_Integer,
    mut lim: *const TValue,
    mut p: *mut lua_Integer,
    mut step: lua_Integer,
) -> i32 {
    let res = if step < 0 {
        luaV_tointeger::<F2Iceil>(lim, p)
    } else {
        luaV_tointeger::<F2Ifloor>(lim, p)
    };

    if res == 0 {
        let mut flim: lua_Number = 0.;
        if if (*lim).tt_ as i32 == 3 as i32 | (1 as i32) << 4 as i32 {
            flim = (*lim).value_.n;
            1 as i32
        } else {
            luaV_tonumber_(lim, &mut flim)
        } == 0
        {
            luaG_forerror(L, lim, c"limit".as_ptr());
        }
        if (0 as lua_Number) < flim {
            if step < 0 as lua_Integer {
                return 1 as i32;
            }
            *p = 9223372036854775807 as std::ffi::c_longlong;
        } else {
            if step > 0 as lua_Integer {
                return 1 as i32;
            }
            *p = -(9223372036854775807 as std::ffi::c_longlong) - 1 as std::ffi::c_longlong;
        }
    }
    return if step > 0 as lua_Integer {
        (init > *p) as i32
    } else {
        (init < *p) as i32
    };
}
unsafe extern "C-unwind" fn forprep(mut L: *mut lua_State, mut ra: StkId) -> i32 {
    let mut pinit: *mut TValue = &mut (*ra).val;
    let mut plimit: *mut TValue = &mut (*ra.offset(1)).val;
    let mut pstep: *mut TValue = &mut (*ra.offset(2)).val;
    if (*pinit).tt_ as i32 == 3 as i32 | (0) << 4 as i32
        && (*pstep).tt_ as i32 == 3 as i32 | (0) << 4 as i32
    {
        let mut init: lua_Integer = (*pinit).value_.i;
        let mut step: lua_Integer = (*pstep).value_.i;
        let mut limit: lua_Integer = 0;
        if step == 0 as lua_Integer {
            luaG_runerror(L, c"'for' step is zero".as_ptr());
        }
        let mut io: *mut TValue = &mut (*ra.offset(3)).val;
        (*io).value_.i = init;
        (*io).tt_ = (3 as i32 | (0) << 4 as i32) as lu_byte;
        if forlimit(L, init, plimit, &mut limit, step) != 0 {
            return 1 as i32;
        } else {
            let mut count: lua_Unsigned = 0;
            if step > 0 as lua_Integer {
                count = (limit as lua_Unsigned).wrapping_sub(init as lua_Unsigned);
                if step != 1 as i32 as lua_Integer {
                    count = count / step as lua_Unsigned;
                }
            } else {
                count = (init as lua_Unsigned).wrapping_sub(limit as lua_Unsigned);
                count = count
                    / (-(step + 1 as i32 as lua_Integer) as lua_Unsigned)
                        .wrapping_add(1 as u32 as lua_Unsigned);
            }
            let mut io_0: *mut TValue = plimit;
            (*io_0).value_.i = count as lua_Integer;
            (*io_0).tt_ = (3 as i32 | (0) << 4 as i32) as lu_byte;
        }
    } else {
        let mut init_0: lua_Number = 0.;
        let mut limit_0: lua_Number = 0.;
        let mut step_0: lua_Number = 0.;
        if (((if (*plimit).tt_ as i32 == 3 as i32 | (1 as i32) << 4 as i32 {
            limit_0 = (*plimit).value_.n;
            1 as i32
        } else {
            luaV_tonumber_(plimit, &mut limit_0)
        }) == 0) as i32
            != 0) as i32 as std::ffi::c_long
            != 0
        {
            luaG_forerror(L, plimit, c"limit".as_ptr());
        }
        if (((if (*pstep).tt_ as i32 == 3 as i32 | (1 as i32) << 4 as i32 {
            step_0 = (*pstep).value_.n;
            1 as i32
        } else {
            luaV_tonumber_(pstep, &mut step_0)
        }) == 0) as i32
            != 0) as i32 as std::ffi::c_long
            != 0
        {
            luaG_forerror(L, pstep, c"step".as_ptr());
        }
        if (((if (*pinit).tt_ as i32 == 3 as i32 | (1 as i32) << 4 as i32 {
            init_0 = (*pinit).value_.n;
            1 as i32
        } else {
            luaV_tonumber_(pinit, &mut init_0)
        }) == 0) as i32
            != 0) as i32 as std::ffi::c_long
            != 0
        {
            luaG_forerror(L, pinit, c"initial value".as_ptr());
        }
        if step_0 == 0 as lua_Number {
            luaG_runerror(L, c"'for' step is zero".as_ptr());
        }
        if if (0 as lua_Number) < step_0 {
            (limit_0 < init_0) as i32
        } else {
            (init_0 < limit_0) as i32
        } != 0
        {
            return 1 as i32;
        } else {
            let mut io_1: *mut TValue = plimit;
            (*io_1).value_.n = limit_0;
            (*io_1).tt_ = (3 as i32 | (1 as i32) << 4 as i32) as lu_byte;
            let mut io_2: *mut TValue = pstep;
            (*io_2).value_.n = step_0;
            (*io_2).tt_ = (3 as i32 | (1 as i32) << 4 as i32) as lu_byte;
            let mut io_3: *mut TValue = &mut (*ra).val;
            (*io_3).value_.n = init_0;
            (*io_3).tt_ = (3 as i32 | (1 as i32) << 4 as i32) as lu_byte;
            let mut io_4: *mut TValue = &mut (*ra.offset(3)).val;
            (*io_4).value_.n = init_0;
            (*io_4).tt_ = (3 as i32 | (1 as i32) << 4 as i32) as lu_byte;
        }
    }
    return 0;
}
unsafe extern "C-unwind" fn floatforloop(mut ra: StkId) -> i32 {
    let mut step: lua_Number = (*ra.offset(2)).val.value_.n;
    let mut limit: lua_Number = (*ra.offset(1)).val.value_.n;
    let mut idx: lua_Number = (*ra).val.value_.n;
    idx = idx + step;
    if if (0 as lua_Number) < step {
        (idx <= limit) as i32
    } else {
        (limit <= idx) as i32
    } != 0
    {
        let mut io: *mut TValue = &mut (*ra).val;
        (*io).value_.n = idx;
        let mut io_0: *mut TValue = &mut (*ra.offset(3)).val;
        (*io_0).value_.n = idx;
        (*io_0).tt_ = (3 as i32 | (1 as i32) << 4 as i32) as lu_byte;
        return 1 as i32;
    } else {
        return 0;
    };
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaV_finishget(
    mut L: *mut lua_State,
    mut t: *const TValue,
    mut key: *mut TValue,
    mut val: StkId,
    mut slot: *const TValue,
) {
    let mut loop_0: i32 = 0;
    let mut tm: *const TValue = 0 as *const TValue;
    loop_0 = 0;
    while loop_0 < 2000 {
        if slot.is_null() {
            tm = luaT_gettmbyobj(L, t, TM_INDEX);
            if (((*tm).tt_ as i32 & 0xf as i32 == 0) as i32 != 0) as i32 as std::ffi::c_long != 0 {
                luaG_typeerror(L, t, c"index".as_ptr());
            }
        } else {
            tm = if ((*(&mut (*((*t).value_.gc as *mut GCUnion)).h as *mut Table)).metatable)
                .is_null()
            {
                0 as *const TValue
            } else if (*(*(&mut (*((*t).value_.gc as *mut GCUnion)).h as *mut Table)).metatable)
                .flags as u32
                & (1 as u32) << TM_INDEX as i32
                != 0
            {
                0 as *const TValue
            } else {
                luaT_gettm(
                    (*&mut (*((*t).value_.gc as *mut GCUnion)).h).metatable,
                    TM_INDEX,
                    (*(*L).l_G).tmname[TM_INDEX as i32 as usize],
                )
            };
            if tm.is_null() {
                (*val).val.tt_ = (0 | (0) << 4 as i32) as lu_byte;
                return;
            }
        }
        if (*tm).tt_ as i32 & 0xf as i32 == 6 as i32 {
            luaT_callTMres(L, tm, t, key, val);
            return;
        }
        t = tm;
        if if !((*t).tt_ as i32 == 5 as i32 | (0) << 4 as i32 | (1 as i32) << 6 as i32) {
            slot = 0 as *const TValue;
            0
        } else {
            slot = luaH_get(&mut (*((*t).value_.gc as *mut GCUnion)).h, key);
            !((*slot).tt_ as i32 & 0xf as i32 == 0) as i32
        } != 0
        {
            let mut io1: *mut TValue = &mut (*val).val;
            let mut io2: *const TValue = slot;
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
            return;
        }
        loop_0 += 1;
        loop_0;
    }
    luaG_runerror(L, c"'__index' chain too long; possible loop".as_ptr());
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaV_finishset(
    mut L: *mut lua_State,
    mut t: *const TValue,
    mut key: *mut TValue,
    mut val: *mut TValue,
    mut slot: *const TValue,
) {
    let mut loop_0: i32 = 0;
    loop_0 = 0;
    while loop_0 < 2000 {
        let mut tm: *const TValue = 0 as *const TValue;
        if !slot.is_null() {
            let mut h: *mut Table = &mut (*((*t).value_.gc as *mut GCUnion)).h;
            tm = if ((*h).metatable).is_null() {
                0 as *const TValue
            } else if (*(*h).metatable).flags as u32 & (1 as u32) << TM_NEWINDEX as i32 != 0 {
                0 as *const TValue
            } else {
                luaT_gettm(
                    (*h).metatable,
                    TM_NEWINDEX,
                    (*(*L).l_G).tmname[TM_NEWINDEX as i32 as usize],
                )
            };
            if tm.is_null() {
                luaH_finishset(L, h, key, slot, val);
                (*h).flags =
                    ((*h).flags as u32 & !!(!(0 as u32) << TM_EQ as i32 + 1 as i32)) as lu_byte;
                if (*val).tt_ as i32 & (1 as i32) << 6 as i32 != 0 {
                    if (*(&mut (*(h as *mut GCUnion)).gc as *mut GCObject)).marked as i32
                        & (1 as i32) << 5 as i32
                        != 0
                        && (*(*val).value_.gc).marked as i32
                            & ((1 as i32) << 3 as i32 | (1 as i32) << 4 as i32)
                            != 0
                    {
                        luaC_barrierback_(L, &mut (*(h as *mut GCUnion)).gc);
                    } else {
                    };
                } else {
                };
                return;
            }
        } else {
            tm = luaT_gettmbyobj(L, t, TM_NEWINDEX);
            if (((*tm).tt_ as i32 & 0xf as i32 == 0) as i32 != 0) as i32 as std::ffi::c_long != 0 {
                luaG_typeerror(L, t, c"index".as_ptr());
            }
        }
        if (*tm).tt_ as i32 & 0xf as i32 == 6 as i32 {
            luaT_callTM(L, tm, t, key, val);
            return;
        }
        t = tm;
        if if !((*t).tt_ as i32 == 5 as i32 | (0) << 4 as i32 | (1 as i32) << 6 as i32) {
            slot = 0 as *const TValue;
            0
        } else {
            slot = luaH_get(&mut (*((*t).value_.gc as *mut GCUnion)).h, key);
            !((*slot).tt_ as i32 & 0xf as i32 == 0) as i32
        } != 0
        {
            let mut io1: *mut TValue = slot as *mut TValue;
            let mut io2: *const TValue = val;
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
            if (*val).tt_ as i32 & (1 as i32) << 6 as i32 != 0 {
                if (*(*t).value_.gc).marked as i32 & (1 as i32) << 5 as i32 != 0
                    && (*(*val).value_.gc).marked as i32
                        & ((1 as i32) << 3 as i32 | (1 as i32) << 4 as i32)
                        != 0
                {
                    luaC_barrierback_(L, (*t).value_.gc);
                } else {
                };
            } else {
            };
            return;
        }
        loop_0 += 1;
        loop_0;
    }
    luaG_runerror(L, c"'__newindex' chain too long; possible loop".as_ptr());
}
unsafe extern "C-unwind" fn l_strcmp(mut ts1: *const TString, mut ts2: *const TString) -> i32 {
    let mut s1: *const std::ffi::c_char = ((*ts1).contents).as_ptr();
    let mut rl1: size_t = if (*ts1).shrlen as i32 != 0xff as i32 {
        (*ts1).shrlen as size_t
    } else {
        (*ts1).u.lnglen
    };
    let mut s2: *const std::ffi::c_char = ((*ts2).contents).as_ptr();
    let mut rl2: size_t = if (*ts2).shrlen as i32 != 0xff as i32 {
        (*ts2).shrlen as size_t
    } else {
        (*ts2).u.lnglen
    };
    loop {
        let mut temp: i32 = strcoll(s1, s2);
        if temp != 0 {
            return temp;
        } else {
            let mut zl1: size_t = strlen(s1);
            let mut zl2: size_t = strlen(s2);
            if zl2 == rl2 {
                return if zl1 == rl1 { 0 } else { 1 as i32 };
            } else if zl1 == rl1 {
                return -(1 as i32);
            }
            zl1 = zl1.wrapping_add(1);
            zl1;
            zl2 = zl2.wrapping_add(1);
            zl2;
            s1 = s1.offset(zl1 as isize);
            rl1 = rl1.wrapping_sub(zl1);
            s2 = s2.offset(zl2 as isize);
            rl2 = rl2.wrapping_sub(zl2);
        }
    }
}
#[inline]
unsafe extern "C-unwind" fn LTintfloat(mut i: lua_Integer, mut f: lua_Number) -> i32 {
    if ((1 as i32 as lua_Unsigned) << 53 as i32).wrapping_add(i as lua_Unsigned)
        <= 2 as i32 as lua_Unsigned * ((1 as i32 as lua_Unsigned) << 53 as i32)
    {
        return ((i as lua_Number) < f) as i32;
    } else {
        let mut fi: lua_Integer = 0;
        if luaV_flttointeger::<F2Iceil>(f, &mut fi) != 0 {
            return (i < fi) as i32;
        } else {
            return (f > 0 as lua_Number) as i32;
        }
    };
}
#[inline]
unsafe extern "C-unwind" fn LEintfloat(mut i: lua_Integer, mut f: lua_Number) -> i32 {
    if ((1 as i32 as lua_Unsigned) << 53 as i32).wrapping_add(i as lua_Unsigned)
        <= 2 as i32 as lua_Unsigned * ((1 as i32 as lua_Unsigned) << 53 as i32)
    {
        return (i as lua_Number <= f) as i32;
    } else {
        let mut fi: lua_Integer = 0;
        if luaV_flttointeger::<F2Ifloor>(f, &mut fi) != 0 {
            return (i <= fi) as i32;
        } else {
            return (f > 0 as lua_Number) as i32;
        }
    };
}
#[inline]
unsafe extern "C-unwind" fn LTfloatint(mut f: lua_Number, mut i: lua_Integer) -> i32 {
    if ((1 as i32 as lua_Unsigned) << 53 as i32).wrapping_add(i as lua_Unsigned)
        <= 2 as i32 as lua_Unsigned * ((1 as i32 as lua_Unsigned) << 53 as i32)
    {
        return (f < i as lua_Number) as i32;
    } else {
        let mut fi: lua_Integer = 0;
        if luaV_flttointeger::<F2Ifloor>(f, &mut fi) != 0 {
            return (fi < i) as i32;
        } else {
            return (f < 0 as lua_Number) as i32;
        }
    };
}
#[inline]
unsafe extern "C-unwind" fn LEfloatint(mut f: lua_Number, mut i: lua_Integer) -> i32 {
    if ((1 as i32 as lua_Unsigned) << 53 as i32).wrapping_add(i as lua_Unsigned)
        <= 2 as i32 as lua_Unsigned * ((1 as i32 as lua_Unsigned) << 53 as i32)
    {
        return (f <= i as lua_Number) as i32;
    } else {
        let mut fi: lua_Integer = 0;
        if luaV_flttointeger::<F2Iceil>(f, &mut fi) != 0 {
            return (fi <= i) as i32;
        } else {
            return (f < 0 as lua_Number) as i32;
        }
    };
}
#[inline]
unsafe extern "C-unwind" fn LTnum(mut l: *const TValue, mut r: *const TValue) -> i32 {
    if (*l).tt_ as i32 == 3 as i32 | (0) << 4 as i32 {
        let mut li: lua_Integer = (*l).value_.i;
        if (*r).tt_ as i32 == 3 as i32 | (0) << 4 as i32 {
            return (li < (*r).value_.i) as i32;
        } else {
            return LTintfloat(li, (*r).value_.n);
        }
    } else {
        let mut lf: lua_Number = (*l).value_.n;
        if (*r).tt_ as i32 == 3 as i32 | (1 as i32) << 4 as i32 {
            return (lf < (*r).value_.n) as i32;
        } else {
            return LTfloatint(lf, (*r).value_.i);
        }
    };
}
#[inline]
unsafe extern "C-unwind" fn LEnum(mut l: *const TValue, mut r: *const TValue) -> i32 {
    if (*l).tt_ as i32 == 3 as i32 | (0) << 4 as i32 {
        let mut li: lua_Integer = (*l).value_.i;
        if (*r).tt_ as i32 == 3 as i32 | (0) << 4 as i32 {
            return (li <= (*r).value_.i) as i32;
        } else {
            return LEintfloat(li, (*r).value_.n);
        }
    } else {
        let mut lf: lua_Number = (*l).value_.n;
        if (*r).tt_ as i32 == 3 as i32 | (1 as i32) << 4 as i32 {
            return (lf <= (*r).value_.n) as i32;
        } else {
            return LEfloatint(lf, (*r).value_.i);
        }
    };
}
unsafe extern "C-unwind" fn lessthanothers(
    mut L: *mut lua_State,
    mut l: *const TValue,
    mut r: *const TValue,
) -> i32 {
    if (*l).tt_ as i32 & 0xf as i32 == 4 as i32 && (*r).tt_ as i32 & 0xf as i32 == 4 as i32 {
        return (l_strcmp(
            &mut (*((*l).value_.gc as *mut GCUnion)).ts,
            &mut (*((*r).value_.gc as *mut GCUnion)).ts,
        ) < 0) as i32;
    } else {
        return luaT_callorderTM(L, l, r, TM_LT);
    };
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaV_lessthan(
    mut L: *mut lua_State,
    mut l: *const TValue,
    mut r: *const TValue,
) -> i32 {
    if (*l).tt_ as i32 & 0xf as i32 == 3 as i32 && (*r).tt_ as i32 & 0xf as i32 == 3 as i32 {
        return LTnum(l, r);
    } else {
        return lessthanothers(L, l, r);
    };
}
unsafe extern "C-unwind" fn lessequalothers(
    mut L: *mut lua_State,
    mut l: *const TValue,
    mut r: *const TValue,
) -> i32 {
    if (*l).tt_ as i32 & 0xf as i32 == 4 as i32 && (*r).tt_ as i32 & 0xf as i32 == 4 as i32 {
        return (l_strcmp(
            &mut (*((*l).value_.gc as *mut GCUnion)).ts,
            &mut (*((*r).value_.gc as *mut GCUnion)).ts,
        ) <= 0) as i32;
    } else {
        return luaT_callorderTM(L, l, r, TM_LE);
    };
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaV_lessequal(
    mut L: *mut lua_State,
    mut l: *const TValue,
    mut r: *const TValue,
) -> i32 {
    if (*l).tt_ as i32 & 0xf as i32 == 3 as i32 && (*r).tt_ as i32 & 0xf as i32 == 3 as i32 {
        return LEnum(l, r);
    } else {
        return lessequalothers(L, l, r);
    };
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaV_equalobj(
    mut L: *mut lua_State,
    mut t1: *const TValue,
    mut t2: *const TValue,
) -> i32 {
    let mut tm: *const TValue = 0 as *const TValue;
    if (*t1).tt_ as i32 & 0x3f as i32 != (*t2).tt_ as i32 & 0x3f as i32 {
        if (*t1).tt_ as i32 & 0xf as i32 != (*t2).tt_ as i32 & 0xf as i32
            || (*t1).tt_ as i32 & 0xf as i32 != 3 as i32
        {
            return 0;
        } else {
            let mut i1: lua_Integer = 0;
            let mut i2: lua_Integer = 0;
            return (luaV_tointegerns::<F2Ieq>(t1, &mut i1) != 0
                && luaV_tointegerns::<F2Ieq>(t2, &mut i2) != 0
                && i1 == i2) as i32;
        }
    }
    match (*t1).tt_ as i32 & 0x3f as i32 {
        0 | 1 | 17 => return 1 as i32,
        3 => return ((*t1).value_.i == (*t2).value_.i) as i32,
        19 => return ((*t1).value_.n == (*t2).value_.n) as i32,
        2 => return ((*t1).value_.p == (*t2).value_.p) as i32,
        22 => return ((*t1).value_.f == (*t2).value_.f) as i32,
        4 => {
            return (&mut (*((*t1).value_.gc as *mut GCUnion)).ts as *mut TString
                == &mut (*((*t2).value_.gc as *mut GCUnion)).ts as *mut TString)
                as i32;
        }
        20 => {
            return luaS_eqlngstr(
                &mut (*((*t1).value_.gc as *mut GCUnion)).ts,
                &mut (*((*t2).value_.gc as *mut GCUnion)).ts,
            );
        }
        7 => {
            if &mut (*((*t1).value_.gc as *mut GCUnion)).u as *mut Udata
                == &mut (*((*t2).value_.gc as *mut GCUnion)).u as *mut Udata
            {
                return 1 as i32;
            } else if L.is_null() {
                return 0;
            }
            tm = if ((*(&mut (*((*t1).value_.gc as *mut GCUnion)).u as *mut Udata)).metatable)
                .is_null()
            {
                0 as *const TValue
            } else if (*(*(&mut (*((*t1).value_.gc as *mut GCUnion)).u as *mut Udata)).metatable)
                .flags as u32
                & (1 as u32) << TM_EQ as i32
                != 0
            {
                0 as *const TValue
            } else {
                luaT_gettm(
                    (*&mut (*((*t1).value_.gc as *mut GCUnion)).u).metatable,
                    TM_EQ,
                    (*(*L).l_G).tmname[TM_EQ as i32 as usize],
                )
            };
            if tm.is_null() {
                tm = if ((*(&mut (*((*t2).value_.gc as *mut GCUnion)).u as *mut Udata)).metatable)
                    .is_null()
                {
                    0 as *const TValue
                } else if (*(*(&mut (*((*t2).value_.gc as *mut GCUnion)).u as *mut Udata))
                    .metatable)
                    .flags as u32
                    & (1 as u32) << TM_EQ as i32
                    != 0
                {
                    0 as *const TValue
                } else {
                    luaT_gettm(
                        (*&mut (*((*t2).value_.gc as *mut GCUnion)).u).metatable,
                        TM_EQ,
                        (*(*L).l_G).tmname[TM_EQ as i32 as usize],
                    )
                };
            }
        }
        5 => {
            if &mut (*((*t1).value_.gc as *mut GCUnion)).h as *mut Table
                == &mut (*((*t2).value_.gc as *mut GCUnion)).h as *mut Table
            {
                return 1 as i32;
            } else if L.is_null() {
                return 0;
            }
            tm = if ((*(&mut (*((*t1).value_.gc as *mut GCUnion)).h as *mut Table)).metatable)
                .is_null()
            {
                0 as *const TValue
            } else if (*(*(&mut (*((*t1).value_.gc as *mut GCUnion)).h as *mut Table)).metatable)
                .flags as u32
                & (1 as u32) << TM_EQ as i32
                != 0
            {
                0 as *const TValue
            } else {
                luaT_gettm(
                    (*&mut (*((*t1).value_.gc as *mut GCUnion)).h).metatable,
                    TM_EQ,
                    (*(*L).l_G).tmname[TM_EQ as i32 as usize],
                )
            };
            if tm.is_null() {
                tm = if ((*(&mut (*((*t2).value_.gc as *mut GCUnion)).h as *mut Table)).metatable)
                    .is_null()
                {
                    0 as *const TValue
                } else if (*(*(&mut (*((*t2).value_.gc as *mut GCUnion)).h as *mut Table))
                    .metatable)
                    .flags as u32
                    & (1 as u32) << TM_EQ as i32
                    != 0
                {
                    0 as *const TValue
                } else {
                    luaT_gettm(
                        (*&mut (*((*t2).value_.gc as *mut GCUnion)).h).metatable,
                        TM_EQ,
                        (*(*L).l_G).tmname[TM_EQ as i32 as usize],
                    )
                };
            }
        }
        _ => return ((*t1).value_.gc == (*t2).value_.gc) as i32,
    }
    if tm.is_null() {
        return 0;
    } else {
        luaT_callTMres(L, tm, t1, t2, (*L).top.p);
        return !((*(*L).top.p).val.tt_ as i32 == 1 as i32 | (0) << 4 as i32
            || (*(*L).top.p).val.tt_ as i32 & 0xf as i32 == 0) as i32;
    };
}
unsafe extern "C-unwind" fn copy2buff(mut top: StkId, mut n: i32, mut buff: *mut std::ffi::c_char) {
    let mut tl: size_t = 0 as size_t;
    loop {
        let mut st: *mut TString =
            &mut (*((*top.offset(-(n as isize))).val.value_.gc as *mut GCUnion)).ts;
        let mut l: size_t = if (*st).shrlen as i32 != 0xff as i32 {
            (*st).shrlen as size_t
        } else {
            (*st).u.lnglen
        };
        memcpy(
            buff.offset(tl as isize) as *mut c_void,
            ((*st).contents).as_mut_ptr() as *const c_void,
            l.wrapping_mul(size_of::<std::ffi::c_char>() as usize),
        );
        tl = tl.wrapping_add(l);
        n -= 1;
        if !(n > 0) {
            break;
        }
    }
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaV_concat(mut L: *mut lua_State, mut total: i32) {
    if total == 1 as i32 {
        return;
    }
    loop {
        let mut top: StkId = (*L).top.p;
        let mut n: i32 = 2 as i32;
        if !((*top.offset(-(2))).val.tt_ as i32 & 0xf as i32 == 4 as i32
            || (*top.offset(-(2))).val.tt_ as i32 & 0xf as i32 == 3 as i32)
            || !((*top.offset(-(1))).val.tt_ as i32 & 0xf as i32 == 4 as i32
                || (*top.offset(-(1))).val.tt_ as i32 & 0xf as i32 == 3 as i32 && {
                    luaO_tostring(L, &mut (*top.offset(-(1))).val);
                    1 as i32 != 0
                })
        {
            luaT_tryconcatTM(L);
        } else if (*top.offset(-(1))).val.tt_ as i32
            == 4 as i32 | (0) << 4 as i32 | (1 as i32) << 6 as i32
            && (*(&mut (*((*top.offset(-(1))).val.value_.gc as *mut GCUnion)).ts as *mut TString))
                .shrlen as i32
                == 0
        {
            ((*top.offset(-(2))).val.tt_ as i32 & 0xf as i32 == 4 as i32
                || (*top.offset(-(2))).val.tt_ as i32 & 0xf as i32 == 3 as i32 && {
                    luaO_tostring(L, &mut (*top.offset(-(2))).val);
                    1 as i32 != 0
                }) as i32;
        } else if (*top.offset(-(2))).val.tt_ as i32
            == 4 as i32 | (0) << 4 as i32 | (1 as i32) << 6 as i32
            && (*(&mut (*((*top.offset(-(2))).val.value_.gc as *mut GCUnion)).ts as *mut TString))
                .shrlen as i32
                == 0
        {
            let mut io1: *mut TValue = &mut (*top.offset(-(2))).val;
            let mut io2: *const TValue = &mut (*top.offset(-(1))).val;
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
        } else {
            let mut tl: size_t = if (*(&mut (*((*top.offset(-(1))).val.value_.gc as *mut GCUnion))
                .ts as *mut TString))
                .shrlen as i32
                != 0xff as i32
            {
                (*&mut (*((*top.offset(-(1))).val.value_.gc as *mut GCUnion)).ts).shrlen as size_t
            } else {
                (*&mut (*((*top.offset(-(1))).val.value_.gc as *mut GCUnion)).ts)
                    .u
                    .lnglen
            };
            let mut ts: *mut TString = 0 as *mut TString;
            n = 1 as i32;
            while n < total
                && ((*top.offset(-(n as isize)).offset(-(1))).val.tt_ as i32 & 0xf as i32
                    == 4 as i32
                    || (*top.offset(-(n as isize)).offset(-(1))).val.tt_ as i32 & 0xf as i32
                        == 3 as i32
                        && {
                            luaO_tostring(L, &mut (*top.offset(-(n as isize)).offset(-(1))).val);
                            1 as i32 != 0
                        })
            {
                let mut l: size_t =
                    if (*(&mut (*((*top.offset(-(n as isize)).offset(-(1))).val.value_.gc
                        as *mut GCUnion))
                        .ts as *mut TString))
                        .shrlen as i32
                        != 0xff as i32
                    {
                        (*&mut (*((*top.offset(-(n as isize)).offset(-(1))).val.value_.gc
                            as *mut GCUnion))
                            .ts)
                            .shrlen as size_t
                    } else {
                        (*&mut (*((*top.offset(-(n as isize)).offset(-(1))).val.value_.gc
                            as *mut GCUnion))
                            .ts)
                            .u
                            .lnglen
                    };
                if ((l
                    >= (if (size_of::<size_t>() as usize) < size_of::<lua_Integer>() as usize {
                        !(0 as size_t)
                    } else {
                        9223372036854775807 as std::ffi::c_longlong as size_t
                    })
                    .wrapping_sub(size_of::<TString>() as usize)
                    .wrapping_sub(tl)) as i32
                    != 0) as i32 as std::ffi::c_long
                    != 0
                {
                    (*L).top.p = top.offset(-(total as isize));
                    luaG_runerror(L, c"string length overflow".as_ptr());
                }
                tl = tl.wrapping_add(l);
                n += 1;
                n;
            }
            if tl <= 40 as size_t {
                let mut buff: [std::ffi::c_char; 40] = [0; 40];
                copy2buff(top, n, buff.as_mut_ptr());
                ts = luaS_newlstr(L, buff.as_mut_ptr(), tl);
            } else {
                ts = luaS_createlngstrobj(L, tl);
                copy2buff(top, n, ((*ts).contents).as_mut_ptr());
            }
            let mut io: *mut TValue = &mut (*top.offset(-(n as isize))).val;
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
        }
        total -= n - 1 as i32;
        (*L).top.p = ((*L).top.p).offset(-((n - 1 as i32) as isize));
        if !(total > 1 as i32) {
            break;
        }
    }
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaV_objlen(
    mut L: *mut lua_State,
    mut ra: StkId,
    mut rb: *const TValue,
) {
    let mut tm: *const TValue = 0 as *const TValue;
    match (*rb).tt_ as i32 & 0x3f as i32 {
        5 => {
            let mut h: *mut Table = &mut (*((*rb).value_.gc as *mut GCUnion)).h;
            tm = if ((*h).metatable).is_null() {
                0 as *const TValue
            } else if (*(*h).metatable).flags as u32 & (1 as u32) << TM_LEN as i32 != 0 {
                0 as *const TValue
            } else {
                luaT_gettm(
                    (*h).metatable,
                    TM_LEN,
                    (*(*L).l_G).tmname[TM_LEN as i32 as usize],
                )
            };
            if tm.is_null() {
                let mut io: *mut TValue = &mut (*ra).val;
                (*io).value_.i = luaH_getn(h) as lua_Integer;
                (*io).tt_ = (3 as i32 | (0) << 4 as i32) as lu_byte;
                return;
            }
        }
        4 => {
            let mut io_0: *mut TValue = &mut (*ra).val;
            (*io_0).value_.i =
                (*&mut (*((*rb).value_.gc as *mut GCUnion)).ts).shrlen as lua_Integer;
            (*io_0).tt_ = (3 as i32 | (0) << 4 as i32) as lu_byte;
            return;
        }
        20 => {
            let mut io_1: *mut TValue = &mut (*ra).val;
            (*io_1).value_.i =
                (*&mut (*((*rb).value_.gc as *mut GCUnion)).ts).u.lnglen as lua_Integer;
            (*io_1).tt_ = (3 as i32 | (0) << 4 as i32) as lu_byte;
            return;
        }
        _ => {
            tm = luaT_gettmbyobj(L, rb, TM_LEN);
            if (((*tm).tt_ as i32 & 0xf as i32 == 0) as i32 != 0) as i32 as std::ffi::c_long != 0 {
                luaG_typeerror(L, rb, c"get length of".as_ptr());
            }
        }
    }
    luaT_callTMres(L, tm, rb, rb, ra);
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaV_idiv(
    mut L: *mut lua_State,
    mut m: lua_Integer,
    mut n: lua_Integer,
) -> lua_Integer {
    if (((n as lua_Unsigned).wrapping_add(1 as u32 as lua_Unsigned) <= 1 as u32 as lua_Unsigned)
        as i32
        != 0) as i32 as std::ffi::c_long
        != 0
    {
        if n == 0 as lua_Integer {
            luaG_runerror(L, c"attempt to divide by zero".as_ptr());
        }
        return (0 as lua_Unsigned).wrapping_sub(m as lua_Unsigned) as lua_Integer;
    } else {
        let mut q: lua_Integer = m / n;
        if m ^ n < 0 as lua_Integer && m % n != 0 as lua_Integer {
            q -= 1 as i32 as lua_Integer;
        }
        return q;
    };
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaV_mod(
    mut L: *mut lua_State,
    mut m: lua_Integer,
    mut n: lua_Integer,
) -> lua_Integer {
    if (((n as lua_Unsigned).wrapping_add(1 as u32 as lua_Unsigned) <= 1 as u32 as lua_Unsigned)
        as i32
        != 0) as i32 as std::ffi::c_long
        != 0
    {
        if n == 0 as lua_Integer {
            luaG_runerror(L, c"attempt to perform 'n%%0'".as_ptr());
        }
        return 0 as lua_Integer;
    } else {
        let mut r: lua_Integer = m % n;
        if r != 0 as lua_Integer && r ^ n < 0 as lua_Integer {
            r += n;
        }
        return r;
    };
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaV_modf(
    mut L: *mut lua_State,
    mut m: lua_Number,
    mut n: lua_Number,
) -> lua_Number {
    let mut r: lua_Number = 0.;
    r = fmod(m, n);
    if if r > 0 as lua_Number {
        (n < 0 as lua_Number) as i32
    } else {
        (r < 0 as lua_Number && n > 0 as lua_Number) as i32
    } != 0
    {
        r += n;
    }
    return r;
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaV_shiftl(mut x: lua_Integer, mut y: lua_Integer) -> lua_Integer {
    if y < 0 as lua_Integer {
        if y <= -((size_of::<lua_Integer>() as usize).wrapping_mul(8) as i32) as lua_Integer {
            return 0 as lua_Integer;
        } else {
            return (x as lua_Unsigned >> -y as lua_Unsigned) as lua_Integer;
        }
    } else if y >= (size_of::<lua_Integer>() as usize).wrapping_mul(8) as i32 as lua_Integer {
        return 0 as lua_Integer;
    } else {
        return ((x as lua_Unsigned) << y as lua_Unsigned) as lua_Integer;
    };
}
unsafe extern "C-unwind" fn pushclosure(
    mut L: *mut lua_State,
    mut p: *mut Proto,
    mut encup: *mut *mut UpVal,
    mut base: StkId,
    mut ra: StkId,
) {
    let mut nup: i32 = (*p).sizeupvalues;
    let mut uv: *mut Upvaldesc = (*p).upvalues;
    let mut i: i32 = 0;
    let mut ncl: *mut LClosure = luaF_newLclosure(L, nup);
    (*ncl).p = p;
    let mut io: *mut TValue = &mut (*ra).val;
    let mut x_: *mut LClosure = ncl;
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
    i = 0;
    while i < nup {
        if (*uv.offset(i as isize)).instack != 0 {
            let ref mut fresh132 = *((*ncl).upvals).as_mut_ptr().offset(i as isize);
            *fresh132 =
                luaF_findupval(L, base.offset((*uv.offset(i as isize)).idx as i32 as isize));
        } else {
            let ref mut fresh133 = *((*ncl).upvals).as_mut_ptr().offset(i as isize);
            *fresh133 = *encup.offset((*uv.offset(i as isize)).idx as isize);
        }
        if (*ncl).marked as i32 & (1 as i32) << 5 as i32 != 0
            && (**((*ncl).upvals).as_mut_ptr().offset(i as isize)).marked as i32
                & ((1 as i32) << 3 as i32 | (1 as i32) << 4 as i32)
                != 0
        {
            luaC_barrier_(
                L,
                &mut (*(ncl as *mut GCUnion)).gc,
                &mut (*(*((*ncl).upvals).as_mut_ptr().offset(i as isize) as *mut GCUnion)).gc,
            );
        } else {
        };
        i += 1;
        i;
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaV_finishOp(mut L: *mut lua_State) {
    let mut ci: *mut CallInfo = (*L).ci;
    let mut base: StkId = ((*ci).func.p).offset(1);
    let mut inst: Instruction = *((*ci).u.l.savedpc).offset(-(1));
    let mut op: OpCode = (inst >> 0 & !(!(0 as Instruction) << 7 as i32) << 0) as OpCode;
    match op as u32 {
        OP_MMBIN | OP_MMBINI | OP_MMBINK => {
            let mut io1: *mut TValue = &mut (*base.offset(
                (*((*ci).u.l.savedpc).offset(-(2)) >> 0 + 7 as i32
                    & !(!(0 as Instruction) << 8 as i32) << 0) as i32 as isize,
            ))
            .val;
            (*L).top.p = ((*L).top.p).offset(-1);
            let mut io2: *const TValue = &mut (*(*L).top.p).val;
            (*io1).value_ = (*io2).value_;
            (*io1).tt_ = (*io2).tt_;
        }
        OP_UNM | OP_BNOT | OP_LEN | OP_GETTABUP | OP_GETTABLE | OP_GETI | OP_GETFIELD | OP_SELF => {
            let mut io1_0: *mut TValue = &mut (*base.offset(
                (inst >> 0 + 7 as i32 & !(!(0 as Instruction) << 8 as i32) << 0) as i32 as isize,
            ))
            .val;
            (*L).top.p = ((*L).top.p).offset(-1);
            let mut io2_0: *const TValue = &mut (*(*L).top.p).val;
            (*io1_0).value_ = (*io2_0).value_;
            (*io1_0).tt_ = (*io2_0).tt_;
        }
        OP_LT | OP_LE | OP_LTI | OP_LEI | OP_GTI | OP_GEI | OP_EQ => {
            let mut res: i32 = !((*((*L).top.p).offset(-(1))).val.tt_ as i32
                == 1 as i32 | (0) << 4 as i32
                || (*((*L).top.p).offset(-(1))).val.tt_ as i32 & 0xf as i32 == 0)
                as i32;
            (*L).top.p = ((*L).top.p).offset(-1);
            (*L).top.p;
            if res
                != (inst >> 0 + 7 as i32 + 8 as i32 & !(!(0 as Instruction) << 1 as i32) << 0)
                    as i32
            {
                (*ci).u.l.savedpc = ((*ci).u.l.savedpc).offset(1);
                (*ci).u.l.savedpc;
            }
        }
        OP_CONCAT => {
            let mut top: StkId = ((*L).top.p).offset(-(1));
            let mut a: i32 =
                (inst >> 0 + 7 as i32 & !(!(0 as Instruction) << 8 as i32) << 0) as i32;
            let mut total: i32 =
                top.offset(-(1)).offset_from(base.offset(a as isize)) as std::ffi::c_long as i32;
            let mut io1_1: *mut TValue = &mut (*top.offset(-(2))).val;
            let mut io2_1: *const TValue = &mut (*top).val;
            (*io1_1).value_ = (*io2_1).value_;
            (*io1_1).tt_ = (*io2_1).tt_;

            (*L).top.p = top.offset(-(1));
            luaV_concat(L, total);
        }
        OP_CLOSE => {
            (*ci).u.l.savedpc = ((*ci).u.l.savedpc).offset(-1);
            (*ci).u.l.savedpc;
        }
        OP_RETURN => {
            let mut ra: StkId = base.offset(
                (inst >> 0 + 7 as i32 & !(!(0 as Instruction) << 8 as i32) << 0) as i32 as isize,
            );
            (*L).top.p = ra.offset((*ci).u2.nres as isize);
            (*ci).u.l.savedpc = ((*ci).u.l.savedpc).offset(-1);
            (*ci).u.l.savedpc;
        }
        _ => {}
    };
}

macro_rules! intop {
    ($name:ident, $op:tt) => {
        #[inline(always)]
        fn $name(a: lua_Integer, b: lua_Integer) -> lua_Integer {
            ((a as lua_Unsigned) $op (b as lua_Unsigned)) as lua_Integer
        }
    };
}

macro_rules! fltop {
    ($name:ident, $op:tt) => {
        #[inline(always)]
        fn $name(a: lua_Number, b: lua_Number) -> lua_Number {
            a $op b
        }
    };
}

intop!(l_addi, +);
intop!(l_subi, -);
intop!(l_muli, *);
intop!(l_band, &);
intop!(l_bor, |);
intop!(l_bxor, ^);

fltop!(luai_numadd, +);
fltop!(luai_numsub, -);
fltop!(luai_nummul, *);
fltop!(luai_numdiv, /);

#[inline]
fn luai_numpow(a: lua_Number, b: lua_Number) -> lua_Number {
    if b == 2.0 { a * a } else { a.powf(b) }
}

#[inline(always)]
unsafe fn op_arith(
    i: u32,
    base: StkId,
    pc: *const Instruction,
    iop: impl Fn(lua_Integer, lua_Integer) -> lua_Integer,
    fop: impl Fn(lua_Number, lua_Number) -> lua_Number,
) -> *const Instruction {
    let mut v1: *mut TValue = &mut (*base.add(getarg_b(i) as usize)).val;
    let mut v2: *mut TValue = &mut (*base.add(getarg_c(i) as usize)).val;
    let mut ra: StkId = base.add(getarg_a(i) as usize);

    let a_tt = (*v1).tt_;
    let b_tt = (*v2).tt_;

    if std::hint::likely(a_tt == b_tt) {
        if a_tt == LUA_VNUMINT {
            setivalue(ra, iop((*v1).value_.i, (*v2).value_.i));
            pc.add(1)
        } else if a_tt == LUA_VNUMFLT {
            setfltvalue(ra, fop((*v1).value_.n, (*v2).value_.n));
            pc.add(1)
        } else {
            pc
        }
    } else {
        match (a_tt, b_tt) {
            (LUA_VNUMINT, LUA_VNUMFLT) => {
                setfltvalue(ra, fop((*v1).value_.i as lua_Number, (*v2).value_.n));
                pc.add(1)
            }
            (LUA_VNUMFLT, LUA_VNUMINT) => {
                setfltvalue(ra, fop((*v1).value_.n, (*v2).value_.i as lua_Number));
                pc.add(1)
            }
            _ => pc,
        }
    }
}

#[inline(always)]
unsafe fn op_arith_k(
    i: u32,
    base: StkId,
    k: *mut TValue,
    pc: *const Instruction,
    iop: impl Fn(lua_Integer, lua_Integer) -> lua_Integer,
    fop: impl Fn(lua_Number, lua_Number) -> lua_Number,
) -> *const Instruction {
    let mut v1: *mut TValue = &mut (*base.add(getarg_b(i) as usize)).val;
    let mut v2: *mut TValue = k.add(getarg_c(i) as usize);
    let mut ra: StkId = base.add(getarg_a(i) as usize);

    let a_tt = (*v1).tt_;
    let b_tt = (*v2).tt_;

    if std::hint::likely(a_tt == b_tt) {
        if a_tt == LUA_VNUMINT {
            setivalue(ra, iop((*v1).value_.i, (*v2).value_.i));
            pc.add(1)
        } else if a_tt == LUA_VNUMFLT {
            setfltvalue(ra, fop((*v1).value_.n, (*v2).value_.n));
            pc.add(1)
        } else {
            pc
        }
    } else {
        match (a_tt, b_tt) {
            (LUA_VNUMINT, LUA_VNUMFLT) => {
                setfltvalue(ra, fop((*v1).value_.i as lua_Number, (*v2).value_.n));
                pc.add(1)
            }
            (LUA_VNUMFLT, LUA_VNUMINT) => {
                setfltvalue(ra, fop((*v1).value_.n, (*v2).value_.i as lua_Number));
                pc.add(1)
            }
            _ => pc,
        }
    }
}

#[inline(always)]
unsafe fn op_arithf(
    i: u32,
    base: StkId,
    pc: *const Instruction,
    fop: impl Fn(lua_Number, lua_Number) -> lua_Number,
) -> *const Instruction {
    let mut v1: *mut TValue = &mut (*base.add(getarg_b(i) as usize)).val;
    let mut v2: *mut TValue = &mut (*base.add(getarg_c(i) as usize)).val;
    let mut ra: StkId = base.add(getarg_a(i) as usize);

    let a_tt = (*v1).tt_;
    let b_tt = (*v2).tt_;

    let a = match a_tt {
        LUA_VNUMINT => (*v1).value_.i as lua_Number,
        LUA_VNUMFLT => (*v1).value_.n,
        _ => return pc,
    };

    let b = match b_tt {
        LUA_VNUMINT => (*v2).value_.i as lua_Number,
        LUA_VNUMFLT => (*v2).value_.n,
        _ => return pc,
    };

    setfltvalue(ra, fop(a, b));
    pc.add(1)
}

#[inline(always)]
unsafe fn op_bitwise(
    i: u32,
    base: StkId,
    pc: *const Instruction,
    op: impl Fn(lua_Integer, lua_Integer) -> lua_Integer,
) -> *const Instruction {
    let mut ra: StkId = base.add(getarg_a(i) as usize);
    let mut v1: *mut TValue = &mut (*base.add(getarg_b(i) as usize)).val;
    let mut v2: *mut TValue = &mut (*base.add(getarg_c(i) as usize)).val;

    let a_tt = (*v1).tt_;
    let b_tt = (*v2).tt_;

    if std::hint::likely(a_tt == LUA_VNUMINT && b_tt == LUA_VNUMINT) {
        let a = (*v1).value_.i;
        let b = (*v2).value_.i;
        setivalue(ra, op(a, b));
        return pc.add(1);
    }

    op_bitwise_slow(ra, v1, v2, pc, op)
}

#[inline(always)]
unsafe fn op_bitwise_slow(
    ra: StkId,
    v1: *mut TValue,
    v2: *mut TValue,
    pc: *const Instruction,
    op: impl Fn(lua_Integer, lua_Integer) -> lua_Integer,
) -> *const Instruction {
    let mut i1: lua_Integer = 0;
    let mut i2: lua_Integer = 0;

    if (if (*v1).tt_ == LUA_VNUMINT {
        i1 = (*v1).value_.i;
        1 as i32
    } else {
        luaV_tointegerns::<F2Ieq>(v1, &mut i1)
    }) != 0
        && (if (*v2).tt_ == LUA_VNUMINT {
            i2 = (*v2).value_.i;
            1 as i32
        } else {
            luaV_tointegerns::<F2Ieq>(v2, &mut i2)
        }) != 0
    {
        let mut io_37: *mut TValue = &mut (*ra).val;
        (*io_37).value_.i = op(i1, i2);
        (*io_37).tt_ = (3 as i32 | (0) << 4 as i32) as lu_byte;
        pc.add(1)
    } else {
        pc
    }
}

#[cfg(feature = "jit")]
unsafe fn luaV_record_loop(
    proto: *mut Proto,
    pc: *const Instruction,
    tr: &mut trace::TraceRecorder,
) -> Option<NonNull<trace::Trace>> {
    let pc = pc.offset_from_unsigned((*proto).code) as u32;
    let loop_counters =
        ptr::slice_from_raw_parts_mut((*proto).loop_cnts, (*proto).size_loop_cnts as usize);
    if let Some(loop_counters) = loop_counters.as_mut() {
        if let Ok(idx) = loop_counters.binary_search_by_key(&pc, |lc| lc.pc) {
            let lc = loop_counters.get_unchecked_mut(idx);
            lc.count += 1;
            if !tr.recording && lc.count > 56 && lc.trace.is_none() {
                tr.begin_recording(&mut lc.trace);
            }
            // TODO: Could stitch trace here
            return lc.trace;
        }
    }
    None
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaV_execute(mut L: *mut lua_State, mut ci: *mut CallInfo) {
    let mut i: Instruction = 0;
    let mut ra_65: StkId = 0 as *mut StackValue;
    let mut newci: *mut CallInfo = 0 as *mut CallInfo;
    let mut b_4: i32 = 0;
    let mut nresults: i32 = 0;
    let mut current_block: u64;
    let mut cl: *mut LClosure = 0 as *mut LClosure;
    let mut k: *mut TValue = 0 as *mut TValue;
    let mut base: StkId = 0 as *mut StackValue;
    let mut pc: *const Instruction = 0 as *const Instruction;
    let mut trap: i32 = 0;
    #[cfg(feature = "jit")]
    let mut trace_recorder = trace::TraceRecorder::new();
    #[cfg(feature = "jit")]
    let mut next_trace: Option<NonNull<trace::Trace>> = None;
    '_startfunc: loop {
        trap = (*L).hookmask;
        '_returning: loop {
            cl = &mut (*((*(*ci).func.p).val.value_.gc as *mut GCUnion)).cl.l;
            k = (*(*cl).p).k;
            pc = (*ci).u.l.savedpc;
            if (trap != 0) as i32 as std::ffi::c_long != 0 {
                trap = luaG_tracecall(L);
            }
            base = ((*ci).func.p).offset(1);
            #[cfg(feature = "jit")]
            {
                next_trace = luaV_record_loop((*cl).p, pc, &mut trace_recorder);
            }
            loop {
                i = 0;
                if (trap != 0) as i32 as std::ffi::c_long != 0 {
                    trap = luaG_traceexec(L, pc);
                    base = ((*ci).func.p).offset(1);
                }

                #[cfg(feature = "jit")]
                if trace_recorder.recording {
                    let ok = trace_recorder.record_start(L, pc, ci, NonNull::new_unchecked(cl));
                    if !ok {
                        trace_recorder.end_recording(L);
                    }
                } else {
                    while let Some(trace) = next_trace.take() {
                        let entrypoint = (*(trace.as_ptr())).entrypoint;
                        let result = entrypoint(base, L, ci, cl);
                        if result < 0 {
                            pc = trace.as_ref().bail(result);
                        } else {
                            pc = (*(trace.as_ptr())).last_pc;

                            next_trace = luaV_record_loop((*cl).p, pc, &mut trace_recorder)
                        }
                    }
                }

                i = *pc;
                pc = pc.offset(1);
                match (i >> 0 & !(!(0 as Instruction) << 7 as i32) << 0) as OpCode as u32 {
                    OP_MOVE => {
                        let mut ra: StkId = base.add(getarg_a(i) as usize);
                        let mut io2: *const TValue = &(*base.add(getarg_b(i) as usize)).val;
                        setobj(ra, io2);
                        continue;
                    }
                    OP_LOADI => {
                        let mut ra: StkId = base.add(getarg_a(i) as usize);
                        let b = getarg_sbx(i) as lua_Integer;
                        setivalue(ra, b);
                        continue;
                    }
                    OP_LOADF => {
                        let mut ra: StkId = base.add(getarg_a(i) as usize);
                        let b = getarg_sbx(i);
                        setfltvalue(ra, b as lua_Number);
                        continue;
                    }
                    OP_LOADK => {
                        let mut ra: StkId = base.add(getarg_a(i) as usize);
                        let mut rb: *const TValue = k.add(getarg_bx(i) as usize);
                        setobj(ra, rb);
                        continue;
                    }
                    OP_LOADKX => {
                        let mut ra: StkId = base.add(getarg_a(i) as usize);
                        let mut rb: *const TValue = k.add(getarg_ax(*pc) as usize);
                        pc = pc.add(1);
                        setobj(ra, rb);
                        continue;
                    }
                    OP_LOADFALSE => {
                        let mut ra: StkId = base.add(getarg_a(i) as usize);
                        tt_set_false(ra);
                        continue;
                    }
                    OP_LFALSESKIP => {
                        let mut ra: StkId = base.add(getarg_a(i) as usize);
                        tt_set_false(ra);
                        pc = pc.add(1);
                        continue;
                    }
                    OP_LOADTRUE => {
                        let mut ra: StkId = base.add(getarg_a(i) as usize);
                        tt_set_true(ra);
                        continue;
                    }
                    OP_LOADNIL => {
                        let mut ra: StkId = base.add(getarg_a(i) as usize);
                        let mut b = getarg_b(i);
                        loop {
                            set_nil_value(ra);
                            ra = ra.add(1);
                            if b == 0 {
                                break;
                            }
                            b = b - 1;
                        }
                        continue;
                    }
                    OP_GETUPVAL => {
                        let mut ra: StkId = base.add(getarg_a(i) as usize);
                        let mut uv = getupval(cl, getarg_b(i) as usize);
                        let mut io1: *mut TValue = &mut (*ra).val;
                        let mut io2: *const TValue = (*uv).v.p;
                        setobj(io1, io2);
                        continue;
                    }
                    OP_SETUPVAL => {
                        let mut ra: StkId = base.add(getarg_a(i) as usize);
                        let mut uv: *mut UpVal = getupval(cl, getarg_b(i) as usize);
                        let mut io1: *mut TValue = (*uv).v.p;
                        let mut io2: *const TValue = &mut (*ra).val;
                        setobj(io1, io2);
                        luaC_barrier(L, uv, &raw mut (*ra).val);
                        continue;
                    }
                    OP_GETTABUP => {
                        let mut ra: StkId = base.add(getarg_a(i) as usize);
                        let mut slot: *const TValue = 0 as *const TValue;
                        let mut upval: *mut TValue = (*getupval(cl, getarg_b(i) as usize)).v.p;
                        let mut rc: *mut TValue = k.add(getarg_c(i) as usize);
                        let mut key: *mut TString = &mut (*((*rc).value_.gc as *mut GCUnion)).ts;

                        if luaV_fastget(upval, key, &mut slot, |t, k| luaH_getshortstr(t, k)) {
                            setobj(ra, slot);
                        } else {
                            // TODO: Protect macro
                            (*ci).u.l.savedpc = pc;
                            (*L).top.p = (*ci).top.p;
                            luaV_finishget(L, upval, rc, ra, slot);
                            trap = (*ci).u.l.trap;
                        }

                        continue;
                    }
                    OP_GETTABLE => {
                        let mut ra: StkId = base.add(getarg_a(i) as usize);
                        let mut slot: *const TValue = 0 as *const TValue;
                        let mut rb: *mut TValue = &mut (*base.add(getarg_b(i) as usize)).val;
                        let mut rc: *mut TValue = &mut (*base.add(getarg_c(i) as usize)).val;

                        let ok = if tt_is_integer(rc) {
                            luaV_fastgeti(rb, (*rc).value_.i, &mut slot)
                        } else {
                            luaV_fastget(rb, rc, &mut slot, |t, k| luaH_get(t, k))
                        };

                        if ok {
                            setobj(ra, slot);
                        } else {
                            (*ci).u.l.savedpc = pc;
                            (*L).top.p = (*ci).top.p;
                            luaV_finishget(L, rb, rc, ra, slot);
                            trap = (*ci).u.l.trap;
                        }
                        continue;
                    }
                    OP_GETI => {
                        let mut ra: StkId = base.add(getarg_a(i) as usize);
                        let mut slot: *const TValue = 0 as *const TValue;
                        let mut rb: *mut TValue = &mut (*base.add(getarg_b(i) as usize)).val;
                        let mut key: i32 = getarg_c(i) as i32;
                        if luaV_fastgeti(rb, key as i64, &mut slot) {
                            setobj(ra, slot);
                        } else {
                            let mut key_val: TValue = TValue {
                                value_: Value {
                                    gc: 0 as *mut GCObject,
                                },
                                tt_: 0,
                            };
                            setivalue(&raw mut key_val, key as i64);
                            (*ci).u.l.savedpc = pc;
                            (*L).top.p = (*ci).top.p;
                            luaV_finishget(L, rb, &mut key_val, ra, slot);
                            trap = (*ci).u.l.trap;
                        }
                        continue;
                    }
                    OP_GETFIELD => {
                        let mut ra: StkId = base.add(getarg_a(i) as usize);
                        let mut slot: *const TValue = 0 as *const TValue;
                        let mut rb: *mut TValue = &mut (*base.add(getarg_b(i) as usize)).val;
                        let mut rc: *mut TValue = k.add(getarg_c(i) as usize);
                        let mut key: *mut TString = &mut (*((*rc).value_.gc as *mut GCUnion)).ts;

                        if luaV_fastget(rb, key, &mut slot, |t, k| luaH_getshortstr(t, k)) {
                            setobj(ra, slot);
                        } else {
                            // TODO: Protect macro
                            (*ci).u.l.savedpc = pc;
                            (*L).top.p = (*ci).top.p;
                            luaV_finishget(L, rb, rc, ra, slot);
                            trap = (*ci).u.l.trap;
                        }
                        continue;
                    }
                    OP_SETTABUP => {
                        let mut slot: *const TValue = 0 as *const TValue;
                        let mut upval: *mut TValue = (*getupval(cl, getarg_a(i) as usize)).v.p;
                        let mut rb: *mut TValue = k.add(getarg_b(i) as usize);
                        let c = getarg_c(i) as usize;
                        let mut rc: *mut TValue = if getarg_k(i) {
                            k.add(c)
                        } else {
                            &mut (*base.add(c)).val
                        };
                        let mut key: *mut TString = &mut (*((*rb).value_.gc as *mut GCUnion)).ts;

                        if luaV_fastget(upval, key, &mut slot, |t, k| luaH_getshortstr(t, k)) {
                            luaV_finishfastset(L, upval, slot.cast_mut(), rc);
                        } else {
                            // TODO: Protect macro
                            (*ci).u.l.savedpc = pc;
                            (*L).top.p = (*ci).top.p;
                            luaV_finishset(L, upval, rb, rc, slot);
                            trap = (*ci).u.l.trap;
                        }
                        continue;
                    }
                    OP_SETTABLE => {
                        let mut ra: StkId = base.add(getarg_a(i) as usize);
                        let mut slot: *const TValue = 0 as *const TValue;
                        let mut rb: *mut TValue = &mut (*base.add(getarg_b(i) as usize)).val;
                        let c = getarg_c(i) as usize;
                        let mut rc: *mut TValue = if getarg_k(i) {
                            k.add(c)
                        } else {
                            &mut (*base.add(c)).val
                        };

                        let ok = if tt_is_integer(rb) {
                            luaV_fastgeti(&raw mut (*ra).val, (*rb).value_.i, &mut slot)
                        } else {
                            luaV_fastget(&raw mut (*ra).val, rb, &mut slot, |t, k| luaH_get(t, k))
                        };

                        if ok {
                            luaV_finishfastset(L, &raw mut (*ra).val, slot.cast_mut(), rc);
                        } else {
                            (*ci).u.l.savedpc = pc;
                            (*L).top.p = (*ci).top.p;
                            luaV_finishset(L, &raw mut (*ra).val, rb, rc, slot);
                            trap = (*ci).u.l.trap;
                        }
                        continue;
                    }
                    OP_SETI => {
                        let mut ra: StkId = base.add(getarg_a(i) as usize);
                        let mut slot: *const TValue = 0 as *const TValue;
                        let mut key = getarg_b(i);
                        let c = getarg_c(i) as usize;
                        let mut rc: *mut TValue = if getarg_k(i) {
                            k.add(c)
                        } else {
                            &mut (*base.add(c)).val
                        };

                        if luaV_fastgeti(&raw mut (*ra).val, key as i64, &mut slot) {
                            luaV_finishfastset(L, &raw mut (*ra).val, slot.cast_mut(), rc);
                        } else {
                            let mut key_val: TValue = TValue {
                                value_: Value {
                                    gc: 0 as *mut GCObject,
                                },
                                tt_: 0,
                            };
                            setivalue(&raw mut key_val, key as i64);
                            (*ci).u.l.savedpc = pc;
                            (*L).top.p = (*ci).top.p;
                            luaV_finishset(L, &raw mut (*ra).val, &mut key_val, rc, slot);
                            trap = (*ci).u.l.trap;
                        }
                        continue;
                    }
                    OP_SETFIELD => {
                        let mut ra: StkId = base.add(getarg_a(i) as usize);
                        let mut slot: *const TValue = 0 as *const TValue;
                        let mut rb: *mut TValue = k.add(getarg_b(i) as usize);
                        let c = getarg_c(i) as usize;
                        let mut rc: *mut TValue = if getarg_k(i) {
                            k.add(c)
                        } else {
                            &mut (*base.add(c)).val
                        };
                        let mut key: *mut TString = &mut (*((*rb).value_.gc as *mut GCUnion)).ts;

                        if luaV_fastget(&raw mut (*ra).val, key, &mut slot, |t, k| {
                            luaH_getshortstr(t, k)
                        }) {
                            luaV_finishfastset(L, &raw mut (*ra).val, slot.cast_mut(), rc);
                        } else {
                            // TODO: Protect macro
                            (*ci).u.l.savedpc = pc;
                            (*L).top.p = (*ci).top.p;
                            luaV_finishset(L, &raw mut (*ra).val, rb, rc, slot);
                            trap = (*ci).u.l.trap;
                        }
                        continue;
                    }
                    OP_NEWTABLE => {
                        let mut ra: StkId = base.add(getarg_a(i) as usize);
                        let mut b = getarg_b(i);
                        let mut c = getarg_c(i);
                        let mut t: *mut Table = 0 as *mut Table;

                        if b > 0 {
                            b = 1 << (b - 1);
                        }

                        if getarg_k(i) {
                            // Add non-zero extra argument
                            c += getarg_ax(*pc) * (MAXARG_C + 1);
                        }
                        pc = pc.add(1); // Skip extra argument

                        (*L).top.p = ra.add(1); // correct top in case of emergency GC
                        t = luaH_new(L);
                        sethvalue(ra, t);

                        if b != 0 || c != 0 {
                            luaH_resize(L, t, c, b);
                        }

                        // checkGC(L, ra + 1)
                        if (*(*L).l_G).GCdebt > 0 as l_mem {
                            (*ci).u.l.savedpc = pc;
                            (*L).top.p = ra.add(1);
                            luaC_step(L);
                            trap = (*ci).u.l.trap;
                        }
                        continue;
                    }
                    OP_SELF => {
                        let mut ra_18: StkId = base.offset(
                            (i >> 0 + 7 as i32 & !(!(0 as Instruction) << 8 as i32) << 0) as i32
                                as isize,
                        );
                        let mut slot_7: *const TValue = 0 as *const TValue;
                        let mut rb_7: *mut TValue = &mut (*base.offset(
                            (i >> 0 + 7 as i32 + 8 as i32 + 1 as i32
                                & !(!(0 as Instruction) << 8 as i32) << 0)
                                as i32 as isize,
                        ))
                        .val;
                        let mut rc_6: *mut TValue =
                            if (i & (1 as u32) << 0 + 7 as i32 + 8 as i32) as i32 != 0 {
                                k.offset(
                                    (i >> 0 + 7 as i32 + 8 as i32 + 1 as i32 + 8 as i32
                                        & !(!(0 as Instruction) << 8 as i32) << 0)
                                        as i32 as isize,
                                )
                            } else {
                                &mut (*base.offset(
                                    (i >> 0 + 7 as i32 + 8 as i32 + 1 as i32 + 8 as i32
                                        & !(!(0 as Instruction) << 8 as i32) << 0)
                                        as i32 as isize,
                                ))
                                .val
                            };
                        let mut key_5: *mut TString =
                            &mut (*((*rc_6).value_.gc as *mut GCUnion)).ts;
                        let mut io1_12: *mut TValue = &mut (*ra_18.offset(1)).val;
                        let mut io2_12: *const TValue = rb_7;
                        (*io1_12).value_ = (*io2_12).value_;
                        (*io1_12).tt_ = (*io2_12).tt_;

                        if if !((*rb_7).tt_ as i32
                            == 5 as i32 | (0) << 4 as i32 | (1 as i32) << 6 as i32)
                        {
                            slot_7 = 0 as *const TValue;
                            0
                        } else {
                            slot_7 =
                                luaH_getstr(&mut (*((*rb_7).value_.gc as *mut GCUnion)).h, key_5);
                            !((*slot_7).tt_ as i32 & 0xf as i32 == 0) as i32
                        } != 0
                        {
                            let mut io1_13: *mut TValue = &mut (*ra_18).val;
                            let mut io2_13: *const TValue = slot_7;
                            (*io1_13).value_ = (*io2_13).value_;
                            (*io1_13).tt_ = (*io2_13).tt_;
                        } else {
                            (*ci).u.l.savedpc = pc;
                            (*L).top.p = (*ci).top.p;
                            luaV_finishget(L, rb_7, rc_6, ra_18, slot_7);
                            trap = (*ci).u.l.trap;
                        }
                        continue;
                    }
                    OP_ADDI => {
                        let mut ra_19: StkId = base.offset(
                            (i >> 0 + 7 as i32 & !(!(0 as Instruction) << 8 as i32) << 0) as i32
                                as isize,
                        );
                        let mut v1: *mut TValue = &mut (*base.offset(
                            (i >> 0 + 7 as i32 + 8 as i32 + 1 as i32
                                & !(!(0 as Instruction) << 8 as i32) << 0)
                                as i32 as isize,
                        ))
                        .val;
                        let mut imm: i32 = (i >> 0 + 7 as i32 + 8 as i32 + 1 as i32 + 8 as i32
                            & !(!(0 as Instruction) << 8 as i32) << 0)
                            as i32
                            - (((1 as i32) << 8 as i32) - 1 as i32 >> 1 as i32);
                        if (*v1).tt_ as i32 == 3 as i32 | (0) << 4 as i32 {
                            let mut iv1: lua_Integer = (*v1).value_.i;
                            pc = pc.offset(1);
                            let mut io_4: *mut TValue = &mut (*ra_19).val;
                            (*io_4).value_.i = (iv1 as lua_Unsigned)
                                .wrapping_add(imm as lua_Unsigned)
                                as lua_Integer;
                            (*io_4).tt_ = (3 as i32 | (0) << 4 as i32) as lu_byte;
                        } else if (*v1).tt_ as i32 == 3 as i32 | (1 as i32) << 4 as i32 {
                            let mut nb: lua_Number = (*v1).value_.n;
                            let mut fimm: lua_Number = imm as lua_Number;
                            pc = pc.offset(1);
                            let mut io_5: *mut TValue = &mut (*ra_19).val;
                            (*io_5).value_.n = nb + fimm;
                            (*io_5).tt_ = (3 as i32 | (1 as i32) << 4 as i32) as lu_byte;
                        }
                        continue;
                    }
                    OP_ADDK => {
                        pc = op_arith_k(i, base, k, pc, l_addi, luai_numadd);
                        continue;
                    }
                    OP_SUBK => {
                        pc = op_arith_k(i, base, k, pc, l_subi, luai_numsub);
                        continue;
                    }
                    OP_MULK => {
                        pc = op_arith_k(i, base, k, pc, l_muli, luai_nummul);
                        continue;
                    }
                    OP_MODK => {
                        (*ci).u.l.savedpc = pc;
                        (*L).top.p = (*ci).top.p;
                        let mut v1_3: *mut TValue = &mut (*base.offset(
                            (i >> 0 + 7 as i32 + 8 as i32 + 1 as i32
                                & !(!(0 as Instruction) << 8 as i32) << 0)
                                as i32 as isize,
                        ))
                        .val;
                        let mut v2_2: *mut TValue = k.offset(
                            (i >> 0 + 7 as i32 + 8 as i32 + 1 as i32 + 8 as i32
                                & !(!(0 as Instruction) << 8 as i32) << 0)
                                as i32 as isize,
                        );
                        let mut ra_23: StkId = base.offset(
                            (i >> 0 + 7 as i32 & !(!(0 as Instruction) << 8 as i32) << 0) as i32
                                as isize,
                        );
                        if (*v1_3).tt_ as i32 == 3 as i32 | (0) << 4 as i32
                            && (*v2_2).tt_ as i32 == 3 as i32 | (0) << 4 as i32
                        {
                            let mut i1_2: lua_Integer = (*v1_3).value_.i;
                            let mut i2_2: lua_Integer = (*v2_2).value_.i;
                            pc = pc.offset(1);
                            let mut io_12: *mut TValue = &mut (*ra_23).val;
                            (*io_12).value_.i = luaV_mod(L, i1_2, i2_2);
                            (*io_12).tt_ = (3 as i32 | (0) << 4 as i32) as lu_byte;
                        } else {
                            let mut n1_2: lua_Number = 0.;
                            let mut n2_2: lua_Number = 0.;
                            if (if (*v1_3).tt_ as i32 == 3 as i32 | (1 as i32) << 4 as i32 {
                                n1_2 = (*v1_3).value_.n;
                                1 as i32
                            } else {
                                (if (*v1_3).tt_ as i32 == 3 as i32 | (0) << 4 as i32 {
                                    n1_2 = (*v1_3).value_.i as lua_Number;
                                    1 as i32
                                } else {
                                    0
                                })
                            }) != 0
                                && (if (*v2_2).tt_ as i32 == 3 as i32 | (1 as i32) << 4 as i32 {
                                    n2_2 = (*v2_2).value_.n;
                                    1 as i32
                                } else {
                                    (if (*v2_2).tt_ as i32 == 3 as i32 | (0) << 4 as i32 {
                                        n2_2 = (*v2_2).value_.i as lua_Number;
                                        1 as i32
                                    } else {
                                        0
                                    })
                                }) != 0
                            {
                                pc = pc.offset(1);
                                let mut io_13: *mut TValue = &mut (*ra_23).val;
                                (*io_13).value_.n = luaV_modf(L, n1_2, n2_2);
                                (*io_13).tt_ = (3 as i32 | (1 as i32) << 4 as i32) as lu_byte;
                            }
                        }
                        continue;
                    }
                    OP_POWK => {
                        let mut ra_24: StkId = base.offset(
                            (i >> 0 + 7 as i32 & !(!(0 as Instruction) << 8 as i32) << 0) as i32
                                as isize,
                        );
                        let mut v1_4: *mut TValue = &mut (*base.offset(
                            (i >> 0 + 7 as i32 + 8 as i32 + 1 as i32
                                & !(!(0 as Instruction) << 8 as i32) << 0)
                                as i32 as isize,
                        ))
                        .val;
                        let mut v2_3: *mut TValue = k.offset(
                            (i >> 0 + 7 as i32 + 8 as i32 + 1 as i32 + 8 as i32
                                & !(!(0 as Instruction) << 8 as i32) << 0)
                                as i32 as isize,
                        );
                        let mut n1_3: lua_Number = 0.;
                        let mut n2_3: lua_Number = 0.;
                        if (if (*v1_4).tt_ as i32 == 3 as i32 | (1 as i32) << 4 as i32 {
                            n1_3 = (*v1_4).value_.n;
                            1 as i32
                        } else {
                            (if (*v1_4).tt_ as i32 == 3 as i32 | (0) << 4 as i32 {
                                n1_3 = (*v1_4).value_.i as lua_Number;
                                1 as i32
                            } else {
                                0
                            })
                        }) != 0
                            && (if (*v2_3).tt_ as i32 == 3 as i32 | (1 as i32) << 4 as i32 {
                                n2_3 = (*v2_3).value_.n;
                                1 as i32
                            } else {
                                (if (*v2_3).tt_ as i32 == 3 as i32 | (0) << 4 as i32 {
                                    n2_3 = (*v2_3).value_.i as lua_Number;
                                    1 as i32
                                } else {
                                    0
                                })
                            }) != 0
                        {
                            pc = pc.offset(1);
                            let mut io_14: *mut TValue = &mut (*ra_24).val;
                            (*io_14).value_.n = (if n2_3 == 2 as i32 as lua_Number {
                                n1_3 * n1_3
                            } else {
                                n1_3.powf(n2_3)
                            });
                            (*io_14).tt_ = (3 as i32 | (1 as i32) << 4 as i32) as lu_byte;
                        }
                        continue;
                    }
                    OP_DIVK => {
                        let mut ra_25: StkId = base.offset(
                            (i >> 0 + 7 as i32 & !(!(0 as Instruction) << 8 as i32) << 0) as i32
                                as isize,
                        );
                        let mut v1_5: *mut TValue = &mut (*base.offset(
                            (i >> 0 + 7 as i32 + 8 as i32 + 1 as i32
                                & !(!(0 as Instruction) << 8 as i32) << 0)
                                as i32 as isize,
                        ))
                        .val;
                        let mut v2_4: *mut TValue = k.offset(
                            (i >> 0 + 7 as i32 + 8 as i32 + 1 as i32 + 8 as i32
                                & !(!(0 as Instruction) << 8 as i32) << 0)
                                as i32 as isize,
                        );
                        let mut n1_4: lua_Number = 0.;
                        let mut n2_4: lua_Number = 0.;
                        if (if (*v1_5).tt_ as i32 == 3 as i32 | (1 as i32) << 4 as i32 {
                            n1_4 = (*v1_5).value_.n;
                            1 as i32
                        } else {
                            (if (*v1_5).tt_ as i32 == 3 as i32 | (0) << 4 as i32 {
                                n1_4 = (*v1_5).value_.i as lua_Number;
                                1 as i32
                            } else {
                                0
                            })
                        }) != 0
                            && (if (*v2_4).tt_ as i32 == 3 as i32 | (1 as i32) << 4 as i32 {
                                n2_4 = (*v2_4).value_.n;
                                1 as i32
                            } else {
                                (if (*v2_4).tt_ as i32 == 3 as i32 | (0) << 4 as i32 {
                                    n2_4 = (*v2_4).value_.i as lua_Number;
                                    1 as i32
                                } else {
                                    0
                                })
                            }) != 0
                        {
                            pc = pc.offset(1);
                            let mut io_15: *mut TValue = &mut (*ra_25).val;
                            (*io_15).value_.n = n1_4 / n2_4;
                            (*io_15).tt_ = (3 as i32 | (1 as i32) << 4 as i32) as lu_byte;
                        }
                        continue;
                    }
                    OP_IDIVK => {
                        (*ci).u.l.savedpc = pc;
                        (*L).top.p = (*ci).top.p;
                        let mut v1_6: *mut TValue = &mut (*base.offset(
                            (i >> 0 + 7 as i32 + 8 as i32 + 1 as i32
                                & !(!(0 as Instruction) << 8 as i32) << 0)
                                as i32 as isize,
                        ))
                        .val;
                        let mut v2_5: *mut TValue = k.offset(
                            (i >> 0 + 7 as i32 + 8 as i32 + 1 as i32 + 8 as i32
                                & !(!(0 as Instruction) << 8 as i32) << 0)
                                as i32 as isize,
                        );
                        let mut ra_26: StkId = base.offset(
                            (i >> 0 + 7 as i32 & !(!(0 as Instruction) << 8 as i32) << 0) as i32
                                as isize,
                        );
                        if (*v1_6).tt_ as i32 == 3 as i32 | (0) << 4 as i32
                            && (*v2_5).tt_ as i32 == 3 as i32 | (0) << 4 as i32
                        {
                            let mut i1_3: lua_Integer = (*v1_6).value_.i;
                            let mut i2_3: lua_Integer = (*v2_5).value_.i;
                            pc = pc.offset(1);
                            let mut io_16: *mut TValue = &mut (*ra_26).val;
                            (*io_16).value_.i = luaV_idiv(L, i1_3, i2_3);
                            (*io_16).tt_ = (3 as i32 | (0) << 4 as i32) as lu_byte;
                        } else {
                            let mut n1_5: lua_Number = 0.;
                            let mut n2_5: lua_Number = 0.;
                            if (if (*v1_6).tt_ as i32 == 3 as i32 | (1 as i32) << 4 as i32 {
                                n1_5 = (*v1_6).value_.n;
                                1 as i32
                            } else {
                                (if (*v1_6).tt_ as i32 == 3 as i32 | (0) << 4 as i32 {
                                    n1_5 = (*v1_6).value_.i as lua_Number;
                                    1 as i32
                                } else {
                                    0
                                })
                            }) != 0
                                && (if (*v2_5).tt_ as i32 == 3 as i32 | (1 as i32) << 4 as i32 {
                                    n2_5 = (*v2_5).value_.n;
                                    1 as i32
                                } else {
                                    (if (*v2_5).tt_ as i32 == 3 as i32 | (0) << 4 as i32 {
                                        n2_5 = (*v2_5).value_.i as lua_Number;
                                        1 as i32
                                    } else {
                                        0
                                    })
                                }) != 0
                            {
                                pc = pc.offset(1);
                                let mut io_17: *mut TValue = &mut (*ra_26).val;
                                (*io_17).value_.n = (n1_5 / n2_5).floor();
                                (*io_17).tt_ = (3 as i32 | (1 as i32) << 4 as i32) as lu_byte;
                            }
                        }
                        continue;
                    }
                    OP_BANDK => {
                        let mut ra_27: StkId = base.offset(
                            (i >> 0 + 7 as i32 & !(!(0 as Instruction) << 8 as i32) << 0) as i32
                                as isize,
                        );
                        let mut v1_7: *mut TValue = &mut (*base.offset(
                            (i >> 0 + 7 as i32 + 8 as i32 + 1 as i32
                                & !(!(0 as Instruction) << 8 as i32) << 0)
                                as i32 as isize,
                        ))
                        .val;
                        let mut v2_6: *mut TValue = k.offset(
                            (i >> 0 + 7 as i32 + 8 as i32 + 1 as i32 + 8 as i32
                                & !(!(0 as Instruction) << 8 as i32) << 0)
                                as i32 as isize,
                        );
                        let mut i1_4: lua_Integer = 0;
                        let mut i2_4: lua_Integer = (*v2_6).value_.i;
                        if if (((*v1_7).tt_ as i32 == 3 as i32 | (0) << 4 as i32) as i32 != 0)
                            as i32 as std::ffi::c_long
                            != 0
                        {
                            i1_4 = (*v1_7).value_.i;
                            1 as i32
                        } else {
                            luaV_tointegerns::<F2Ieq>(v1_7, &mut i1_4)
                        } != 0
                        {
                            pc = pc.offset(1);
                            let mut io_18: *mut TValue = &mut (*ra_27).val;
                            (*io_18).value_.i =
                                (i1_4 as lua_Unsigned & i2_4 as lua_Unsigned) as lua_Integer;
                            (*io_18).tt_ = (3 as i32 | (0) << 4 as i32) as lu_byte;
                        }
                        continue;
                    }
                    OP_BORK => {
                        let mut ra_28: StkId = base.offset(
                            (i >> 0 + 7 as i32 & !(!(0 as Instruction) << 8 as i32) << 0) as i32
                                as isize,
                        );
                        let mut v1_8: *mut TValue = &mut (*base.offset(
                            (i >> 0 + 7 as i32 + 8 as i32 + 1 as i32
                                & !(!(0 as Instruction) << 8 as i32) << 0)
                                as i32 as isize,
                        ))
                        .val;
                        let mut v2_7: *mut TValue = k.offset(
                            (i >> 0 + 7 as i32 + 8 as i32 + 1 as i32 + 8 as i32
                                & !(!(0 as Instruction) << 8 as i32) << 0)
                                as i32 as isize,
                        );
                        let mut i1_5: lua_Integer = 0;
                        let mut i2_5: lua_Integer = (*v2_7).value_.i;
                        if if (((*v1_8).tt_ as i32 == 3 as i32 | (0) << 4 as i32) as i32 != 0)
                            as i32 as std::ffi::c_long
                            != 0
                        {
                            i1_5 = (*v1_8).value_.i;
                            1 as i32
                        } else {
                            luaV_tointegerns::<F2Ieq>(v1_8, &mut i1_5)
                        } != 0
                        {
                            pc = pc.offset(1);
                            let mut io_19: *mut TValue = &mut (*ra_28).val;
                            (*io_19).value_.i =
                                (i1_5 as lua_Unsigned | i2_5 as lua_Unsigned) as lua_Integer;
                            (*io_19).tt_ = (3 as i32 | (0) << 4 as i32) as lu_byte;
                        }
                        continue;
                    }
                    OP_BXORK => {
                        let mut ra_29: StkId = base.offset(
                            (i >> 0 + 7 as i32 & !(!(0 as Instruction) << 8 as i32) << 0) as i32
                                as isize,
                        );
                        let mut v1_9: *mut TValue = &mut (*base.offset(
                            (i >> 0 + 7 as i32 + 8 as i32 + 1 as i32
                                & !(!(0 as Instruction) << 8 as i32) << 0)
                                as i32 as isize,
                        ))
                        .val;
                        let mut v2_8: *mut TValue = k.offset(
                            (i >> 0 + 7 as i32 + 8 as i32 + 1 as i32 + 8 as i32
                                & !(!(0 as Instruction) << 8 as i32) << 0)
                                as i32 as isize,
                        );
                        let mut i1_6: lua_Integer = 0;
                        let mut i2_6: lua_Integer = (*v2_8).value_.i;
                        if if (((*v1_9).tt_ as i32 == 3 as i32 | (0) << 4 as i32) as i32 != 0)
                            as i32 as std::ffi::c_long
                            != 0
                        {
                            i1_6 = (*v1_9).value_.i;
                            1 as i32
                        } else {
                            luaV_tointegerns::<F2Ieq>(v1_9, &mut i1_6)
                        } != 0
                        {
                            pc = pc.offset(1);
                            let mut io_20: *mut TValue = &mut (*ra_29).val;
                            (*io_20).value_.i =
                                (i1_6 as lua_Unsigned ^ i2_6 as lua_Unsigned) as lua_Integer;
                            (*io_20).tt_ = (3 as i32 | (0) << 4 as i32) as lu_byte;
                        }
                        continue;
                    }
                    OP_SHRI => {
                        let mut ra_30: StkId = base.offset(
                            (i >> 0 + 7 as i32 & !(!(0 as Instruction) << 8 as i32) << 0) as i32
                                as isize,
                        );
                        let mut rb_8: *mut TValue = &mut (*base.offset(
                            (i >> 0 + 7 as i32 + 8 as i32 + 1 as i32
                                & !(!(0 as Instruction) << 8 as i32) << 0)
                                as i32 as isize,
                        ))
                        .val;
                        let mut ic: i32 = (i >> 0 + 7 as i32 + 8 as i32 + 1 as i32 + 8 as i32
                            & !(!(0 as Instruction) << 8 as i32) << 0)
                            as i32
                            - (((1 as i32) << 8 as i32) - 1 as i32 >> 1 as i32);
                        let mut ib: lua_Integer = 0;
                        if if (((*rb_8).tt_ as i32 == 3 as i32 | (0) << 4 as i32) as i32 != 0)
                            as i32 as std::ffi::c_long
                            != 0
                        {
                            ib = (*rb_8).value_.i;
                            1 as i32
                        } else {
                            luaV_tointegerns::<F2Ieq>(rb_8, &mut ib)
                        } != 0
                        {
                            pc = pc.offset(1);
                            let mut io_21: *mut TValue = &mut (*ra_30).val;
                            (*io_21).value_.i = luaV_shiftl(ib, -ic as lua_Integer);
                            (*io_21).tt_ = (3 as i32 | (0) << 4 as i32) as lu_byte;
                        }
                        continue;
                    }
                    OP_SHLI => {
                        let mut ra_31: StkId = base.offset(
                            (i >> 0 + 7 as i32 & !(!(0 as Instruction) << 8 as i32) << 0) as i32
                                as isize,
                        );
                        let mut rb_9: *mut TValue = &mut (*base.offset(
                            (i >> 0 + 7 as i32 + 8 as i32 + 1 as i32
                                & !(!(0 as Instruction) << 8 as i32) << 0)
                                as i32 as isize,
                        ))
                        .val;
                        let mut ic_0: i32 = (i >> 0 + 7 as i32 + 8 as i32 + 1 as i32 + 8 as i32
                            & !(!(0 as Instruction) << 8 as i32) << 0)
                            as i32
                            - (((1 as i32) << 8 as i32) - 1 as i32 >> 1 as i32);
                        let mut ib_0: lua_Integer = 0;
                        if if (((*rb_9).tt_ as i32 == 3 as i32 | (0) << 4 as i32) as i32 != 0)
                            as i32 as std::ffi::c_long
                            != 0
                        {
                            ib_0 = (*rb_9).value_.i;
                            1 as i32
                        } else {
                            luaV_tointegerns::<F2Ieq>(rb_9, &mut ib_0)
                        } != 0
                        {
                            pc = pc.offset(1);
                            let mut io_22: *mut TValue = &mut (*ra_31).val;
                            (*io_22).value_.i = luaV_shiftl(ic_0 as lua_Integer, ib_0);
                            (*io_22).tt_ = (3 as i32 | (0) << 4 as i32) as lu_byte;
                        }
                        continue;
                    }
                    OP_ADD => {
                        pc = op_arith(i, base, pc, l_addi, luai_numadd);
                        continue;
                    }
                    OP_SUB => {
                        pc = op_arith(i, base, pc, l_subi, luai_numsub);
                        continue;
                    }
                    OP_MUL => {
                        pc = op_arith(i, base, pc, l_muli, luai_nummul);
                        continue;
                    }
                    OP_MOD => {
                        (*ci).u.l.savedpc = pc;
                        (*L).top.p = (*ci).top.p;
                        let mut v1_13: *mut TValue = &mut (*base.offset(
                            (i >> 0 + 7 as i32 + 8 as i32 + 1 as i32
                                & !(!(0 as Instruction) << 8 as i32) << 0)
                                as i32 as isize,
                        ))
                        .val;
                        let mut v2_12: *mut TValue = &mut (*base.offset(
                            (i >> 0 + 7 as i32 + 8 as i32 + 1 as i32 + 8 as i32
                                & !(!(0 as Instruction) << 8 as i32) << 0)
                                as i32 as isize,
                        ))
                        .val;
                        let mut ra_35: StkId = base.offset(
                            (i >> 0 + 7 as i32 & !(!(0 as Instruction) << 8 as i32) << 0) as i32
                                as isize,
                        );
                        if (*v1_13).tt_ as i32 == 3 as i32 | (0) << 4 as i32
                            && (*v2_12).tt_ as i32 == 3 as i32 | (0) << 4 as i32
                        {
                            let mut i1_10: lua_Integer = (*v1_13).value_.i;
                            let mut i2_10: lua_Integer = (*v2_12).value_.i;
                            pc = pc.offset(1);
                            let mut io_29: *mut TValue = &mut (*ra_35).val;
                            (*io_29).value_.i = luaV_mod(L, i1_10, i2_10);
                            (*io_29).tt_ = (3 as i32 | (0) << 4 as i32) as lu_byte;
                        } else {
                            let mut n1_9: lua_Number = 0.;
                            let mut n2_9: lua_Number = 0.;
                            if (if (*v1_13).tt_ as i32 == 3 as i32 | (1 as i32) << 4 as i32 {
                                n1_9 = (*v1_13).value_.n;
                                1 as i32
                            } else {
                                (if (*v1_13).tt_ as i32 == 3 as i32 | (0) << 4 as i32 {
                                    n1_9 = (*v1_13).value_.i as lua_Number;
                                    1 as i32
                                } else {
                                    0
                                })
                            }) != 0
                                && (if (*v2_12).tt_ as i32 == 3 as i32 | (1 as i32) << 4 as i32 {
                                    n2_9 = (*v2_12).value_.n;
                                    1 as i32
                                } else {
                                    (if (*v2_12).tt_ as i32 == 3 as i32 | (0) << 4 as i32 {
                                        n2_9 = (*v2_12).value_.i as lua_Number;
                                        1 as i32
                                    } else {
                                        0
                                    })
                                }) != 0
                            {
                                pc = pc.offset(1);
                                let mut io_30: *mut TValue = &mut (*ra_35).val;
                                (*io_30).value_.n = luaV_modf(L, n1_9, n2_9);
                                (*io_30).tt_ = (3 as i32 | (1 as i32) << 4 as i32) as lu_byte;
                            }
                        }
                        continue;
                    }
                    OP_POW => {
                        pc = op_arithf(i, base, pc, luai_numpow);
                        continue;
                    }
                    OP_DIV => {
                        pc = op_arithf(i, base, pc, luai_numdiv);
                        continue;
                    }
                    OP_IDIV => {
                        (*ci).u.l.savedpc = pc;
                        (*L).top.p = (*ci).top.p;
                        let mut v1_16: *mut TValue = &mut (*base.offset(
                            (i >> 0 + 7 as i32 + 8 as i32 + 1 as i32
                                & !(!(0 as Instruction) << 8 as i32) << 0)
                                as i32 as isize,
                        ))
                        .val;
                        let mut v2_15: *mut TValue = &mut (*base.offset(
                            (i >> 0 + 7 as i32 + 8 as i32 + 1 as i32 + 8 as i32
                                & !(!(0 as Instruction) << 8 as i32) << 0)
                                as i32 as isize,
                        ))
                        .val;
                        let mut ra_38: StkId = base.offset(
                            (i >> 0 + 7 as i32 & !(!(0 as Instruction) << 8 as i32) << 0) as i32
                                as isize,
                        );
                        if (*v1_16).tt_ as i32 == 3 as i32 | (0) << 4 as i32
                            && (*v2_15).tt_ as i32 == 3 as i32 | (0) << 4 as i32
                        {
                            let mut i1_11: lua_Integer = (*v1_16).value_.i;
                            let mut i2_11: lua_Integer = (*v2_15).value_.i;
                            pc = pc.offset(1);
                            let mut io_33: *mut TValue = &mut (*ra_38).val;
                            (*io_33).value_.i = luaV_idiv(L, i1_11, i2_11);
                            (*io_33).tt_ = (3 as i32 | (0) << 4 as i32) as lu_byte;
                        } else {
                            let mut n1_12: lua_Number = 0.;
                            let mut n2_12: lua_Number = 0.;
                            if (if (*v1_16).tt_ as i32 == 3 as i32 | (1 as i32) << 4 as i32 {
                                n1_12 = (*v1_16).value_.n;
                                1 as i32
                            } else {
                                (if (*v1_16).tt_ as i32 == 3 as i32 | (0) << 4 as i32 {
                                    n1_12 = (*v1_16).value_.i as lua_Number;
                                    1 as i32
                                } else {
                                    0
                                })
                            }) != 0
                                && (if (*v2_15).tt_ as i32 == 3 as i32 | (1 as i32) << 4 as i32 {
                                    n2_12 = (*v2_15).value_.n;
                                    1 as i32
                                } else {
                                    (if (*v2_15).tt_ as i32 == 3 as i32 | (0) << 4 as i32 {
                                        n2_12 = (*v2_15).value_.i as lua_Number;
                                        1 as i32
                                    } else {
                                        0
                                    })
                                }) != 0
                            {
                                pc = pc.offset(1);
                                let mut io_34: *mut TValue = &mut (*ra_38).val;
                                (*io_34).value_.n = (n1_12 / n2_12).floor();
                                (*io_34).tt_ = (3 as i32 | (1 as i32) << 4 as i32) as lu_byte;
                            }
                        }
                        continue;
                    }
                    OP_BAND => {
                        pc = op_bitwise(i, base, pc, l_band);
                        continue;
                    }
                    OP_BOR => {
                        pc = op_bitwise(i, base, pc, l_bor);
                        continue;
                    }
                    OP_BXOR => {
                        pc = op_bitwise(i, base, pc, l_bxor);
                        continue;
                    }
                    OP_SHR => {
                        let mut ra_42: StkId = base.offset(
                            (i >> 0 + 7 as i32 & !(!(0 as Instruction) << 8 as i32) << 0) as i32
                                as isize,
                        );
                        let mut v1_20: *mut TValue = &mut (*base.offset(
                            (i >> 0 + 7 as i32 + 8 as i32 + 1 as i32
                                & !(!(0 as Instruction) << 8 as i32) << 0)
                                as i32 as isize,
                        ))
                        .val;
                        let mut v2_19: *mut TValue = &mut (*base.offset(
                            (i >> 0 + 7 as i32 + 8 as i32 + 1 as i32 + 8 as i32
                                & !(!(0 as Instruction) << 8 as i32) << 0)
                                as i32 as isize,
                        ))
                        .val;
                        let mut i1_15: lua_Integer = 0;
                        let mut i2_15: lua_Integer = 0;
                        if (if (((*v1_20).tt_ as i32 == 3 as i32 | (0) << 4 as i32) as i32 != 0)
                            as i32 as std::ffi::c_long
                            != 0
                        {
                            i1_15 = (*v1_20).value_.i;
                            1 as i32
                        } else {
                            luaV_tointegerns::<F2Ieq>(v1_20, &mut i1_15)
                        }) != 0
                            && (if (((*v2_19).tt_ as i32 == 3 as i32 | (0) << 4 as i32) as i32 != 0)
                                as i32 as std::ffi::c_long
                                != 0
                            {
                                i2_15 = (*v2_19).value_.i;
                                1 as i32
                            } else {
                                luaV_tointegerns::<F2Ieq>(v2_19, &mut i2_15)
                            }) != 0
                        {
                            pc = pc.offset(1);
                            let mut io_38: *mut TValue = &mut (*ra_42).val;
                            (*io_38).value_.i = luaV_shiftl(
                                i1_15,
                                (0 as lua_Unsigned).wrapping_sub(i2_15 as lua_Unsigned)
                                    as lua_Integer,
                            );
                            (*io_38).tt_ = (3 as i32 | (0) << 4 as i32) as lu_byte;
                        }
                        continue;
                    }
                    OP_SHL => {
                        let mut ra_43: StkId = base.offset(
                            (i >> 0 + 7 as i32 & !(!(0 as Instruction) << 8 as i32) << 0) as i32
                                as isize,
                        );
                        let mut v1_21: *mut TValue = &mut (*base.offset(
                            (i >> 0 + 7 as i32 + 8 as i32 + 1 as i32
                                & !(!(0 as Instruction) << 8 as i32) << 0)
                                as i32 as isize,
                        ))
                        .val;
                        let mut v2_20: *mut TValue = &mut (*base.offset(
                            (i >> 0 + 7 as i32 + 8 as i32 + 1 as i32 + 8 as i32
                                & !(!(0 as Instruction) << 8 as i32) << 0)
                                as i32 as isize,
                        ))
                        .val;
                        let mut i1_16: lua_Integer = 0;
                        let mut i2_16: lua_Integer = 0;
                        if (if (((*v1_21).tt_ as i32 == 3 as i32 | (0) << 4 as i32) as i32 != 0)
                            as i32 as std::ffi::c_long
                            != 0
                        {
                            i1_16 = (*v1_21).value_.i;
                            1 as i32
                        } else {
                            luaV_tointegerns::<F2Ieq>(v1_21, &mut i1_16)
                        }) != 0
                            && (if (((*v2_20).tt_ as i32 == 3 as i32 | (0) << 4 as i32) as i32 != 0)
                                as i32 as std::ffi::c_long
                                != 0
                            {
                                i2_16 = (*v2_20).value_.i;
                                1 as i32
                            } else {
                                luaV_tointegerns::<F2Ieq>(v2_20, &mut i2_16)
                            }) != 0
                        {
                            pc = pc.offset(1);
                            let mut io_39: *mut TValue = &mut (*ra_43).val;
                            (*io_39).value_.i = luaV_shiftl(i1_16, i2_16);
                            (*io_39).tt_ = (3 as i32 | (0) << 4 as i32) as lu_byte;
                        }
                        continue;
                    }
                    OP_MMBIN => {
                        let mut ra_44: StkId = base.offset(
                            (i >> 0 + 7 as i32 & !(!(0 as Instruction) << 8 as i32) << 0) as i32
                                as isize,
                        );
                        let mut pi: Instruction = *pc.offset(-(2));
                        let mut rb_10: *mut TValue = &mut (*base.offset(
                            (i >> 0 + 7 as i32 + 8 as i32 + 1 as i32
                                & !(!(0 as Instruction) << 8 as i32) << 0)
                                as i32 as isize,
                        ))
                        .val;
                        let mut tm: TMS = (i >> 0 + 7 as i32 + 8 as i32 + 1 as i32 + 8 as i32
                            & !(!(0 as Instruction) << 8 as i32) << 0)
                            as i32 as TMS;
                        let mut result: StkId = base.offset(
                            (pi >> 0 + 7 as i32 & !(!(0 as Instruction) << 8 as i32) << 0) as i32
                                as isize,
                        );
                        (*ci).u.l.savedpc = pc;
                        (*L).top.p = (*ci).top.p;
                        luaT_trybinTM(L, &mut (*ra_44).val, rb_10, result, tm);
                        trap = (*ci).u.l.trap;
                        continue;
                    }
                    OP_MMBINI => {
                        let mut ra_45: StkId = base.offset(
                            (i >> 0 + 7 as i32 & !(!(0 as Instruction) << 8 as i32) << 0) as i32
                                as isize,
                        );
                        let mut pi_0: Instruction = *pc.offset(-(2));
                        let mut imm_0: i32 = (i >> 0 + 7 as i32 + 8 as i32 + 1 as i32
                            & !(!(0 as Instruction) << 8 as i32) << 0)
                            as i32
                            - (((1 as i32) << 8 as i32) - 1 as i32 >> 1 as i32);
                        let mut tm_0: TMS = (i >> 0 + 7 as i32 + 8 as i32 + 1 as i32 + 8 as i32
                            & !(!(0 as Instruction) << 8 as i32) << 0)
                            as i32 as TMS;
                        let mut flip: i32 = (i >> 0 + 7 as i32 + 8 as i32
                            & !(!(0 as Instruction) << 1 as i32) << 0)
                            as i32;
                        let mut result_0: StkId = base.offset(
                            (pi_0 >> 0 + 7 as i32 & !(!(0 as Instruction) << 8 as i32) << 0) as i32
                                as isize,
                        );
                        (*ci).u.l.savedpc = pc;
                        (*L).top.p = (*ci).top.p;
                        luaT_trybiniTM(
                            L,
                            &mut (*ra_45).val,
                            imm_0 as lua_Integer,
                            flip,
                            result_0,
                            tm_0,
                        );
                        trap = (*ci).u.l.trap;
                        continue;
                    }
                    OP_MMBINK => {
                        let mut ra_46: StkId = base.offset(
                            (i >> 0 + 7 as i32 & !(!(0 as Instruction) << 8 as i32) << 0) as i32
                                as isize,
                        );
                        let mut pi_1: Instruction = *pc.offset(-(2));
                        let mut imm_1: *mut TValue = k.offset(
                            (i >> 0 + 7 as i32 + 8 as i32 + 1 as i32
                                & !(!(0 as Instruction) << 8 as i32) << 0)
                                as i32 as isize,
                        );
                        let mut tm_1: TMS = (i >> 0 + 7 as i32 + 8 as i32 + 1 as i32 + 8 as i32
                            & !(!(0 as Instruction) << 8 as i32) << 0)
                            as i32 as TMS;
                        let mut flip_0: i32 = (i >> 0 + 7 as i32 + 8 as i32
                            & !(!(0 as Instruction) << 1 as i32) << 0)
                            as i32;
                        let mut result_1: StkId = base.offset(
                            (pi_1 >> 0 + 7 as i32 & !(!(0 as Instruction) << 8 as i32) << 0) as i32
                                as isize,
                        );
                        (*ci).u.l.savedpc = pc;
                        (*L).top.p = (*ci).top.p;
                        luaT_trybinassocTM(L, &mut (*ra_46).val, imm_1, flip_0, result_1, tm_1);
                        trap = (*ci).u.l.trap;
                        continue;
                    }
                    OP_UNM => {
                        let mut ra_47: StkId = base.offset(
                            (i >> 0 + 7 as i32 & !(!(0 as Instruction) << 8 as i32) << 0) as i32
                                as isize,
                        );
                        let mut rb_11: *mut TValue = &mut (*base.offset(
                            (i >> 0 + 7 as i32 + 8 as i32 + 1 as i32
                                & !(!(0 as Instruction) << 8 as i32) << 0)
                                as i32 as isize,
                        ))
                        .val;
                        let mut nb_0: lua_Number = 0.;
                        if (*rb_11).tt_ as i32 == 3 as i32 | (0) << 4 as i32 {
                            let mut ib_1: lua_Integer = (*rb_11).value_.i;
                            let mut io_40: *mut TValue = &mut (*ra_47).val;
                            (*io_40).value_.i = (0 as lua_Unsigned)
                                .wrapping_sub(ib_1 as lua_Unsigned)
                                as lua_Integer;
                            (*io_40).tt_ = (3 as i32 | (0) << 4 as i32) as lu_byte;
                        } else if if (*rb_11).tt_ as i32 == 3 as i32 | (1 as i32) << 4 as i32 {
                            nb_0 = (*rb_11).value_.n;
                            1 as i32
                        } else if (*rb_11).tt_ as i32 == 3 as i32 | (0) << 4 as i32 {
                            nb_0 = (*rb_11).value_.i as lua_Number;
                            1 as i32
                        } else {
                            0
                        } != 0
                        {
                            let mut io_41: *mut TValue = &mut (*ra_47).val;
                            (*io_41).value_.n = -nb_0;
                            (*io_41).tt_ = (3 as i32 | (1 as i32) << 4 as i32) as lu_byte;
                        } else {
                            (*ci).u.l.savedpc = pc;
                            (*L).top.p = (*ci).top.p;
                            luaT_trybinTM(L, rb_11, rb_11, ra_47, TM_UNM);
                            trap = (*ci).u.l.trap;
                        }
                        continue;
                    }
                    OP_BNOT => {
                        let mut ra_48: StkId = base.offset(
                            (i >> 0 + 7 as i32 & !(!(0 as Instruction) << 8 as i32) << 0) as i32
                                as isize,
                        );
                        let mut rb_12: *mut TValue = &mut (*base.offset(
                            (i >> 0 + 7 as i32 + 8 as i32 + 1 as i32
                                & !(!(0 as Instruction) << 8 as i32) << 0)
                                as i32 as isize,
                        ))
                        .val;
                        let mut ib_2: lua_Integer = 0;
                        if if (((*rb_12).tt_ as i32 == 3 as i32 | (0) << 4 as i32) as i32 != 0)
                            as i32 as std::ffi::c_long
                            != 0
                        {
                            ib_2 = (*rb_12).value_.i;
                            1 as i32
                        } else {
                            luaV_tointegerns::<F2Ieq>(rb_12, &mut ib_2)
                        } != 0
                        {
                            let mut io_42: *mut TValue = &mut (*ra_48).val;
                            (*io_42).value_.i =
                                (!(0 as lua_Unsigned) ^ ib_2 as lua_Unsigned) as lua_Integer;
                            (*io_42).tt_ = (3 as i32 | (0) << 4 as i32) as lu_byte;
                        } else {
                            (*ci).u.l.savedpc = pc;
                            (*L).top.p = (*ci).top.p;
                            luaT_trybinTM(L, rb_12, rb_12, ra_48, TM_BNOT);
                            trap = (*ci).u.l.trap;
                        }
                        continue;
                    }
                    OP_NOT => {
                        let mut ra_49: StkId = base.offset(
                            (i >> 0 + 7 as i32 & !(!(0 as Instruction) << 8 as i32) << 0) as i32
                                as isize,
                        );
                        let mut rb_13: *mut TValue = &mut (*base.offset(
                            (i >> 0 + 7 as i32 + 8 as i32 + 1 as i32
                                & !(!(0 as Instruction) << 8 as i32) << 0)
                                as i32 as isize,
                        ))
                        .val;
                        if (*rb_13).tt_ as i32 == 1 as i32 | (0) << 4 as i32
                            || (*rb_13).tt_ as i32 & 0xf as i32 == 0
                        {
                            (*ra_49).val.tt_ = (1 as i32 | (1 as i32) << 4 as i32) as lu_byte;
                        } else {
                            (*ra_49).val.tt_ = (1 as i32 | (0) << 4 as i32) as lu_byte;
                        }
                        continue;
                    }
                    OP_LEN => {
                        let mut ra_50: StkId = base.offset(
                            (i >> 0 + 7 as i32 & !(!(0 as Instruction) << 8 as i32) << 0) as i32
                                as isize,
                        );
                        (*ci).u.l.savedpc = pc;
                        (*L).top.p = (*ci).top.p;
                        luaV_objlen(
                            L,
                            ra_50,
                            &mut (*base.offset(
                                (i >> 0 + 7 as i32 + 8 as i32 + 1 as i32
                                    & !(!(0 as Instruction) << 8 as i32) << 0)
                                    as i32 as isize,
                            ))
                            .val,
                        );
                        trap = (*ci).u.l.trap;
                        continue;
                    }
                    OP_CONCAT => {
                        let mut ra_51: StkId = base.offset(
                            (i >> 0 + 7 as i32 & !(!(0 as Instruction) << 8 as i32) << 0) as i32
                                as isize,
                        );
                        let mut n_1: i32 = (i >> 0 + 7 as i32 + 8 as i32 + 1 as i32
                            & !(!(0 as Instruction) << 8 as i32) << 0)
                            as i32;
                        (*L).top.p = ra_51.offset(n_1 as isize);
                        (*ci).u.l.savedpc = pc;
                        luaV_concat(L, n_1);
                        trap = (*ci).u.l.trap;
                        if (*(*L).l_G).GCdebt > 0 as l_mem {
                            (*ci).u.l.savedpc = pc;
                            (*L).top.p = (*L).top.p;
                            luaC_step(L);
                            trap = (*ci).u.l.trap;
                        }
                        continue;
                    }
                    OP_CLOSE => {
                        let mut ra_52: StkId = base.offset(
                            (i >> 0 + 7 as i32 & !(!(0 as Instruction) << 8 as i32) << 0) as i32
                                as isize,
                        );
                        (*ci).u.l.savedpc = pc;
                        (*L).top.p = (*ci).top.p;
                        luaF_close(L, ra_52, 0, 1 as i32);
                        trap = (*ci).u.l.trap;
                        continue;
                    }
                    OP_TBC => {
                        let mut ra_53: StkId = base.offset(
                            (i >> 0 + 7 as i32 & !(!(0 as Instruction) << 8 as i32) << 0) as i32
                                as isize,
                        );
                        (*ci).u.l.savedpc = pc;
                        (*L).top.p = (*ci).top.p;
                        luaF_newtbcupval(L, ra_53);
                        continue;
                    }
                    OP_JMP => {
                        let offset = getarg_sj(i) as isize;
                        pc = pc.offset(offset);
                        #[cfg(feature = "jit")]
                        if offset.is_negative() {
                            next_trace = luaV_record_loop((*cl).p, pc, &mut trace_recorder);
                        }
                        trap = (*ci).u.l.trap;
                        continue;
                    }
                    OP_EQ => {
                        let mut ra_54: StkId = base.offset(
                            (i >> 0 + 7 as i32 & !(!(0 as Instruction) << 8 as i32) << 0) as i32
                                as isize,
                        );
                        let mut cond_0: i32 = 0;
                        let mut rb_14: *mut TValue = &mut (*base.offset(
                            (i >> 0 + 7 as i32 + 8 as i32 + 1 as i32
                                & !(!(0 as Instruction) << 8 as i32) << 0)
                                as i32 as isize,
                        ))
                        .val;
                        (*ci).u.l.savedpc = pc;
                        (*L).top.p = (*ci).top.p;
                        cond_0 = luaV_equalobj(L, &mut (*ra_54).val, rb_14);
                        trap = (*ci).u.l.trap;
                        if cond_0
                            != (i >> 0 + 7 as i32 + 8 as i32
                                & !(!(0 as Instruction) << 1 as i32) << 0)
                                as i32
                        {
                            pc = pc.offset(1);
                        } else {
                            let mut ni: Instruction = *pc;
                            pc = pc.offset(
                                ((ni >> 0 + 7 as i32
                                    & !(!(0 as Instruction)
                                        << 8 as i32 + 8 as i32 + 1 as i32 + 8 as i32)
                                        << 0) as i32
                                    - (((1 as i32) << 8 as i32 + 8 as i32 + 1 as i32 + 8 as i32)
                                        - 1 as i32
                                        >> 1 as i32)
                                    + 1 as i32) as isize,
                            );
                            trap = (*ci).u.l.trap;
                        }
                        continue;
                    }
                    OP_LT => {
                        let mut ra_55: StkId = base.offset(
                            (i >> 0 + 7 as i32 & !(!(0 as Instruction) << 8 as i32) << 0) as i32
                                as isize,
                        );
                        let mut cond_1: i32 = 0;
                        let mut rb_15: *mut TValue = &mut (*base.offset(
                            (i >> 0 + 7 as i32 + 8 as i32 + 1 as i32
                                & !(!(0 as Instruction) << 8 as i32) << 0)
                                as i32 as isize,
                        ))
                        .val;
                        if (*ra_55).val.tt_ as i32 == 3 as i32 | (0) << 4 as i32
                            && (*rb_15).tt_ as i32 == 3 as i32 | (0) << 4 as i32
                        {
                            let mut ia: lua_Integer = (*ra_55).val.value_.i;
                            let mut ib_3: lua_Integer = (*rb_15).value_.i;
                            cond_1 = (ia < ib_3) as i32;
                        } else if (*ra_55).val.tt_ as i32 & 0xf as i32 == 3 as i32
                            && (*rb_15).tt_ as i32 & 0xf as i32 == 3 as i32
                        {
                            cond_1 = LTnum(&mut (*ra_55).val, rb_15);
                        } else {
                            (*ci).u.l.savedpc = pc;
                            (*L).top.p = (*ci).top.p;
                            cond_1 = lessthanothers(L, &mut (*ra_55).val, rb_15);
                            trap = (*ci).u.l.trap;
                        }
                        if cond_1
                            != (i >> 0 + 7 as i32 + 8 as i32
                                & !(!(0 as Instruction) << 1 as i32) << 0)
                                as i32
                        {
                            pc = pc.offset(1);
                        } else {
                            let mut ni_0: Instruction = *pc;
                            pc = pc.offset(
                                ((ni_0 >> 0 + 7 as i32
                                    & !(!(0 as Instruction)
                                        << 8 as i32 + 8 as i32 + 1 as i32 + 8 as i32)
                                        << 0) as i32
                                    - (((1 as i32) << 8 as i32 + 8 as i32 + 1 as i32 + 8 as i32)
                                        - 1 as i32
                                        >> 1 as i32)
                                    + 1 as i32) as isize,
                            );
                            trap = (*ci).u.l.trap;
                        }
                        continue;
                    }
                    OP_LE => {
                        let mut ra_56: StkId = base.offset(
                            (i >> 0 + 7 as i32 & !(!(0 as Instruction) << 8 as i32) << 0) as i32
                                as isize,
                        );
                        let mut cond_2: i32 = 0;
                        let mut rb_16: *mut TValue = &mut (*base.offset(
                            (i >> 0 + 7 as i32 + 8 as i32 + 1 as i32
                                & !(!(0 as Instruction) << 8 as i32) << 0)
                                as i32 as isize,
                        ))
                        .val;
                        if (*ra_56).val.tt_ as i32 == 3 as i32 | (0) << 4 as i32
                            && (*rb_16).tt_ as i32 == 3 as i32 | (0) << 4 as i32
                        {
                            let mut ia_0: lua_Integer = (*ra_56).val.value_.i;
                            let mut ib_4: lua_Integer = (*rb_16).value_.i;
                            cond_2 = (ia_0 <= ib_4) as i32;
                        } else if (*ra_56).val.tt_ as i32 & 0xf as i32 == 3 as i32
                            && (*rb_16).tt_ as i32 & 0xf as i32 == 3 as i32
                        {
                            cond_2 = LEnum(&mut (*ra_56).val, rb_16);
                        } else {
                            (*ci).u.l.savedpc = pc;
                            (*L).top.p = (*ci).top.p;
                            cond_2 = lessequalothers(L, &mut (*ra_56).val, rb_16);
                            trap = (*ci).u.l.trap;
                        }
                        if cond_2
                            != (i >> 0 + 7 as i32 + 8 as i32
                                & !(!(0 as Instruction) << 1 as i32) << 0)
                                as i32
                        {
                            pc = pc.offset(1);
                        } else {
                            let mut ni_1: Instruction = *pc;
                            pc = pc.offset(
                                ((ni_1 >> 0 + 7 as i32
                                    & !(!(0 as Instruction)
                                        << 8 as i32 + 8 as i32 + 1 as i32 + 8 as i32)
                                        << 0) as i32
                                    - (((1 as i32) << 8 as i32 + 8 as i32 + 1 as i32 + 8 as i32)
                                        - 1 as i32
                                        >> 1 as i32)
                                    + 1 as i32) as isize,
                            );
                            trap = (*ci).u.l.trap;
                        }
                        continue;
                    }
                    OP_EQK => {
                        let mut ra_57: StkId = base.offset(
                            (i >> 0 + 7 as i32 & !(!(0 as Instruction) << 8 as i32) << 0) as i32
                                as isize,
                        );
                        let mut rb_17: *mut TValue = k.offset(
                            (i >> 0 + 7 as i32 + 8 as i32 + 1 as i32
                                & !(!(0 as Instruction) << 8 as i32) << 0)
                                as i32 as isize,
                        );
                        let mut cond_3: i32 =
                            luaV_equalobj(0 as *mut lua_State, &mut (*ra_57).val, rb_17);
                        if cond_3
                            != (i >> 0 + 7 as i32 + 8 as i32
                                & !(!(0 as Instruction) << 1 as i32) << 0)
                                as i32
                        {
                            pc = pc.offset(1);
                        } else {
                            let mut ni_2: Instruction = *pc;
                            pc = pc.offset(
                                ((ni_2 >> 0 + 7 as i32
                                    & !(!(0 as Instruction)
                                        << 8 as i32 + 8 as i32 + 1 as i32 + 8 as i32)
                                        << 0) as i32
                                    - (((1 as i32) << 8 as i32 + 8 as i32 + 1 as i32 + 8 as i32)
                                        - 1 as i32
                                        >> 1 as i32)
                                    + 1 as i32) as isize,
                            );
                            trap = (*ci).u.l.trap;
                        }
                        continue;
                    }
                    OP_EQI => {
                        let mut ra_58: StkId = base.offset(
                            (i >> 0 + 7 as i32 & !(!(0 as Instruction) << 8 as i32) << 0) as i32
                                as isize,
                        );
                        let mut cond_4: i32 = 0;
                        let mut im: i32 = (i >> 0 + 7 as i32 + 8 as i32 + 1 as i32
                            & !(!(0 as Instruction) << 8 as i32) << 0)
                            as i32
                            - (((1 as i32) << 8 as i32) - 1 as i32 >> 1 as i32);
                        if (*ra_58).val.tt_ as i32 == 3 as i32 | (0) << 4 as i32 {
                            cond_4 = ((*ra_58).val.value_.i == im as lua_Integer) as i32;
                        } else if (*ra_58).val.tt_ as i32 == 3 as i32 | (1 as i32) << 4 as i32 {
                            cond_4 = ((*ra_58).val.value_.n == im as lua_Number) as i32;
                        } else {
                            cond_4 = 0;
                        }
                        if cond_4
                            != (i >> 0 + 7 as i32 + 8 as i32
                                & !(!(0 as Instruction) << 1 as i32) << 0)
                                as i32
                        {
                            pc = pc.offset(1);
                        } else {
                            let mut ni_3: Instruction = *pc;
                            pc = pc.offset(
                                ((ni_3 >> 0 + 7 as i32
                                    & !(!(0 as Instruction)
                                        << 8 as i32 + 8 as i32 + 1 as i32 + 8 as i32)
                                        << 0) as i32
                                    - (((1 as i32) << 8 as i32 + 8 as i32 + 1 as i32 + 8 as i32)
                                        - 1 as i32
                                        >> 1 as i32)
                                    + 1 as i32) as isize,
                            );
                            trap = (*ci).u.l.trap;
                        }
                        continue;
                    }
                    OP_LTI => {
                        let mut ra_59: StkId = base.offset(
                            (i >> 0 + 7 as i32 & !(!(0 as Instruction) << 8 as i32) << 0) as i32
                                as isize,
                        );
                        let mut cond_5: i32 = 0;
                        let mut im_0: i32 = (i >> 0 + 7 as i32 + 8 as i32 + 1 as i32
                            & !(!(0 as Instruction) << 8 as i32) << 0)
                            as i32
                            - (((1 as i32) << 8 as i32) - 1 as i32 >> 1 as i32);
                        if (*ra_59).val.tt_ as i32 == 3 as i32 | (0) << 4 as i32 {
                            cond_5 = ((*ra_59).val.value_.i < im_0 as lua_Integer) as i32;
                        } else if (*ra_59).val.tt_ as i32 == 3 as i32 | (1 as i32) << 4 as i32 {
                            let mut fa: lua_Number = (*ra_59).val.value_.n;
                            let mut fim: lua_Number = im_0 as lua_Number;
                            cond_5 = (fa < fim) as i32;
                        } else {
                            let mut isf: i32 = (i >> 0 + 7 as i32 + 8 as i32 + 1 as i32 + 8 as i32
                                & !(!(0 as Instruction) << 8 as i32) << 0)
                                as i32;
                            (*ci).u.l.savedpc = pc;
                            (*L).top.p = (*ci).top.p;
                            cond_5 = luaT_callorderiTM(L, &mut (*ra_59).val, im_0, 0, isf, TM_LT);
                            trap = (*ci).u.l.trap;
                        }
                        if cond_5
                            != (i >> 0 + 7 as i32 + 8 as i32
                                & !(!(0 as Instruction) << 1 as i32) << 0)
                                as i32
                        {
                            pc = pc.offset(1);
                        } else {
                            let mut ni_4: Instruction = *pc;
                            pc = pc.offset(
                                ((ni_4 >> 0 + 7 as i32
                                    & !(!(0 as Instruction)
                                        << 8 as i32 + 8 as i32 + 1 as i32 + 8 as i32)
                                        << 0) as i32
                                    - (((1 as i32) << 8 as i32 + 8 as i32 + 1 as i32 + 8 as i32)
                                        - 1 as i32
                                        >> 1 as i32)
                                    + 1 as i32) as isize,
                            );
                            trap = (*ci).u.l.trap;
                        }
                        continue;
                    }
                    OP_LEI => {
                        let mut ra_60: StkId = base.offset(
                            (i >> 0 + 7 as i32 & !(!(0 as Instruction) << 8 as i32) << 0) as i32
                                as isize,
                        );
                        let mut cond_6: i32 = 0;
                        let mut im_1: i32 = (i >> 0 + 7 as i32 + 8 as i32 + 1 as i32
                            & !(!(0 as Instruction) << 8 as i32) << 0)
                            as i32
                            - (((1 as i32) << 8 as i32) - 1 as i32 >> 1 as i32);
                        if (*ra_60).val.tt_ as i32 == 3 as i32 | (0) << 4 as i32 {
                            cond_6 = ((*ra_60).val.value_.i <= im_1 as lua_Integer) as i32;
                        } else if (*ra_60).val.tt_ as i32 == 3 as i32 | (1 as i32) << 4 as i32 {
                            let mut fa_0: lua_Number = (*ra_60).val.value_.n;
                            let mut fim_0: lua_Number = im_1 as lua_Number;
                            cond_6 = (fa_0 <= fim_0) as i32;
                        } else {
                            let mut isf_0: i32 = (i
                                >> 0 + 7 as i32 + 8 as i32 + 1 as i32 + 8 as i32
                                & !(!(0 as Instruction) << 8 as i32) << 0)
                                as i32;
                            (*ci).u.l.savedpc = pc;
                            (*L).top.p = (*ci).top.p;
                            cond_6 = luaT_callorderiTM(L, &mut (*ra_60).val, im_1, 0, isf_0, TM_LE);
                            trap = (*ci).u.l.trap;
                        }
                        if cond_6
                            != (i >> 0 + 7 as i32 + 8 as i32
                                & !(!(0 as Instruction) << 1 as i32) << 0)
                                as i32
                        {
                            pc = pc.offset(1);
                        } else {
                            let mut ni_5: Instruction = *pc;
                            pc = pc.offset(
                                ((ni_5 >> 0 + 7 as i32
                                    & !(!(0 as Instruction)
                                        << 8 as i32 + 8 as i32 + 1 as i32 + 8 as i32)
                                        << 0) as i32
                                    - (((1 as i32) << 8 as i32 + 8 as i32 + 1 as i32 + 8 as i32)
                                        - 1 as i32
                                        >> 1 as i32)
                                    + 1 as i32) as isize,
                            );
                            trap = (*ci).u.l.trap;
                        }
                        continue;
                    }
                    OP_GTI => {
                        let mut ra_61: StkId = base.offset(
                            (i >> 0 + 7 as i32 & !(!(0 as Instruction) << 8 as i32) << 0) as i32
                                as isize,
                        );
                        let mut cond_7: i32 = 0;
                        let mut im_2: i32 = (i >> 0 + 7 as i32 + 8 as i32 + 1 as i32
                            & !(!(0 as Instruction) << 8 as i32) << 0)
                            as i32
                            - (((1 as i32) << 8 as i32) - 1 as i32 >> 1 as i32);
                        if (*ra_61).val.tt_ as i32 == 3 as i32 | (0) << 4 as i32 {
                            cond_7 = ((*ra_61).val.value_.i > im_2 as lua_Integer) as i32;
                        } else if (*ra_61).val.tt_ as i32 == 3 as i32 | (1 as i32) << 4 as i32 {
                            let mut fa_1: lua_Number = (*ra_61).val.value_.n;
                            let mut fim_1: lua_Number = im_2 as lua_Number;
                            cond_7 = (fa_1 > fim_1) as i32;
                        } else {
                            let mut isf_1: i32 = (i
                                >> 0 + 7 as i32 + 8 as i32 + 1 as i32 + 8 as i32
                                & !(!(0 as Instruction) << 8 as i32) << 0)
                                as i32;
                            (*ci).u.l.savedpc = pc;
                            (*L).top.p = (*ci).top.p;
                            cond_7 = luaT_callorderiTM(
                                L,
                                &mut (*ra_61).val,
                                im_2,
                                1 as i32,
                                isf_1,
                                TM_LT,
                            );
                            trap = (*ci).u.l.trap;
                        }
                        if cond_7
                            != (i >> 0 + 7 as i32 + 8 as i32
                                & !(!(0 as Instruction) << 1 as i32) << 0)
                                as i32
                        {
                            pc = pc.offset(1);
                        } else {
                            let mut ni_6: Instruction = *pc;
                            pc = pc.offset(
                                ((ni_6 >> 0 + 7 as i32
                                    & !(!(0 as Instruction)
                                        << 8 as i32 + 8 as i32 + 1 as i32 + 8 as i32)
                                        << 0) as i32
                                    - (((1 as i32) << 8 as i32 + 8 as i32 + 1 as i32 + 8 as i32)
                                        - 1 as i32
                                        >> 1 as i32)
                                    + 1 as i32) as isize,
                            );
                            trap = (*ci).u.l.trap;
                        }
                        continue;
                    }
                    OP_GEI => {
                        let mut ra_62: StkId = base.offset(
                            (i >> 0 + 7 as i32 & !(!(0 as Instruction) << 8 as i32) << 0) as i32
                                as isize,
                        );
                        let mut cond_8: i32 = 0;
                        let mut im_3: i32 = (i >> 0 + 7 as i32 + 8 as i32 + 1 as i32
                            & !(!(0 as Instruction) << 8 as i32) << 0)
                            as i32
                            - (((1 as i32) << 8 as i32) - 1 as i32 >> 1 as i32);
                        if (*ra_62).val.tt_ as i32 == 3 as i32 | (0) << 4 as i32 {
                            cond_8 = ((*ra_62).val.value_.i >= im_3 as lua_Integer) as i32;
                        } else if (*ra_62).val.tt_ as i32 == 3 as i32 | (1 as i32) << 4 as i32 {
                            let mut fa_2: lua_Number = (*ra_62).val.value_.n;
                            let mut fim_2: lua_Number = im_3 as lua_Number;
                            cond_8 = (fa_2 >= fim_2) as i32;
                        } else {
                            let mut isf_2: i32 = (i
                                >> 0 + 7 as i32 + 8 as i32 + 1 as i32 + 8 as i32
                                & !(!(0 as Instruction) << 8 as i32) << 0)
                                as i32;
                            (*ci).u.l.savedpc = pc;
                            (*L).top.p = (*ci).top.p;
                            cond_8 = luaT_callorderiTM(
                                L,
                                &mut (*ra_62).val,
                                im_3,
                                1 as i32,
                                isf_2,
                                TM_LE,
                            );
                            trap = (*ci).u.l.trap;
                        }
                        if cond_8
                            != (i >> 0 + 7 as i32 + 8 as i32
                                & !(!(0 as Instruction) << 1 as i32) << 0)
                                as i32
                        {
                            pc = pc.offset(1);
                        } else {
                            let mut ni_7: Instruction = *pc;
                            pc = pc.offset(
                                ((ni_7 >> 0 + 7 as i32
                                    & !(!(0 as Instruction)
                                        << 8 as i32 + 8 as i32 + 1 as i32 + 8 as i32)
                                        << 0) as i32
                                    - (((1 as i32) << 8 as i32 + 8 as i32 + 1 as i32 + 8 as i32)
                                        - 1 as i32
                                        >> 1 as i32)
                                    + 1 as i32) as isize,
                            );
                            trap = (*ci).u.l.trap;
                        }
                        continue;
                    }
                    OP_TEST => {
                        let mut ra_63: StkId = base.offset(
                            (i >> 0 + 7 as i32 & !(!(0 as Instruction) << 8 as i32) << 0) as i32
                                as isize,
                        );
                        let mut cond_9: i32 = !((*ra_63).val.tt_ as i32
                            == 1 as i32 | (0) << 4 as i32
                            || (*ra_63).val.tt_ as i32 & 0xf as i32 == 0)
                            as i32;
                        if cond_9
                            != (i >> 0 + 7 as i32 + 8 as i32
                                & !(!(0 as Instruction) << 1 as i32) << 0)
                                as i32
                        {
                            pc = pc.offset(1);
                        } else {
                            let mut ni_8: Instruction = *pc;
                            pc = pc.offset(
                                ((ni_8 >> 0 + 7 as i32
                                    & !(!(0 as Instruction)
                                        << 8 as i32 + 8 as i32 + 1 as i32 + 8 as i32)
                                        << 0) as i32
                                    - (((1 as i32) << 8 as i32 + 8 as i32 + 1 as i32 + 8 as i32)
                                        - 1 as i32
                                        >> 1 as i32)
                                    + 1 as i32) as isize,
                            );
                            trap = (*ci).u.l.trap;
                        }
                        continue;
                    }
                    OP_TESTSET => {
                        let mut ra_64: StkId = base.offset(
                            (i >> 0 + 7 as i32 & !(!(0 as Instruction) << 8 as i32) << 0) as i32
                                as isize,
                        );
                        let mut rb_18: *mut TValue = &mut (*base.offset(
                            (i >> 0 + 7 as i32 + 8 as i32 + 1 as i32
                                & !(!(0 as Instruction) << 8 as i32) << 0)
                                as i32 as isize,
                        ))
                        .val;
                        if ((*rb_18).tt_ as i32 == 1 as i32 | (0) << 4 as i32
                            || (*rb_18).tt_ as i32 & 0xf as i32 == 0)
                            as i32
                            == (i >> 0 + 7 as i32 + 8 as i32
                                & !(!(0 as Instruction) << 1 as i32) << 0)
                                as i32
                        {
                            pc = pc.offset(1);
                        } else {
                            let mut io1_14: *mut TValue = &mut (*ra_64).val;
                            let mut io2_14: *const TValue = rb_18;
                            (*io1_14).value_ = (*io2_14).value_;
                            (*io1_14).tt_ = (*io2_14).tt_;
                            if (*io1_14).tt_ as i32 & (1 as i32) << 6 as i32 == 0
                                || (*io1_14).tt_ as i32 & 0x3f as i32
                                    == (*(*io1_14).value_.gc).tt as i32
                                    && (L.is_null()
                                        || (*(*io1_14).value_.gc).marked as i32
                                            & ((*(*L).l_G).currentwhite as i32
                                                ^ ((1 as i32) << 3 as i32
                                                    | (1 as i32) << 4 as i32))
                                            == 0)
                            {
                            } else {
                            };
                            let mut ni_9: Instruction = *pc;
                            pc = pc.offset(
                                ((ni_9 >> 0 + 7 as i32
                                    & !(!(0 as Instruction)
                                        << 8 as i32 + 8 as i32 + 1 as i32 + 8 as i32)
                                        << 0) as i32
                                    - (((1 as i32) << 8 as i32 + 8 as i32 + 1 as i32 + 8 as i32)
                                        - 1 as i32
                                        >> 1 as i32)
                                    + 1 as i32) as isize,
                            );
                            trap = (*ci).u.l.trap;
                        }
                        continue;
                    }
                    OP_CALL => {
                        ra_65 = base.offset(
                            (i >> 0 + 7 as i32 & !(!(0 as Instruction) << 8 as i32) << 0) as i32
                                as isize,
                        );
                        newci = 0 as *mut CallInfo;
                        b_4 = (i >> 0 + 7 as i32 + 8 as i32 + 1 as i32
                            & !(!(0 as Instruction) << 8 as i32) << 0)
                            as i32;
                        nresults = (i >> 0 + 7 as i32 + 8 as i32 + 1 as i32 + 8 as i32
                            & !(!(0 as Instruction) << 8 as i32) << 0)
                            as i32
                            - 1 as i32;
                        if b_4 != 0 {
                            (*L).top.p = ra_65.offset(b_4 as isize);
                        }
                        (*ci).u.l.savedpc = pc;
                        newci = luaD_precall(L, ra_65, nresults);
                        if !newci.is_null() {
                            break '_returning;
                        }
                        trap = (*ci).u.l.trap;
                        continue;
                    }
                    OP_TAILCALL => {
                        let mut ra_66: StkId = base.offset(
                            (i >> 0 + 7 as i32 & !(!(0 as Instruction) << 8 as i32) << 0) as i32
                                as isize,
                        );
                        let mut b_5: i32 = (i >> 0 + 7 as i32 + 8 as i32 + 1 as i32
                            & !(!(0 as Instruction) << 8 as i32) << 0)
                            as i32;
                        let mut n_2: i32 = 0;
                        let mut nparams1: i32 = (i >> 0 + 7 as i32 + 8 as i32 + 1 as i32 + 8 as i32
                            & !(!(0 as Instruction) << 8 as i32) << 0)
                            as i32;
                        let mut delta: i32 = if nparams1 != 0 {
                            (*ci).u.l.nextraargs + nparams1
                        } else {
                            0
                        };
                        if b_5 != 0 {
                            (*L).top.p = ra_66.offset(b_5 as isize);
                        } else {
                            b_5 = ((*L).top.p).offset_from(ra_66) as std::ffi::c_long as i32;
                        }
                        (*ci).u.l.savedpc = pc;
                        if (i & (1 as u32) << 0 + 7 as i32 + 8 as i32) as i32 != 0 {
                            luaF_closeupval(L, base);
                        }
                        n_2 = luaD_pretailcall(L, ci, ra_66, b_5, delta);
                        if n_2 < 0 {
                            continue '_startfunc;
                        }
                        (*ci).func.p = ((*ci).func.p).offset(-(delta as isize));
                        luaD_poscall(L, ci, n_2);
                        trap = (*ci).u.l.trap;
                        break;
                    }
                    OP_RETURN => {
                        let mut ra_67: StkId = base.offset(
                            (i >> 0 + 7 as i32 & !(!(0 as Instruction) << 8 as i32) << 0) as i32
                                as isize,
                        );
                        let mut n_3: i32 = (i >> 0 + 7 as i32 + 8 as i32 + 1 as i32
                            & !(!(0 as Instruction) << 8 as i32) << 0)
                            as i32
                            - 1 as i32;
                        let mut nparams1_0: i32 = (i
                            >> 0 + 7 as i32 + 8 as i32 + 1 as i32 + 8 as i32
                            & !(!(0 as Instruction) << 8 as i32) << 0)
                            as i32;
                        if n_3 < 0 {
                            n_3 = ((*L).top.p).offset_from(ra_67) as std::ffi::c_long as i32;
                        }
                        (*ci).u.l.savedpc = pc;
                        if (i & (1 as u32) << 0 + 7 as i32 + 8 as i32) as i32 != 0 {
                            (*ci).u2.nres = n_3;
                            if (*L).top.p < (*ci).top.p {
                                (*L).top.p = (*ci).top.p;
                            }
                            luaF_close(L, base, -(1 as i32), 1 as i32);
                            trap = (*ci).u.l.trap;
                            if (trap != 0) as i32 as std::ffi::c_long != 0 {
                                base = ((*ci).func.p).offset(1);
                                ra_67 = base.offset(
                                    (i >> 0 + 7 as i32 & !(!(0 as Instruction) << 8 as i32) << 0)
                                        as i32 as isize,
                                );
                            }
                        }
                        if nparams1_0 != 0 {
                            (*ci).func.p = ((*ci).func.p)
                                .offset(-(((*ci).u.l.nextraargs + nparams1_0) as isize));
                        }
                        (*L).top.p = ra_67.offset(n_3 as isize);
                        luaD_poscall(L, ci, n_3);
                        trap = (*ci).u.l.trap;
                        break;
                    }
                    OP_RETURN0 => {
                        if ((*L).hookmask != 0) as i32 as std::ffi::c_long != 0 {
                            let mut ra_68: StkId = base.offset(
                                (i >> 0 + 7 as i32 & !(!(0 as Instruction) << 8 as i32) << 0) as i32
                                    as isize,
                            );
                            (*L).top.p = ra_68;
                            (*ci).u.l.savedpc = pc;
                            luaD_poscall(L, ci, 0);
                            trap = 1 as i32;
                        } else {
                            let mut nres: i32 = 0;
                            (*L).ci = (*ci).previous;
                            (*L).top.p = base.offset(-(1));
                            nres = (*ci).nresults as i32;
                            while ((nres > 0) as i32 != 0) as i32 as std::ffi::c_long != 0 {
                                let fresh137 = (*L).top.p;
                                (*L).top.p = ((*L).top.p).offset(1);
                                (*fresh137).val.tt_ = (0 | (0) << 4 as i32) as lu_byte;
                                nres -= 1;
                            }
                        }
                        break;
                    }
                    OP_RETURN1 => {
                        if ((*L).hookmask != 0) as i32 as std::ffi::c_long != 0 {
                            let mut ra_69: StkId = base.offset(
                                (i >> 0 + 7 as i32 & !(!(0 as Instruction) << 8 as i32) << 0) as i32
                                    as isize,
                            );
                            (*L).top.p = ra_69.offset(1);
                            (*ci).u.l.savedpc = pc;
                            luaD_poscall(L, ci, 1 as i32);
                            trap = 1 as i32;
                        } else {
                            let mut nres_0: i32 = (*ci).nresults as i32;
                            (*L).ci = (*ci).previous;
                            if nres_0 == 0 {
                                (*L).top.p = base.offset(-(1));
                            } else {
                                let mut ra_70: StkId = base.offset(
                                    (i >> 0 + 7 as i32 & !(!(0 as Instruction) << 8 as i32) << 0)
                                        as i32 as isize,
                                );
                                let mut io1_15: *mut TValue = &mut (*base.offset(-(1))).val;
                                let mut io2_15: *const TValue = &mut (*ra_70).val;
                                (*io1_15).value_ = (*io2_15).value_;
                                (*io1_15).tt_ = (*io2_15).tt_;
                                if (*io1_15).tt_ as i32 & (1 as i32) << 6 as i32 == 0
                                    || (*io1_15).tt_ as i32 & 0x3f as i32
                                        == (*(*io1_15).value_.gc).tt as i32
                                        && (L.is_null()
                                            || (*(*io1_15).value_.gc).marked as i32
                                                & ((*(*L).l_G).currentwhite as i32
                                                    ^ ((1 as i32) << 3 as i32
                                                        | (1 as i32) << 4 as i32))
                                                == 0)
                                {
                                } else {
                                };
                                (*L).top.p = base;
                                while ((nres_0 > 1 as i32) as i32 != 0) as i32 as std::ffi::c_long
                                    != 0
                                {
                                    let fresh138 = (*L).top.p;
                                    (*L).top.p = ((*L).top.p).offset(1);
                                    (*fresh138).val.tt_ = (0 | (0) << 4 as i32) as lu_byte;
                                    nres_0 -= 1;
                                }
                            }
                        }
                        break;
                    }
                    OP_FORLOOP => {
                        let mut ra_71: StkId = base.offset(
                            (i >> 0 + 7 as i32 & !(!(0 as Instruction) << 8 as i32) << 0) as i32
                                as isize,
                        );
                        if (*ra_71.offset(2)).val.tt_ as i32 == 3 as i32 | (0) << 4 as i32 {
                            let mut count: lua_Unsigned =
                                (*ra_71.offset(1)).val.value_.i as lua_Unsigned;
                            if count > 0 as lua_Unsigned {
                                let mut step: lua_Integer = (*ra_71.offset(2)).val.value_.i;
                                let mut idx: lua_Integer = (*ra_71).val.value_.i;
                                let mut io_43: *mut TValue = &mut (*ra_71.offset(1)).val;
                                (*io_43).value_.i =
                                    count.wrapping_sub(1 as i32 as lua_Unsigned) as lua_Integer;
                                idx = (idx as lua_Unsigned).wrapping_add(step as lua_Unsigned)
                                    as lua_Integer;
                                let mut io_44: *mut TValue = &mut (*ra_71).val;
                                (*io_44).value_.i = idx;
                                let mut io_45: *mut TValue = &mut (*ra_71.offset(3)).val;
                                (*io_45).value_.i = idx;
                                (*io_45).tt_ = (3 as i32 | (0) << 4 as i32) as lu_byte;
                                pc = pc.offset(
                                    -((i >> 0 + 7 as i32 + 8 as i32
                                        & !(!(0 as Instruction) << 8 as i32 + 8 as i32 + 1 as i32)
                                            << 0) as i32
                                        as isize),
                                );
                            }
                        } else if floatforloop(ra_71) != 0 {
                            pc = pc.offset(
                                -((i >> 0 + 7 as i32 + 8 as i32
                                    & !(!(0 as Instruction) << 8 as i32 + 8 as i32 + 1 as i32) << 0)
                                    as i32 as isize),
                            );
                        }
                        #[cfg(feature = "jit")]
                        {
                            next_trace = luaV_record_loop((*cl).p, pc, &mut trace_recorder)
                        };
                        trap = (*ci).u.l.trap;
                        continue;
                    }
                    OP_FORPREP => {
                        let mut ra_72: StkId = base.offset(
                            (i >> 0 + 7 as i32 & !(!(0 as Instruction) << 8 as i32) << 0) as i32
                                as isize,
                        );
                        (*ci).u.l.savedpc = pc;
                        (*L).top.p = (*ci).top.p;
                        if forprep(L, ra_72) != 0 {
                            pc = pc.offset(
                                ((i >> 0 + 7 as i32 + 8 as i32
                                    & !(!(0 as Instruction) << 8 as i32 + 8 as i32 + 1 as i32) << 0)
                                    as i32
                                    + 1 as i32) as isize,
                            );
                        }
                        #[cfg(feature = "jit")]
                        {
                            next_trace = luaV_record_loop((*cl).p, pc, &mut trace_recorder)
                        };
                        continue;
                    }
                    OP_TFORPREP => {
                        let mut ra_73: StkId = base.offset(
                            (i >> 0 + 7 as i32 & !(!(0 as Instruction) << 8 as i32) << 0) as i32
                                as isize,
                        );
                        (*ci).u.l.savedpc = pc;
                        (*L).top.p = (*ci).top.p;
                        luaF_newtbcupval(L, ra_73.offset(3));
                        pc = pc.offset(
                            (i >> 0 + 7 as i32 + 8 as i32
                                & !(!(0 as Instruction) << 8 as i32 + 8 as i32 + 1 as i32) << 0)
                                as i32 as isize,
                        );
                        let fresh139 = pc;
                        pc = pc.offset(1);
                        i = *fresh139;
                        current_block = 13973394567113199817;
                    }
                    OP_TFORCALL => {
                        current_block = 13973394567113199817;
                    }
                    OP_TFORLOOP => {
                        current_block = 15611964311717037170;
                    }
                    OP_SETLIST => {
                        let mut ra_76: StkId = base.offset(
                            (i >> 0 + 7 as i32 & !(!(0 as Instruction) << 8 as i32) << 0) as i32
                                as isize,
                        );
                        let mut n_4: i32 = (i >> 0 + 7 as i32 + 8 as i32 + 1 as i32
                            & !(!(0 as Instruction) << 8 as i32) << 0)
                            as i32;
                        let mut last: u32 = (i >> 0 + 7 as i32 + 8 as i32 + 1 as i32 + 8 as i32
                            & !(!(0 as Instruction) << 8 as i32) << 0)
                            as i32 as u32;
                        let mut h: *mut Table = &mut (*((*ra_76).val.value_.gc as *mut GCUnion)).h;
                        if n_4 == 0 {
                            n_4 = ((*L).top.p).offset_from(ra_76) as std::ffi::c_long as i32
                                - 1 as i32;
                        } else {
                            (*L).top.p = (*ci).top.p;
                        }
                        last = last.wrapping_add(n_4 as u32);
                        if (i & (1 as u32) << 0 + 7 as i32 + 8 as i32) as i32 != 0 {
                            last = last.wrapping_add(
                                ((*pc >> 0 + 7 as i32
                                    & !(!(0 as Instruction)
                                        << 8 as i32 + 8 as i32 + 1 as i32 + 8 as i32)
                                        << 0) as i32
                                    * (((1 as i32) << 8 as i32) - 1 as i32 + 1 as i32))
                                    as u32,
                            );
                            pc = pc.offset(1);
                            pc;
                        }
                        if last > luaH_realasize(h) {
                            luaH_resizearray(L, h, last);
                        }
                        while n_4 > 0 {
                            let mut val: *mut TValue = &mut (*ra_76.offset(n_4 as isize)).val;
                            let mut io1_17: *mut TValue = &mut *((*h).array)
                                .offset(last.wrapping_sub(1) as isize)
                                as *mut TValue;
                            let mut io2_17: *const TValue = val;
                            (*io1_17).value_ = (*io2_17).value_;
                            (*io1_17).tt_ = (*io2_17).tt_;
                            if (*io1_17).tt_ as i32 & (1 as i32) << 6 as i32 == 0
                                || (*io1_17).tt_ as i32 & 0x3f as i32
                                    == (*(*io1_17).value_.gc).tt as i32
                                    && (L.is_null()
                                        || (*(*io1_17).value_.gc).marked as i32
                                            & ((*(*L).l_G).currentwhite as i32
                                                ^ ((1 as i32) << 3 as i32
                                                    | (1 as i32) << 4 as i32))
                                            == 0)
                            {
                            } else {
                            };
                            last = last.wrapping_sub(1);
                            last;
                            if (*val).tt_ as i32 & (1 as i32) << 6 as i32 != 0 {
                                if (*(&mut (*(h as *mut GCUnion)).gc as *mut GCObject)).marked
                                    as i32
                                    & (1 as i32) << 5 as i32
                                    != 0
                                    && (*(*val).value_.gc).marked as i32
                                        & ((1 as i32) << 3 as i32 | (1 as i32) << 4 as i32)
                                        != 0
                                {
                                    luaC_barrierback_(L, &mut (*(h as *mut GCUnion)).gc);
                                } else {
                                };
                            } else {
                            };
                            n_4 -= 1;
                            n_4;
                        }
                        continue;
                    }
                    OP_CLOSURE => {
                        let mut ra_77: StkId = base.offset(
                            (i >> 0 + 7 as i32 & !(!(0 as Instruction) << 8 as i32) << 0) as i32
                                as isize,
                        );
                        let mut p: *mut Proto = *((*(*cl).p).p).offset(
                            (i >> 0 + 7 as i32 + 8 as i32
                                & !(!(0 as Instruction) << 8 as i32 + 8 as i32 + 1 as i32) << 0)
                                as i32 as isize,
                        );
                        (*ci).u.l.savedpc = pc;
                        (*L).top.p = (*ci).top.p;
                        pushclosure(L, p, ((*cl).upvals).as_mut_ptr(), base, ra_77);
                        if (*(*L).l_G).GCdebt > 0 as l_mem {
                            (*ci).u.l.savedpc = pc;
                            (*L).top.p = ra_77.offset(1);
                            luaC_step(L);
                            trap = (*ci).u.l.trap;
                        }
                        continue;
                    }
                    OP_VARARG => {
                        let mut ra_78: StkId = base.offset(
                            (i >> 0 + 7 as i32 & !(!(0 as Instruction) << 8 as i32) << 0) as i32
                                as isize,
                        );
                        let mut n_5: i32 = (i >> 0 + 7 as i32 + 8 as i32 + 1 as i32 + 8 as i32
                            & !(!(0 as Instruction) << 8 as i32) << 0)
                            as i32
                            - 1 as i32;
                        (*ci).u.l.savedpc = pc;
                        (*L).top.p = (*ci).top.p;
                        luaT_getvarargs(L, ci, ra_78, n_5);
                        trap = (*ci).u.l.trap;
                        continue;
                    }
                    OP_VARARGPREP => {
                        (*ci).u.l.savedpc = pc;
                        luaT_adjustvarargs(
                            L,
                            (i >> 0 + 7 as i32 & !(!(0 as Instruction) << 8 as i32) << 0) as i32,
                            ci,
                            (*cl).p,
                        );
                        trap = (*ci).u.l.trap;
                        if (trap != 0) as i32 as std::ffi::c_long != 0 {
                            luaD_hookcall(L, ci);
                            (*L).oldpc = 1 as i32;
                        }
                        base = ((*ci).func.p).offset(1);
                        continue;
                    }
                    OP_EXTRAARG | _ => {
                        continue;
                    }
                }
                match current_block {
                    13973394567113199817 => {
                        let mut ra_74: StkId = base.offset(
                            (i >> 0 + 7 as i32 & !(!(0 as Instruction) << 8 as i32) << 0) as i32
                                as isize,
                        );
                        memcpy(
                            ra_74.offset(4) as *mut c_void,
                            ra_74 as *const c_void,
                            (3usize).wrapping_mul(size_of::<StackValue>() as usize),
                        );
                        (*L).top.p = ra_74.offset(4).offset(3);
                        (*ci).u.l.savedpc = pc;
                        luaD_call(
                            L,
                            ra_74.offset(4),
                            (i >> 0 + 7 as i32 + 8 as i32 + 1 as i32 + 8 as i32
                                & !(!(0 as Instruction) << 8 as i32) << 0)
                                as i32,
                        );
                        trap = (*ci).u.l.trap;
                        if (trap != 0) as i32 as std::ffi::c_long != 0 {
                            base = ((*ci).func.p).offset(1);
                            ra_74 = base.offset(
                                (i >> 0 + 7 as i32 & !(!(0 as Instruction) << 8 as i32) << 0) as i32
                                    as isize,
                            );
                        }
                        let fresh140 = pc;
                        pc = pc.offset(1);
                        i = *fresh140;
                    }
                    _ => {}
                }
                let mut ra_75: StkId = base.offset(
                    (i >> 0 + 7 as i32 & !(!(0 as Instruction) << 8 as i32) << 0) as i32 as isize,
                );
                if !((*ra_75.offset(4)).val.tt_ as i32 & 0xf as i32 == 0) {
                    let mut io1_16: *mut TValue = &mut (*ra_75.offset(2)).val;
                    let mut io2_16: *const TValue = &mut (*ra_75.offset(4)).val;
                    (*io1_16).value_ = (*io2_16).value_;
                    (*io1_16).tt_ = (*io2_16).tt_;
                    if (*io1_16).tt_ as i32 & (1 as i32) << 6 as i32 == 0
                        || (*io1_16).tt_ as i32 & 0x3f as i32 == (*(*io1_16).value_.gc).tt as i32
                            && (L.is_null()
                                || (*(*io1_16).value_.gc).marked as i32
                                    & ((*(*L).l_G).currentwhite as i32
                                        ^ ((1 as i32) << 3 as i32 | (1 as i32) << 4 as i32))
                                    == 0)
                    {
                    } else {
                    };
                    pc = pc.offset(
                        -((i >> 0 + 7 as i32 + 8 as i32
                            & !(!(0 as Instruction) << 8 as i32 + 8 as i32 + 1 as i32) << 0)
                            as i32 as isize),
                    );
                    #[cfg(feature = "jit")]
                    {
                        next_trace = luaV_record_loop((*cl).p, pc, &mut trace_recorder);
                    }
                }
            }
            if (*ci).callstatus as i32 & (1 as i32) << 2 as i32 != 0 {
                break '_startfunc;
            }
            ci = (*ci).previous;
        }
        ci = newci;
    }
}
