use crate::*;

static mut dummynode_: Node = Node {
    u: {
        let mut init = NodeKey {
            value_: Value {
                gc: 0 as *const GCObject as *mut GCObject,
            },
            tt_: (0 | (1 as i32) << 4 as i32) as lu_byte,
            key_tt: (0 | (0) << 4 as i32) as lu_byte,
            next: 0,
            key_val: Value {
                gc: 0 as *const GCObject as *mut GCObject,
            },
        };
        init
    },
};

static mut absentkey: TValue = {
    let mut init = TValue {
        value_: Value {
            gc: 0 as *const GCObject as *mut GCObject,
        },
        tt_: (0 | (2 as i32) << 4 as i32) as lu_byte,
    };
    init
};

unsafe extern "C-unwind" fn hashint(mut t: *const Table, mut i: lua_Integer) -> *mut Node {
    let mut ui: lua_Unsigned = i as lua_Unsigned;
    if ui <= 2147483647 as lua_Unsigned {
        return &mut *((*t).node).offset(
            (ui as i32 % (((1 as i32) << (*t).lsizenode as i32) - 1 as i32 | 1 as i32)) as isize,
        ) as *mut Node;
    } else {
        return &mut *((*t).node).offset(
            (ui % (((1 as i32) << (*t).lsizenode as i32) - 1 as i32 | 1 as i32) as lua_Unsigned)
                as isize,
        ) as *mut Node;
    };
}

unsafe extern "C-unwind" fn l_hashfloat(mut n: lua_Number) -> i32 {
    let mut i: i32 = 0;
    let mut ni: lua_Integer = 0;
    n = frexp(n, &mut i) * -((-(2147483647 as i32) - 1 as i32) as lua_Number);
    if !(n
        >= (-(9223372036854775807 as std::ffi::c_longlong) - 1 as std::ffi::c_longlong)
            as std::ffi::c_double
        && n < -((-(9223372036854775807 as std::ffi::c_longlong) - 1 as std::ffi::c_longlong)
            as std::ffi::c_double)
        && {
            ni = n as std::ffi::c_longlong;
            1 as i32 != 0
        })
    {
        return 0;
    } else {
        let mut u: u32 = (i as u32).wrapping_add(ni as u32);
        return (if u <= 2147483647 { u } else { !u }) as i32;
    };
}

unsafe extern "C-unwind" fn mainpositionTV(
    mut t: *const Table,
    mut key: *const TValue,
) -> *mut Node {
    match (*key).tt_ & 0x3f {
        LUA_VNUMINT => {
            let mut i: lua_Integer = (*key).value_.i;
            return hashint(t, i);
        }
        LUA_VNUMFLT => {
            let mut n: lua_Number = (*key).value_.n;
            return &mut *((*t).node).offset(
                ((l_hashfloat as unsafe extern "C-unwind" fn(lua_Number) -> i32)(n)
                    % (((1 as i32) << (*t).lsizenode as i32) - 1 as i32 | 1 as i32))
                    as isize,
            ) as *mut Node;
        }
        LUA_VSHRSTR => {
            let mut ts: *mut TString = &mut (*((*key).value_.gc as *mut GCUnion)).ts;
            return &mut *((*t).node).offset(
                ((*ts).hash & (((1 as i32) << (*t).lsizenode as i32) - 1 as i32) as u32) as i32
                    as isize,
            ) as *mut Node;
        }
        LUA_VLNGSTR => {
            let mut ts_0: *mut TString = &mut (*((*key).value_.gc as *mut GCUnion)).ts;
            return &mut *((*t).node).offset(
                ((luaS_hashlongstr as unsafe extern "C-unwind" fn(*mut TString) -> u32)(ts_0)
                    & (((1 as i32) << (*t).lsizenode as i32) - 1 as i32) as u32)
                    as i32 as isize,
            ) as *mut Node;
        }
        LUA_VFALSE => {
            return &mut *((*t).node)
                .offset((0 & ((1 as i32) << (*t).lsizenode as i32) - 1 as i32) as isize)
                as *mut Node;
        }
        LUA_VTRUE => {
            return &mut *((*t).node)
                .offset((1 as i32 & ((1 as i32) << (*t).lsizenode as i32) - 1 as i32) as isize)
                as *mut Node;
        }
        LUA_VLIGHTUSERDATA => {
            let mut p: *mut c_void = (*key).value_.p;
            return &mut *((*t).node).offset(
                ((p as uintptr_t
                    & (2147483647u32)
                        .wrapping_mul(2 as u32)
                        .wrapping_add(1 as u32) as uintptr_t) as u32)
                    .wrapping_rem(
                        (((1 as i32) << (*t).lsizenode as i32) - 1 as i32 | 1 as i32) as u32,
                    ) as isize,
            ) as *mut Node;
        }
        LUA_VLCF => {
            let mut f: lua_CFunction = (*key).value_.f;
            return &mut *((*t).node).offset(
                ((::core::mem::transmute::<lua_CFunction, uintptr_t>(f)
                    & (2147483647u32)
                        .wrapping_mul(2 as u32)
                        .wrapping_add(1 as u32) as uintptr_t) as u32)
                    .wrapping_rem(
                        (((1 as i32) << (*t).lsizenode as i32) - 1 as i32 | 1 as i32) as u32,
                    ) as isize,
            ) as *mut Node;
        }
        _ => {
            let mut o: *mut GCObject = (*key).value_.gc;
            return &mut *((*t).node).offset(
                ((o as uintptr_t
                    & (2147483647u32)
                        .wrapping_mul(2 as u32)
                        .wrapping_add(1 as u32) as uintptr_t) as u32)
                    .wrapping_rem(
                        (((1 as i32) << (*t).lsizenode as i32) - 1 as i32 | 1 as i32) as u32,
                    ) as isize,
            ) as *mut Node;
        }
    };
}

