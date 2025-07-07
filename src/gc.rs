use crate::*;

// Possible states of the Garbage Collector
pub(super) const GCSpropagate: u8 = 0;
pub(super) const GCSenteratomic: u8 = 1;
pub(super) const GCSatomic: u8 = 2;
pub(super) const GCSswpallgc: u8 = 3;
pub(super) const GCSswpfinobj: u8 = 4;
pub(super) const GCSswptobefnz: u8 = 5;
pub(super) const GCSswpend: u8 = 6;
pub(super) const GCScallfin: u8 = 7;
pub(super) const GCSpause: u8 = 8;

#[inline]
unsafe fn is_sweep_phase(g: *mut global_State) -> bool {
    let gcstate = (*g).gcstate;
    GCSswpallgc <= gcstate && gcstate <= GCSswpend
}

/// When main invariant (white objects cannot point to black ones) must be kept.
///
/// During a collection, the sweep phase may break the invariant, as objects turned
/// white may point to still-black objects. The invariant is restored when sweep
/// ends and all objects are white again.
#[inline]
unsafe fn keep_invariant(g: *mut global_State) -> bool {
    (*g).gcstate <= GCSatomic
}

#[inline]
fn resetbits(x: &mut u8, m: u8) {
    *x &= !m;
}

#[inline]
fn setbits(x: &mut u8, m: u8) {
    *x |= m;
}

#[inline]
fn testbits(x: u8, m: u8) -> bool {
    (x & m) != 0
}

#[inline]
const fn bitmask(b: u32) -> u8 {
    1 << b
}

#[inline]
const fn bit2mask(b1: u32, b2: u32) -> u8 {
    bitmask(b1) | bitmask(b2)
}

#[inline]
fn setbit<const B: u32>(x: &mut u8) {
    setbits(x, const { bitmask(B) })
}

#[inline]
fn resetbit<const B: u32>(x: &mut u8) {
    resetbits(x, const { bitmask(B) })
}

#[inline]
fn testbit<const B: u32>(x: u8) -> bool {
    testbits(x, const { bitmask(B) })
}

// Layout for bit use in 'marked' field. First three bits are used for object "age"
// in generational mode. Last bit is used by tests.
const WHITE0BIT: u32 = 3;
const WHITE1BIT: u32 = 4;
const BLACKBIT: u32 = 5;
const FINALIZEDBIT: u32 = 6;

const WHITEBITS: u8 = bit2mask(WHITE0BIT, WHITE1BIT);

pub(super) trait IsGCObject {}

impl IsGCObject for GCObject {}
impl IsGCObject for UpVal {}

#[inline]
unsafe fn gco(ptr: *const impl IsGCObject) -> *mut GCObject {
    ptr.cast_mut().cast()
}

#[inline]
unsafe fn is_white(x: *mut GCObject) -> bool {
    testbits((*x).marked, WHITEBITS)
}

#[inline]
unsafe fn is_black(x: *mut GCObject) -> bool {
    testbit::<BLACKBIT>((*x).marked)
}

#[inline]
unsafe fn is_grey(x: *mut GCObject) -> bool {
    testbits((*x).marked, const { WHITEBITS | bitmask(BLACKBIT) })
}

#[inline]
unsafe fn to_finalize(x: *mut GCObject) -> bool {
    testbit::<FINALIZEDBIT>((*x).marked)
}

unsafe fn luaC_white(g: *mut global_State) -> u8 {
    (*g).currentwhite & WHITEBITS
}

// Object age in generational mode
const G_NEW: u8 = 0;
const G_SURVIVAL: u8 = 1;
const G_OLD0: u8 = 2;
const G_OLD1: u8 = 3;
const G_OLD: u8 = 4;
const G_TOUCHED1: u8 = 5;
const G_TOUCHED2: u8 = 6;

const AGEBITS: u8 = 7;

const MASKCOLORS: u8 = bitmask(BLACKBIT) | WHITEBITS;
const MASKGCBITS: u8 = MASKCOLORS | AGEBITS;

/// erase all color bits then set only the current white bit
#[inline]
unsafe fn makewhite(g: *mut global_State, x: *mut GCObject) {
    (*x).marked = ((*x).marked & !MASKCOLORS) | luaC_white(g);
}

/// make an object gray (neither white nor black)
#[inline]
unsafe fn set2gray(x: *mut GCObject) {
    resetbits(&mut (*x).marked, MASKCOLORS);
}

/// make an object black (coming from any color)
#[inline]
unsafe fn set2black(x: *mut GCObject) {
    (*x).marked = ((*x).marked & !WHITEBITS) | const { bitmask(BLACKBIT) };
}

#[inline]
unsafe fn valiswhite(x: impl TValueFields) -> bool {
    iscollectable(x) && is_white((*x.value()).gc)
}

#[inline]
unsafe fn try_gcvalue(x: impl TValueFields) -> Option<NonNull<GCObject>> {
    if iscollectable(x) {
        Some(unsafe { NonNull::new_unchecked((*x.value()).gc) })
    } else {
        None
    }
}

#[inline]
unsafe fn markvalue(g: *mut global_State, o: impl TValueFields) {
    if valiswhite(o) {
        reallymarkobject(g, (*o.value()).gc);
    }
}

#[inline]
unsafe fn markobject(g: *mut global_State, t: *mut GCObject) {
    if is_white(t) {
        reallymarkobject(g, t);
    }
}

#[inline]
pub(super) unsafe fn luaC_objbarrier(L: *mut lua_State, p: *mut GCObject, o: *mut GCObject) {
    if is_black(p) && is_white(o) {
        luaC_barrier_(L, p, o)
    }
}

/// Barrier that moves collector forward, that is, marks the white object
/// `v` being pointed bt the black object `o`. In the generational
/// mode, `v` must also become old, if `o` is old; however, it cannot
/// be changed directly to OLD, because it may still point to non-old
/// objects. So, it is marked as OLD0. In the next cycle it will become
/// OLD1, and in the next it will finally become OLD (regular old). By
/// then, any object it points to will also be old. If called in the
/// incremental sweep phase, it clears the black object to white (sweep
/// it) to avoid other barrier calls for this same object. (That cannot
/// be done in generational mode, as its sweep does not distinguish
/// whites from deads.)
#[inline]
pub(super) unsafe fn luaC_barrier(L: *mut lua_State, p: *mut impl IsGCObject, v: *mut TValue) {
    if iscollectable(v) {
        luaC_objbarrier(L, gco(p), (*v).value_.gc)
    }
}

#[inline]
pub(super) unsafe fn luaC_objbarrierback(L: *mut lua_State, p: *mut GCObject, o: *mut GCObject) {
    if is_black(p) && is_white(o) {
        luaC_barrierback_(L, p)
    }
}

/// Barrier that moves collector backwards, that is, mark the black object
/// pointing to a white object as gray again.
#[inline]
pub(super) unsafe fn luaC_barrierback(L: *mut lua_State, p: *mut impl IsGCObject, v: *mut TValue) {
    if iscollectable(v) {
        luaC_objbarrierback(L, gco(p), (*v).value_.gc)
    }
}

unsafe extern "C-unwind" fn getgclist(mut o: *mut GCObject) -> *mut *mut GCObject {
    match (*o).tt {
        LUA_VTABLE => return &mut (*&mut (*(o as *mut GCUnion)).h).gclist,
        LUA_VLCL => return &mut (*&mut (*(o as *mut GCUnion)).cl.l).gclist,
        LUA_VCCL => return &mut (*&mut (*(o as *mut GCUnion)).cl.c).gclist,
        LUA_VTHREAD => return &mut (*&mut (*(o as *mut GCUnion)).th).gclist,
        LUA_VPROTO => return &mut (*&mut (*(o as *mut GCUnion)).p).gclist,
        LUA_VUSERDATA => {
            let mut u: *mut Udata = &mut (*(o as *mut GCUnion)).u;
            return &mut (*u).gclist;
        }
        _ => return ptr::null_mut(),
    };
}
unsafe extern "C-unwind" fn linkgclist_(
    mut o: *mut GCObject,
    mut pnext: *mut *mut GCObject,
    mut list: *mut *mut GCObject,
) {
    *pnext = *list;
    *list = o;
    set2gray(o);
}

/// Clear keys for empty entries in tables. If entry is empty, mark its
/// entry as dead. This allows the collectio nof the key, but keeps its
/// entry in the table: its removal could break a chain and could break
/// a table traversal. Other places never manipulate dead keys, because
/// its associated empty value is enough to signal that the entry is
/// logically empty.
unsafe extern "C-unwind" fn clearkey(mut n: *mut Node) {
    if iscollectable(KeyTV(n)) {
        setdeadkey(n)
    }
}

