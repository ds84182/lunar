use super::*;

pub(super) const LUA_TUPVAL: i8 = LUA_NUMTYPES;
pub(super) const LUA_TPROTO: i8 = LUA_NUMTYPES + 1;
pub(super) const LUA_TDEADKEY: i8 = LUA_NUMTYPES + 2;

pub(super) const LUA_TOTALTYPES: i8 = LUA_TPROTO + 2;

/*
** tags for Tagged Values have the following use of bits:
** bits 0-3: actual tag (a LUA_T* constant)
** bits 4-5: variant bits
** bit 6: whether value is collectable
*/

#[inline]
pub(super) const fn makevariant(t: i8, v: u8) -> u8 {
    t as u8 | (v << 4)
}

#[inline]
pub(super) const fn novariant(t: u8) -> u8 {
    t & 0x0F
}

#[inline]
pub(super) const fn withvariant(t: u8) -> u8 {
    t & 0x3F
}

/// Standard nil
pub(super) const LUA_VNIL: u8 = makevariant(LUA_TNIL, 0);

/// Empty slot (which might be different from a slot containing nil)
pub(super) const LUA_VEMPTY: u8 = makevariant(LUA_TNIL, 1);

/// Value returned for a key not found in a table (absent key)
pub(super) const LUA_VABSTKEY: u8 = makevariant(LUA_TNIL, 2);

pub(super) const LUA_VFALSE: u8 = makevariant(LUA_TBOOLEAN, 0);
pub(super) const LUA_VTRUE: u8 = makevariant(LUA_TBOOLEAN, 1);

pub(super) const LUA_VTHREAD: u8 = makevariant(LUA_TTHREAD, 0);

pub(super) const BIT_ISCOLLECTABLE: u8 = 1 << 6;

pub(super) const LUA_VNUMINT: u8 = makevariant(LUA_TNUMBER, 0);
pub(super) const LUA_VNUMFLT: u8 = makevariant(LUA_TNUMBER, 1);

pub(super) const LUA_VSHRSTR: u8 = makevariant(LUA_TSTRING, 0);
pub(super) const LUA_VLNGSTR: u8 = makevariant(LUA_TSTRING, 1);

pub(super) const LUA_VLIGHTUSERDATA: u8 = makevariant(LUA_TLIGHTUSERDATA, 0);
pub(super) const LUA_VUSERDATA: u8 = makevariant(LUA_TUSERDATA, 0);

pub(super) const LUA_VPROTO: u8 = makevariant(LUA_TPROTO, 0);

pub(super) const LUA_VUPVAL: u8 = makevariant(LUA_TUPVAL, 0);

/// Lua closure
pub(super) const LUA_VLCL: u8 = makevariant(LUA_TFUNCTION, 0);
/// Light C function
pub(super) const LUA_VLCF: u8 = makevariant(LUA_TFUNCTION, 1);
/// C closure
pub(super) const LUA_VCCL: u8 = makevariant(LUA_TFUNCTION, 2);

pub(super) const LUA_VTABLE: u8 = makevariant(LUA_TTABLE, 0);