#[inline]
unsafe extern "C-unwind" fn mainpositionfromnode(
    mut t: *const Table,
    mut nd: *mut Node,
) -> *mut Node {
    let mut key: TValue = TValue {
        value_: Value {
            gc: 0 as *mut GCObject,
        },
        tt_: 0,
    };
    let mut io_: *mut TValue = &mut key;
    let mut n_: *const Node = nd;
    (*io_).value_ = (*n_).u.key_val;
    (*io_).tt_ = (*n_).u.key_tt;
    return mainpositionTV(t, &mut key);
}

unsafe extern "C-unwind" fn equalkey(
    mut k1: *const TValue,
    mut n2: *const Node,
    mut deadok: i32,
) -> i32 {
    if (*k1).tt_ as i32 != (*n2).u.key_tt as i32
        && !(deadok != 0
            && (*n2).u.key_tt as i32 == 9 as i32 + 2 as i32
            && (*k1).tt_ as i32 & (1 as i32) << 6 as i32 != 0)
    {
        return 0;
    }
    match (*n2).u.key_tt {
        LUA_VNIL | LUA_VFALSE | LUA_VTRUE => return 1 as i32,
        LUA_VNUMINT => return ((*k1).value_.i == (*n2).u.key_val.i) as i32,
        LUA_VNUMFLT => return ((*k1).value_.n == (*n2).u.key_val.n) as i32,
        LUA_VLIGHTUSERDATA => return ((*k1).value_.p == (*n2).u.key_val.p) as i32,
        LUA_VLCF => return ((*k1).value_.f == (*n2).u.key_val.f) as i32,
        LUA_VLNGSTR_CTB => {
            return luaS_eqlngstr(
                &mut (*((*k1).value_.gc as *mut GCUnion)).ts,
                &mut (*((*n2).u.key_val.gc as *mut GCUnion)).ts,
            );
        }
        _ => return ((*k1).value_.gc == (*n2).u.key_val.gc) as i32,
    };
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaH_realasize(mut t: *const Table) -> u32 {
    if (*t).flags as i32 & (1 as i32) << 7 as i32 == 0
        || (*t).alimit & ((*t).alimit).wrapping_sub(1) == 0 as u32
    {
        return (*t).alimit;
    } else {
        let mut size: u32 = (*t).alimit;
        size |= size >> 1 as i32;
        size |= size >> 2 as i32;
        size |= size >> 4 as i32;
        size |= size >> 8 as i32;
        size |= size >> 16 as i32;
        size = size.wrapping_add(1);
        return size;
    };
}

unsafe extern "C-unwind" fn ispow2realasize(mut t: *const Table) -> i32 {
    return ((*t).flags as i32 & (1 as i32) << 7 as i32 != 0
        || (*t).alimit & ((*t).alimit).wrapping_sub(1) == 0 as u32) as i32;
}

unsafe extern "C-unwind" fn setlimittosize(mut t: *mut Table) -> u32 {
    (*t).alimit = luaH_realasize(t);
    (*t).flags = ((*t).flags as i32 & !((1 as i32) << 7 as i32) as lu_byte as i32) as lu_byte;
    return (*t).alimit;
}

unsafe extern "C-unwind" fn getgeneric(
    mut t: *mut Table,
    mut key: *const TValue,
    mut deadok: i32,
) -> *const TValue {
    let mut n: *mut Node = mainpositionTV(t, key);
    loop {
        if equalkey(key, n, deadok) != 0 {
            return &mut (*n).i_val;
        } else {
            let mut nx: i32 = (*n).u.next;
            if nx == 0 {
                return &raw const absentkey;
            }
            n = n.offset(nx as isize);
        }
    }
}

unsafe extern "C-unwind" fn arrayindex(mut k: lua_Integer) -> u32 {
    if (k as lua_Unsigned).wrapping_sub(1 as u32 as lua_Unsigned)
        < (if ((1 as u32) << (size_of::<i32>() as usize).wrapping_mul(8).wrapping_sub(1) as i32)
            as size_t
            <= (!(0 as size_t)).wrapping_div(size_of::<TValue>() as usize)
        {
            (1 as u32) << (size_of::<i32>() as usize).wrapping_mul(8).wrapping_sub(1) as i32
        } else {
            (!(0 as size_t)).wrapping_div(size_of::<TValue>() as usize) as u32
        }) as lua_Unsigned
    {
        return k as u32;
    } else {
        return 0 as u32;
    };
}

unsafe extern "C-unwind" fn findindex(
    mut L: *mut lua_State,
    mut t: *mut Table,
    mut key: *mut TValue,
    mut asize: u32,
) -> u32 {
    let mut i: u32 = 0;
    if (*key).tt_ as i32 & 0xf as i32 == 0 {
        return 0 as u32;
    }
    i = if (*key).tt_ as i32 == 3 as i32 | (0) << 4 as i32 {
        arrayindex((*key).value_.i)
    } else {
        0 as u32
    };
    if i.wrapping_sub(1 as u32) < asize {
        return i;
    } else {
        let mut n: *const TValue = getgeneric(t, key, 1 as i32);
        if (((*n).tt_ as i32 == 0 | (2 as i32) << 4 as i32) as i32 != 0) as i32 as std::ffi::c_long
            != 0
        {
            luaG_runerror(L, c"invalid key to 'next'".as_ptr());
        }
        i = (n as *mut Node).offset_from(&mut *((*t).node).offset(0 as isize) as *mut Node)
            as std::ffi::c_long as i32 as u32;
        return i.wrapping_add(1).wrapping_add(asize);
    };
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaH_next(
    mut L: *mut lua_State,
    mut t: *mut Table,
    mut key: StkId,
) -> i32 {
    let mut asize: u32 = luaH_realasize(t);
    let mut i: u32 = findindex(L, t, &mut (*key).val, asize);
    while i < asize {
        if !((*((*t).array).offset(i as isize)).tt_ as i32 & 0xf as i32 == 0) {
            let mut io: *mut TValue = &mut (*key).val;
            (*io).value_.i = i.wrapping_add(1) as lua_Integer;
            (*io).tt_ = (3 as i32 | (0) << 4 as i32) as lu_byte;
            let mut io1: *mut TValue = &mut (*key.offset(1)).val;
            let mut io2: *const TValue = &mut *((*t).array).offset(i as isize) as *mut TValue;
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
            return 1 as i32;
        }
        i = i.wrapping_add(1);
    }
    i = i.wrapping_sub(asize);
    while (i as i32) < (1 as i32) << (*t).lsizenode as i32 {
        if !((*((*t).node).offset(i as isize)).i_val.tt_ as i32 & 0xf as i32 == 0) {
            let mut n: *mut Node = &mut *((*t).node).offset(i as isize) as *mut Node;
            let mut io_: *mut TValue = &mut (*key).val;
            let mut n_: *const Node = n;
            (*io_).value_ = (*n_).u.key_val;
            (*io_).tt_ = (*n_).u.key_tt;
            if (*io_).tt_ as i32 & (1 as i32) << 6 as i32 == 0
                || (*io_).tt_ as i32 & 0x3f as i32 == (*(*io_).value_.gc).tt as i32
                    && (L.is_null()
                        || (*(*io_).value_.gc).marked as i32
                            & ((*(*L).l_G).currentwhite as i32
                                ^ ((1 as i32) << 3 as i32 | (1 as i32) << 4 as i32))
                            == 0)
            {
            } else {
            };
            let mut io1_0: *mut TValue = &mut (*key.offset(1)).val;
            let mut io2_0: *const TValue = &mut (*n).i_val;
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
            return 1 as i32;
        }
        i = i.wrapping_add(1);
    }
    return 0;
}

unsafe extern "C-unwind" fn freehash(mut L: *mut lua_State, mut t: *mut Table) {
    if !((*t).lastfree).is_null() {
        luaM_free_(
            L,
            (*t).node as *mut c_void,
            (((1 as i32) << (*t).lsizenode as i32) as size_t)
                .wrapping_mul(size_of::<Node>() as usize),
        );
    }
}

unsafe extern "C-unwind" fn computesizes(mut nums: *mut u32, mut pna: *mut u32) -> u32 {
    let mut i: i32 = 0;
    let mut twotoi: u32 = 0;
    let mut a: u32 = 0 as u32;
    let mut na: u32 = 0 as u32;
    let mut optimal: u32 = 0 as u32;
    i = 0;
    twotoi = 1;
    while twotoi > 0 as u32 && *pna > twotoi.wrapping_div(2) {
        a = a.wrapping_add(*nums.offset(i as isize));
        if a > twotoi.wrapping_div(2) {
            optimal = twotoi;
            na = a;
        }
        i += 1;
        twotoi = twotoi.wrapping_mul(2);
    }
    *pna = na;
    return optimal;
}

unsafe extern "C-unwind" fn countint(mut key: lua_Integer, mut nums: *mut u32) -> i32 {
    let mut k: u32 = arrayindex(key);
    if k != 0 {
        let ref mut num = *nums.offset(luaO_ceillog2(k) as isize);
        *num = (*num).wrapping_add(1);
        return 1;
    } else {
        return 0;
    }
}

unsafe extern "C-unwind" fn numusearray(mut t: *const Table, mut nums: *mut u32) -> u32 {
    let mut lg: i32 = 0;
    let mut ttlg: u32 = 0;
    let mut ause: u32 = 0 as u32;
    let mut i: u32 = 1;
    let mut asize: u32 = (*t).alimit;
    lg = 0;
    ttlg = 1;
    while lg <= (size_of::<i32>() as usize).wrapping_mul(8).wrapping_sub(1) as i32 {
        let mut lc: u32 = 0 as u32;
        let mut lim: u32 = ttlg;
        if lim > asize {
            lim = asize;
            if i > lim {
                break;
            }
        }
        while i <= lim {
            if !((*((*t).array).offset(i.wrapping_sub(1) as isize)).tt_ as i32 & 0xf as i32 == 0) {
                lc = lc.wrapping_add(1);
            }
            i = i.wrapping_add(1);
        }
        let ref mut fresh126 = *nums.offset(lg as isize);
        *fresh126 = (*fresh126).wrapping_add(lc);
        ause = ause.wrapping_add(lc);
        lg += 1;
        ttlg = ttlg.wrapping_mul(2);
    }
    return ause;
}

unsafe extern "C-unwind" fn numusehash(
    mut t: *const Table,
    mut nums: *mut u32,
    mut pna: *mut u32,
) -> i32 {
    let mut totaluse: i32 = 0;
    let mut ause: i32 = 0;
    let mut i: i32 = (1 as i32) << (*t).lsizenode as i32;
    loop {
        let fresh127 = i;
        i = i - 1;
        if !(fresh127 != 0) {
            break;
        }
        let mut n: *mut Node = &mut *((*t).node).offset(i as isize) as *mut Node;
        if !((*n).i_val.tt_ as i32 & 0xf as i32 == 0) {
            if (*n).u.key_tt as i32 == 3 as i32 | (0) << 4 as i32 {
                ause += countint((*n).u.key_val.i, nums);
            }
            totaluse += 1;
        }
    }
    *pna = (*pna).wrapping_add(ause as u32);
    return totaluse;
}

unsafe extern "C-unwind" fn setnodevector(mut L: *mut lua_State, mut t: *mut Table, mut size: u32) {
    if size == 0 as u32 {
        (*t).node = &raw const dummynode_ as *const Node as *mut Node;
        (*t).lsizenode = 0 as lu_byte;
        (*t).lastfree = 0 as *mut Node;
    } else {
        let mut i: i32 = 0;
        let mut lsize: i32 = luaO_ceillog2(size);
        if lsize > (size_of::<i32>() as usize).wrapping_mul(8).wrapping_sub(1) as i32 - 1 as i32
            || (1 as u32) << lsize
                > (if ((1 as u32)
                    << (size_of::<i32>() as usize).wrapping_mul(8).wrapping_sub(1) as i32
                        - 1 as i32) as size_t
                    <= (!(0 as size_t)).wrapping_div(size_of::<Node>() as usize)
                {
                    (1 as u32)
                        << (size_of::<i32>() as usize).wrapping_mul(8).wrapping_sub(1) as i32
                            - 1 as i32
                } else {
                    (!(0 as size_t)).wrapping_div(size_of::<Node>() as usize) as u32
                })
        {
            luaG_runerror(L, c"table overflow".as_ptr());
        }
        size = ((1 as i32) << lsize) as u32;
        (*t).node = luaM_malloc_(
            L,
            (size as usize).wrapping_mul(size_of::<Node>() as usize),
            0,
        ) as *mut Node;
        i = 0;
        while i < size as i32 {
            let mut n: *mut Node = &mut *((*t).node).offset(i as isize) as *mut Node;
            (*n).u.next = 0;
            (*n).u.key_tt = 0 as lu_byte;
            (*n).i_val.tt_ = (0 | (1 as i32) << 4 as i32) as lu_byte;
            i += 1;
        }
        (*t).lsizenode = lsize as lu_byte;
        (*t).lastfree = &mut *((*t).node).offset(size as isize) as *mut Node;
    };
}

unsafe extern "C-unwind" fn reinsert(mut L: *mut lua_State, mut ot: *mut Table, mut t: *mut Table) {
    let mut j: i32 = 0;
    let mut size: i32 = (1 as i32) << (*ot).lsizenode as i32;
    j = 0;
    while j < size {
        let mut old: *mut Node = &mut *((*ot).node).offset(j as isize) as *mut Node;
        if !((*old).i_val.tt_ as i32 & 0xf as i32 == 0) {
            let mut k: TValue = TValue {
                value_: Value {
                    gc: 0 as *mut GCObject,
                },
                tt_: 0,
            };
            let mut io_: *mut TValue = &mut k;
            let mut n_: *const Node = old;
            (*io_).value_ = (*n_).u.key_val;
            (*io_).tt_ = (*n_).u.key_tt;
            if (*io_).tt_ as i32 & (1 as i32) << 6 as i32 == 0
                || (*io_).tt_ as i32 & 0x3f as i32 == (*(*io_).value_.gc).tt as i32
                    && (L.is_null()
                        || (*(*io_).value_.gc).marked as i32
                            & ((*(*L).l_G).currentwhite as i32
                                ^ ((1 as i32) << 3 as i32 | (1 as i32) << 4 as i32))
                            == 0)
            {
            } else {
            };
            luaH_set(L, t, &mut k, &mut (*old).i_val);
        }
        j += 1;
    }
}

unsafe extern "C-unwind" fn exchangehashpart(mut t1: *mut Table, mut t2: *mut Table) {
    let mut lsizenode: lu_byte = (*t1).lsizenode;
    let mut node: *mut Node = (*t1).node;
    let mut lastfree: *mut Node = (*t1).lastfree;
    (*t1).lsizenode = (*t2).lsizenode;
    (*t1).node = (*t2).node;
    (*t1).lastfree = (*t2).lastfree;
    (*t2).lsizenode = lsizenode;
    (*t2).node = node;
    (*t2).lastfree = lastfree;
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaH_resize(
    mut L: *mut lua_State,
    mut t: *mut Table,
    mut newasize: u32,
    mut nhsize: u32,
) {
    let mut i: u32 = 0;
    let mut newt: Table = Table {
        next: 0 as *mut GCObject,
        tt: 0,
        marked: 0,
        flags: 0,
        lsizenode: 0,
        alimit: 0,
        array: 0 as *mut TValue,
        node: 0 as *mut Node,
        lastfree: 0 as *mut Node,
        metatable: 0 as *mut Table,
        gclist: 0 as *mut GCObject,
    };
    let mut oldasize: u32 = setlimittosize(t);
    let mut newarray: *mut TValue = 0 as *mut TValue;
    setnodevector(L, &mut newt, nhsize);
    if newasize < oldasize {
        (*t).alimit = newasize;
        exchangehashpart(t, &mut newt);
        i = newasize;
        while i < oldasize {
            if !((*((*t).array).offset(i as isize)).tt_ as i32 & 0xf as i32 == 0) {
                luaH_setint(
                    L,
                    t,
                    i.wrapping_add(1) as lua_Integer,
                    &mut *((*t).array).offset(i as isize),
                );
            }
            i = i.wrapping_add(1);
        }
        (*t).alimit = oldasize;
        exchangehashpart(t, &mut newt);
    }
    newarray = luaM_realloc_(
        L,
        (*t).array as *mut c_void,
        (oldasize as size_t).wrapping_mul(size_of::<TValue>() as usize),
        (newasize as size_t).wrapping_mul(size_of::<TValue>() as usize),
    ) as *mut TValue;
    if ((newarray.is_null() && newasize > 0 as u32) as i32 != 0) as i32 as std::ffi::c_long != 0 {
        freehash(L, &mut newt);
        luaD_throw(L, 4 as i32);
    }
    exchangehashpart(t, &mut newt);
    (*t).array = newarray;
    (*t).alimit = newasize;
    i = oldasize;
    while i < newasize {
        (*((*t).array).offset(i as isize)).tt_ = (0 | (1 as i32) << 4 as i32) as lu_byte;
        i = i.wrapping_add(1);
    }
    reinsert(L, &mut newt, t);
    freehash(L, &mut newt);
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaH_resizearray(
    mut L: *mut lua_State,
    mut t: *mut Table,
    mut nasize: u32,
) {
    let mut nsize: i32 = if ((*t).lastfree).is_null() {
        0
    } else {
        (1 as i32) << (*t).lsizenode as i32
    };
    luaH_resize(L, t, nasize, nsize as u32);
}

unsafe extern "C-unwind" fn rehash(
    mut L: *mut lua_State,
    mut t: *mut Table,
    mut ek: *const TValue,
) {
    let mut asize: u32 = 0;
    let mut na: u32 = 0;
    let mut nums: [u32; 32] = [0; 32];
    let mut i: i32 = 0;
    let mut totaluse: i32 = 0;
    i = 0;
    while i <= (size_of::<i32>() as usize).wrapping_mul(8).wrapping_sub(1) as i32 {
        nums[i as usize] = 0 as u32;
        i += 1;
    }
    setlimittosize(t);
    na = numusearray(t, nums.as_mut_ptr());
    totaluse = na as i32;
    totaluse += numusehash(t, nums.as_mut_ptr(), &mut na);
    if (*ek).tt_ as i32 == 3 as i32 | (0) << 4 as i32 {
        na = na.wrapping_add(countint((*ek).value_.i, nums.as_mut_ptr()) as u32);
    }
    totaluse += 1;
    asize = computesizes(nums.as_mut_ptr(), &mut na);
    luaH_resize(L, t, asize, (totaluse as u32).wrapping_sub(na));
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaH_new(mut L: *mut lua_State) -> *mut Table {
    let mut o: *mut GCObject =
        luaC_newobj(L, 5 as i32 | (0) << 4 as i32, size_of::<Table>() as usize);
    let mut t: *mut Table = &mut (*(o as *mut GCUnion)).h;
    (*t).metatable = 0 as *mut Table;
    (*t).flags = !(!(0 as u32) << TM_EQ as i32 + 1 as i32) as lu_byte;
    (*t).array = 0 as *mut TValue;
    (*t).alimit = 0 as u32;
    setnodevector(L, t, 0 as u32);
    return t;
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaH_free(mut L: *mut lua_State, mut t: *mut Table) {
    freehash(L, t);
    luaM_free_(
        L,
        (*t).array as *mut c_void,
        (luaH_realasize(t) as usize).wrapping_mul(size_of::<TValue>() as usize),
    );
    luaM_free_(L, t as *mut c_void, size_of::<Table>() as usize);
}

unsafe extern "C-unwind" fn getfreepos(mut t: *mut Table) -> *mut Node {
    if !((*t).lastfree).is_null() {
        while (*t).lastfree > (*t).node {
            (*t).lastfree = ((*t).lastfree).offset(-1);
            (*t).lastfree;
            if (*(*t).lastfree).u.key_tt as i32 == 0 {
                return (*t).lastfree;
            }
        }
    }
    return 0 as *mut Node;
}

unsafe extern "C-unwind" fn luaH_newkey(
    mut L: *mut lua_State,
    mut t: *mut Table,
    mut key: *const TValue,
    mut value: *mut TValue,
) {
    let mut mp: *mut Node = 0 as *mut Node;
    let mut aux: TValue = TValue {
        value_: Value {
            gc: 0 as *mut GCObject,
        },
        tt_: 0,
    };
    if (((*key).tt_ as i32 & 0xf as i32 == 0) as i32 != 0) as i32 as std::ffi::c_long != 0 {
        luaG_runerror(L, c"table index is nil".as_ptr());
    } else if (*key).tt_ as i32 == 3 as i32 | (1 as i32) << 4 as i32 {
        let mut f: lua_Number = (*key).value_.n;
        let mut k: lua_Integer = 0;
        if luaV_flttointeger(f, &mut k, F2Ieq) != 0 {
            let mut io: *mut TValue = &mut aux;
            (*io).value_.i = k;
            (*io).tt_ = (3 as i32 | (0) << 4 as i32) as lu_byte;
            key = &mut aux;
        } else if (!(f == f) as i32 != 0) as i32 as std::ffi::c_long != 0 {
            luaG_runerror(L, c"table index is NaN".as_ptr());
        }
    }
    if (*value).tt_ as i32 & 0xf as i32 == 0 {
        return;
    }
    mp = mainpositionTV(t, key);
    if !((*mp).i_val.tt_ as i32 & 0xf as i32 == 0) || ((*t).lastfree).is_null() {
        let mut othern: *mut Node = 0 as *mut Node;
        let mut f_0: *mut Node = getfreepos(t);
        if f_0.is_null() {
            rehash(L, t, key);
            luaH_set(L, t, key, value);
            return;
        }
        othern = mainpositionfromnode(t, mp);
        if othern != mp {
            while othern.offset((*othern).u.next as isize) != mp {
                othern = othern.offset((*othern).u.next as isize);
            }
            (*othern).u.next = f_0.offset_from(othern) as std::ffi::c_long as i32;
            *f_0 = *mp;
            if (*mp).u.next != 0 {
                (*f_0).u.next += mp.offset_from(f_0) as std::ffi::c_long as i32;
                (*mp).u.next = 0;
            }
            (*mp).i_val.tt_ = (0 | (1 as i32) << 4 as i32) as lu_byte;
        } else {
            if (*mp).u.next != 0 {
                (*f_0).u.next =
                    mp.offset((*mp).u.next as isize).offset_from(f_0) as std::ffi::c_long as i32;
            }
            (*mp).u.next = f_0.offset_from(mp) as std::ffi::c_long as i32;
            mp = f_0;
        }
    }
    let mut n_: *mut Node = mp;
    let mut io_: *const TValue = key;
    (*n_).u.key_val = (*io_).value_;
    (*n_).u.key_tt = (*io_).tt_;
    if (*io_).tt_ as i32 & (1 as i32) << 6 as i32 == 0
        || (*io_).tt_ as i32 & 0x3f as i32 == (*(*io_).value_.gc).tt as i32
            && (L.is_null()
                || (*(*io_).value_.gc).marked as i32
                    & ((*(*L).l_G).currentwhite as i32
                        ^ ((1 as i32) << 3 as i32 | (1 as i32) << 4 as i32))
                    == 0)
    {
    } else {
    };
    if (*key).tt_ as i32 & (1 as i32) << 6 as i32 != 0 {
        if (*(&mut (*(t as *mut GCUnion)).gc as *mut GCObject)).marked as i32
            & (1 as i32) << 5 as i32
            != 0
            && (*(*key).value_.gc).marked as i32 & ((1 as i32) << 3 as i32 | (1 as i32) << 4 as i32)
                != 0
        {
            luaC_barrierback_(L, &mut (*(t as *mut GCUnion)).gc);
        } else {
        };
    } else {
    };
    let mut io1: *mut TValue = &mut (*mp).i_val;
    let mut io2: *const TValue = value;
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

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaH_getint(
    mut t: *mut Table,
    mut key: lua_Integer,
) -> *const TValue {
    let mut alimit: lua_Unsigned = (*t).alimit as lua_Unsigned;
    if (key as lua_Unsigned).wrapping_sub(1 as u32 as lua_Unsigned) < alimit {
        return &mut *((*t).array).offset((key - 1 as i32 as lua_Integer) as isize) as *mut TValue;
    } else if (*t).flags as i32 & (1 as i32) << 7 as i32 != 0
        && (key as lua_Unsigned).wrapping_sub(1 as u32 as lua_Unsigned)
            & !alimit.wrapping_sub(1 as u32 as lua_Unsigned)
            < alimit
    {
        (*t).alimit = key as u32;
        return &mut *((*t).array).offset((key - 1 as i32 as lua_Integer) as isize) as *mut TValue;
    } else {
        let mut n: *mut Node = hashint(t, key);
        loop {
            if (*n).u.key_tt as i32 == 3 as i32 | (0) << 4 as i32 && (*n).u.key_val.i == key {
                return &mut (*n).i_val;
            } else {
                let mut nx: i32 = (*n).u.next;
                if nx == 0 {
                    break;
                }
                n = n.offset(nx as isize);
            }
        }
        return &raw const absentkey;
    };
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaH_getshortstr(
    mut t: *mut Table,
    mut key: *mut TString,
) -> *const TValue {
    let mut n: *mut Node = &mut *((*t).node).offset(
        ((*key).hash & (((1 as i32) << (*t).lsizenode as i32) - 1 as i32) as u32) as i32 as isize,
    ) as *mut Node;
    loop {
        if (*n).u.key_tt as i32 == 4 as i32 | (0) << 4 as i32 | (1 as i32) << 6 as i32
            && &mut (*((*n).u.key_val.gc as *mut GCUnion)).ts as *mut TString == key
        {
            return &mut (*n).i_val;
        } else {
            let mut nx: i32 = (*n).u.next;
            if nx == 0 {
                return &raw const absentkey;
            }
            n = n.offset(nx as isize);
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaH_getstr(
    mut t: *mut Table,
    mut key: *mut TString,
) -> *const TValue {
    if (*key).tt as i32 == 4 as i32 | (0) << 4 as i32 {
        return luaH_getshortstr(t, key);
    } else {
        let mut ko: TValue = TValue {
            value_: Value {
                gc: 0 as *mut GCObject,
            },
            tt_: 0,
        };
        let mut io: *mut TValue = &mut ko;
        let mut x_: *mut TString = key;
        (*io).value_.gc = &mut (*(x_ as *mut GCUnion)).gc;
        (*io).tt_ = ((*x_).tt as i32 | (1 as i32) << 6 as i32) as lu_byte;

        return getgeneric(t, &mut ko, 0);
    };
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaH_get(
    mut t: *mut Table,
    mut key: *const TValue,
) -> *const TValue {
    match (*key).tt_ as i32 & 0x3f as i32 {
        4 => return luaH_getshortstr(t, &mut (*((*key).value_.gc as *mut GCUnion)).ts),
        3 => return luaH_getint(t, (*key).value_.i),
        0 => return &raw const absentkey,
        19 => {
            let mut k: lua_Integer = 0;
            if luaV_flttointeger((*key).value_.n, &mut k, F2Ieq) != 0 {
                return luaH_getint(t, k);
            }
        }
        _ => {}
    }
    return getgeneric(t, key, 0);
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaH_finishset(
    mut L: *mut lua_State,
    mut t: *mut Table,
    mut key: *const TValue,
    mut slot: *const TValue,
    mut value: *mut TValue,
) {
    if (*slot).tt_ as i32 == 0 | (2 as i32) << 4 as i32 {
        luaH_newkey(L, t, key, value);
    } else {
        let mut io1: *mut TValue = slot as *mut TValue;
        let mut io2: *const TValue = value;
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
    };
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaH_set(
    mut L: *mut lua_State,
    mut t: *mut Table,
    mut key: *const TValue,
    mut value: *mut TValue,
) {
    let mut slot: *const TValue = luaH_get(t, key);
    luaH_finishset(L, t, key, slot, value);
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaH_setint(
    mut L: *mut lua_State,
    mut t: *mut Table,
    mut key: lua_Integer,
    mut value: *mut TValue,
) {
    let mut p: *const TValue = luaH_getint(t, key);
    if (*p).tt_ as i32 == 0 | (2 as i32) << 4 as i32 {
        let mut k: TValue = TValue {
            value_: Value {
                gc: 0 as *mut GCObject,
            },
            tt_: 0,
        };
        let mut io: *mut TValue = &mut k;
        (*io).value_.i = key;
        (*io).tt_ = (3 as i32 | (0) << 4 as i32) as lu_byte;
        luaH_newkey(L, t, &mut k, value);
    } else {
        let mut io1: *mut TValue = p as *mut TValue;
        let mut io2: *const TValue = value;
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
    };
}

unsafe extern "C-unwind" fn hash_search(mut t: *mut Table, mut j: lua_Unsigned) -> lua_Unsigned {
    let mut i: lua_Unsigned = 0;
    if j == 0 as lua_Unsigned {
        j = j.wrapping_add(1);
    }
    loop {
        i = j;
        if j <= 9223372036854775807 as std::ffi::c_longlong as lua_Unsigned
            / 2 as i32 as lua_Unsigned
        {
            j = j * 2 as i32 as lua_Unsigned;
            if (*luaH_getint(t, j as lua_Integer)).tt_ as i32 & 0xf as i32 == 0 {
                break;
            }
        } else {
            j = 9223372036854775807 as std::ffi::c_longlong as lua_Unsigned;
            if (*luaH_getint(t, j as lua_Integer)).tt_ as i32 & 0xf as i32 == 0 {
                break;
            }
            return j;
        }
    }
    while j.wrapping_sub(i) > 1 as u32 as lua_Unsigned {
        let mut m: lua_Unsigned = i.wrapping_add(j) / 2 as i32 as lua_Unsigned;
        if (*luaH_getint(t, m as lua_Integer)).tt_ as i32 & 0xf as i32 == 0 {
            j = m;
        } else {
            i = m;
        }
    }
    return i;
}

unsafe extern "C-unwind" fn binsearch(mut array: *const TValue, mut i: u32, mut j: u32) -> u32 {
    while j.wrapping_sub(i) > 1 as u32 {
        let mut m: u32 = i.wrapping_add(j).wrapping_div(2);
        if (*array.offset(m.wrapping_sub(1) as isize)).tt_ as i32 & 0xf as i32 == 0 {
            j = m;
        } else {
            i = m;
        }
    }
    return i;
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaH_getn(mut t: *mut Table) -> lua_Unsigned {
    let mut limit: u32 = (*t).alimit;
    if limit > 0 as u32
        && (*((*t).array).offset(limit.wrapping_sub(1) as isize)).tt_ as i32 & 0xf as i32 == 0
    {
        if limit >= 2
            && !((*((*t).array).offset(limit.wrapping_sub(2) as isize)).tt_ as i32 & 0xf as i32
                == 0)
        {
            if ispow2realasize(t) != 0
                && !(limit.wrapping_sub(1) & limit.wrapping_sub(1).wrapping_sub(1) == 0 as u32)
            {
                (*t).alimit = limit.wrapping_sub(1);
                (*t).flags = ((*t).flags as i32 | (1 as i32) << 7 as i32) as lu_byte;
            }
            return limit.wrapping_sub(1) as lua_Unsigned;
        } else {
            let mut boundary: u32 = binsearch((*t).array, 0 as u32, limit);
            if ispow2realasize(t) != 0 && boundary > (luaH_realasize(t)).wrapping_div(2) {
                (*t).alimit = boundary;
                (*t).flags = ((*t).flags as i32 | (1 as i32) << 7 as i32) as lu_byte;
            }
            return boundary as lua_Unsigned;
        }
    }
    if !((*t).flags as i32 & (1 as i32) << 7 as i32 == 0
        || (*t).alimit & ((*t).alimit).wrapping_sub(1) == 0 as u32)
    {
        if (*((*t).array).offset(limit as isize)).tt_ as i32 & 0xf as i32 == 0 {
            return limit as lua_Unsigned;
        }
        limit = luaH_realasize(t);
        if (*((*t).array).offset(limit.wrapping_sub(1) as isize)).tt_ as i32 & 0xf as i32 == 0 {
            let mut boundary_0: u32 = binsearch((*t).array, (*t).alimit, limit);
            (*t).alimit = boundary_0;
            return boundary_0 as lua_Unsigned;
        }
    }
    if ((*t).lastfree).is_null()
        || (*luaH_getint(t, limit.wrapping_add(1) as lua_Integer)).tt_ as i32 & 0xf as i32 == 0
    {
        return limit as lua_Unsigned;
    } else {
        return hash_search(t, limit as lua_Unsigned);
    };
}