/// Tells whether a key or value can be cleared from a weak
/// table. Non-collectable objects are never removed from weak
/// tables. Strings behave as 'values', so are never removed too. For
/// other objects: if really collected, cannot keep them; for objects
/// being finalized, keep them in keys, but not in values.
unsafe extern "C-unwind" fn iscleared(mut g: *mut global_State, mut o: *const GCObject) -> i32 {
    if o.is_null() {
        return 0;
    } else if novariant((*o).tt) == LUA_TSTRING as u8 {
        // strings are 'values', so are never weak
        markobject(g, o.cast_mut());
        return 0;
    } else {
        return (*o).marked as i32 & ((1 as i32) << 3 as i32 | (1 as i32) << 4 as i32);
    };
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaC_barrier_(
    mut L: *mut lua_State,
    mut o: *mut GCObject,
    mut v: *mut GCObject,
) {
    let mut g: *mut global_State = (*L).l_G;
    if keep_invariant(g) {
        reallymarkobject(g, v);
        // if is_old(o) {
        //   setage(v, G_OLD0)
        // }
        if (*o).marked as i32 & 7 as i32 > 1 as i32 {
            (*v).marked = ((*v).marked as i32 & !(7 as i32) | 2 as i32) as lu_byte;
        }
    } else if (*g).gckind as i32 == 0 {
        makewhite(g, o);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaC_barrierback_(mut L: *mut lua_State, mut o: *mut GCObject) {
    let mut g: *mut global_State = (*L).l_G;
    if (*o).marked as i32 & 7 as i32 == 6 as i32 {
        (*o).marked = ((*o).marked as i32
            & !((1 as i32) << 5 as i32 | ((1 as i32) << 3 as i32 | (1 as i32) << 4 as i32))
                as lu_byte as i32) as lu_byte;
    } else {
        linkgclist_(
            &mut (*(o as *mut GCUnion)).gc,
            getgclist(o),
            &mut (*g).grayagain,
        );
    }
    if (*o).marked as i32 & 7 as i32 > 1 as i32 {
        (*o).marked = ((*o).marked as i32 & !(7 as i32) | 5 as i32) as lu_byte;
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaC_fix(mut L: *mut lua_State, mut o: *mut GCObject) {
    let mut g: *mut global_State = (*L).l_G;
    set2gray(o);
    // setage(o, G_OLD);
    (*o).marked = ((*o).marked as i32 & !(7 as i32) | 4 as i32) as lu_byte;
    (*g).allgc = (*o).next;
    (*o).next = (*g).fixedgc;
    (*g).fixedgc = o;
}

/// Create a new collectable object (with given type, size, and offset)
/// and link it to 'allgc' list.
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaC_newobjdt(
    mut L: *mut lua_State,
    mut tt: i32,
    mut sz: size_t,
    mut offset: size_t,
) -> *mut GCObject {
    let mut g: *mut global_State = (*L).l_G;
    let mut p: *mut std::ffi::c_char =
        luaM_malloc_(L, sz, novariant(tt as u8) as i32) as *mut std::ffi::c_char;
    let mut o: *mut GCObject = p.add(offset) as *mut GCObject;
    (*o).marked = luaC_white(g);
    (*o).tt = tt as lu_byte;
    (*o).next = (*g).allgc;
    (*g).allgc = o;
    return o;
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaC_newobj(
    mut L: *mut lua_State,
    mut tt: i32,
    mut sz: size_t,
) -> *mut GCObject {
    return luaC_newobjdt(L, tt, sz, 0);
}

/// Mark an object. Userdata with no user values, string, and closed
/// upvalues are visited and turned black here. Open upvalues are
/// already indirectly linked through their respective threads in the
/// 'twups' list, so they don't go to the gray list; nevertheless, they
/// are kept gray to avoid barriers, as their values will be revisited
/// by the thread or by 'remarkupvals'. Other objects are added to the
/// gray list to be visited (and turned black) later. Both userdata and
/// upvalues can call this function recursively, but this recursion goes
/// for at most two levels: An upvalue cannot refer to another upvalue
/// (only closures can), and a userdata's metatable must be a table.
unsafe extern "C-unwind" fn reallymarkobject(mut g: *mut global_State, mut o: *mut GCObject) {
    let mut current_block_18: u64;
    match (*o).tt {
        LUA_VSHRSTR | LUA_VLNGSTR => {
            (*o).marked = ((*o).marked as i32 & !((1 as i32) << 3 as i32 | (1 as i32) << 4 as i32)
                | (1 as i32) << 5 as i32) as lu_byte;
            // Nothing to visit
            set2black(o);
            current_block_18 = 18317007320854588510;
        }
        LUA_VUPVAL => {
            let mut uv: *mut UpVal = &mut (*(o as *mut GCUnion)).upv;
            if (*uv).v.p != &mut (*uv).u.value as *mut TValue {
                set2gray(uv.cast());
            } else {
                set2black(uv.cast());
            }
            markvalue(g, (*uv).v.p);
            current_block_18 = 18317007320854588510;
        }
        LUA_VUSERDATA => {
            let mut u: *mut Udata = &mut (*(o as *mut GCUnion)).u;
            if (*u).nuvalue as i32 == 0 {
                if !((*u).metatable).is_null() {
                    markobject(g, (*u).metatable.cast());
                }
                set2black(u.cast());
                current_block_18 = 18317007320854588510;
            } else {
                current_block_18 = 15904375183555213903;
            }
        }
        LUA_VLCL | LUA_VCCL | LUA_VTABLE | LUA_VTHREAD | LUA_VPROTO => {
            current_block_18 = 15904375183555213903;
        }
        _ => {
            current_block_18 = 18317007320854588510;
        }
    }
    match current_block_18 {
        15904375183555213903 => {
            linkgclist_(&mut (*(o as *mut GCUnion)).gc, getgclist(o), &mut (*g).gray);
        }
        _ => {}
    };
}

/// Mark metamethods for basic types.
unsafe extern "C-unwind" fn markmt(mut g: *mut global_State) {
    let mut i: i32 = 0;
    i = 0;
    while i < 9 as i32 {
        let o = (*g).mt[i as usize];
        if !o.is_null() {
            markobject(g, o.cast());
        }
        i += 1;
    }
}

/// Mark all objects in list of being-finalized.
unsafe extern "C-unwind" fn markbeingfnz(mut g: *mut global_State) -> lu_mem {
    let mut o: *mut GCObject = 0 as *mut GCObject;
    let mut count: lu_mem = 0 as lu_mem;
    o = (*g).tobefnz;
    while !o.is_null() {
        count = count.wrapping_add(1);
        markobject(g, o);
        o = (*o).next;
    }
    return count;
}

/// For each non-marked thread, simulates a barrier between each open
/// upvalue and its value. (If the thread is collected, the value will be
/// assigned to the upvalue, but then it can be too late for the barrier
/// to act. The "barrier" does not need to check colors: A non-marked
/// thread must be young; upvalues cannot be older than their threads; so
/// any visited upvalue must be young too.) Also removes the thread from
/// the list, as it was already visited. Removes also threads with no
/// upvalues, as they have nothing to be checked. (If the thread gets an
/// upvalue later, it will be linked in the list again.)
unsafe extern "C-unwind" fn remarkupvals(mut g: *mut global_State) -> i32 {
    let mut thread: *mut lua_State = 0 as *mut lua_State;
    let mut p: *mut *mut lua_State = &mut (*g).twups;
    // estimate of how much work was done here
    let mut work: i32 = 0;
    loop {
        thread = *p;
        if thread.is_null() {
            break;
        }
        work += 1;
        if !is_white(thread.cast()) && !((*thread).openupval).is_null() {
            // keep marked thread with upvalues in the list
            p = &mut (*thread).twups;
        } else {
            // thread is not marked or without upvalues
            let mut uv: *mut UpVal = 0 as *mut UpVal;
            *p = (*thread).twups;
            (*thread).twups = thread;
            uv = (*thread).openupval;
            while !uv.is_null() {
                work += 1;
                if !is_white(uv.cast()) {
                    // upvalue already visited?
                    markvalue(g, (*uv).v.p); // mark its value
                }
                uv = (*uv).u.open.next;
            }
        }
    }
    return work;
}

unsafe extern "C-unwind" fn cleargraylists(mut g: *mut global_State) {
    (*g).grayagain = 0 as *mut GCObject;
    (*g).gray = (*g).grayagain;
    (*g).ephemeron = 0 as *mut GCObject;
    (*g).allweak = (*g).ephemeron;
    (*g).weak = (*g).allweak;
}

/// Mark root set and reset all gray lists, to start a new collection.
unsafe extern "C-unwind" fn restartcollection(mut g: *mut global_State) {
    cleargraylists(g);
    markobject(g, (*g).mainthread.cast());
    markvalue(g, &raw mut (*g).l_registry);
    markmt(g);
    markbeingfnz(g); // mark any finalizing object left from previous cycle
}

unsafe extern "C-unwind" fn genlink(mut g: *mut global_State, mut o: *mut GCObject) {
    if (*o).marked as i32 & 7 as i32 == 5 as i32 {
        linkgclist_(
            &mut (*(o as *mut GCUnion)).gc,
            getgclist(o),
            &mut (*g).grayagain,
        );
    } else if (*o).marked as i32 & 7 as i32 == 6 as i32 {
        (*o).marked = ((*o).marked as i32 ^ (6 as i32 ^ 4 as i32)) as lu_byte;
    }
}
unsafe extern "C-unwind" fn traverseweakvalue(mut g: *mut global_State, mut h: *mut Table) {
    let mut n: *mut Node = 0 as *mut Node;
    let mut limit: *mut Node = &mut *((*h).node)
        .offset(((1 as i32) << (*h).lsizenode as i32) as size_t as isize)
        as *mut Node;
    let mut hasclears: i32 = ((*h).alimit > 0 as u32) as i32;
    n = &mut *((*h).node).offset(0 as isize) as *mut Node;
    while n < limit {
        if (*n).i_val.tt_ as i32 & 0xf as i32 == 0 {
            clearkey(n);
        } else {
            if (*n).u.key_tt as i32 & (1 as i32) << 6 as i32 != 0
                && (*(*n).u.key_val.gc).marked as i32
                    & ((1 as i32) << 3 as i32 | (1 as i32) << 4 as i32)
                    != 0
            {
                reallymarkobject(g, (*n).u.key_val.gc);
            }
            if hasclears == 0
                && iscleared(
                    g,
                    (if (*n).i_val.tt_ as i32 & (1 as i32) << 6 as i32 != 0 {
                        (*n).i_val.value_.gc
                    } else {
                        0 as *mut GCObject
                    }),
                ) != 0
            {
                hasclears = 1 as i32;
            }
        }
        n = n.offset(1);
        n;
    }
    if (*g).gcstate as i32 == 2 as i32 && hasclears != 0 {
        linkgclist_(
            &mut (*(h as *mut GCUnion)).gc,
            &mut (*h).gclist,
            &mut (*g).weak,
        );
    } else {
        linkgclist_(
            &mut (*(h as *mut GCUnion)).gc,
            &mut (*h).gclist,
            &mut (*g).grayagain,
        );
    };
}
unsafe extern "C-unwind" fn traverseephemeron(
    mut g: *mut global_State,
    mut h: *mut Table,
    mut inv: i32,
) -> i32 {
    let mut marked: i32 = 0;
    let mut hasclears: i32 = 0;
    let mut hasww: i32 = 0;
    let mut i: u32 = 0;
    let mut asize: u32 = luaH_realasize(h);
    let mut nsize: u32 = ((1 as i32) << (*h).lsizenode as i32) as u32;
    i = 0 as u32;
    while i < asize {
        if (*((*h).array).offset(i as isize)).tt_ as i32 & (1 as i32) << 6 as i32 != 0
            && (*(*((*h).array).offset(i as isize)).value_.gc).marked as i32
                & ((1 as i32) << 3 as i32 | (1 as i32) << 4 as i32)
                != 0
        {
            marked = 1 as i32;
            reallymarkobject(g, (*((*h).array).offset(i as isize)).value_.gc);
        }
        i = i.wrapping_add(1);
        i;
    }
    i = 0 as u32;
    while i < nsize {
        let mut n: *mut Node = if inv != 0 {
            &mut *((*h).node).offset(nsize.wrapping_sub(1).wrapping_sub(i) as isize) as *mut Node
        } else {
            &mut *((*h).node).offset(i as isize) as *mut Node
        };
        if (*n).i_val.tt_ as i32 & 0xf as i32 == 0 {
            clearkey(n);
        } else if iscleared(
            g,
            if (*n).u.key_tt as i32 & (1 as i32) << 6 as i32 != 0 {
                (*n).u.key_val.gc
            } else {
                0 as *mut GCObject
            },
        ) != 0
        {
            hasclears = 1 as i32;
            if (*n).i_val.tt_ as i32 & (1 as i32) << 6 as i32 != 0
                && (*(*n).i_val.value_.gc).marked as i32
                    & ((1 as i32) << 3 as i32 | (1 as i32) << 4 as i32)
                    != 0
            {
                hasww = 1 as i32;
            }
        } else if (*n).i_val.tt_ as i32 & (1 as i32) << 6 as i32 != 0
            && (*(*n).i_val.value_.gc).marked as i32
                & ((1 as i32) << 3 as i32 | (1 as i32) << 4 as i32)
                != 0
        {
            marked = 1 as i32;
            reallymarkobject(g, (*n).i_val.value_.gc);
        }
        i = i.wrapping_add(1);
        i;
    }
    if (*g).gcstate as i32 == 0 {
        linkgclist_(
            &mut (*(h as *mut GCUnion)).gc,
            &mut (*h).gclist,
            &mut (*g).grayagain,
        );
    } else if hasww != 0 {
        linkgclist_(
            &mut (*(h as *mut GCUnion)).gc,
            &mut (*h).gclist,
            &mut (*g).ephemeron,
        );
    } else if hasclears != 0 {
        linkgclist_(
            &mut (*(h as *mut GCUnion)).gc,
            &mut (*h).gclist,
            &mut (*g).allweak,
        );
    } else {
        genlink(g, &mut (*(h as *mut GCUnion)).gc);
    }
    return marked;
}
unsafe extern "C-unwind" fn traversestrongtable(mut g: *mut global_State, mut h: *mut Table) {
    let mut n: *mut Node = 0 as *mut Node;
    let mut limit: *mut Node = &mut *((*h).node)
        .offset(((1 as i32) << (*h).lsizenode as i32) as size_t as isize)
        as *mut Node;
    let mut i: u32 = 0;
    let mut asize: u32 = luaH_realasize(h);
    i = 0 as u32;
    while i < asize {
        if (*((*h).array).offset(i as isize)).tt_ as i32 & (1 as i32) << 6 as i32 == 0
            || (*((*h).array).offset(i as isize)).tt_ as i32 & 0x3f as i32
                == (*(*((*h).array).offset(i as isize)).value_.gc).tt as i32
                && (((*g).mainthread).is_null()
                    || (*(*((*h).array).offset(i as isize)).value_.gc).marked as i32
                        & ((*(*(*g).mainthread).l_G).currentwhite as i32
                            ^ ((1 as i32) << 3 as i32 | (1 as i32) << 4 as i32))
                        == 0)
        {
        } else {
        };
        if (*((*h).array).offset(i as isize)).tt_ as i32 & (1 as i32) << 6 as i32 != 0
            && (*(*((*h).array).offset(i as isize)).value_.gc).marked as i32
                & ((1 as i32) << 3 as i32 | (1 as i32) << 4 as i32)
                != 0
        {
            reallymarkobject(g, (*((*h).array).offset(i as isize)).value_.gc);
        }
        i = i.wrapping_add(1);
        i;
    }
    n = &mut *((*h).node).offset(0 as isize) as *mut Node;
    while n < limit {
        if (*n).i_val.tt_ as i32 & 0xf as i32 == 0 {
            clearkey(n);
        } else {
            if (*n).u.key_tt as i32 & (1 as i32) << 6 as i32 != 0
                && (*(*n).u.key_val.gc).marked as i32
                    & ((1 as i32) << 3 as i32 | (1 as i32) << 4 as i32)
                    != 0
            {
                reallymarkobject(g, (*n).u.key_val.gc);
            }
            if (*n).i_val.tt_ as i32 & (1 as i32) << 6 as i32 == 0
                || (*n).i_val.tt_ as i32 & 0x3f as i32 == (*(*n).i_val.value_.gc).tt as i32
                    && (((*g).mainthread).is_null()
                        || (*(*n).i_val.value_.gc).marked as i32
                            & ((*(*(*g).mainthread).l_G).currentwhite as i32
                                ^ ((1 as i32) << 3 as i32 | (1 as i32) << 4 as i32))
                            == 0)
            {
            } else {
            };
            if (*n).i_val.tt_ as i32 & (1 as i32) << 6 as i32 != 0
                && (*(*n).i_val.value_.gc).marked as i32
                    & ((1 as i32) << 3 as i32 | (1 as i32) << 4 as i32)
                    != 0
            {
                reallymarkobject(g, (*n).i_val.value_.gc);
            }
        }
        n = n.offset(1);
        n;
    }
    genlink(g, &mut (*(h as *mut GCUnion)).gc);
}
unsafe extern "C-unwind" fn traversetable(mut g: *mut global_State, mut h: *mut Table) -> lu_mem {
    let mut weakkey: *const std::ffi::c_char = 0 as *const std::ffi::c_char;
    let mut weakvalue: *const std::ffi::c_char = 0 as *const std::ffi::c_char;
    let mut mode: *const TValue = if ((*h).metatable).is_null() {
        0 as *const TValue
    } else if (*(*h).metatable).flags as u32 & (1 as u32) << TM_MODE as i32 != 0 {
        0 as *const TValue
    } else {
        luaT_gettm(
            (*h).metatable,
            TM_MODE,
            (*g).tmname[TM_MODE as i32 as usize],
        )
    };
    let mut smode: *mut TString = 0 as *mut TString;
    if !((*h).metatable).is_null() {
        if (*(*h).metatable).marked as i32 & ((1 as i32) << 3 as i32 | (1 as i32) << 4 as i32) != 0
        {
            reallymarkobject(g, &mut (*((*h).metatable as *mut GCUnion)).gc);
        }
    }
    if !mode.is_null()
        && (*mode).tt_ as i32 == 4 as i32 | (0) << 4 as i32 | (1 as i32) << 6 as i32
        && {
            smode = &mut (*((*mode).value_.gc as *mut GCUnion)).ts as *mut TString;
            weakkey = strchr(((*smode).contents).as_mut_ptr(), 'k' as i32);
            weakvalue = strchr(((*smode).contents).as_mut_ptr(), 'v' as i32);
            !weakkey.is_null() || !weakvalue.is_null()
        }
    {
        if weakkey.is_null() {
            traverseweakvalue(g, h);
        } else if weakvalue.is_null() {
            traverseephemeron(g, h, 0);
        } else {
            linkgclist_(
                &mut (*(h as *mut GCUnion)).gc,
                &mut (*h).gclist,
                &mut (*g).allweak,
            );
        }
    } else {
        traversestrongtable(g, h);
    }
    return (1u32).wrapping_add((*h).alimit).wrapping_add(
        (2 as i32
            * (if ((*h).lastfree).is_null() {
                0
            } else {
                (1 as i32) << (*h).lsizenode as i32
            })) as u32,
    ) as lu_mem;
}
unsafe extern "C-unwind" fn traverseudata(mut g: *mut global_State, mut u: *mut Udata) -> i32 {
    let mut i: i32 = 0;
    if !((*u).metatable).is_null() {
        if (*(*u).metatable).marked as i32 & ((1 as i32) << 3 as i32 | (1 as i32) << 4 as i32) != 0
        {
            reallymarkobject(g, &mut (*((*u).metatable as *mut GCUnion)).gc);
        }
    }
    i = 0;
    while i < (*u).nuvalue as i32 {
        if (*((*u).uv).as_mut_ptr().offset(i as isize)).uv.tt_ as i32 & (1 as i32) << 6 as i32 == 0
            || (*((*u).uv).as_mut_ptr().offset(i as isize)).uv.tt_ as i32 & 0x3f as i32
                == (*(*((*u).uv).as_mut_ptr().offset(i as isize)).uv.value_.gc).tt as i32
                && (((*g).mainthread).is_null()
                    || (*(*((*u).uv).as_mut_ptr().offset(i as isize)).uv.value_.gc).marked as i32
                        & ((*(*(*g).mainthread).l_G).currentwhite as i32
                            ^ ((1 as i32) << 3 as i32 | (1 as i32) << 4 as i32))
                        == 0)
        {
        } else {
        };
        if (*((*u).uv).as_mut_ptr().offset(i as isize)).uv.tt_ as i32 & (1 as i32) << 6 as i32 != 0
            && (*(*((*u).uv).as_mut_ptr().offset(i as isize)).uv.value_.gc).marked as i32
                & ((1 as i32) << 3 as i32 | (1 as i32) << 4 as i32)
                != 0
        {
            reallymarkobject(g, (*((*u).uv).as_mut_ptr().offset(i as isize)).uv.value_.gc);
        }
        i += 1;
        i;
    }
    genlink(g, &mut (*(u as *mut GCUnion)).gc);
    return 1 as i32 + (*u).nuvalue as i32;
}
unsafe extern "C-unwind" fn traverseproto(mut g: *mut global_State, mut f: *mut Proto) -> i32 {
    let mut i: i32 = 0;
    if !((*f).source).is_null() {
        if (*(*f).source).marked as i32 & ((1 as i32) << 3 as i32 | (1 as i32) << 4 as i32) != 0 {
            reallymarkobject(g, &mut (*((*f).source as *mut GCUnion)).gc);
        }
    }
    i = 0;
    while i < (*f).sizek {
        if (*((*f).k).offset(i as isize)).tt_ as i32 & (1 as i32) << 6 as i32 == 0
            || (*((*f).k).offset(i as isize)).tt_ as i32 & 0x3f as i32
                == (*(*((*f).k).offset(i as isize)).value_.gc).tt as i32
                && (((*g).mainthread).is_null()
                    || (*(*((*f).k).offset(i as isize)).value_.gc).marked as i32
                        & ((*(*(*g).mainthread).l_G).currentwhite as i32
                            ^ ((1 as i32) << 3 as i32 | (1 as i32) << 4 as i32))
                        == 0)
        {
        } else {
        };
        if (*((*f).k).offset(i as isize)).tt_ as i32 & (1 as i32) << 6 as i32 != 0
            && (*(*((*f).k).offset(i as isize)).value_.gc).marked as i32
                & ((1 as i32) << 3 as i32 | (1 as i32) << 4 as i32)
                != 0
        {
            reallymarkobject(g, (*((*f).k).offset(i as isize)).value_.gc);
        }
        i += 1;
        i;
    }
    i = 0;
    while i < (*f).sizeupvalues {
        if !((*((*f).upvalues).offset(i as isize)).name).is_null() {
            if (*(*((*f).upvalues).offset(i as isize)).name).marked as i32
                & ((1 as i32) << 3 as i32 | (1 as i32) << 4 as i32)
                != 0
            {
                reallymarkobject(
                    g,
                    &mut (*((*((*f).upvalues).offset(i as isize)).name as *mut GCUnion)).gc,
                );
            }
        }
        i += 1;
        i;
    }
    i = 0;
    while i < (*f).sizep {
        if !(*((*f).p).offset(i as isize)).is_null() {
            if (**((*f).p).offset(i as isize)).marked as i32
                & ((1 as i32) << 3 as i32 | (1 as i32) << 4 as i32)
                != 0
            {
                reallymarkobject(g, &mut (*(*((*f).p).offset(i as isize) as *mut GCUnion)).gc);
            }
        }
        i += 1;
        i;
    }
    i = 0;
    while i < (*f).sizelocvars {
        if !((*((*f).locvars).offset(i as isize)).varname).is_null() {
            if (*(*((*f).locvars).offset(i as isize)).varname).marked as i32
                & ((1 as i32) << 3 as i32 | (1 as i32) << 4 as i32)
                != 0
            {
                reallymarkobject(
                    g,
                    &mut (*((*((*f).locvars).offset(i as isize)).varname as *mut GCUnion)).gc,
                );
            }
        }
        i += 1;
        i;
    }
    return 1 as i32 + (*f).sizek + (*f).sizeupvalues + (*f).sizep + (*f).sizelocvars;
}
unsafe extern "C-unwind" fn traverseCclosure(
    mut g: *mut global_State,
    mut cl: *mut CClosure,
) -> i32 {
    let mut i: i32 = 0;
    i = 0;
    while i < (*cl).nupvalues as i32 {
        if (*((*cl).upvalue).as_mut_ptr().offset(i as isize)).tt_ as i32 & (1 as i32) << 6 as i32
            == 0
            || (*((*cl).upvalue).as_mut_ptr().offset(i as isize)).tt_ as i32 & 0x3f as i32
                == (*(*((*cl).upvalue).as_mut_ptr().offset(i as isize)).value_.gc).tt as i32
                && (((*g).mainthread).is_null()
                    || (*(*((*cl).upvalue).as_mut_ptr().offset(i as isize)).value_.gc).marked
                        as i32
                        & ((*(*(*g).mainthread).l_G).currentwhite as i32
                            ^ ((1 as i32) << 3 as i32 | (1 as i32) << 4 as i32))
                        == 0)
        {
        } else {
        };
        if (*((*cl).upvalue).as_mut_ptr().offset(i as isize)).tt_ as i32 & (1 as i32) << 6 as i32
            != 0
            && (*(*((*cl).upvalue).as_mut_ptr().offset(i as isize)).value_.gc).marked as i32
                & ((1 as i32) << 3 as i32 | (1 as i32) << 4 as i32)
                != 0
        {
            reallymarkobject(
                g,
                (*((*cl).upvalue).as_mut_ptr().offset(i as isize)).value_.gc,
            );
        }
        i += 1;
        i;
    }
    return 1 as i32 + (*cl).nupvalues as i32;
}
unsafe extern "C-unwind" fn traverseLclosure(
    mut g: *mut global_State,
    mut cl: *mut LClosure,
) -> i32 {
    let mut i: i32 = 0;
    if !((*cl).p).is_null() {
        if (*(*cl).p).marked as i32 & ((1 as i32) << 3 as i32 | (1 as i32) << 4 as i32) != 0 {
            reallymarkobject(g, &mut (*((*cl).p as *mut GCUnion)).gc);
        }
    }
    i = 0;
    while i < (*cl).nupvalues as i32 {
        let mut uv: *mut UpVal = *((*cl).upvals).as_mut_ptr().offset(i as isize);
        if !uv.is_null() {
            if (*uv).marked as i32 & ((1 as i32) << 3 as i32 | (1 as i32) << 4 as i32) != 0 {
                reallymarkobject(g, &mut (*(uv as *mut GCUnion)).gc);
            }
        }
        i += 1;
        i;
    }
    return 1 as i32 + (*cl).nupvalues as i32;
}
unsafe extern "C-unwind" fn traversethread(
    mut g: *mut global_State,
    mut th: *mut lua_State,
) -> i32 {
    let mut uv: *mut UpVal = 0 as *mut UpVal;
    let mut o: StkId = (*th).stack.p;
    if (*th).marked as i32 & 7 as i32 > 1 as i32 || (*g).gcstate as i32 == 0 {
        linkgclist_(
            &mut (*(th as *mut GCUnion)).gc,
            &mut (*th).gclist,
            &mut (*g).grayagain,
        );
    }
    if o.is_null() {
        return 1 as i32;
    }
    while o < (*th).top.p {
        if (*o).val.tt_ as i32 & (1 as i32) << 6 as i32 == 0
            || (*o).val.tt_ as i32 & 0x3f as i32 == (*(*o).val.value_.gc).tt as i32
                && (((*g).mainthread).is_null()
                    || (*(*o).val.value_.gc).marked as i32
                        & ((*(*(*g).mainthread).l_G).currentwhite as i32
                            ^ ((1 as i32) << 3 as i32 | (1 as i32) << 4 as i32))
                        == 0)
        {
        } else {
        };
        if (*o).val.tt_ as i32 & (1 as i32) << 6 as i32 != 0
            && (*(*o).val.value_.gc).marked as i32
                & ((1 as i32) << 3 as i32 | (1 as i32) << 4 as i32)
                != 0
        {
            reallymarkobject(g, (*o).val.value_.gc);
        }
        o = o.offset(1);
        o;
    }
    uv = (*th).openupval;
    while !uv.is_null() {
        if (*uv).marked as i32 & ((1 as i32) << 3 as i32 | (1 as i32) << 4 as i32) != 0 {
            reallymarkobject(g, &mut (*(uv as *mut GCUnion)).gc);
        }
        uv = (*uv).u.open.next;
    }
    if (*g).gcstate as i32 == 2 as i32 {
        if (*g).gcemergency == 0 {
            luaD_shrinkstack(th);
        }
        o = (*th).top.p;
        while o < ((*th).stack_last.p).offset(5) {
            (*o).val.tt_ = (0 | (0) << 4 as i32) as lu_byte;
            o = o.offset(1);
            o;
        }
        if !((*th).twups != th) && !((*th).openupval).is_null() {
            (*th).twups = (*g).twups;
            (*g).twups = th;
        }
    }
    return 1 as i32 + ((*th).stack_last.p).offset_from((*th).stack.p) as std::ffi::c_long as i32;
}
unsafe extern "C-unwind" fn propagatemark(mut g: *mut global_State) -> lu_mem {
    let mut o: *mut GCObject = (*g).gray;
    (*o).marked = ((*o).marked as i32 | (1 as i32) << 5 as i32) as lu_byte;
    (*g).gray = *getgclist(o);
    match (*o).tt {
        LUA_VTABLE => return traversetable(g, &mut (*(o as *mut GCUnion)).h),
        LUA_VUSERDATA => return traverseudata(g, &mut (*(o as *mut GCUnion)).u) as lu_mem,
        LUA_VLCL => return traverseLclosure(g, &mut (*(o as *mut GCUnion)).cl.l) as lu_mem,
        LUA_VCCL => return traverseCclosure(g, &mut (*(o as *mut GCUnion)).cl.c) as lu_mem,
        LUA_VPROTO => return traverseproto(g, &mut (*(o as *mut GCUnion)).p) as lu_mem,
        LUA_VTHREAD => return traversethread(g, &mut (*(o as *mut GCUnion)).th) as lu_mem,
        _ => return 0 as lu_mem,
    };
}
unsafe extern "C-unwind" fn propagateall(mut g: *mut global_State) -> lu_mem {
    let mut tot: lu_mem = 0 as lu_mem;
    while !((*g).gray).is_null() {
        tot = tot.wrapping_add(propagatemark(g));
    }
    return tot;
}
unsafe extern "C-unwind" fn convergeephemerons(mut g: *mut global_State) {
    let mut changed: i32 = 0;
    let mut dir: i32 = 0;
    loop {
        let mut w: *mut GCObject = 0 as *mut GCObject;
        let mut next: *mut GCObject = (*g).ephemeron;
        (*g).ephemeron = 0 as *mut GCObject;
        changed = 0;
        loop {
            w = next;
            if w.is_null() {
                break;
            }
            let mut h: *mut Table = &mut (*(w as *mut GCUnion)).h;
            next = (*h).gclist;
            (*h).marked = ((*h).marked as i32 | (1 as i32) << 5 as i32) as lu_byte;
            if traverseephemeron(g, h, dir) != 0 {
                propagateall(g);
                changed = 1 as i32;
            }
        }
        dir = (dir == 0) as i32;
        if !(changed != 0) {
            break;
        }
    }
}
unsafe extern "C-unwind" fn clearbykeys(mut g: *mut global_State, mut l: *mut GCObject) {
    while !l.is_null() {
        let mut h: *mut Table = &mut (*(l as *mut GCUnion)).h;
        let mut limit: *mut Node = &mut *((*h).node)
            .offset(((1 as i32) << (*h).lsizenode as i32) as size_t as isize)
            as *mut Node;
        let mut n: *mut Node = 0 as *mut Node;
        n = &mut *((*h).node).offset(0 as isize) as *mut Node;
        while n < limit {
            if iscleared(
                g,
                if (*n).u.key_tt as i32 & (1 as i32) << 6 as i32 != 0 {
                    (*n).u.key_val.gc
                } else {
                    0 as *mut GCObject
                },
            ) != 0
            {
                (*n).i_val.tt_ = (0 | (1 as i32) << 4 as i32) as lu_byte;
            }
            if (*n).i_val.tt_ as i32 & 0xf as i32 == 0 {
                clearkey(n);
            }
            n = n.offset(1);
            n;
        }
        l = (*&mut (*(l as *mut GCUnion)).h).gclist;
    }
}
unsafe extern "C-unwind" fn clearbyvalues(
    mut g: *mut global_State,
    mut l: *mut GCObject,
    mut f: *mut GCObject,
) {
    while l != f {
        let mut h: *mut Table = &mut (*(l as *mut GCUnion)).h;
        let mut n: *mut Node = 0 as *mut Node;
        let mut limit: *mut Node = &mut *((*h).node)
            .offset(((1 as i32) << (*h).lsizenode as i32) as size_t as isize)
            as *mut Node;
        let mut i: u32 = 0;
        let mut asize: u32 = luaH_realasize(h);
        i = 0 as u32;
        while i < asize {
            let mut o: *mut TValue = &mut *((*h).array).offset(i as isize) as *mut TValue;
            if iscleared(
                g,
                if (*o).tt_ as i32 & (1 as i32) << 6 as i32 != 0 {
                    (*o).value_.gc
                } else {
                    0 as *mut GCObject
                },
            ) != 0
            {
                (*o).tt_ = (0 | (1 as i32) << 4 as i32) as lu_byte;
            }
            i = i.wrapping_add(1);
            i;
        }
        n = &mut *((*h).node).offset(0 as isize) as *mut Node;
        while n < limit {
            if iscleared(
                g,
                if (*n).i_val.tt_ as i32 & (1 as i32) << 6 as i32 != 0 {
                    (*n).i_val.value_.gc
                } else {
                    0 as *mut GCObject
                },
            ) != 0
            {
                (*n).i_val.tt_ = (0 | (1 as i32) << 4 as i32) as lu_byte;
            }
            if (*n).i_val.tt_ as i32 & 0xf as i32 == 0 {
                clearkey(n);
            }
            n = n.offset(1);
            n;
        }
        l = (*&mut (*(l as *mut GCUnion)).h).gclist;
    }
}
unsafe extern "C-unwind" fn freeupval(mut L: *mut lua_State, mut uv: *mut UpVal) {
    if (*uv).v.p != &mut (*uv).u.value as *mut TValue {
        luaF_unlinkupval(uv);
    }
    luaM_free_(L, uv as *mut c_void, size_of::<UpVal>() as usize);
}
unsafe extern "C-unwind" fn freeobj(mut L: *mut lua_State, mut o: *mut GCObject) {
    match (*o).tt {
        LUA_VPROTO => {
            luaF_freeproto(L, &mut (*(o as *mut GCUnion)).p);
        }
        LUA_VUPVAL => {
            freeupval(L, &mut (*(o as *mut GCUnion)).upv);
        }
        LUA_VLCL => {
            let mut cl: *mut LClosure = &mut (*(o as *mut GCUnion)).cl.l;
            luaM_free_(
                L,
                cl as *mut c_void,
                (32 as usize as i32
                    + size_of::<*mut TValue>() as usize as i32 * (*cl).nupvalues as i32)
                    as size_t,
            );
        }
        LUA_VCCL => {
            let mut cl_0: *mut CClosure = &mut (*(o as *mut GCUnion)).cl.c;
            luaM_free_(
                L,
                cl_0 as *mut c_void,
                (32 as usize as i32
                    + size_of::<TValue>() as usize as i32 * (*cl_0).nupvalues as i32)
                    as size_t,
            );
        }
        LUA_VTABLE => {
            luaH_free(L, &mut (*(o as *mut GCUnion)).h);
        }
        LUA_VTHREAD => {
            luaE_freethread(L, &mut (*(o as *mut GCUnion)).th);
        }
        LUA_VUSERDATA => {
            let mut u: *mut Udata = &mut (*(o as *mut GCUnion)).u;
            luaM_free_(
                L,
                o as *mut c_void,
                (if (*u).nuvalue as i32 == 0 {
                    32 as usize
                } else {
                    (40 as usize).wrapping_add(
                        (size_of::<UValue>() as usize).wrapping_mul((*u).nuvalue as usize),
                    )
                })
                .wrapping_add((*u).len),
            );
        }
        LUA_VSHRSTR => {
            let mut ts: *mut TString = &mut (*(o as *mut GCUnion)).ts;
            luaS_remove(L, ts);
            luaM_free_(
                L,
                ts as *mut c_void,
                (24 as usize).wrapping_add(
                    (((*ts).shrlen as i32 + 1 as i32) as usize)
                        .wrapping_mul(size_of::<std::ffi::c_char>() as usize),
                ),
            );
        }
        LUA_VLNGSTR => {
            let mut ts_0: *mut TString = &mut (*(o as *mut GCUnion)).ts;
            luaM_free_(
                L,
                ts_0 as *mut c_void,
                (24 as usize).wrapping_add(
                    ((*ts_0).u.lnglen)
                        .wrapping_add(1 as i32 as size_t)
                        .wrapping_mul(size_of::<std::ffi::c_char>() as usize),
                ),
            );
        }
        _ => {}
    };
}
unsafe extern "C-unwind" fn sweeplist(
    mut L: *mut lua_State,
    mut p: *mut *mut GCObject,
    mut countin: i32,
    mut countout: *mut i32,
) -> *mut *mut GCObject {
    let mut g: *mut global_State = (*L).l_G;
    let mut ow: i32 = (*g).currentwhite as i32 ^ ((1 as i32) << 3 as i32 | (1 as i32) << 4 as i32);
    let mut i: i32 = 0;
    let mut white: i32 = ((*g).currentwhite as i32
        & ((1 as i32) << 3 as i32 | (1 as i32) << 4 as i32)) as lu_byte
        as i32;
    i = 0;
    while !(*p).is_null() && i < countin {
        let mut curr: *mut GCObject = *p;
        let mut marked: i32 = (*curr).marked as i32;
        if marked & ow != 0 {
            *p = (*curr).next;
            freeobj(L, curr);
        } else {
            (*curr).marked = (marked
                & !((1 as i32) << 5 as i32
                    | ((1 as i32) << 3 as i32 | (1 as i32) << 4 as i32)
                    | 7 as i32)
                | white) as lu_byte;
            p = &mut (*curr).next;
        }
        i += 1;
        i;
    }
    if !countout.is_null() {
        *countout = i;
    }
    return if (*p).is_null() {
        0 as *mut *mut GCObject
    } else {
        p
    };
}
unsafe extern "C-unwind" fn sweeptolive(
    mut L: *mut lua_State,
    mut p: *mut *mut GCObject,
) -> *mut *mut GCObject {
    let mut old: *mut *mut GCObject = p;
    loop {
        p = sweeplist(L, p, 1 as i32, 0 as *mut i32);
        if !(p == old) {
            break;
        }
    }
    return p;
}
unsafe extern "C-unwind" fn checkSizes(mut L: *mut lua_State, mut g: *mut global_State) {
    if (*g).gcemergency == 0 {
        if (*g).strt.nuse < (*g).strt.size / 4 as i32 {
            let mut olddebt: l_mem = (*g).GCdebt;
            luaS_resize(L, (*g).strt.size / 2 as i32);
            (*g).GCestimate = ((*g).GCestimate).wrapping_add(((*g).GCdebt - olddebt) as lu_mem);
        }
    }
}
unsafe extern "C-unwind" fn udata2finalize(mut g: *mut global_State) -> *mut GCObject {
    let mut o: *mut GCObject = (*g).tobefnz;
    (*g).tobefnz = (*o).next;
    (*o).next = (*g).allgc;
    (*g).allgc = o;
    (*o).marked = ((*o).marked as i32 & !((1 as i32) << 6 as i32) as lu_byte as i32) as lu_byte;
    if 3 as i32 <= (*g).gcstate as i32 && (*g).gcstate as i32 <= 6 as i32 {
        (*o).marked = ((*o).marked as i32
            & !((1 as i32) << 5 as i32 | ((1 as i32) << 3 as i32 | (1 as i32) << 4 as i32))
            | ((*g).currentwhite as i32 & ((1 as i32) << 3 as i32 | (1 as i32) << 4 as i32))
                as lu_byte as i32) as lu_byte;
    } else if (*o).marked as i32 & 7 as i32 == 3 as i32 {
        (*g).firstold1 = o;
    }
    return o;
}
unsafe extern "C-unwind" fn dothecall(mut L: *mut lua_State, mut ud: *mut c_void) {
    luaD_callnoyield(L, ((*L).top.p).offset(-(2)), 0);
}
unsafe extern "C-unwind" fn GCTM(mut L: *mut lua_State) {
    let mut g: *mut global_State = (*L).l_G;
    let mut tm: *const TValue = 0 as *const TValue;
    let mut v: TValue = TValue {
        value_: Value {
            gc: 0 as *mut GCObject,
        },
        tt_: 0,
    };
    let mut io: *mut TValue = &mut v;
    let mut i_g: *mut GCObject = udata2finalize(g);
    (*io).value_.gc = i_g;
    (*io).tt_ = ((*i_g).tt as i32 | (1 as i32) << 6 as i32) as lu_byte;
    tm = luaT_gettmbyobj(L, &mut v, TM_GC);
    if !((*tm).tt_ as i32 & 0xf as i32 == 0) {
        let mut status: i32 = 0;
        let mut oldah: lu_byte = (*L).allowhook;
        let mut oldgcstp: i32 = (*g).gcstp as i32;
        (*g).gcstp = ((*g).gcstp as i32 | 2 as i32) as lu_byte;
        (*L).allowhook = 0 as lu_byte;
        let fresh9 = (*L).top.p;
        (*L).top.p = ((*L).top.p).offset(1);
        let mut io1: *mut TValue = &mut (*fresh9).val;
        let mut io2: *const TValue = tm;
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
        let fresh10 = (*L).top.p;
        (*L).top.p = ((*L).top.p).offset(1);
        let mut io1_0: *mut TValue = &mut (*fresh10).val;
        let mut io2_0: *const TValue = &mut v;
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
        (*(*L).ci).callstatus = ((*(*L).ci).callstatus as i32 | (1 as i32) << 7 as i32) as u16;
        status = luaD_pcall(
            L,
            Some(dothecall as unsafe extern "C-unwind" fn(*mut lua_State, *mut c_void) -> ()),
            0 as *mut c_void,
            (((*L).top.p).offset(-(2)) as *mut std::ffi::c_char)
                .offset_from((*L).stack.p as *mut std::ffi::c_char),
            0 as ptrdiff_t,
        );
        (*(*L).ci).callstatus = ((*(*L).ci).callstatus as i32 & !((1 as i32) << 7 as i32)) as u16;
        (*L).allowhook = oldah;
        (*g).gcstp = oldgcstp as lu_byte;
        if ((status != 0) as i32 != 0) as i32 as std::ffi::c_long != 0 {
            luaE_warnerror(L, c"__gc".as_ptr());
            (*L).top.p = ((*L).top.p).offset(-1);
            (*L).top.p;
        }
    }
}
unsafe extern "C-unwind" fn runafewfinalizers(mut L: *mut lua_State, mut n: i32) -> i32 {
    let mut g: *mut global_State = (*L).l_G;
    let mut i: i32 = 0;
    i = 0;
    while i < n && !((*g).tobefnz).is_null() {
        GCTM(L);
        i += 1;
        i;
    }
    return i;
}
unsafe extern "C-unwind" fn callallpendingfinalizers(mut L: *mut lua_State) {
    let mut g: *mut global_State = (*L).l_G;
    while !((*g).tobefnz).is_null() {
        GCTM(L);
    }
}
unsafe extern "C-unwind" fn findlast(mut p: *mut *mut GCObject) -> *mut *mut GCObject {
    while !(*p).is_null() {
        p = &mut (**p).next;
    }
    return p;
}
unsafe extern "C-unwind" fn separatetobefnz(mut g: *mut global_State, mut all: i32) {
    let mut curr: *mut GCObject = 0 as *mut GCObject;
    let mut p: *mut *mut GCObject = &mut (*g).finobj;
    let mut lastnext: *mut *mut GCObject = findlast(&mut (*g).tobefnz);
    loop {
        curr = *p;
        if !(curr != (*g).finobjold1) {
            break;
        }
        if !((*curr).marked as i32 & ((1 as i32) << 3 as i32 | (1 as i32) << 4 as i32) != 0
            || all != 0)
        {
            p = &mut (*curr).next;
        } else {
            if curr == (*g).finobjsur {
                (*g).finobjsur = (*curr).next;
            }
            *p = (*curr).next;
            (*curr).next = *lastnext;
            *lastnext = curr;
            lastnext = &mut (*curr).next;
        }
    }
}
unsafe extern "C-unwind" fn checkpointer(mut p: *mut *mut GCObject, mut o: *mut GCObject) {
    if o == *p {
        *p = (*o).next;
    }
}
unsafe extern "C-unwind" fn correctpointers(mut g: *mut global_State, mut o: *mut GCObject) {
    checkpointer(&mut (*g).survival, o);
    checkpointer(&mut (*g).old1, o);
    checkpointer(&mut (*g).reallyold, o);
    checkpointer(&mut (*g).firstold1, o);
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaC_checkfinalizer(
    mut L: *mut lua_State,
    mut o: *mut GCObject,
    mut mt: *mut Table,
) {
    let mut g: *mut global_State = (*L).l_G;
    if (*o).marked as i32 & (1 as i32) << 6 as i32 != 0
        || (if mt.is_null() {
            0 as *const TValue
        } else {
            (if (*mt).flags as u32 & (1 as u32) << TM_GC as i32 != 0 {
                0 as *const TValue
            } else {
                luaT_gettm(mt, TM_GC, (*g).tmname[TM_GC as i32 as usize])
            })
        })
        .is_null()
        || (*g).gcstp as i32 & 4 as i32 != 0
    {
        return;
    } else {
        let mut p: *mut *mut GCObject = 0 as *mut *mut GCObject;
        if 3 as i32 <= (*g).gcstate as i32 && (*g).gcstate as i32 <= 6 as i32 {
            (*o).marked = ((*o).marked as i32
                & !((1 as i32) << 5 as i32 | ((1 as i32) << 3 as i32 | (1 as i32) << 4 as i32))
                | ((*g).currentwhite as i32 & ((1 as i32) << 3 as i32 | (1 as i32) << 4 as i32))
                    as lu_byte as i32) as lu_byte;
            if (*g).sweepgc == &mut (*o).next as *mut *mut GCObject {
                (*g).sweepgc = sweeptolive(L, (*g).sweepgc);
            }
        } else {
            correctpointers(g, o);
        }
        p = &mut (*g).allgc;
        while *p != o {
            p = &mut (**p).next;
        }
        *p = (*o).next;
        (*o).next = (*g).finobj;
        (*g).finobj = o;
        (*o).marked = ((*o).marked as i32 | (1 as i32) << 6 as i32) as lu_byte;
    };
}
unsafe extern "C-unwind" fn setpause(mut g: *mut global_State) {
    let mut threshold: l_mem = 0;
    let mut debt: l_mem = 0;
    let mut pause: i32 = (*g).gcpause as i32 * 4 as i32;
    let mut estimate: l_mem = ((*g).GCestimate / 100 as lu_mem) as l_mem;
    threshold = if (pause as l_mem) < (!(0 as lu_mem) >> 1 as i32) as l_mem / estimate {
        estimate * pause as l_mem
    } else {
        (!(0 as lu_mem) >> 1 as i32) as l_mem
    };
    debt = (((*g).totalbytes + (*g).GCdebt) as lu_mem).wrapping_sub(threshold as lu_mem) as l_mem;
    if debt > 0 as l_mem {
        debt = 0 as l_mem;
    }
    luaE_setdebt(g, debt);
}
unsafe extern "C-unwind" fn sweep2old(mut L: *mut lua_State, mut p: *mut *mut GCObject) {
    let mut curr: *mut GCObject = 0 as *mut GCObject;
    let mut g: *mut global_State = (*L).l_G;
    loop {
        curr = *p;
        if curr.is_null() {
            break;
        }
        if (*curr).marked as i32 & ((1 as i32) << 3 as i32 | (1 as i32) << 4 as i32) != 0 {
            *p = (*curr).next;
            freeobj(L, curr);
        } else {
            (*curr).marked = ((*curr).marked as i32 & !(7 as i32) | 4 as i32) as lu_byte;
            if (*curr).tt as i32 == 8 as i32 | (0) << 4 as i32 {
                let mut th: *mut lua_State = &mut (*(curr as *mut GCUnion)).th;
                linkgclist_(
                    &mut (*(th as *mut GCUnion)).gc,
                    &mut (*th).gclist,
                    &mut (*g).grayagain,
                );
            } else if (*curr).tt as i32 == 9 as i32 | (0) << 4 as i32
                && (*(&mut (*(curr as *mut GCUnion)).upv as *mut UpVal)).v.p
                    != &mut (*(&mut (*(curr as *mut GCUnion)).upv as *mut UpVal))
                        .u
                        .value as *mut TValue
            {
                (*curr).marked = ((*curr).marked as i32
                    & !((1 as i32) << 5 as i32 | ((1 as i32) << 3 as i32 | (1 as i32) << 4 as i32))
                        as lu_byte as i32) as lu_byte;
            } else {
                (*curr).marked = ((*curr).marked as i32 | (1 as i32) << 5 as i32) as lu_byte;
            }
            p = &mut (*curr).next;
        }
    }
}
unsafe extern "C-unwind" fn sweepgen(
    mut L: *mut lua_State,
    mut g: *mut global_State,
    mut p: *mut *mut GCObject,
    mut limit: *mut GCObject,
    mut pfirstold1: *mut *mut GCObject,
) -> *mut *mut GCObject {
    static mut nextage: [lu_byte; 7] = [
        1 as i32 as lu_byte,
        3 as i32 as lu_byte,
        3 as i32 as lu_byte,
        4 as i32 as lu_byte,
        4 as i32 as lu_byte,
        5 as i32 as lu_byte,
        6 as i32 as lu_byte,
    ];
    let mut white: i32 = ((*g).currentwhite as i32
        & ((1 as i32) << 3 as i32 | (1 as i32) << 4 as i32)) as lu_byte
        as i32;
    let mut curr: *mut GCObject = 0 as *mut GCObject;
    loop {
        curr = *p;
        if !(curr != limit) {
            break;
        }
        if (*curr).marked as i32 & ((1 as i32) << 3 as i32 | (1 as i32) << 4 as i32) != 0 {
            *p = (*curr).next;
            freeobj(L, curr);
        } else {
            if (*curr).marked as i32 & 7 as i32 == 0 {
                let mut marked: i32 = (*curr).marked as i32
                    & !((1 as i32) << 5 as i32
                        | ((1 as i32) << 3 as i32 | (1 as i32) << 4 as i32)
                        | 7 as i32);
                (*curr).marked = (marked | 1 as i32 | white) as lu_byte;
            } else {
                (*curr).marked = ((*curr).marked as i32 & !(7 as i32)
                    | nextage[((*curr).marked as i32 & 7 as i32) as usize] as i32)
                    as lu_byte;
                if (*curr).marked as i32 & 7 as i32 == 3 as i32 && (*pfirstold1).is_null() {
                    *pfirstold1 = curr;
                }
            }
            p = &mut (*curr).next;
        }
    }
    return p;
}
unsafe extern "C-unwind" fn whitelist(mut g: *mut global_State, mut p: *mut GCObject) {
    let mut white: i32 = ((*g).currentwhite as i32
        & ((1 as i32) << 3 as i32 | (1 as i32) << 4 as i32)) as lu_byte
        as i32;
    while !p.is_null() {
        (*p).marked = ((*p).marked as i32
            & !((1 as i32) << 5 as i32
                | ((1 as i32) << 3 as i32 | (1 as i32) << 4 as i32)
                | 7 as i32)
            | white) as lu_byte;
        p = (*p).next;
    }
}
unsafe extern "C-unwind" fn correctgraylist(mut p: *mut *mut GCObject) -> *mut *mut GCObject {
    let mut current_block: u64;
    let mut curr: *mut GCObject = 0 as *mut GCObject;
    loop {
        curr = *p;
        if curr.is_null() {
            break;
        }
        let mut next: *mut *mut GCObject = getgclist(curr);
        if !((*curr).marked as i32 & ((1 as i32) << 3 as i32 | (1 as i32) << 4 as i32) != 0) {
            if (*curr).marked as i32 & 7 as i32 == 5 as i32 {
                (*curr).marked = ((*curr).marked as i32 | (1 as i32) << 5 as i32) as lu_byte;
                (*curr).marked = ((*curr).marked as i32 ^ (5 as i32 ^ 6 as i32)) as lu_byte;
                current_block = 8466888583218658643;
            } else if (*curr).tt as i32 == 8 as i32 | (0) << 4 as i32 {
                current_block = 8466888583218658643;
            } else {
                if (*curr).marked as i32 & 7 as i32 == 6 as i32 {
                    (*curr).marked = ((*curr).marked as i32 ^ (6 as i32 ^ 4 as i32)) as lu_byte;
                }
                (*curr).marked = ((*curr).marked as i32 | (1 as i32) << 5 as i32) as lu_byte;
                current_block = 17465232319020127661;
            }
            match current_block {
                17465232319020127661 => {}
                _ => {
                    p = next;
                    continue;
                }
            }
        }
        *p = *next;
    }
    return p;
}
unsafe extern "C-unwind" fn correctgraylists(mut g: *mut global_State) {
    let mut list: *mut *mut GCObject = correctgraylist(&mut (*g).grayagain);
    *list = (*g).weak;
    (*g).weak = 0 as *mut GCObject;
    list = correctgraylist(list);
    *list = (*g).allweak;
    (*g).allweak = 0 as *mut GCObject;
    list = correctgraylist(list);
    *list = (*g).ephemeron;
    (*g).ephemeron = 0 as *mut GCObject;
    correctgraylist(list);
}
unsafe extern "C-unwind" fn markold(
    mut g: *mut global_State,
    mut from: *mut GCObject,
    mut to: *mut GCObject,
) {
    let mut p: *mut GCObject = 0 as *mut GCObject;
    p = from;
    while p != to {
        if (*p).marked as i32 & 7 as i32 == 3 as i32 {
            (*p).marked = ((*p).marked as i32 ^ (3 as i32 ^ 4 as i32)) as lu_byte;
            if (*p).marked as i32 & (1 as i32) << 5 as i32 != 0 {
                reallymarkobject(g, p);
            }
        }
        p = (*p).next;
    }
}
unsafe extern "C-unwind" fn finishgencycle(mut L: *mut lua_State, mut g: *mut global_State) {
    correctgraylists(g);
    checkSizes(L, g);
    (*g).gcstate = 0 as lu_byte;
    if (*g).gcemergency == 0 {
        callallpendingfinalizers(L);
    }
}
unsafe extern "C-unwind" fn youngcollection(mut L: *mut lua_State, mut g: *mut global_State) {
    let mut psurvival: *mut *mut GCObject = 0 as *mut *mut GCObject;
    let mut dummy: *mut GCObject = 0 as *mut GCObject;
    if !((*g).firstold1).is_null() {
        markold(g, (*g).firstold1, (*g).reallyold);
        (*g).firstold1 = 0 as *mut GCObject;
    }
    markold(g, (*g).finobj, (*g).finobjrold);
    markold(g, (*g).tobefnz, 0 as *mut GCObject);
    atomic(L);
    (*g).gcstate = 3 as i32 as lu_byte;
    psurvival = sweepgen(L, g, &mut (*g).allgc, (*g).survival, &mut (*g).firstold1);
    sweepgen(L, g, psurvival, (*g).old1, &mut (*g).firstold1);
    (*g).reallyold = (*g).old1;
    (*g).old1 = *psurvival;
    (*g).survival = (*g).allgc;
    dummy = 0 as *mut GCObject;
    psurvival = sweepgen(L, g, &mut (*g).finobj, (*g).finobjsur, &mut dummy);
    sweepgen(L, g, psurvival, (*g).finobjold1, &mut dummy);
    (*g).finobjrold = (*g).finobjold1;
    (*g).finobjold1 = *psurvival;
    (*g).finobjsur = (*g).finobj;
    sweepgen(L, g, &mut (*g).tobefnz, 0 as *mut GCObject, &mut dummy);
    finishgencycle(L, g);
}
unsafe extern "C-unwind" fn atomic2gen(mut L: *mut lua_State, mut g: *mut global_State) {
    cleargraylists(g);
    (*g).gcstate = 3 as i32 as lu_byte;
    sweep2old(L, &mut (*g).allgc);
    (*g).survival = (*g).allgc;
    (*g).old1 = (*g).survival;
    (*g).reallyold = (*g).old1;
    (*g).firstold1 = 0 as *mut GCObject;
    sweep2old(L, &mut (*g).finobj);
    (*g).finobjsur = (*g).finobj;
    (*g).finobjold1 = (*g).finobjsur;
    (*g).finobjrold = (*g).finobjold1;
    sweep2old(L, &mut (*g).tobefnz);
    (*g).gckind = 1 as i32 as lu_byte;
    (*g).lastatomic = 0 as lu_mem;
    (*g).GCestimate = ((*g).totalbytes + (*g).GCdebt) as lu_mem;
    finishgencycle(L, g);
}
unsafe extern "C-unwind" fn setminordebt(mut g: *mut global_State) {
    luaE_setdebt(
        g,
        -((((*g).totalbytes + (*g).GCdebt) as lu_mem / 100 as lu_mem) as l_mem
            * (*g).genminormul as l_mem),
    );
}
unsafe extern "C-unwind" fn entergen(mut L: *mut lua_State, mut g: *mut global_State) -> lu_mem {
    let mut numobjs: lu_mem = 0;
    luaC_runtilstate(L, (1 as i32) << 8 as i32);
    luaC_runtilstate(L, (1 as i32) << 0);
    numobjs = atomic(L);
    atomic2gen(L, g);
    setminordebt(g);
    return numobjs;
}
unsafe extern "C-unwind" fn enterinc(mut g: *mut global_State) {
    whitelist(g, (*g).allgc);
    (*g).survival = 0 as *mut GCObject;
    (*g).old1 = (*g).survival;
    (*g).reallyold = (*g).old1;
    whitelist(g, (*g).finobj);
    whitelist(g, (*g).tobefnz);
    (*g).finobjsur = 0 as *mut GCObject;
    (*g).finobjold1 = (*g).finobjsur;
    (*g).finobjrold = (*g).finobjold1;
    (*g).gcstate = 8 as i32 as lu_byte;
    (*g).gckind = 0 as lu_byte;
    (*g).lastatomic = 0 as lu_mem;
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaC_changemode(mut L: *mut lua_State, mut newmode: i32) {
    let mut g: *mut global_State = (*L).l_G;
    if newmode != (*g).gckind as i32 {
        if newmode == 1 as i32 {
            entergen(L, g);
        } else {
            enterinc(g);
        }
    }
    (*g).lastatomic = 0 as lu_mem;
}
unsafe extern "C-unwind" fn fullgen(mut L: *mut lua_State, mut g: *mut global_State) -> lu_mem {
    enterinc(g);
    return entergen(L, g);
}
unsafe extern "C-unwind" fn stepgenfull(mut L: *mut lua_State, mut g: *mut global_State) {
    let mut newatomic: lu_mem = 0;
    let mut lastatomic: lu_mem = (*g).lastatomic;
    if (*g).gckind as i32 == 1 as i32 {
        enterinc(g);
    }
    luaC_runtilstate(L, (1 as i32) << 0);
    newatomic = atomic(L);
    if newatomic < lastatomic.wrapping_add(lastatomic >> 3 as i32) {
        atomic2gen(L, g);
        setminordebt(g);
    } else {
        (*g).GCestimate = ((*g).totalbytes + (*g).GCdebt) as lu_mem;
        entersweep(L);
        luaC_runtilstate(L, (1 as i32) << 8 as i32);
        setpause(g);
        (*g).lastatomic = newatomic;
    };
}
unsafe extern "C-unwind" fn genstep(mut L: *mut lua_State, mut g: *mut global_State) {
    if (*g).lastatomic != 0 as lu_mem {
        stepgenfull(L, g);
    } else {
        let mut majorbase: lu_mem = (*g).GCestimate;
        let mut majorinc: lu_mem =
            majorbase / 100 as lu_mem * ((*g).genmajormul as i32 * 4 as i32) as lu_mem;
        if (*g).GCdebt > 0 as l_mem
            && ((*g).totalbytes + (*g).GCdebt) as lu_mem > majorbase.wrapping_add(majorinc)
        {
            let mut numobjs: lu_mem = fullgen(L, g);
            if !((((*g).totalbytes + (*g).GCdebt) as lu_mem)
                < majorbase.wrapping_add(majorinc / 2 as i32 as lu_mem))
            {
                (*g).lastatomic = numobjs;
                setpause(g);
            }
        } else {
            youngcollection(L, g);
            setminordebt(g);
            (*g).GCestimate = majorbase;
        }
    };
}
unsafe extern "C-unwind" fn entersweep(mut L: *mut lua_State) {
    let mut g: *mut global_State = (*L).l_G;
    (*g).gcstate = 3 as i32 as lu_byte;
    (*g).sweepgc = sweeptolive(L, &mut (*g).allgc);
}
unsafe extern "C-unwind" fn deletelist(
    mut L: *mut lua_State,
    mut p: *mut GCObject,
    mut limit: *mut GCObject,
) {
    while p != limit {
        let mut next: *mut GCObject = (*p).next;
        freeobj(L, p);
        p = next;
    }
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaC_freeallobjects(mut L: *mut lua_State) {
    let mut g: *mut global_State = (*L).l_G;
    (*g).gcstp = 4 as i32 as lu_byte;
    luaC_changemode(L, 0);
    separatetobefnz(g, 1 as i32);
    callallpendingfinalizers(L);
    deletelist(L, (*g).allgc, &mut (*((*g).mainthread as *mut GCUnion)).gc);
    deletelist(L, (*g).fixedgc, 0 as *mut GCObject);
}
unsafe extern "C-unwind" fn atomic(mut L: *mut lua_State) -> lu_mem {
    let mut g: *mut global_State = (*L).l_G;
    let mut work: lu_mem = 0 as lu_mem;
    let mut origweak: *mut GCObject = 0 as *mut GCObject;
    let mut origall: *mut GCObject = 0 as *mut GCObject;
    let mut grayagain: *mut GCObject = (*g).grayagain;
    (*g).grayagain = 0 as *mut GCObject;
    (*g).gcstate = 2 as i32 as lu_byte;
    if (*L).marked as i32 & ((1 as i32) << 3 as i32 | (1 as i32) << 4 as i32) != 0 {
        reallymarkobject(g, &mut (*(L as *mut GCUnion)).gc);
    }
    if (*g).l_registry.tt_ as i32 & (1 as i32) << 6 as i32 == 0
        || (*g).l_registry.tt_ as i32 & 0x3f as i32 == (*(*g).l_registry.value_.gc).tt as i32
            && (((*g).mainthread).is_null()
                || (*(*g).l_registry.value_.gc).marked as i32
                    & ((*(*(*g).mainthread).l_G).currentwhite as i32
                        ^ ((1 as i32) << 3 as i32 | (1 as i32) << 4 as i32))
                    == 0)
    {
    } else {
    };
    if (*g).l_registry.tt_ as i32 & (1 as i32) << 6 as i32 != 0
        && (*(*g).l_registry.value_.gc).marked as i32
            & ((1 as i32) << 3 as i32 | (1 as i32) << 4 as i32)
            != 0
    {
        reallymarkobject(g, (*g).l_registry.value_.gc);
    }
    markmt(g);
    work = work.wrapping_add(propagateall(g));
    work = work.wrapping_add(remarkupvals(g) as lu_mem);
    work = work.wrapping_add(propagateall(g));
    (*g).gray = grayagain;
    work = work.wrapping_add(propagateall(g));
    convergeephemerons(g);
    clearbyvalues(g, (*g).weak, 0 as *mut GCObject);
    clearbyvalues(g, (*g).allweak, 0 as *mut GCObject);
    origweak = (*g).weak;
    origall = (*g).allweak;
    separatetobefnz(g, 0);
    work = work.wrapping_add(markbeingfnz(g));
    work = work.wrapping_add(propagateall(g));
    convergeephemerons(g);
    clearbykeys(g, (*g).ephemeron);
    clearbykeys(g, (*g).allweak);
    clearbyvalues(g, (*g).weak, origweak);
    clearbyvalues(g, (*g).allweak, origall);
    luaS_clearcache(g);
    (*g).currentwhite =
        ((*g).currentwhite as i32 ^ ((1 as i32) << 3 as i32 | (1 as i32) << 4 as i32)) as lu_byte;
    return work;
}
unsafe extern "C-unwind" fn sweepstep(
    mut L: *mut lua_State,
    mut g: *mut global_State,
    mut nextstate: i32,
    mut nextlist: *mut *mut GCObject,
) -> i32 {
    if !((*g).sweepgc).is_null() {
        let mut olddebt: l_mem = (*g).GCdebt;
        let mut count: i32 = 0;
        (*g).sweepgc = sweeplist(L, (*g).sweepgc, 100, &mut count);
        (*g).GCestimate = ((*g).GCestimate).wrapping_add(((*g).GCdebt - olddebt) as lu_mem);
        return count;
    } else {
        (*g).gcstate = nextstate as lu_byte;
        (*g).sweepgc = nextlist;
        return 0;
    };
}
unsafe extern "C-unwind" fn singlestep(mut L: *mut lua_State) -> lu_mem {
    let mut g: *mut global_State = (*L).l_G;
    let mut work: lu_mem = 0;
    (*g).gcstopem = 1 as i32 as lu_byte;
    match (*g).gcstate as i32 {
        8 => {
            restartcollection(g);
            (*g).gcstate = 0 as lu_byte;
            work = 1 as i32 as lu_mem;
        }
        0 => {
            if ((*g).gray).is_null() {
                (*g).gcstate = 1 as i32 as lu_byte;
                work = 0 as lu_mem;
            } else {
                work = propagatemark(g);
            }
        }
        1 => {
            work = atomic(L);
            entersweep(L);
            (*g).GCestimate = ((*g).totalbytes + (*g).GCdebt) as lu_mem;
        }
        3 => {
            work = sweepstep(L, g, 4 as i32, &mut (*g).finobj) as lu_mem;
        }
        4 => {
            work = sweepstep(L, g, 5 as i32, &mut (*g).tobefnz) as lu_mem;
        }
        5 => {
            work = sweepstep(L, g, 6 as i32, 0 as *mut *mut GCObject) as lu_mem;
        }
        6 => {
            checkSizes(L, g);
            (*g).gcstate = 7 as i32 as lu_byte;
            work = 0 as lu_mem;
        }
        7 => {
            if !((*g).tobefnz).is_null() && (*g).gcemergency == 0 {
                (*g).gcstopem = 0 as lu_byte;
                work = (runafewfinalizers(L, 10) * 50) as lu_mem;
            } else {
                (*g).gcstate = 8 as i32 as lu_byte;
                work = 0 as lu_mem;
            }
        }
        _ => return 0 as lu_mem,
    }
    (*g).gcstopem = 0 as lu_byte;
    return work;
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaC_runtilstate(mut L: *mut lua_State, mut statesmask: i32) {
    let mut g: *mut global_State = (*L).l_G;
    while statesmask & (1 as i32) << (*g).gcstate as i32 == 0 {
        singlestep(L);
    }
}
unsafe extern "C-unwind" fn incstep(mut L: *mut lua_State, mut g: *mut global_State) {
    let mut stepmul: i32 = (*g).gcstepmul as i32 * 4 as i32 | 1 as i32;
    let mut debt: l_mem = ((*g).GCdebt as usize)
        .wrapping_div(size_of::<TValue>() as usize)
        .wrapping_mul(stepmul as usize) as l_mem;
    let mut stepsize: l_mem = (if (*g).gcstepsize as usize
        <= (size_of::<l_mem>() as usize)
            .wrapping_mul(8)
            .wrapping_sub(2)
    {
        (((1 as i32 as l_mem) << (*g).gcstepsize as i32) as usize)
            .wrapping_div(size_of::<TValue>() as usize)
            .wrapping_mul(stepmul as usize)
    } else {
        (!(0 as lu_mem) >> 1 as i32) as l_mem as usize
    }) as l_mem;
    loop {
        let mut work: lu_mem = singlestep(L);
        debt = (debt as lu_mem).wrapping_sub(work) as l_mem as l_mem;
        if !(debt > -stepsize && (*g).gcstate as i32 != 8 as i32) {
            break;
        }
    }
    if (*g).gcstate as i32 == 8 as i32 {
        setpause(g);
    } else {
        debt = ((debt / stepmul as l_mem) as usize).wrapping_mul(size_of::<TValue>() as usize)
            as l_mem;
        luaE_setdebt(g, debt);
    };
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaC_step(mut L: *mut lua_State) {
    let mut g: *mut global_State = (*L).l_G;
    if !((*g).gcstp as i32 == 0) {
        luaE_setdebt(g, -(2000) as l_mem);
    } else if (*g).gckind as i32 == 1 as i32 || (*g).lastatomic != 0 as lu_mem {
        genstep(L, g);
    } else {
        incstep(L, g);
    };
}
unsafe extern "C-unwind" fn fullinc(mut L: *mut lua_State, mut g: *mut global_State) {
    if (*g).gcstate as i32 <= 2 as i32 {
        entersweep(L);
    }
    luaC_runtilstate(L, (1 as i32) << 8 as i32);
    luaC_runtilstate(L, (1 as i32) << 0);
    (*g).gcstate = 1 as i32 as lu_byte;
    luaC_runtilstate(L, (1 as i32) << 7 as i32);
    luaC_runtilstate(L, (1 as i32) << 8 as i32);
    setpause(g);
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaC_fullgc(mut L: *mut lua_State, mut isemergency: i32) {
    let mut g: *mut global_State = (*L).l_G;
    (*g).gcemergency = isemergency as lu_byte;
    if (*g).gckind as i32 == 0 {
        fullinc(L, g);
    } else {
        fullgen(L, g);
    }
    (*g).gcemergency = 0 as lu_byte;
}