// #[inline]
// pub(super) fn iscollectable()

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaO_ceillog2(mut x: u32) -> i32 {
    static mut log_2: [lu_byte; 256] = [
        0, 1, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 4, 4, 4, 4, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5,
        5, 5, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6,
        6, 6, 6, 6, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
        7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
        7, 7, 7, 7, 7, 7, 7, 7, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8,
        8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8,
        8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8,
        8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8,
        8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8,
    ];
    let mut l: i32 = 0;
    x = x.wrapping_sub(1);
    x;
    while x >= 256 {
        l += 8 as i32;
        x >>= 8 as i32;
    }
    return l + log_2[x as usize] as i32;
}
unsafe extern "C-unwind" fn intarith(
    mut L: *mut lua_State,
    mut op: i32,
    mut v1: lua_Integer,
    mut v2: lua_Integer,
) -> lua_Integer {
    match op {
        0 => return (v1 as lua_Unsigned).wrapping_add(v2 as lua_Unsigned) as lua_Integer,
        1 => return (v1 as lua_Unsigned).wrapping_sub(v2 as lua_Unsigned) as lua_Integer,
        2 => return ((v1 as lua_Unsigned).wrapping_mul(v2 as lua_Unsigned)) as lua_Integer,
        3 => return luaV_mod(L, v1, v2),
        6 => return luaV_idiv(L, v1, v2),
        7 => return (v1 as lua_Unsigned & v2 as lua_Unsigned) as lua_Integer,
        8 => return (v1 as lua_Unsigned | v2 as lua_Unsigned) as lua_Integer,
        9 => return (v1 as lua_Unsigned ^ v2 as lua_Unsigned) as lua_Integer,
        10 => return luaV_shiftl(v1, v2),
        11 => {
            return luaV_shiftl(
                v1,
                (0 as lua_Unsigned).wrapping_sub(v2 as lua_Unsigned) as lua_Integer,
            );
        }
        12 => {
            return (0 as lua_Unsigned).wrapping_sub(v1 as lua_Unsigned) as lua_Integer;
        }
        13 => {
            return (!(0 as lua_Unsigned) ^ v1 as lua_Unsigned) as lua_Integer;
        }
        _ => return 0 as lua_Integer,
    };
}
unsafe extern "C-unwind" fn numarith(
    mut L: *mut lua_State,
    mut op: i32,
    mut v1: lua_Number,
    mut v2: lua_Number,
) -> lua_Number {
    match op {
        0 => return v1 + v2,
        1 => return v1 - v2,
        2 => return v1 * v2,
        5 => return v1 / v2,
        4 => {
            return (if v2 == 2 as i32 as lua_Number {
                v1 * v1
            } else {
                v1.powf(v2)
            });
        }
        6 => return (v1 / v2).floor(),
        12 => return -v1,
        3 => return luaV_modf(L, v1, v2),
        _ => return 0 as lua_Number,
    };
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaO_rawarith(
    mut L: *mut lua_State,
    mut op: i32,
    mut p1: *const TValue,
    mut p2: *const TValue,
    mut res: *mut TValue,
) -> i32 {
    match op {
        7 | 8 | 9 | 10 | 11 | 13 => {
            let mut i1: lua_Integer = 0;
            let mut i2: lua_Integer = 0;
            if (if (((*p1).tt_ as i32 == 3 as i32 | (0) << 4 as i32) as i32 != 0) as i32
                as std::ffi::c_long
                != 0
            {
                i1 = (*p1).value_.i;
                1 as i32
            } else {
                luaV_tointegerns(p1, &mut i1, F2Ieq)
            }) != 0
                && (if (((*p2).tt_ as i32 == 3 as i32 | (0) << 4 as i32) as i32 != 0) as i32
                    as std::ffi::c_long
                    != 0
                {
                    i2 = (*p2).value_.i;
                    1 as i32
                } else {
                    luaV_tointegerns(p2, &mut i2, F2Ieq)
                }) != 0
            {
                let mut io: *mut TValue = res;
                (*io).value_.i = intarith(L, op, i1, i2);
                (*io).tt_ = (3 as i32 | (0) << 4 as i32) as lu_byte;
                return 1 as i32;
            } else {
                return 0;
            }
        }
        5 | 4 => {
            let mut n1: lua_Number = 0.;
            let mut n2: lua_Number = 0.;
            if (if (*p1).tt_ as i32 == 3 as i32 | (1 as i32) << 4 as i32 {
                n1 = (*p1).value_.n;
                1 as i32
            } else {
                (if (*p1).tt_ as i32 == 3 as i32 | (0) << 4 as i32 {
                    n1 = (*p1).value_.i as lua_Number;
                    1 as i32
                } else {
                    0
                })
            }) != 0
                && (if (*p2).tt_ as i32 == 3 as i32 | (1 as i32) << 4 as i32 {
                    n2 = (*p2).value_.n;
                    1 as i32
                } else {
                    (if (*p2).tt_ as i32 == 3 as i32 | (0) << 4 as i32 {
                        n2 = (*p2).value_.i as lua_Number;
                        1 as i32
                    } else {
                        0
                    })
                }) != 0
            {
                let mut io_0: *mut TValue = res;
                (*io_0).value_.n = numarith(L, op, n1, n2);
                (*io_0).tt_ = (3 as i32 | (1 as i32) << 4 as i32) as lu_byte;
                return 1 as i32;
            } else {
                return 0;
            }
        }
        _ => {
            let mut n1_0: lua_Number = 0.;
            let mut n2_0: lua_Number = 0.;
            if (*p1).tt_ as i32 == 3 as i32 | (0) << 4 as i32
                && (*p2).tt_ as i32 == 3 as i32 | (0) << 4 as i32
            {
                let mut io_1: *mut TValue = res;
                (*io_1).value_.i = intarith(L, op, (*p1).value_.i, (*p2).value_.i);
                (*io_1).tt_ = (3 as i32 | (0) << 4 as i32) as lu_byte;
                return 1 as i32;
            } else if (if (*p1).tt_ as i32 == 3 as i32 | (1 as i32) << 4 as i32 {
                n1_0 = (*p1).value_.n;
                1 as i32
            } else {
                (if (*p1).tt_ as i32 == 3 as i32 | (0) << 4 as i32 {
                    n1_0 = (*p1).value_.i as lua_Number;
                    1 as i32
                } else {
                    0
                })
            }) != 0
                && (if (*p2).tt_ as i32 == 3 as i32 | (1 as i32) << 4 as i32 {
                    n2_0 = (*p2).value_.n;
                    1 as i32
                } else {
                    (if (*p2).tt_ as i32 == 3 as i32 | (0) << 4 as i32 {
                        n2_0 = (*p2).value_.i as lua_Number;
                        1 as i32
                    } else {
                        0
                    })
                }) != 0
            {
                let mut io_2: *mut TValue = res;
                (*io_2).value_.n = numarith(L, op, n1_0, n2_0);
                (*io_2).tt_ = (3 as i32 | (1 as i32) << 4 as i32) as lu_byte;
                return 1 as i32;
            } else {
                return 0;
            }
        }
    };
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaO_arith(
    mut L: *mut lua_State,
    mut op: i32,
    mut p1: *const TValue,
    mut p2: *const TValue,
    mut res: StkId,
) {
    if luaO_rawarith(L, op, p1, p2, &mut (*res).val) == 0 {
        luaT_trybinTM(L, p1, p2, res, (op - 0 + TM_ADD as i32) as TMS);
    }
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaO_hexavalue(mut c: i32) -> i32 {
    if luai_ctype_[(c + 1 as i32) as usize] as i32 & (1 as i32) << 1 as i32 != 0 {
        return c - '0' as i32;
    } else {
        return (c | 'A' as i32 ^ 'a' as i32) - 'a' as i32 + 10;
    };
}
unsafe extern "C-unwind" fn isneg(mut s: *mut *const std::ffi::c_char) -> i32 {
    if **s as i32 == '-' as i32 {
        *s = (*s).offset(1);
        *s;
        return 1 as i32;
    } else if **s as i32 == '+' as i32 {
        *s = (*s).offset(1);
        *s;
    }
    return 0;
}
unsafe extern "C-unwind" fn l_str2dloc(
    mut s: *const std::ffi::c_char,
    mut result: *mut lua_Number,
    mut mode: i32,
) -> *const std::ffi::c_char {
    let mut endptr: *mut std::ffi::c_char = 0 as *mut std::ffi::c_char;
    *result = if mode == 'x' as i32 {
        strtod(s, &mut endptr)
    } else {
        strtod(s, &mut endptr)
    };
    if endptr == s as *mut std::ffi::c_char {
        return 0 as *const std::ffi::c_char;
    }
    while luai_ctype_[(*endptr as u8 as i32 + 1 as i32) as usize] as i32 & (1 as i32) << 3 as i32
        != 0
    {
        endptr = endptr.offset(1);
        endptr;
    }
    return if *endptr as i32 == '\0' as i32 {
        endptr
    } else {
        0 as *mut std::ffi::c_char
    };
}
unsafe extern "C-unwind" fn l_str2d(
    mut s: *const std::ffi::c_char,
    mut result: *mut lua_Number,
) -> *const std::ffi::c_char {
    let mut endptr: *const std::ffi::c_char = 0 as *const std::ffi::c_char;
    let mut pmode: *const std::ffi::c_char = strpbrk(s, c".xXnN".as_ptr());
    let mut mode: i32 = if !pmode.is_null() {
        *pmode as u8 as i32 | 'A' as i32 ^ 'a' as i32
    } else {
        0
    };
    if mode == 'n' as i32 {
        return 0 as *const std::ffi::c_char;
    }
    endptr = l_str2dloc(s, result, mode);
    if endptr.is_null() {
        let mut buff: [std::ffi::c_char; 201] = [0; 201];
        let mut pdot: *const std::ffi::c_char = strchr(s, '.' as i32);
        if pdot.is_null() || strlen(s) > 200 as usize {
            return 0 as *const std::ffi::c_char;
        }
        strcpy(buff.as_mut_ptr(), s);
        buff[pdot.offset_from(s) as std::ffi::c_long as usize] =
            *((*localeconv()).decimal_point).offset(0 as isize);
        endptr = l_str2dloc(buff.as_mut_ptr(), result, mode);
        if !endptr.is_null() {
            endptr = s.offset(endptr.offset_from(buff.as_mut_ptr()) as std::ffi::c_long as isize);
        }
    }
    return endptr;
}
unsafe extern "C-unwind" fn l_str2int(
    mut s: *const std::ffi::c_char,
    mut result: *mut lua_Integer,
) -> *const std::ffi::c_char {
    let mut a: lua_Unsigned = 0 as lua_Unsigned;
    let mut empty: i32 = 1 as i32;
    let mut neg: i32 = 0;
    while luai_ctype_[(*s as u8 as i32 + 1 as i32) as usize] as i32 & (1 as i32) << 3 as i32 != 0 {
        s = s.offset(1);
        s;
    }
    neg = isneg(&mut s);
    if *s.offset(0 as isize) as i32 == '0' as i32
        && (*s.offset(1) as i32 == 'x' as i32 || *s.offset(1) as i32 == 'X' as i32)
    {
        s = s.offset(2);
        while luai_ctype_[(*s as u8 as i32 + 1 as i32) as usize] as i32 & (1 as i32) << 4 as i32
            != 0
        {
            a = a
                .wrapping_mul(16)
                .wrapping_add(luaO_hexavalue(*s as i32) as lua_Unsigned);
            empty = 0;
            s = s.offset(1);
            s;
        }
    } else {
        while luai_ctype_[(*s as u8 as i32 + 1 as i32) as usize] as i32 & (1 as i32) << 1 as i32
            != 0
        {
            let mut d: i32 = *s as i32 - '0' as i32;
            if a >= (9223372036854775807 as std::ffi::c_longlong / 10 as std::ffi::c_longlong)
                as lua_Unsigned
                && (a
                    > (9223372036854775807 as std::ffi::c_longlong / 10 as std::ffi::c_longlong)
                        as lua_Unsigned
                    || d > (9223372036854775807 as std::ffi::c_longlong
                        % 10 as std::ffi::c_longlong) as i32
                        + neg)
            {
                return 0 as *const std::ffi::c_char;
            }
            a = (a * 10 as lua_Unsigned).wrapping_add(d as lua_Unsigned);
            empty = 0;
            s = s.offset(1);
            s;
        }
    }
    while luai_ctype_[(*s as u8 as i32 + 1 as i32) as usize] as i32 & (1 as i32) << 3 as i32 != 0 {
        s = s.offset(1);
        s;
    }
    if empty != 0 || *s as i32 != '\0' as i32 {
        return 0 as *const std::ffi::c_char;
    } else {
        *result = (if neg != 0 {
            (0 as u32 as lua_Unsigned).wrapping_sub(a)
        } else {
            a
        }) as lua_Integer;
        return s;
    };
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaO_str2num(
    mut s: *const std::ffi::c_char,
    mut o: *mut TValue,
) -> size_t {
    let mut i: lua_Integer = 0;
    let mut n: lua_Number = 0.;
    let mut e: *const std::ffi::c_char = 0 as *const std::ffi::c_char;
    e = l_str2int(s, &mut i);
    if !e.is_null() {
        let mut io: *mut TValue = o;
        (*io).value_.i = i;
        (*io).tt_ = (3 as i32 | (0) << 4 as i32) as lu_byte;
    } else {
        e = l_str2d(s, &mut n);
        if !e.is_null() {
            let mut io_0: *mut TValue = o;
            (*io_0).value_.n = n;
            (*io_0).tt_ = (3 as i32 | (1 as i32) << 4 as i32) as lu_byte;
        } else {
            return 0 as size_t;
        }
    }
    return (e.offset_from(s) + 1) as size_t;
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaO_utf8esc(mut buff: *mut std::ffi::c_char, mut x: usize) -> i32 {
    let mut n: i32 = 1 as i32;
    if x < 0x80 as usize {
        *buff.offset((8 as i32 - 1 as i32) as isize) = x as std::ffi::c_char;
    } else {
        let mut mfb: u32 = 0x3f as i32 as u32;
        loop {
            let fresh115 = n;
            n = n + 1;
            *buff.offset((8 as i32 - fresh115) as isize) =
                (0x80 as usize | x & 0x3f as i32 as usize) as std::ffi::c_char;
            x >>= 6 as i32;
            mfb >>= 1 as i32;
            if !(x > mfb as usize) {
                break;
            }
        }
        *buff.offset((8 as i32 - n) as isize) =
            ((!mfb << 1 as i32) as usize | x) as std::ffi::c_char;
    }
    return n;
}
unsafe extern "C-unwind" fn tostringbuff(
    mut obj: *mut TValue,
    mut buff: *mut std::ffi::c_char,
) -> i32 {
    let mut len: i32 = 0;
    if (*obj).tt_ as i32 == 3 as i32 | (0) << 4 as i32 {
        len = snprintf(buff, 44, c"%lld".as_ptr(), (*obj).value_.i);
    } else {
        len = snprintf(buff, 44, c"%.14g".as_ptr(), (*obj).value_.n);
        if *buff.offset(strspn(buff, c"-0123456789".as_ptr()) as isize) as i32 == '\0' as i32 {
            let fresh116 = len;
            len = len + 1;
            *buff.offset(fresh116 as isize) = *((*localeconv()).decimal_point).offset(0 as isize);
            let fresh117 = len;
            len = len + 1;
            *buff.offset(fresh117 as isize) = '0' as i32 as std::ffi::c_char;
        }
    }
    return len;
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaO_tostring(mut L: *mut lua_State, mut obj: *mut TValue) {
    let mut buff: [std::ffi::c_char; 44] = [0; 44];
    let mut len: i32 = tostringbuff(obj, buff.as_mut_ptr());
    let mut io: *mut TValue = obj;
    let mut x_: *mut TString = luaS_newlstr(L, buff.as_mut_ptr(), len as size_t);
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
unsafe extern "C-unwind" fn pushstr(
    mut buff: *mut BuffFS,
    mut str: *const std::ffi::c_char,
    mut lstr: size_t,
) {
    let mut L: *mut lua_State = (*buff).L;
    let mut io: *mut TValue = &mut (*(*L).top.p).val;
    let mut x_: *mut TString = luaS_newlstr(L, str, lstr);
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
    if (*buff).pushed == 0 {
        (*buff).pushed = 1 as i32;
    } else {
        luaV_concat(L, 2 as i32);
    };
}
unsafe extern "C-unwind" fn clearbuff(mut buff: *mut BuffFS) {
    pushstr(buff, ((*buff).space).as_mut_ptr(), (*buff).blen as size_t);
    (*buff).blen = 0;
}
unsafe extern "C-unwind" fn getbuff(mut buff: *mut BuffFS, mut sz: i32) -> *mut std::ffi::c_char {
    if sz > 60 + 44 as i32 + 95 as i32 - (*buff).blen {
        clearbuff(buff);
    }
    return ((*buff).space).as_mut_ptr().offset((*buff).blen as isize);
}
unsafe extern "C-unwind" fn addstr2buff(
    mut buff: *mut BuffFS,
    mut str: *const std::ffi::c_char,
    mut slen: size_t,
) {
    if slen <= (60 + 44 as i32 + 95 as i32) as size_t {
        let mut bf: *mut std::ffi::c_char = getbuff(buff, slen as i32);
        memcpy(bf as *mut c_void, str as *const c_void, slen);
        (*buff).blen += slen as i32;
    } else {
        clearbuff(buff);
        pushstr(buff, str, slen);
    };
}
unsafe extern "C-unwind" fn addnum2buff(mut buff: *mut BuffFS, mut num: *mut TValue) {
    let mut numbuff: *mut std::ffi::c_char = getbuff(buff, 44 as i32);
    let mut len: i32 = tostringbuff(num, numbuff);
    (*buff).blen += len;
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaO_pushvfstring(
    mut L: *mut lua_State,
    mut fmt: *const std::ffi::c_char,
    mut argp: ::core::ffi::VaList,
) -> *const std::ffi::c_char {
    let mut buff: BuffFS = BuffFS {
        L: 0 as *mut lua_State,
        pushed: 0,
        blen: 0,
        space: [0; 199],
    };
    let mut e: *const std::ffi::c_char = 0 as *const std::ffi::c_char;
    buff.blen = 0;
    buff.pushed = buff.blen;
    buff.L = L;
    loop {
        e = strchr(fmt, '%' as i32);
        if e.is_null() {
            break;
        }
        addstr2buff(
            &mut buff,
            fmt,
            e.offset_from(fmt) as std::ffi::c_long as size_t,
        );
        match *e.offset(1) as i32 {
            115 => {
                let mut s: *const std::ffi::c_char = argp.arg::<*mut std::ffi::c_char>();
                if s.is_null() {
                    s = c"(null)".as_ptr();
                }
                addstr2buff(&mut buff, s, strlen(s));
            }
            99 => {
                let mut c: std::ffi::c_char = argp.arg::<i32>() as u8 as std::ffi::c_char;
                addstr2buff(
                    &mut buff,
                    &mut c,
                    ::core::mem::size_of::<std::ffi::c_char>() as usize,
                );
            }
            100 => {
                let mut num: TValue = TValue {
                    value_: Value {
                        gc: 0 as *mut GCObject,
                    },
                    tt_: 0,
                };
                let mut io: *mut TValue = &mut num;
                (*io).value_.i = argp.arg::<i32>() as lua_Integer;
                (*io).tt_ = (3 as i32 | (0) << 4 as i32) as lu_byte;
                addnum2buff(&mut buff, &mut num);
            }
            73 => {
                let mut num_0: TValue = TValue {
                    value_: Value {
                        gc: 0 as *mut GCObject,
                    },
                    tt_: 0,
                };
                let mut io_0: *mut TValue = &mut num_0;
                (*io_0).value_.i = argp.arg::<l_uacInt>();
                (*io_0).tt_ = (3 as i32 | (0) << 4 as i32) as lu_byte;
                addnum2buff(&mut buff, &mut num_0);
            }
            102 => {
                let mut num_1: TValue = TValue {
                    value_: Value {
                        gc: 0 as *mut GCObject,
                    },
                    tt_: 0,
                };
                let mut io_1: *mut TValue = &mut num_1;
                (*io_1).value_.n = argp.arg::<l_uacNumber>();
                (*io_1).tt_ = (3 as i32 | (1 as i32) << 4 as i32) as lu_byte;
                addnum2buff(&mut buff, &mut num_1);
            }
            112 => {
                let sz: i32 = (3usize)
                    .wrapping_mul(::core::mem::size_of::<*mut c_void>() as usize)
                    .wrapping_add(8) as i32;
                let mut bf: *mut std::ffi::c_char = getbuff(&mut buff, sz);
                let mut p: *mut c_void = argp.arg::<*mut c_void>();
                let mut len: i32 = snprintf(bf, sz as usize, c"%p".as_ptr(), p);
                buff.blen += len;
            }
            85 => {
                let mut bf_0: [std::ffi::c_char; 8] = [0; 8];
                let mut len_0: i32 =
                    luaO_utf8esc(bf_0.as_mut_ptr(), argp.arg::<std::ffi::c_long>() as usize);
                addstr2buff(
                    &mut buff,
                    bf_0.as_mut_ptr().offset(8).offset(-(len_0 as isize)),
                    len_0 as size_t,
                );
            }
            37 => {
                addstr2buff(&mut buff, c"%".as_ptr(), 1 as i32 as size_t);
            }
            _ => {
                luaG_runerror(
                    L,
                    c"invalid option '%%%c' to 'lua_pushfstring'".as_ptr(),
                    *e.offset(1) as i32,
                );
            }
        }
        fmt = e.offset(2);
    }
    addstr2buff(&mut buff, fmt, strlen(fmt));
    clearbuff(&mut buff);
    return ((*&mut (*((*((*L).top.p).offset(-(1))).val.value_.gc as *mut GCUnion)).ts).contents)
        .as_mut_ptr();
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaO_pushfstring(
    mut L: *mut lua_State,
    mut fmt: *const std::ffi::c_char,
    mut args: ...
) -> *const std::ffi::c_char {
    let mut msg: *const std::ffi::c_char = 0 as *const std::ffi::c_char;
    let mut argp: ::core::ffi::VaListImpl;
    argp = args.clone();
    msg = luaO_pushvfstring(L, fmt, argp.as_va_list());
    return msg;
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaO_chunkid(
    mut out: *mut std::ffi::c_char,
    mut source: *const std::ffi::c_char,
    mut srclen: size_t,
) {
    let mut bufflen: size_t = 60 as size_t;
    if *source as i32 == '=' as i32 {
        if srclen <= bufflen {
            memcpy(
                out as *mut c_void,
                source.offset(1) as *const c_void,
                srclen.wrapping_mul(::core::mem::size_of::<std::ffi::c_char>() as usize),
            );
        } else {
            memcpy(
                out as *mut c_void,
                source.offset(1) as *const c_void,
                bufflen
                    .wrapping_sub(1 as i32 as size_t)
                    .wrapping_mul(::core::mem::size_of::<std::ffi::c_char>() as usize),
            );
            out = out.offset(bufflen.wrapping_sub(1 as i32 as size_t) as isize);
            *out = b'\0' as std::ffi::c_char;
        }
    } else if *source as i32 == '@' as i32 {
        if srclen <= bufflen {
            memcpy(
                out as *mut c_void,
                source.offset(1) as *const c_void,
                srclen.wrapping_mul(::core::mem::size_of::<std::ffi::c_char>() as usize),
            );
        } else {
            memcpy(
                out as *mut c_void,
                c"...".as_ptr() as *const c_void,
                (::core::mem::size_of::<[std::ffi::c_char; 4]>() as usize)
                    .wrapping_div(::core::mem::size_of::<std::ffi::c_char>() as usize)
                    .wrapping_sub(1)
                    .wrapping_mul(::core::mem::size_of::<std::ffi::c_char>() as usize),
            );
            out = out.offset(
                (::core::mem::size_of::<[std::ffi::c_char; 4]>() as usize)
                    .wrapping_div(::core::mem::size_of::<std::ffi::c_char>() as usize)
                    .wrapping_sub(1) as isize,
            );
            bufflen = (bufflen as usize).wrapping_sub(
                (::core::mem::size_of::<[std::ffi::c_char; 4]>() as usize)
                    .wrapping_div(::core::mem::size_of::<std::ffi::c_char>() as usize)
                    .wrapping_sub(1),
            ) as size_t as size_t;
            memcpy(
                out as *mut c_void,
                source
                    .offset(1)
                    .offset(srclen as isize)
                    .offset(-(bufflen as isize)) as *const c_void,
                bufflen.wrapping_mul(::core::mem::size_of::<std::ffi::c_char>() as usize),
            );
        }
    } else {
        let mut nl: *const std::ffi::c_char = strchr(source, '\n' as i32);
        memcpy(
            out as *mut c_void,
            c"[string \"".as_ptr() as *const c_void,
            (::core::mem::size_of::<[std::ffi::c_char; 10]>() as usize)
                .wrapping_div(::core::mem::size_of::<std::ffi::c_char>() as usize)
                .wrapping_sub(1)
                .wrapping_mul(::core::mem::size_of::<std::ffi::c_char>() as usize),
        );
        out = out.offset(
            (::core::mem::size_of::<[std::ffi::c_char; 10]>() as usize)
                .wrapping_div(::core::mem::size_of::<std::ffi::c_char>() as usize)
                .wrapping_sub(1) as isize,
        );
        bufflen = (bufflen as usize).wrapping_sub(
            (::core::mem::size_of::<[std::ffi::c_char; 15]>() as usize)
                .wrapping_div(::core::mem::size_of::<std::ffi::c_char>() as usize)
                .wrapping_sub(1)
                .wrapping_add(1),
        ) as size_t as size_t;
        if srclen < bufflen && nl.is_null() {
            memcpy(
                out as *mut c_void,
                source as *const c_void,
                srclen.wrapping_mul(::core::mem::size_of::<std::ffi::c_char>() as usize),
            );
            out = out.offset(srclen as isize);
        } else {
            if !nl.is_null() {
                srclen = nl.offset_from(source) as std::ffi::c_long as size_t;
            }
            if srclen > bufflen {
                srclen = bufflen;
            }
            memcpy(
                out as *mut c_void,
                source as *const c_void,
                srclen.wrapping_mul(::core::mem::size_of::<std::ffi::c_char>() as usize),
            );
            out = out.offset(srclen as isize);
            memcpy(
                out as *mut c_void,
                c"...".as_ptr() as *const c_void,
                (::core::mem::size_of::<[std::ffi::c_char; 4]>() as usize)
                    .wrapping_div(::core::mem::size_of::<std::ffi::c_char>() as usize)
                    .wrapping_sub(1)
                    .wrapping_mul(::core::mem::size_of::<std::ffi::c_char>() as usize),
            );
            out = out.offset(
                (::core::mem::size_of::<[std::ffi::c_char; 4]>() as usize)
                    .wrapping_div(::core::mem::size_of::<std::ffi::c_char>() as usize)
                    .wrapping_sub(1) as isize,
            );
        }
        memcpy(
            out as *mut c_void,
            c"\"]".as_ptr() as *const c_void,
            (::core::mem::size_of::<[std::ffi::c_char; 3]>() as usize)
                .wrapping_div(::core::mem::size_of::<std::ffi::c_char>() as usize)
                .wrapping_sub(1)
                .wrapping_add(1)
                .wrapping_mul(::core::mem::size_of::<std::ffi::c_char>() as usize),
        );
    };
}
