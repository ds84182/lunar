#![allow(
    dead_code,
    mutable_transmutes,
    non_camel_case_types,
    non_snake_case,
    non_upper_case_globals,
    unused_assignments,
    unused_mut,
    unsafe_op_in_unsafe_fn
)]
#![feature(c_variadic, extern_types)]
#![feature(likely_unlikely)]

mod code;
mod gc;
mod ldo;
mod mem;
mod object;
mod opcodes;
mod parser;
mod table;
mod undump;
mod utils;
mod vm;
mod zio;

use libc::{
    __errno_location, FILE, abort, abs, clearerr, close, difftime, dlclose, dlerror, dlopen, dlsym,
    exit, fclose, feof, ferror, fflush, fgets, fopen, fprintf, fread, free, freopen, fseeko,
    ftello, fwrite, getenv, gmtime_r, localeconv, localtime_r, memchr, memcmp, memcpy, mkstemp,
    mktime, pclose, popen, realloc, remove, rename, setlocale, setvbuf, snprintf, strchr, strcmp,
    strcoll, strcpy, strerror, strftime, strlen, strncmp, strpbrk, strspn, strstr, strtod, system,
    time, tm, tmpfile, tolower, toupper, ungetc,
};
use std::{
    ffi::c_void,
    ptr::{self, NonNull},
};

use code::*;
use gc::*;
use ldo::*;
use mem::*;
use object::*;
use opcodes::*;
use parser::*;
use table::*;
use undump::*;
use vm::*;
use zio::*;

unsafe extern "C-unwind" {
    static mut stdin: *mut FILE;
    static mut stdout: *mut FILE;
    static mut stderr: *mut FILE;
    fn getc(__stream: *mut FILE) -> i32;
    fn getc_unlocked(__stream: *mut FILE) -> i32;
    fn flockfile(__stream: *mut FILE);
    fn funlockfile(__stream: *mut FILE);
    fn clock() -> clock_t;
    fn frexp(_: std::ffi::c_double, _: *mut i32) -> std::ffi::c_double;
    fn ldexp(_: std::ffi::c_double, _: i32) -> std::ffi::c_double;
    fn fmod(_: std::ffi::c_double, _: std::ffi::c_double) -> std::ffi::c_double;
    fn __ctype_b_loc() -> *mut *const u16;
}
pub type ptrdiff_t = isize;
pub type size_t = usize;
pub type __off_t = std::ffi::c_long;
pub type __off64_t = std::ffi::c_long;
pub type __clock_t = std::ffi::c_long;
pub type __time_t = std::ffi::c_long;
pub type __sig_atomic_t = i32;
pub type intptr_t = isize;
pub type uintptr_t = usize;
pub const LUA_MULTRET: i32 = -1;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct lua_State {
    pub next: *mut GCObject,
    pub tt: lu_byte,
    pub marked: lu_byte,
    pub status: lu_byte,
    pub allowhook: lu_byte,
    pub nci: u16,
    pub top: StkIdRel,
    pub l_G: *mut global_State,
    pub ci: *mut CallInfo,
    pub stack_last: StkIdRel,
    pub stack: StkIdRel,
    pub openupval: *mut UpVal,
    pub tbclist: StkIdRel,
    pub gclist: *mut GCObject,
    pub twups: *mut lua_State,
    pub errorJmp: *mut lua_longjmp,
    pub base_ci: CallInfo,
    pub hook: lua_Hook,
    pub errfunc: ptrdiff_t,
    pub nCcalls: l_uint32,
    pub oldpc: i32,
    pub basehookcount: i32,
    pub hookcount: i32,
    pub hookmask: sig_atomic_t,
}

pub const LUA_TNONE: i8 = -1;
pub const LUA_TNIL: i8 = 0;
pub const LUA_TBOOLEAN: i8 = 1;
pub const LUA_TLIGHTUSERDATA: i8 = 2;
pub const LUA_TNUMBER: i8 = 3;
pub const LUA_TSTRING: i8 = 4;
pub const LUA_TTABLE: i8 = 5;
pub const LUA_TFUNCTION: i8 = 6;
pub const LUA_TUSERDATA: i8 = 7;
pub const LUA_TTHREAD: i8 = 8;
pub const LUA_NUMTYPES: i8 = 9;

pub type sig_atomic_t = __sig_atomic_t;
pub type l_uint32 = u32;
pub type lua_Hook = Option<unsafe extern "C-unwind" fn(*mut lua_State, *mut lua_Debug) -> ()>;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct lua_Debug {
    pub event: i32,
    pub name: *const std::ffi::c_char,
    pub namewhat: *const std::ffi::c_char,
    pub what: *const std::ffi::c_char,
    pub source: *const std::ffi::c_char,
    pub srclen: size_t,
    pub currentline: i32,
    pub linedefined: i32,
    pub lastlinedefined: i32,
    pub nups: u8,
    pub nparams: u8,
    pub isvararg: std::ffi::c_char,
    pub istailcall: std::ffi::c_char,
    pub ftransfer: u16,
    pub ntransfer: u16,
    pub short_src: [std::ffi::c_char; 60],
    pub i_ci: *mut CallInfo,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct CallInfo {
    pub func: StkIdRel,
    pub top: StkIdRel,
    pub previous: *mut CallInfo,
    pub next: *mut CallInfo,
    pub u: CallInfoState,
    pub u2: CallInfoUnion2,
    pub nresults: i16,
    pub callstatus: u16,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union CallInfoUnion2 {
    pub funcidx: i32,
    pub nyield: i32,
    pub nres: i32,
    pub transferinfo: CallInfoTransferredValues,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct CallInfoTransferredValues {
    pub ftransfer: u16,
    pub ntransfer: u16,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union CallInfoState {
    pub l: CallInfoLuaState,
    pub c: CallInfoCState,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct CallInfoCState {
    pub k: lua_KFunction,
    pub old_errfunc: ptrdiff_t,
    pub ctx: lua_KContext,
}
pub type lua_KContext = intptr_t;
pub type lua_KFunction =
    Option<unsafe extern "C-unwind" fn(*mut lua_State, i32, lua_KContext) -> i32>;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct CallInfoLuaState {
    pub savedpc: *const Instruction,
    pub trap: sig_atomic_t,
    pub nextraargs: i32,
}
pub type Instruction = u32;
#[derive(Copy, Clone)]
#[repr(C)]
pub union StkIdRel {
    pub p: StkId,
    pub offset: ptrdiff_t,
}
pub type StkId = *mut StackValue;
#[derive(Copy, Clone)]
#[repr(C)]
pub union StackValue {
    pub val: TValue,
    pub tbclist: ToBeClosedList,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ToBeClosedList {
    pub value_: Value,
    pub tt_: lu_byte,
    pub delta: u16,
}
pub type lu_byte = u8;
#[derive(Copy, Clone)]
#[repr(C)]
pub union Value {
    pub gc: *mut GCObject,
    pub p: *mut c_void,
    pub f: lua_CFunction,
    pub i: lua_Integer,
    pub n: lua_Number,
    pub ub: lu_byte,
}
pub type lua_Number = f64;
pub type lua_Integer = i64;
pub type lua_CFunction = Option<unsafe extern "C-unwind" fn(*mut lua_State) -> i32>;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct GCObject {
    pub next: *mut GCObject,
    pub tt: lu_byte,
    pub marked: lu_byte,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct TValue {
    pub value_: Value,
    pub tt_: lu_byte,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct lua_longjmp {
    pub previous: *mut lua_longjmp,
    pub status: i32,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct UpVal {
    pub next: *mut GCObject,
    pub tt: lu_byte,
    pub marked: lu_byte,
    pub v: UpValValuePtr,
    pub u: UpValLinksOrValue,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union UpValLinksOrValue {
    pub open: UpValLinks,
    pub value: TValue,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct UpValLinks {
    pub next: *mut UpVal,
    pub previous: *mut *mut UpVal,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union UpValValuePtr {
    pub p: *mut TValue,
    pub offset: ptrdiff_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct global_State {
    pub frealloc: lua_Alloc,
    pub ud: *mut c_void,
    pub totalbytes: l_mem,
    pub GCdebt: l_mem,
    pub GCestimate: lu_mem,
    pub lastatomic: lu_mem,
    pub strt: stringtable,
    pub l_registry: TValue,
    pub nilvalue: TValue,
    pub seed: u32,
    pub currentwhite: lu_byte,
    pub gcstate: lu_byte,
    pub gckind: lu_byte,
    pub gcstopem: lu_byte,
    pub genminormul: lu_byte,
    pub genmajormul: lu_byte,
    pub gcstp: lu_byte,
    pub gcemergency: lu_byte,
    pub gcpause: lu_byte,
    pub gcstepmul: lu_byte,
    pub gcstepsize: lu_byte,
    pub allgc: *mut GCObject,
    pub sweepgc: *mut *mut GCObject,
    pub finobj: *mut GCObject,
    pub gray: *mut GCObject,
    pub grayagain: *mut GCObject,
    pub weak: *mut GCObject,
    pub ephemeron: *mut GCObject,
    pub allweak: *mut GCObject,
    pub tobefnz: *mut GCObject,
    pub fixedgc: *mut GCObject,
    pub survival: *mut GCObject,
    pub old1: *mut GCObject,
    pub reallyold: *mut GCObject,
    pub firstold1: *mut GCObject,
    pub finobjsur: *mut GCObject,
    pub finobjold1: *mut GCObject,
    pub finobjrold: *mut GCObject,
    pub twups: *mut lua_State,
    pub panic: lua_CFunction,
    pub mainthread: *mut lua_State,
    pub memerrmsg: *mut TString,
    pub tmname: [*mut TString; 25],
    pub mt: [*mut Table; 9],
    pub strcache: [[*mut TString; 2]; 53],
    pub warnf: lua_WarnFunction,
    pub ud_warn: *mut c_void,
}
pub type lua_WarnFunction =
    Option<unsafe extern "C-unwind" fn(*mut c_void, *const std::ffi::c_char, i32) -> ()>;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct TString {
    pub next: *mut GCObject,
    pub tt: lu_byte,
    pub marked: lu_byte,
    pub extra: lu_byte,
    pub shrlen: lu_byte,
    pub hash: u32,
    pub u: C2RustUnnamed_8,
    pub contents: [std::ffi::c_char; 1],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed_8 {
    pub lnglen: size_t,
    pub hnext: *mut TString,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Table {
    pub next: *mut GCObject,
    pub tt: lu_byte,
    pub marked: lu_byte,
    pub flags: lu_byte,
    pub lsizenode: lu_byte,
    pub alimit: u32,
    pub array: *mut TValue,
    pub node: *mut Node,
    pub lastfree: *mut Node,
    pub metatable: *mut Table,
    pub gclist: *mut GCObject,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union Node {
    pub u: NodeKey,
    pub i_val: TValue,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct NodeKey {
    pub value_: Value,
    pub tt_: lu_byte,
    pub key_tt: lu_byte,
    pub next: i32,
    pub key_val: Value,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct stringtable {
    pub hash: *mut *mut TString,
    pub nuse: i32,
    pub size: i32,
}
pub type lu_mem = size_t;
pub type l_mem = isize;
pub type lua_Alloc =
    Option<unsafe extern "C-unwind" fn(*mut c_void, *mut c_void, size_t, size_t) -> *mut c_void>;
pub type lua_Unsigned = u64;
pub type lua_Reader = Option<
    unsafe extern "C-unwind" fn(
        *mut lua_State,
        *mut c_void,
        *mut size_t,
    ) -> *const std::ffi::c_char,
>;
pub type lua_Writer =
    Option<unsafe extern "C-unwind" fn(*mut lua_State, *const c_void, size_t, *mut c_void) -> i32>;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct LG {
    pub l: LX,
    pub g: global_State,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct LX {
    pub extra_: [lu_byte; 8],
    pub l: lua_State,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union GCUnion {
    pub gc: GCObject,
    pub ts: TString,
    pub u: Udata,
    pub cl: Closure,
    pub h: Table,
    pub p: Proto,
    pub th: lua_State,
    pub upv: UpVal,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Proto {
    pub next: *mut GCObject,
    pub tt: lu_byte,
    pub marked: lu_byte,
    pub numparams: lu_byte,
    pub is_vararg: lu_byte,
    pub maxstacksize: lu_byte,
    pub sizeupvalues: i32,
    pub sizek: i32,
    pub sizecode: i32,
    pub sizelineinfo: i32,
    pub sizep: i32,
    pub sizelocvars: i32,
    pub sizeabslineinfo: i32,
    pub linedefined: i32,
    pub lastlinedefined: i32,
    pub k: *mut TValue,
    pub code: *mut Instruction,
    pub p: *mut *mut Proto,
    pub upvalues: *mut Upvaldesc,
    pub lineinfo: *mut ls_byte,
    pub abslineinfo: *mut AbsLineInfo,
    pub locvars: *mut LocVar,
    pub source: *mut TString,
    pub gclist: *mut GCObject,
    /// Counters for the target of each backwards jump.
    ///
    /// Used to detect hot loops.
    pub loop_cnts: *mut LoopCounter,
    pub size_loop_cnts: u32,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct LocVar {
    pub varname: *mut TString,
    pub startpc: i32,
    pub endpc: i32,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct AbsLineInfo {
    pub pc: i32,
    pub line: i32,
}
pub type ls_byte = std::ffi::c_schar;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Upvaldesc {
    pub name: *mut TString,
    pub instack: lu_byte,
    pub idx: lu_byte,
    pub kind: lu_byte,
}
#[repr(C)]
pub struct LoopCounter {
    pub pc: u32,
    pub count: u32,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union Closure {
    pub c: CClosure,
    pub l: LClosure,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct LClosure {
    pub next: *mut GCObject,
    pub tt: lu_byte,
    pub marked: lu_byte,
    pub nupvalues: lu_byte,
    pub gclist: *mut GCObject,
    pub p: *mut Proto,
    pub upvals: [*mut UpVal; 1],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct CClosure {
    pub next: *mut GCObject,
    pub tt: lu_byte,
    pub marked: lu_byte,
    pub nupvalues: lu_byte,
    pub gclist: *mut GCObject,
    pub f: lua_CFunction,
    pub upvalue: [TValue; 1],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Udata {
    pub next: *mut GCObject,
    pub tt: lu_byte,
    pub marked: lu_byte,
    pub nuvalue: u16,
    pub len: size_t,
    pub metatable: *mut Table,
    pub gclist: *mut GCObject,
    pub uv: [UValue; 1],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union UValue {
    pub uv: TValue,
    pub n: lua_Number,
    pub u: std::ffi::c_double,
    pub s: *mut c_void,
    pub i: lua_Integer,
    pub l: std::ffi::c_long,
}
pub const TM_MODE: TMS = 3;
pub type TMS = u32;
pub const TM_N: TMS = 25;
pub const TM_CLOSE: TMS = 24;
pub const TM_CALL: TMS = 23;
pub const TM_CONCAT: TMS = 22;
pub const TM_LE: TMS = 21;
pub const TM_LT: TMS = 20;
pub const TM_BNOT: TMS = 19;
pub const TM_UNM: TMS = 18;
pub const TM_SHR: TMS = 17;
pub const TM_SHL: TMS = 16;
pub const TM_BXOR: TMS = 15;
pub const TM_BOR: TMS = 14;
pub const TM_BAND: TMS = 13;
pub const TM_IDIV: TMS = 12;
pub const TM_DIV: TMS = 11;
pub const TM_POW: TMS = 10;
pub const TM_MOD: TMS = 9;
pub const TM_MUL: TMS = 8;
pub const TM_SUB: TMS = 7;
pub const TM_ADD: TMS = 6;
pub const TM_EQ: TMS = 5;
pub const TM_LEN: TMS = 4;
pub const TM_GC: TMS = 2;
pub const TM_NEWINDEX: TMS = 1;
pub const TM_INDEX: TMS = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct CloseP {
    pub level: StkId,
    pub status: i32,
}
pub type Pfunc = Option<unsafe extern "C-unwind" fn(*mut lua_State, *mut c_void) -> ()>;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct BuffFS {
    pub L: *mut lua_State,
    pub pushed: i32,
    pub blen: i32,
    pub space: [std::ffi::c_char; 199],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct lconv {
    pub decimal_point: *mut std::ffi::c_char,
    pub thousands_sep: *mut std::ffi::c_char,
    pub grouping: *mut std::ffi::c_char,
    pub int_curr_symbol: *mut std::ffi::c_char,
    pub currency_symbol: *mut std::ffi::c_char,
    pub mon_decimal_point: *mut std::ffi::c_char,
    pub mon_thousands_sep: *mut std::ffi::c_char,
    pub mon_grouping: *mut std::ffi::c_char,
    pub positive_sign: *mut std::ffi::c_char,
    pub negative_sign: *mut std::ffi::c_char,
    pub int_frac_digits: std::ffi::c_char,
    pub frac_digits: std::ffi::c_char,
    pub p_cs_precedes: std::ffi::c_char,
    pub p_sep_by_space: std::ffi::c_char,
    pub n_cs_precedes: std::ffi::c_char,
    pub n_sep_by_space: std::ffi::c_char,
    pub p_sign_posn: std::ffi::c_char,
    pub n_sign_posn: std::ffi::c_char,
    pub int_p_cs_precedes: std::ffi::c_char,
    pub int_p_sep_by_space: std::ffi::c_char,
    pub int_n_cs_precedes: std::ffi::c_char,
    pub int_n_sep_by_space: std::ffi::c_char,
    pub int_p_sign_posn: std::ffi::c_char,
    pub int_n_sign_posn: std::ffi::c_char,
}

pub type l_uacNumber = std::ffi::c_double;
pub type l_uacInt = std::ffi::c_longlong;
pub type F2Imod = u32;
pub const F2Iceil: F2Imod = 2;
pub const F2Ifloor: F2Imod = 1;
pub const F2Ieq: F2Imod = 0;
pub const TK_WHILE: RESERVED = 277;
pub type time_t = __time_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct CallS {
    pub func: StkId,
    pub nresults: i32,
}
pub type ZIO = Zio;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Zio {
    pub n: size_t,
    pub p: *const std::ffi::c_char,
    pub reader: lua_Reader,
    pub data: *mut c_void,
    pub L: *mut lua_State,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Labeldesc {
    pub name: *mut TString,
    pub pc: i32,
    pub line: i32,
    pub nactvar: lu_byte,
    pub close: lu_byte,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Labellist {
    pub arr: *mut Labeldesc,
    pub n: i32,
    pub size: i32,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Dyndata {
    pub actvar: C2RustUnnamed_9,
    pub gt: Labellist,
    pub label: Labellist,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_9 {
    pub arr: *mut Vardesc,
    pub n: i32,
    pub size: i32,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union Vardesc {
    pub vd: C2RustUnnamed_10,
    pub k: TValue,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_10 {
    pub value_: Value,
    pub tt_: lu_byte,
    pub kind: lu_byte,
    pub ridx: lu_byte,
    pub pidx: i16,
    pub name: *mut TString,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct SParser {
    pub z: *mut ZIO,
    pub buff: Mbuffer,
    pub dyd: Dyndata,
    pub mode: *const std::ffi::c_char,
    pub name: *const std::ffi::c_char,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Mbuffer {
    pub buffer: *mut std::ffi::c_char,
    pub n: size_t,
    pub buffsize: size_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct FuncState {
    pub f: *mut Proto,
    pub prev: *mut FuncState,
    pub ls: *mut LexState,
    pub bl: *mut BlockCnt,
    pub pc: i32,
    pub lasttarget: i32,
    pub previousline: i32,
    pub nk: i32,
    pub np: i32,
    pub nabslineinfo: i32,
    pub firstlocal: i32,
    pub firstlabel: i32,
    pub ndebugvars: i16,
    pub nactvar: lu_byte,
    pub nups: lu_byte,
    pub freereg: lu_byte,
    pub iwthabs: lu_byte,
    pub needclose: lu_byte,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct BlockCnt {
    pub previous: *mut BlockCnt,
    pub firstlabel: i32,
    pub firstgoto: i32,
    pub nactvar: lu_byte,
    pub upval: lu_byte,
    pub isloop: lu_byte,
    pub insidetbc: lu_byte,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct LexState {
    pub current: i32,
    pub linenumber: i32,
    pub lastline: i32,
    pub t: Token,
    pub lookahead: Token,
    pub fs: *mut FuncState,
    pub L: *mut lua_State,
    pub z: *mut ZIO,
    pub buff: *mut Mbuffer,
    pub h: *mut Table,
    pub dyd: *mut Dyndata,
    pub source: *mut TString,
    pub envn: *mut TString,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Token {
    pub token: i32,
    pub seminfo: SemInfo,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union SemInfo {
    pub r: lua_Number,
    pub i: lua_Integer,
    pub ts: *mut TString,
}
pub const TK_EOS: RESERVED = 288;
pub const TK_INT: RESERVED = 290;
pub const TK_FLT: RESERVED = 289;
pub const TK_STRING: RESERVED = 292;
pub const TK_NAME: RESERVED = 291;
#[derive(Copy, Clone)]
#[repr(C)]
pub union ExpVariant {
    pub ival: lua_Integer,
    pub nval: lua_Number,
    pub strval: *mut TString,
    pub info: i32,
    pub ind: IndexedVar,
    pub var: LocalVar,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct LocalVar {
    pub ridx: lu_byte,
    pub vidx: u16,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct IndexedVar {
    pub idx: i16,
    pub t: lu_byte,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ExpDesc {
    pub k: ExpKind,
    pub u: ExpVariant,
    pub t: i32,
    pub f: i32,
}
pub type ExpKind = u32;
pub const VVARARG: ExpKind = 19;
pub const VCALL: ExpKind = 18;
pub const VRELOC: ExpKind = 17;
pub const VJMP: ExpKind = 16;
pub const VINDEXSTR: ExpKind = 15;
pub const VINDEXI: ExpKind = 14;
pub const VINDEXUP: ExpKind = 13;
pub const VINDEXED: ExpKind = 12;
pub const VCONST: ExpKind = 11;
pub const VUPVAL: ExpKind = 10;
pub const VLOCAL: ExpKind = 9;
pub const VNONRELOC: ExpKind = 8;
pub const VKSTR: ExpKind = 7;
pub const VKINT: ExpKind = 6;
pub const VKFLT: ExpKind = 5;
pub const VK: ExpKind = 4;
pub const VFALSE: ExpKind = 3;
pub const VTRUE: ExpKind = 2;
pub const VNIL: ExpKind = 1;
pub const VVOID: ExpKind = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct LHS_assign {
    pub prev: *mut LHS_assign,
    pub v: ExpDesc,
}
pub type BinOpr = u32;
pub const OPR_NOBINOPR: BinOpr = 21;
pub const OPR_OR: BinOpr = 20;
pub const OPR_AND: BinOpr = 19;
pub const OPR_GE: BinOpr = 18;
pub const OPR_GT: BinOpr = 17;
pub const OPR_NE: BinOpr = 16;
pub const OPR_LE: BinOpr = 15;
pub const OPR_LT: BinOpr = 14;
pub const OPR_EQ: BinOpr = 13;
pub const OPR_CONCAT: BinOpr = 12;
pub const OPR_SHR: BinOpr = 11;
pub const OPR_SHL: BinOpr = 10;
pub const OPR_BXOR: BinOpr = 9;
pub const OPR_BOR: BinOpr = 8;
pub const OPR_BAND: BinOpr = 7;
pub const OPR_IDIV: BinOpr = 6;
pub const OPR_DIV: BinOpr = 5;
pub const OPR_POW: BinOpr = 4;
pub const OPR_MOD: BinOpr = 3;
pub const OPR_MUL: BinOpr = 2;
pub const OPR_SUB: BinOpr = 1;
pub const OPR_ADD: BinOpr = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_14 {
    pub left: lu_byte,
    pub right: lu_byte,
}
pub const TK_CONCAT: RESERVED = 279;
pub const TK_DOTS: RESERVED = 280;
pub const TK_DBCOLON: RESERVED = 287;
pub const TK_NE: RESERVED = 284;
pub const TK_IDIV: RESERVED = 278;
pub const TK_SHR: RESERVED = 286;
pub const TK_GE: RESERVED = 282;
pub const TK_SHL: RESERVED = 285;
pub const TK_LE: RESERVED = 283;
pub const TK_EQ: RESERVED = 281;
pub const TK_OR: RESERVED = 271;
pub const TK_AND: RESERVED = 256;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ConsControl {
    pub v: ExpDesc,
    pub t: *mut ExpDesc,
    pub nh: i32,
    pub na: i32,
    pub tostore: i32,
}
pub const TK_FUNCTION: RESERVED = 264;
pub const TK_END: RESERVED = 261;
pub const TK_FALSE: RESERVED = 262;
pub const TK_TRUE: RESERVED = 275;
pub const TK_NIL: RESERVED = 269;
pub type UnOpr = u32;
pub const OPR_NOUNOPR: UnOpr = 4;
pub const OPR_LEN: UnOpr = 3;
pub const OPR_NOT: UnOpr = 2;
pub const OPR_BNOT: UnOpr = 1;
pub const OPR_MINUS: UnOpr = 0;
pub const TK_NOT: RESERVED = 270;
pub const TK_GOTO: RESERVED = 265;
pub const TK_BREAK: RESERVED = 257;
pub const TK_UNTIL: RESERVED = 276;
pub const TK_ELSEIF: RESERVED = 260;
pub const TK_ELSE: RESERVED = 259;
pub const TK_RETURN: RESERVED = 273;
pub const TK_LOCAL: RESERVED = 268;
pub const TK_REPEAT: RESERVED = 272;
pub const TK_FOR: RESERVED = 263;
pub const TK_DO: RESERVED = 258;
pub const TK_IN: RESERVED = 267;
pub const TK_IF: RESERVED = 266;
pub const TK_THEN: RESERVED = 274;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct LoadState {
    pub L: *mut lua_State,
    pub Z: *mut ZIO,
    pub name: *const std::ffi::c_char,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct DumpState {
    pub L: *mut lua_State,
    pub writer: lua_Writer,
    pub data: *mut c_void,
    pub strip: i32,
    pub status: i32,
}
pub type off_t = __off64_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct luaL_Buffer {
    pub b: *mut std::ffi::c_char,
    pub size: size_t,
    pub n: size_t,
    pub L: *mut lua_State,
    pub init: C2RustUnnamed_15,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed_15 {
    pub n: lua_Number,
    pub u: std::ffi::c_double,
    pub s: *mut c_void,
    pub i: lua_Integer,
    pub l: std::ffi::c_long,
    pub b: [std::ffi::c_char; 1024],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct luaL_Reg {
    pub name: *const std::ffi::c_char,
    pub func: lua_CFunction,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct LoadF {
    pub n: i32,
    pub f: *mut FILE,
    pub buff: [std::ffi::c_char; 8192],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct LoadS {
    pub s: *const std::ffi::c_char,
    pub size: size_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct UBox {
    pub box_0: *mut c_void,
    pub bsize: size_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct luaL_Stream {
    pub f: *mut FILE,
    pub closef: lua_CFunction,
}
pub const _ISalnum: C2RustUnnamed_19 = 8;
pub const _ISdigit: C2RustUnnamed_19 = 2048;
pub type IdxT = u32;
pub type clock_t = __clock_t;
pub type LStream = luaL_Stream;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct RN {
    pub f: *mut FILE,
    pub c: i32,
    pub n: i32,
    pub buff: [std::ffi::c_char; 201],
}
pub const _ISxdigit: C2RustUnnamed_19 = 4096;
pub const _ISspace: C2RustUnnamed_19 = 8192;
pub const Knop: KOption = 10;
pub const Kpadding: KOption = 8;
pub const Kpaddalign: KOption = 9;
pub const Kzstr: KOption = 7;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Header {
    pub L: *mut lua_State,
    pub islittle: i32,
    pub maxalign: i32,
}
pub const Kstring: KOption = 6;
pub const Kchar: KOption = 5;
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed_16 {
    pub dummy: i32,
    pub little: std::ffi::c_char,
}
pub const Kdouble: KOption = 4;
pub const Knumber: KOption = 3;
pub const Kfloat: KOption = 2;
pub const Kint: KOption = 0;
pub type KOption = u32;
pub const Kuint: KOption = 1;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct cD {
    pub c: std::ffi::c_char,
    pub u: C2RustUnnamed_17,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed_17 {
    pub n: lua_Number,
    pub u: std::ffi::c_double,
    pub s: *mut c_void,
    pub i: lua_Integer,
    pub l: std::ffi::c_long,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct MatchState {
    pub src_init: *const std::ffi::c_char,
    pub src_end: *const std::ffi::c_char,
    pub p_end: *const std::ffi::c_char,
    pub L: *mut lua_State,
    pub matchdepth: i32,
    pub level: u8,
    pub capture: [C2RustUnnamed_18; 32],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_18 {
    pub init: *const std::ffi::c_char,
    pub len: ptrdiff_t,
}
pub const _ISlower: C2RustUnnamed_19 = 512;
pub const _ISupper: C2RustUnnamed_19 = 256;
pub const _ISpunct: C2RustUnnamed_19 = 4;
pub const _ISgraph: C2RustUnnamed_19 = 32768;
pub const _IScntrl: C2RustUnnamed_19 = 2;
pub const _ISalpha: C2RustUnnamed_19 = 1024;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct GMatchState {
    pub src: *const std::ffi::c_char,
    pub p: *const std::ffi::c_char,
    pub lastmatch: *const std::ffi::c_char,
    pub ms: MatchState,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct str_Writer {
    pub init: i32,
    pub B: luaL_Buffer,
}
pub type utfint = u32;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct RanState {
    pub s: [usize; 4],
}
pub type OpMode = u8;
pub type RESERVED = u32;
pub type C2RustUnnamed_19 = u32;
pub const _ISblank: C2RustUnnamed_19 = 1;
pub const _ISprint: C2RustUnnamed_19 = 16384;

#[unsafe(no_mangle)]
pub static mut luai_ctype_: [lu_byte; 257] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x8, 0x8, 0x8, 0x8, 0x8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0xc, 0x4, 0x4, 0x4, 0x4, 0x4, 0x4, 0x4, 0x4, 0x4, 0x4, 0x4, 0x4, 0x4, 0x4, 0x4,
    0x16, 0x16, 0x16, 0x16, 0x16, 0x16, 0x16, 0x16, 0x16, 0x16, 0x4, 0x4, 0x4, 0x4, 0x4, 0x4, 0x4,
    0x15, 0x15, 0x15, 0x15, 0x15, 0x15, 0x5, 0x5, 0x5, 0x5, 0x5, 0x5, 0x5, 0x5, 0x5, 0x5, 0x5, 0x5,
    0x5, 0x5, 0x5, 0x5, 0x5, 0x5, 0x5, 0x5, 0x4, 0x4, 0x4, 0x4, 0x5, 0x4, 0x15, 0x15, 0x15, 0x15,
    0x15, 0x15, 0x5, 0x5, 0x5, 0x5, 0x5, 0x5, 0x5, 0x5, 0x5, 0x5, 0x5, 0x5, 0x5, 0x5, 0x5, 0x5,
    0x5, 0x5, 0x5, 0x5, 0x4, 0x4, 0x4, 0x4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

unsafe extern "C-unwind" fn dumpBlock(
    mut D: *mut DumpState,
    mut b: *const c_void,
    mut size: size_t,
) {
    if (*D).status == 0 && size > 0 as size_t {
        (*D).status = (Some(((*D).writer).expect("non-null function pointer")))
            .expect("non-null function pointer")((*D).L, b, size, (*D).data);
    }
}
unsafe extern "C-unwind" fn dumpByte(mut D: *mut DumpState, mut y: i32) {
    let mut x: lu_byte = y as lu_byte;
    dumpBlock(
        D,
        &mut x as *mut lu_byte as *const c_void,
        (1usize).wrapping_mul(size_of::<lu_byte>() as usize),
    );
}
unsafe extern "C-unwind" fn dumpSize(mut D: *mut DumpState, mut x: size_t) {
    let mut buff: [lu_byte; 10] = [0; 10];
    let mut n: i32 = 0;
    loop {
        n += 1;
        buff[(size_of::<size_t>() as usize)
            .wrapping_mul(8)
            .wrapping_add(6)
            .wrapping_div(7)
            .wrapping_sub(n as usize) as usize] = (x & 0x7f as i32 as size_t) as lu_byte;
        x >>= 7 as i32;
        if !(x != 0 as size_t) {
            break;
        }
    }
    buff[(size_of::<size_t>() as usize)
        .wrapping_mul(8)
        .wrapping_add(6)
        .wrapping_div(7)
        .wrapping_sub(1) as usize] = (buff[(size_of::<size_t>() as usize)
        .wrapping_mul(8)
        .wrapping_add(6)
        .wrapping_div(7)
        .wrapping_sub(1) as usize] as i32
        | 0x80) as lu_byte;
    dumpBlock(
        D,
        buff.as_mut_ptr()
            .offset(
                (size_of::<size_t>() as usize)
                    .wrapping_mul(8)
                    .wrapping_add(6)
                    .wrapping_div(7) as isize,
            )
            .offset(-(n as isize)) as *const c_void,
        (n as usize).wrapping_mul(size_of::<lu_byte>() as usize),
    );
}
unsafe extern "C-unwind" fn dumpInt(mut D: *mut DumpState, mut x: i32) {
    dumpSize(D, x as size_t);
}
unsafe extern "C-unwind" fn dumpNumber(mut D: *mut DumpState, mut x: lua_Number) {
    dumpBlock(
        D,
        &mut x as *mut lua_Number as *const c_void,
        (1usize).wrapping_mul(size_of::<lua_Number>() as usize),
    );
}
unsafe extern "C-unwind" fn dumpInteger(mut D: *mut DumpState, mut x: lua_Integer) {
    dumpBlock(
        D,
        &mut x as *mut lua_Integer as *const c_void,
        (1usize).wrapping_mul(size_of::<lua_Integer>() as usize),
    );
}
unsafe extern "C-unwind" fn dumpString(mut D: *mut DumpState, mut s: *const TString) {
    if s.is_null() {
        dumpSize(D, 0 as size_t);
    } else {
        let mut size: size_t = if (*s).shrlen as i32 != 0xff as i32 {
            (*s).shrlen as size_t
        } else {
            (*s).u.lnglen
        };
        let mut str: *const std::ffi::c_char = ((*s).contents).as_ptr();
        dumpSize(D, size.wrapping_add(1 as i32 as size_t));
        dumpBlock(
            D,
            str as *const c_void,
            size.wrapping_mul(size_of::<std::ffi::c_char>() as usize),
        );
    };
}
unsafe extern "C-unwind" fn dumpCode(mut D: *mut DumpState, mut f: *const Proto) {
    dumpInt(D, (*f).sizecode);
    dumpBlock(
        D,
        (*f).code as *const c_void,
        ((*f).sizecode as usize).wrapping_mul(size_of::<Instruction>() as usize),
    );
}
unsafe extern "C-unwind" fn dumpConstants(mut D: *mut DumpState, mut f: *const Proto) {
    let mut i: i32 = 0;
    let mut n: i32 = (*f).sizek;
    dumpInt(D, n);
    i = 0;
    while i < n {
        let mut o: *const TValue = &mut *((*f).k).offset(i as isize) as *mut TValue;
        let mut tt: i32 = (*o).tt_ as i32 & 0x3f as i32;
        dumpByte(D, tt);
        match tt {
            19 => {
                dumpNumber(D, (*o).value_.n);
            }
            3 => {
                dumpInteger(D, (*o).value_.i);
            }
            4 | 20 => {
                dumpString(D, &mut (*((*o).value_.gc as *mut GCUnion)).ts);
            }
            _ => {}
        }
        i += 1;
        i;
    }
}
unsafe extern "C-unwind" fn dumpProtos(mut D: *mut DumpState, mut f: *const Proto) {
    let mut i: i32 = 0;
    let mut n: i32 = (*f).sizep;
    dumpInt(D, n);
    i = 0;
    while i < n {
        dumpFunction(D, *((*f).p).offset(i as isize), (*f).source);
        i += 1;
        i;
    }
}
unsafe extern "C-unwind" fn dumpUpvalues(mut D: *mut DumpState, mut f: *const Proto) {
    let mut i: i32 = 0;
    let mut n: i32 = (*f).sizeupvalues;
    dumpInt(D, n);
    i = 0;
    while i < n {
        dumpByte(D, (*((*f).upvalues).offset(i as isize)).instack as i32);
        dumpByte(D, (*((*f).upvalues).offset(i as isize)).idx as i32);
        dumpByte(D, (*((*f).upvalues).offset(i as isize)).kind as i32);
        i += 1;
        i;
    }
}
unsafe extern "C-unwind" fn dumpDebug(mut D: *mut DumpState, mut f: *const Proto) {
    let mut i: i32 = 0;
    let mut n: i32 = 0;
    n = if (*D).strip != 0 {
        0
    } else {
        (*f).sizelineinfo
    };
    dumpInt(D, n);
    dumpBlock(
        D,
        (*f).lineinfo as *const c_void,
        (n as usize).wrapping_mul(size_of::<ls_byte>() as usize),
    );
    n = if (*D).strip != 0 {
        0
    } else {
        (*f).sizeabslineinfo
    };
    dumpInt(D, n);
    i = 0;
    while i < n {
        dumpInt(D, (*((*f).abslineinfo).offset(i as isize)).pc);
        dumpInt(D, (*((*f).abslineinfo).offset(i as isize)).line);
        i += 1;
        i;
    }
    n = if (*D).strip != 0 { 0 } else { (*f).sizelocvars };
    dumpInt(D, n);
    i = 0;
    while i < n {
        dumpString(D, (*((*f).locvars).offset(i as isize)).varname);
        dumpInt(D, (*((*f).locvars).offset(i as isize)).startpc);
        dumpInt(D, (*((*f).locvars).offset(i as isize)).endpc);
        i += 1;
        i;
    }
    n = if (*D).strip != 0 {
        0
    } else {
        (*f).sizeupvalues
    };
    dumpInt(D, n);
    i = 0;
    while i < n {
        dumpString(D, (*((*f).upvalues).offset(i as isize)).name);
        i += 1;
        i;
    }
}
unsafe extern "C-unwind" fn dumpFunction(
    mut D: *mut DumpState,
    mut f: *const Proto,
    mut psource: *mut TString,
) {
    if (*D).strip != 0 || (*f).source == psource {
        dumpString(D, 0 as *const TString);
    } else {
        dumpString(D, (*f).source);
    }
    dumpInt(D, (*f).linedefined);
    dumpInt(D, (*f).lastlinedefined);
    dumpByte(D, (*f).numparams as i32);
    dumpByte(D, (*f).is_vararg as i32);
    dumpByte(D, (*f).maxstacksize as i32);
    dumpCode(D, f);
    dumpConstants(D, f);
    dumpUpvalues(D, f);
    dumpProtos(D, f);
    dumpDebug(D, f);
}
unsafe extern "C-unwind" fn dumpHeader(mut D: *mut DumpState) {
    dumpBlock(
        D,
        c"\x1BLua".as_ptr() as *const c_void,
        (size_of::<[std::ffi::c_char; 5]>() as usize)
            .wrapping_sub(size_of::<std::ffi::c_char>() as usize),
    );
    dumpByte(D, 504 as i32 / 100 * 16 as i32 + 504 as i32 % 100);
    dumpByte(D, 0);
    dumpBlock(
        D,
        c"\x19\x93\r\n\x1A\n".as_ptr() as *const c_void,
        (size_of::<[std::ffi::c_char; 7]>() as usize)
            .wrapping_sub(size_of::<std::ffi::c_char>() as usize),
    );
    dumpByte(D, size_of::<Instruction>() as usize as i32);
    dumpByte(D, size_of::<lua_Integer>() as usize as i32);
    dumpByte(D, size_of::<lua_Number>() as usize as i32);
    dumpInteger(D, 0x5678 as i32 as lua_Integer);
    dumpNumber(D, 370.5f64);
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaU_dump(
    mut L: *mut lua_State,
    mut f: *const Proto,
    mut w: lua_Writer,
    mut data: *mut c_void,
    mut strip: i32,
) -> i32 {
    let mut D: DumpState = DumpState {
        L: 0 as *mut lua_State,
        writer: None,
        data: 0 as *mut c_void,
        strip: 0,
        status: 0,
    };
    D.L = L;
    D.writer = w;
    D.data = data;
    D.strip = strip;
    D.status = 0;
    dumpHeader(&mut D);
    dumpByte(&mut D, (*f).sizeupvalues);
    dumpFunction(&mut D, f, 0 as *mut TString);
    return D.status;
}
unsafe extern "C-unwind" fn luai_makeseed(mut L: *mut lua_State) -> u32 {
    let mut buff: [std::ffi::c_char; 24] = [0; 24];
    let mut h: u32 = time(0 as *mut time_t) as u32;
    let mut p: i32 = 0;
    let mut t: size_t = L as size_t;
    memcpy(
        buff.as_mut_ptr().offset(p as isize) as *mut c_void,
        &mut t as *mut size_t as *const c_void,
        size_of::<size_t>() as usize,
    );
    p = (p as usize).wrapping_add(size_of::<size_t>() as usize) as i32 as i32;
    let mut t_0: size_t = &mut h as *mut u32 as size_t;
    memcpy(
        buff.as_mut_ptr().offset(p as isize) as *mut c_void,
        &mut t_0 as *mut size_t as *const c_void,
        size_of::<size_t>() as usize,
    );
    p = (p as usize).wrapping_add(size_of::<size_t>() as usize) as i32 as i32;
    let mut t_1: size_t = ::core::mem::transmute::<
        Option<unsafe extern "C-unwind" fn(lua_Alloc, *mut c_void) -> *mut lua_State>,
        size_t,
    >(Some(
        lua_newstate as unsafe extern "C-unwind" fn(lua_Alloc, *mut c_void) -> *mut lua_State,
    ));
    memcpy(
        buff.as_mut_ptr().offset(p as isize) as *mut c_void,
        &mut t_1 as *mut size_t as *const c_void,
        size_of::<size_t>() as usize,
    );
    p = (p as usize).wrapping_add(size_of::<size_t>() as usize) as i32 as i32;
    return luaS_hash(buff.as_mut_ptr(), p as size_t, h);
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaE_setdebt(mut g: *mut global_State, mut debt: l_mem) {
    let mut tb: l_mem = ((*g).totalbytes + (*g).GCdebt) as lu_mem as l_mem;
    if debt < tb - (!(0 as lu_mem) >> 1 as i32) as l_mem {
        debt = tb - (!(0 as lu_mem) >> 1 as i32) as l_mem;
    }
    (*g).totalbytes = tb - debt;
    (*g).GCdebt = debt;
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn lua_setcstacklimit(mut L: *mut lua_State, mut limit: u32) -> i32 {
    return 200;
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaE_extendCI(mut L: *mut lua_State) -> *mut CallInfo {
    let mut ci: *mut CallInfo = 0 as *mut CallInfo;
    ci = luaM_malloc_(L, size_of::<CallInfo>() as usize, 0) as *mut CallInfo;
    (*(*L).ci).next = ci;
    (*ci).previous = (*L).ci;
    (*ci).next = 0 as *mut CallInfo;
    ::core::ptr::write_volatile(&mut (*ci).u.l.trap as *mut sig_atomic_t, 0);
    (*L).nci = ((*L).nci).wrapping_add(1);
    (*L).nci;
    return ci;
}
unsafe extern "C-unwind" fn freeCI(mut L: *mut lua_State) {
    let mut ci: *mut CallInfo = (*L).ci;
    let mut next: *mut CallInfo = (*ci).next;
    (*ci).next = 0 as *mut CallInfo;
    loop {
        ci = next;
        if ci.is_null() {
            break;
        }
        next = (*ci).next;
        luaM_free_(L, ci as *mut c_void, size_of::<CallInfo>() as usize);
        (*L).nci = ((*L).nci).wrapping_sub(1);
        (*L).nci;
    }
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaE_shrinkCI(mut L: *mut lua_State) {
    let mut ci: *mut CallInfo = (*(*L).ci).next;
    let mut next: *mut CallInfo = 0 as *mut CallInfo;
    if ci.is_null() {
        return;
    }
    loop {
        next = (*ci).next;
        if next.is_null() {
            break;
        }
        let mut next2: *mut CallInfo = (*next).next;
        (*ci).next = next2;
        (*L).nci = ((*L).nci).wrapping_sub(1);
        (*L).nci;
        luaM_free_(L, next as *mut c_void, size_of::<CallInfo>() as usize);
        if next2.is_null() {
            break;
        }
        (*next2).previous = ci;
        ci = next2;
    }
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaE_checkcstack(mut L: *mut lua_State) {
    if (*L).nCcalls & 0xffff as i32 as l_uint32 == 200 as l_uint32 {
        luaG_runerror(L, c"C stack overflow".as_ptr());
    } else if (*L).nCcalls & 0xffff as i32 as l_uint32 >= (200 / 10 * 11 as i32) as l_uint32 {
        luaD_throw(L, 5 as i32);
    }
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaE_incCstack(mut L: *mut lua_State) {
    (*L).nCcalls = ((*L).nCcalls).wrapping_add(1);
    (*L).nCcalls;
    if (((*L).nCcalls & 0xffff as i32 as l_uint32 >= 200 as l_uint32) as i32 != 0) as i32
        as std::ffi::c_long
        != 0
    {
        luaE_checkcstack(L);
    }
}
unsafe extern "C-unwind" fn stack_init(mut L1: *mut lua_State, mut L: *mut lua_State) {
    let mut i: i32 = 0;
    let mut ci: *mut CallInfo = 0 as *mut CallInfo;
    (*L1).stack.p = luaM_malloc_(
        L,
        ((2 as i32 * 20 + 5 as i32) as usize).wrapping_mul(size_of::<StackValue>() as usize),
        0,
    ) as *mut StackValue;
    (*L1).tbclist.p = (*L1).stack.p;
    i = 0;
    while i < 2 as i32 * 20 + 5 as i32 {
        (*((*L1).stack.p).offset(i as isize)).val.tt_ = (0 | (0) << 4 as i32) as lu_byte;
        i += 1;
        i;
    }
    (*L1).top.p = (*L1).stack.p;
    (*L1).stack_last.p = ((*L1).stack.p).offset((2 as i32 * 20) as isize);
    ci = &mut (*L1).base_ci;
    (*ci).previous = 0 as *mut CallInfo;
    (*ci).next = (*ci).previous;
    (*ci).callstatus = ((1 as i32) << 1 as i32) as u16;
    (*ci).func.p = (*L1).top.p;
    (*ci).u.c.k = None;
    (*ci).nresults = 0 as i16;
    (*(*L1).top.p).val.tt_ = (0 | (0) << 4 as i32) as lu_byte;
    (*L1).top.p = ((*L1).top.p).offset(1);
    (*L1).top.p;
    (*ci).top.p = ((*L1).top.p).offset(20 as isize);
    (*L1).ci = ci;
}
unsafe extern "C-unwind" fn freestack(mut L: *mut lua_State) {
    if ((*L).stack.p).is_null() {
        return;
    }
    (*L).ci = &mut (*L).base_ci;
    freeCI(L);
    luaM_free_(
        L,
        (*L).stack.p as *mut c_void,
        ((((*L).stack_last.p).offset_from((*L).stack.p) as std::ffi::c_long as i32 + 5 as i32)
            as usize)
            .wrapping_mul(size_of::<StackValue>() as usize),
    );
}
unsafe extern "C-unwind" fn init_registry(mut L: *mut lua_State, mut g: *mut global_State) {
    let mut registry: *mut Table = luaH_new(L);
    let mut io: *mut TValue = &mut (*g).l_registry;
    let mut x_: *mut Table = registry;
    (*io).value_.gc = &mut (*(x_ as *mut GCUnion)).gc;
    (*io).tt_ = (5 as i32 | (0) << 4 as i32 | (1 as i32) << 6 as i32) as lu_byte;
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
    luaH_resize(L, registry, 2, 0 as u32);
    let mut io_0: *mut TValue =
        &mut *((*registry).array).offset((1 as i32 - 1 as i32) as isize) as *mut TValue;
    let mut x__0: *mut lua_State = L;
    (*io_0).value_.gc = &mut (*(x__0 as *mut GCUnion)).gc;
    (*io_0).tt_ = (8 as i32 | (0) << 4 as i32 | (1 as i32) << 6 as i32) as lu_byte;
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
    let mut io_1: *mut TValue =
        &mut *((*registry).array).offset((2 as i32 - 1 as i32) as isize) as *mut TValue;
    let mut x__1: *mut Table = luaH_new(L);
    (*io_1).value_.gc = &mut (*(x__1 as *mut GCUnion)).gc;
    (*io_1).tt_ = (5 as i32 | (0) << 4 as i32 | (1 as i32) << 6 as i32) as lu_byte;
    if (*io_1).tt_ as i32 & (1 as i32) << 6 as i32 == 0
        || (*io_1).tt_ as i32 & 0x3f as i32 == (*(*io_1).value_.gc).tt as i32
            && (L.is_null()
                || (*(*io_1).value_.gc).marked as i32
                    & ((*(*L).l_G).currentwhite as i32
                        ^ ((1 as i32) << 3 as i32 | (1 as i32) << 4 as i32))
                    == 0)
    {
    } else {
    };
}
unsafe extern "C-unwind" fn f_luaopen(mut L: *mut lua_State, mut ud: *mut c_void) {
    let mut g: *mut global_State = (*L).l_G;
    stack_init(L, L);
    init_registry(L, g);
    luaS_init(L);
    luaT_init(L);
    luaX_init(L);
    (*g).gcstp = 0 as lu_byte;
    (*g).nilvalue.tt_ = (0 | (0) << 4 as i32) as lu_byte;
}
unsafe extern "C-unwind" fn preinit_thread(mut L: *mut lua_State, mut g: *mut global_State) {
    (*L).l_G = g;
    (*L).stack.p = 0 as StkId;
    (*L).ci = 0 as *mut CallInfo;
    (*L).nci = 0 as u16;
    (*L).twups = L;
    (*L).nCcalls = 0 as l_uint32;
    (*L).errorJmp = 0 as *mut lua_longjmp;
    ::core::ptr::write_volatile(&mut (*L).hook as *mut lua_Hook, None);
    ::core::ptr::write_volatile(&mut (*L).hookmask as *mut sig_atomic_t, 0);
    (*L).basehookcount = 0;
    (*L).allowhook = 1 as i32 as lu_byte;
    (*L).hookcount = (*L).basehookcount;
    (*L).openupval = 0 as *mut UpVal;
    (*L).status = 0 as lu_byte;
    (*L).errfunc = 0 as ptrdiff_t;
    (*L).oldpc = 0;
}
unsafe extern "C-unwind" fn close_state(mut L: *mut lua_State) {
    let mut g: *mut global_State = (*L).l_G;
    if !((*g).nilvalue.tt_ as i32 & 0xf as i32 == 0) {
        luaC_freeallobjects(L);
    } else {
        (*L).ci = &mut (*L).base_ci;
        luaD_closeprotected(L, 1 as i32 as ptrdiff_t, 0);
        luaC_freeallobjects(L);
    }
    luaM_free_(
        L,
        (*(*L).l_G).strt.hash as *mut c_void,
        ((*(*L).l_G).strt.size as usize).wrapping_mul(size_of::<*mut TString>() as usize),
    );
    freestack(L);
    (Some(((*g).frealloc).expect("non-null function pointer"))).expect("non-null function pointer")(
        (*g).ud,
        (L as *mut lu_byte).offset(-(8 as usize as isize)) as *mut LX as *mut c_void,
        size_of::<LG>() as usize,
        0 as size_t,
    );
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn lua_newthread(mut L: *mut lua_State) -> *mut lua_State {
    let mut g: *mut global_State = (*L).l_G;
    let mut o: *mut GCObject = 0 as *mut GCObject;
    let mut L1: *mut lua_State = 0 as *mut lua_State;
    if (*(*L).l_G).GCdebt > 0 as l_mem {
        luaC_step(L);
    }
    o = luaC_newobjdt(L, 8 as i32, size_of::<LX>() as usize, 8 as usize);
    L1 = &mut (*(o as *mut GCUnion)).th;
    let mut io: *mut TValue = &mut (*(*L).top.p).val;
    let mut x_: *mut lua_State = L1;
    (*io).value_.gc = &mut (*(x_ as *mut GCUnion)).gc;
    (*io).tt_ = (8 as i32 | (0) << 4 as i32 | (1 as i32) << 6 as i32) as lu_byte;
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
    preinit_thread(L1, g);
    ::core::ptr::write_volatile(&mut (*L1).hookmask as *mut sig_atomic_t, (*L).hookmask);
    (*L1).basehookcount = (*L).basehookcount;
    ::core::ptr::write_volatile(&mut (*L1).hook as *mut lua_Hook, (*L).hook);
    (*L1).hookcount = (*L1).basehookcount;
    memcpy(
        (L1 as *mut std::ffi::c_char).offset(-(size_of::<*mut c_void>() as usize as isize))
            as *mut c_void,
        ((*g).mainthread as *mut std::ffi::c_char)
            .offset(-(size_of::<*mut c_void>() as usize as isize)) as *mut c_void,
        size_of::<*mut c_void>() as usize,
    );
    stack_init(L1, L);
    return L1;
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaE_freethread(mut L: *mut lua_State, mut L1: *mut lua_State) {
    let mut l: *mut LX = (L1 as *mut lu_byte).offset(-(8 as usize as isize)) as *mut LX;
    luaF_closeupval(L1, (*L1).stack.p);
    freestack(L1);
    luaM_free_(L, l as *mut c_void, size_of::<LX>() as usize);
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaE_resetthread(mut L: *mut lua_State, mut status: i32) -> i32 {
    (*L).ci = &mut (*L).base_ci;
    let mut ci: *mut CallInfo = (*L).ci;
    (*(*L).stack.p).val.tt_ = (0 | (0) << 4 as i32) as lu_byte;
    (*ci).func.p = (*L).stack.p;
    (*ci).callstatus = ((1 as i32) << 1 as i32) as u16;
    if status == 1 as i32 {
        status = 0;
    }
    (*L).status = 0 as lu_byte;
    status = luaD_closeprotected(L, 1 as i32 as ptrdiff_t, status);
    if status != 0 {
        luaD_seterrorobj(L, status, ((*L).stack.p).offset(1));
    } else {
        (*L).top.p = ((*L).stack.p).offset(1);
    }
    (*ci).top.p = ((*L).top.p).offset(20 as isize);
    luaD_reallocstack(
        L,
        ((*ci).top.p).offset_from((*L).stack.p) as std::ffi::c_long as i32,
        0,
    );
    return status;
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn lua_closethread(
    mut L: *mut lua_State,
    mut from: *mut lua_State,
) -> i32 {
    let mut status: i32 = 0;
    (*L).nCcalls = if !from.is_null() {
        (*from).nCcalls & 0xffff as i32 as l_uint32
    } else {
        0 as l_uint32
    };
    status = luaE_resetthread(L, (*L).status as i32);
    return status;
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn lua_resetthread(mut L: *mut lua_State) -> i32 {
    return lua_closethread(L, 0 as *mut lua_State);
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn lua_newstate(
    mut f: lua_Alloc,
    mut ud: *mut c_void,
) -> *mut lua_State {
    let mut i: i32 = 0;
    let mut L: *mut lua_State = 0 as *mut lua_State;
    let mut g: *mut global_State = 0 as *mut global_State;
    let mut l: *mut LG = (Some(f.expect("non-null function pointer")))
        .expect("non-null function pointer")(
        ud,
        0 as *mut c_void,
        8 as i32 as size_t,
        size_of::<LG>() as usize,
    ) as *mut LG;
    if l.is_null() {
        return 0 as *mut lua_State;
    }
    L = &mut (*l).l.l;
    g = &mut (*l).g;
    (*L).tt = (8 as i32 | (0) << 4 as i32) as lu_byte;
    (*g).currentwhite = ((1 as i32) << 3 as i32) as lu_byte;
    (*L).marked =
        ((*g).currentwhite as i32 & ((1 as i32) << 3 as i32 | (1 as i32) << 4 as i32)) as lu_byte;
    preinit_thread(L, g);
    (*g).allgc = &mut (*(L as *mut GCUnion)).gc;
    (*L).next = 0 as *mut GCObject;
    (*L).nCcalls = ((*L).nCcalls).wrapping_add(0x10000 as l_uint32);
    (*g).frealloc = f;
    (*g).ud = ud;
    (*g).warnf = None;
    (*g).ud_warn = 0 as *mut c_void;
    (*g).mainthread = L;
    (*g).seed = luai_makeseed(L);
    (*g).gcstp = 2 as i32 as lu_byte;
    (*g).strt.nuse = 0;
    (*g).strt.size = (*g).strt.nuse;
    (*g).strt.hash = 0 as *mut *mut TString;
    (*g).l_registry.tt_ = (0 | (0) << 4 as i32) as lu_byte;
    (*g).panic = None;
    (*g).gcstate = 8 as i32 as lu_byte;
    (*g).gckind = 0 as lu_byte;
    (*g).gcstopem = 0 as lu_byte;
    (*g).gcemergency = 0 as lu_byte;
    (*g).fixedgc = 0 as *mut GCObject;
    (*g).tobefnz = (*g).fixedgc;
    (*g).finobj = (*g).tobefnz;
    (*g).reallyold = 0 as *mut GCObject;
    (*g).old1 = (*g).reallyold;
    (*g).survival = (*g).old1;
    (*g).firstold1 = (*g).survival;
    (*g).finobjrold = 0 as *mut GCObject;
    (*g).finobjold1 = (*g).finobjrold;
    (*g).finobjsur = (*g).finobjold1;
    (*g).sweepgc = 0 as *mut *mut GCObject;
    (*g).grayagain = 0 as *mut GCObject;
    (*g).gray = (*g).grayagain;
    (*g).allweak = 0 as *mut GCObject;
    (*g).ephemeron = (*g).allweak;
    (*g).weak = (*g).ephemeron;
    (*g).twups = 0 as *mut lua_State;
    (*g).totalbytes = size_of::<LG>() as usize as l_mem;
    (*g).GCdebt = 0 as l_mem;
    (*g).lastatomic = 0 as lu_mem;
    let mut io: *mut TValue = &mut (*g).nilvalue;
    (*io).value_.i = 0 as lua_Integer;
    (*io).tt_ = (3 as i32 | (0) << 4 as i32) as lu_byte;
    (*g).gcpause = (200 / 4 as i32) as lu_byte;
    (*g).gcstepmul = (100 / 4 as i32) as lu_byte;
    (*g).gcstepsize = 13 as i32 as lu_byte;
    (*g).genmajormul = (100 / 4 as i32) as lu_byte;
    (*g).genminormul = 20 as lu_byte;
    i = 0;
    while i < 9 as i32 {
        (*g).mt[i as usize] = 0 as *mut Table;
        i += 1;
        i;
    }
    if luaD_rawrunprotected(
        L,
        Some(f_luaopen as unsafe extern "C-unwind" fn(*mut lua_State, *mut c_void) -> ()),
        0 as *mut c_void,
    ) != 0
    {
        close_state(L);
        L = 0 as *mut lua_State;
    }
    return L;
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn lua_close(mut L: *mut lua_State) {
    L = (*(*L).l_G).mainthread;
    close_state(L);
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaE_warning(
    mut L: *mut lua_State,
    mut msg: *const std::ffi::c_char,
    mut tocont: i32,
) {
    let mut wf: lua_WarnFunction = (*(*L).l_G).warnf;
    if wf.is_some() {
        wf.expect("non-null function pointer")((*(*L).l_G).ud_warn, msg, tocont);
    }
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaE_warnerror(
    mut L: *mut lua_State,
    mut where_0: *const std::ffi::c_char,
) {
    let mut errobj: *mut TValue = &mut (*((*L).top.p).offset(-(1))).val;
    let mut msg: *const std::ffi::c_char = if (*errobj).tt_ as i32 & 0xf as i32 == 4 as i32 {
        ((*(&mut (*((*errobj).value_.gc as *mut GCUnion)).ts as *mut TString)).contents)
            .as_mut_ptr() as *const std::ffi::c_char
    } else {
        c"error object is not a string".as_ptr()
    };
    luaE_warning(L, c"error in ".as_ptr(), 1 as i32);
    luaE_warning(L, where_0, 1 as i32);
    luaE_warning(L, c" (".as_ptr(), 1 as i32);
    luaE_warning(L, msg, 1 as i32);
    luaE_warning(L, c")".as_ptr(), 0);
}

static mut luaX_tokens: [*const std::ffi::c_char; 37] = [
    c"and".as_ptr(),
    c"break".as_ptr(),
    c"do".as_ptr(),
    c"else".as_ptr(),
    c"elseif".as_ptr(),
    c"end".as_ptr(),
    c"false".as_ptr(),
    c"for".as_ptr(),
    c"function".as_ptr(),
    c"goto".as_ptr(),
    c"if".as_ptr(),
    c"in".as_ptr(),
    c"local".as_ptr(),
    c"nil".as_ptr(),
    c"not".as_ptr(),
    c"or".as_ptr(),
    c"repeat".as_ptr(),
    c"return".as_ptr(),
    c"then".as_ptr(),
    c"true".as_ptr(),
    c"until".as_ptr(),
    c"while".as_ptr(),
    c"//".as_ptr(),
    c"..".as_ptr(),
    c"...".as_ptr(),
    c"==".as_ptr(),
    c">=".as_ptr(),
    c"<=".as_ptr(),
    c"~=".as_ptr(),
    c"<<".as_ptr(),
    c">>".as_ptr(),
    c"::".as_ptr(),
    c"<eof>".as_ptr(),
    c"<number>".as_ptr(),
    c"<integer>".as_ptr(),
    c"<name>".as_ptr(),
    c"<string>".as_ptr(),
];
unsafe extern "C-unwind" fn save(mut ls: *mut LexState, mut c: i32) {
    let mut b: *mut Mbuffer = (*ls).buff;
    if ((*b).n).wrapping_add(1 as i32 as size_t) > (*b).buffsize {
        let mut newsize: size_t = 0;
        if (*b).buffsize
            >= (if (size_of::<size_t>() as usize) < size_of::<lua_Integer>() as usize {
                !(0 as size_t)
            } else {
                9223372036854775807 as std::ffi::c_longlong as size_t
            }) / 2 as i32 as size_t
        {
            lexerror(ls, c"lexical element too long".as_ptr(), 0);
        }
        newsize = (*b).buffsize * 2 as i32 as size_t;
        (*b).buffer = luaM_saferealloc_(
            (*ls).L,
            (*b).buffer as *mut c_void,
            ((*b).buffsize).wrapping_mul(size_of::<std::ffi::c_char>() as usize),
            newsize.wrapping_mul(size_of::<std::ffi::c_char>() as usize),
        ) as *mut std::ffi::c_char;
        (*b).buffsize = newsize;
    }
    let fresh11 = (*b).n;
    (*b).n = ((*b).n).wrapping_add(1);
    *((*b).buffer).offset(fresh11 as isize) = c as std::ffi::c_char;
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaX_init(mut L: *mut lua_State) {
    let mut i: i32 = 0;
    let mut e: *mut TString = luaS_newlstr(
        L,
        c"_ENV".as_ptr(),
        (size_of::<[std::ffi::c_char; 5]>() as usize)
            .wrapping_div(size_of::<std::ffi::c_char>() as usize)
            .wrapping_sub(1),
    );
    luaC_fix(L, &mut (*(e as *mut GCUnion)).gc);
    i = 0;
    while i < TK_WHILE as i32 - (127 as i32 * 2 as i32 + 1 as i32 + 1 as i32) + 1 as i32 {
        let mut ts: *mut TString = luaS_new(L, luaX_tokens[i as usize]);
        luaC_fix(L, &mut (*(ts as *mut GCUnion)).gc);
        (*ts).extra = (i + 1 as i32) as lu_byte;
        i += 1;
        i;
    }
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaX_token2str(
    mut ls: *mut LexState,
    mut token: i32,
) -> *const std::ffi::c_char {
    if token < 127 as i32 * 2 as i32 + 1 as i32 + 1 as i32 {
        if luai_ctype_[(token + 1 as i32) as usize] as i32 & (1 as i32) << 2 as i32 != 0 {
            return luaO_pushfstring((*ls).L, c"'%c'".as_ptr(), token);
        } else {
            return luaO_pushfstring((*ls).L, c"'<\\%d>'".as_ptr(), token);
        }
    } else {
        let mut s: *const std::ffi::c_char =
            luaX_tokens[(token - (127 as i32 * 2 as i32 + 1 as i32 + 1 as i32)) as usize];
        if token < TK_EOS as i32 {
            return luaO_pushfstring((*ls).L, c"'%s'".as_ptr(), s);
        } else {
            return s;
        }
    };
}
unsafe extern "C-unwind" fn txtToken(
    mut ls: *mut LexState,
    mut token: i32,
) -> *const std::ffi::c_char {
    match token {
        291 | 292 | 289 | 290 => {
            save(ls, '\0' as i32);
            return luaO_pushfstring((*ls).L, c"'%s'".as_ptr(), (*(*ls).buff).buffer);
        }
        _ => return luaX_token2str(ls, token),
    };
}
unsafe extern "C-unwind" fn lexerror(
    mut ls: *mut LexState,
    mut msg: *const std::ffi::c_char,
    mut token: i32,
) -> ! {
    msg = luaG_addinfo((*ls).L, msg, (*ls).source, (*ls).linenumber);
    if token != 0 {
        luaO_pushfstring((*ls).L, c"%s near %s".as_ptr(), msg, txtToken(ls, token));
    }
    luaD_throw((*ls).L, 3 as i32);
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaX_syntaxerror(
    mut ls: *mut LexState,
    mut msg: *const std::ffi::c_char,
) -> ! {
    lexerror(ls, msg, (*ls).t.token);
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaX_newstring(
    mut ls: *mut LexState,
    mut str: *const std::ffi::c_char,
    mut l: size_t,
) -> *mut TString {
    let mut L: *mut lua_State = (*ls).L;
    let mut ts: *mut TString = luaS_newlstr(L, str, l);
    let mut o: *const TValue = luaH_getstr((*ls).h, ts);
    if !((*o).tt_ as i32 & 0xf as i32 == 0) {
        ts = &mut (*((*(o as *mut Node)).u.key_val.gc as *mut GCUnion)).ts;
    } else {
        let fresh12 = (*L).top.p;
        (*L).top.p = ((*L).top.p).offset(1);
        let mut stv: *mut TValue = &mut (*fresh12).val;
        let mut io: *mut TValue = stv;
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
        luaH_finishset(L, (*ls).h, stv, o, stv);
        if (*(*L).l_G).GCdebt > 0 as l_mem {
            luaC_step(L);
        }
        (*L).top.p = ((*L).top.p).offset(-1);
        (*L).top.p;
    }
    return ts;
}
unsafe extern "C-unwind" fn inclinenumber(mut ls: *mut LexState) {
    let mut old: i32 = (*ls).current;
    let fresh13 = (*(*ls).z).n;
    (*(*ls).z).n = ((*(*ls).z).n).wrapping_sub(1);
    (*ls).current = if fresh13 > 0 as size_t {
        let fresh14 = (*(*ls).z).p;
        (*(*ls).z).p = ((*(*ls).z).p).offset(1);
        *fresh14 as u8 as i32
    } else {
        luaZ_fill((*ls).z)
    };
    if ((*ls).current == '\n' as i32 || (*ls).current == '\r' as i32) && (*ls).current != old {
        let fresh15 = (*(*ls).z).n;
        (*(*ls).z).n = ((*(*ls).z).n).wrapping_sub(1);
        (*ls).current = if fresh15 > 0 as size_t {
            let fresh16 = (*(*ls).z).p;
            (*(*ls).z).p = ((*(*ls).z).p).offset(1);
            *fresh16 as u8 as i32
        } else {
            luaZ_fill((*ls).z)
        };
    }
    (*ls).linenumber += 1;
    if (*ls).linenumber >= 2147483647 as i32 {
        lexerror(ls, c"chunk has too many lines".as_ptr(), 0);
    }
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaX_setinput(
    mut L: *mut lua_State,
    mut ls: *mut LexState,
    mut z: *mut ZIO,
    mut source: *mut TString,
    mut firstchar: i32,
) {
    (*ls).t.token = 0;
    (*ls).L = L;
    (*ls).current = firstchar;
    (*ls).lookahead.token = TK_EOS as i32;
    (*ls).z = z;
    (*ls).fs = 0 as *mut FuncState;
    (*ls).linenumber = 1 as i32;
    (*ls).lastline = 1 as i32;
    (*ls).source = source;
    (*ls).envn = luaS_newlstr(
        L,
        c"_ENV".as_ptr(),
        (size_of::<[std::ffi::c_char; 5]>() as usize)
            .wrapping_div(size_of::<std::ffi::c_char>() as usize)
            .wrapping_sub(1),
    );
    (*(*ls).buff).buffer = luaM_saferealloc_(
        (*ls).L,
        (*(*ls).buff).buffer as *mut c_void,
        ((*(*ls).buff).buffsize).wrapping_mul(size_of::<std::ffi::c_char>() as usize),
        (32usize).wrapping_mul(size_of::<std::ffi::c_char>() as usize),
    ) as *mut std::ffi::c_char;
    (*(*ls).buff).buffsize = 32 as i32 as size_t;
}
unsafe extern "C-unwind" fn check_next1(mut ls: *mut LexState, mut c: i32) -> i32 {
    if (*ls).current == c {
        let fresh17 = (*(*ls).z).n;
        (*(*ls).z).n = ((*(*ls).z).n).wrapping_sub(1);
        (*ls).current = if fresh17 > 0 as size_t {
            let fresh18 = (*(*ls).z).p;
            (*(*ls).z).p = ((*(*ls).z).p).offset(1);
            *fresh18 as u8 as i32
        } else {
            luaZ_fill((*ls).z)
        };
        return 1 as i32;
    } else {
        return 0;
    };
}
unsafe extern "C-unwind" fn check_next2(
    mut ls: *mut LexState,
    mut set: *const std::ffi::c_char,
) -> i32 {
    if (*ls).current == *set.offset(0 as isize) as i32 || (*ls).current == *set.offset(1) as i32 {
        save(ls, (*ls).current);
        let fresh19 = (*(*ls).z).n;
        (*(*ls).z).n = ((*(*ls).z).n).wrapping_sub(1);
        (*ls).current = (if fresh19 > 0 as size_t {
            let fresh20 = (*(*ls).z).p;
            (*(*ls).z).p = ((*(*ls).z).p).offset(1);
            *fresh20 as u8 as i32
        } else {
            luaZ_fill((*ls).z)
        });
        return 1 as i32;
    } else {
        return 0;
    };
}
unsafe extern "C-unwind" fn read_numeral(mut ls: *mut LexState, mut seminfo: *mut SemInfo) -> i32 {
    let mut obj: TValue = TValue {
        value_: Value {
            gc: 0 as *mut GCObject,
        },
        tt_: 0,
    };
    let mut expo: *const std::ffi::c_char = c"Ee".as_ptr();
    let mut first: i32 = (*ls).current;
    save(ls, (*ls).current);
    let fresh21 = (*(*ls).z).n;
    (*(*ls).z).n = ((*(*ls).z).n).wrapping_sub(1);
    (*ls).current = (if fresh21 > 0 as size_t {
        let fresh22 = (*(*ls).z).p;
        (*(*ls).z).p = ((*(*ls).z).p).offset(1);
        *fresh22 as u8 as i32
    } else {
        luaZ_fill((*ls).z)
    });
    if first == '0' as i32 && check_next2(ls, c"xX".as_ptr()) != 0 {
        expo = c"Pp".as_ptr();
    }
    loop {
        if check_next2(ls, expo) != 0 {
            check_next2(ls, c"-+".as_ptr());
        } else {
            if !(luai_ctype_[((*ls).current + 1 as i32) as usize] as i32 & (1 as i32) << 4 as i32
                != 0
                || (*ls).current == '.' as i32)
            {
                break;
            }
            save(ls, (*ls).current);
            let fresh23 = (*(*ls).z).n;
            (*(*ls).z).n = ((*(*ls).z).n).wrapping_sub(1);
            (*ls).current = (if fresh23 > 0 as size_t {
                let fresh24 = (*(*ls).z).p;
                (*(*ls).z).p = ((*(*ls).z).p).offset(1);
                *fresh24 as u8 as i32
            } else {
                luaZ_fill((*ls).z)
            });
        }
    }
    if luai_ctype_[((*ls).current + 1 as i32) as usize] as i32 & (1 as i32) << 0 != 0 {
        save(ls, (*ls).current);
        let fresh25 = (*(*ls).z).n;
        (*(*ls).z).n = ((*(*ls).z).n).wrapping_sub(1);
        (*ls).current = (if fresh25 > 0 as size_t {
            let fresh26 = (*(*ls).z).p;
            (*(*ls).z).p = ((*(*ls).z).p).offset(1);
            *fresh26 as u8 as i32
        } else {
            luaZ_fill((*ls).z)
        });
    }
    save(ls, '\0' as i32);
    if luaO_str2num((*(*ls).buff).buffer, &mut obj) == 0 as size_t {
        lexerror(ls, c"malformed number".as_ptr(), TK_FLT as i32);
    }
    if obj.tt_ as i32 == 3 as i32 | (0) << 4 as i32 {
        (*seminfo).i = obj.value_.i;
        return TK_INT as i32;
    } else {
        (*seminfo).r = obj.value_.n;
        return TK_FLT as i32;
    };
}
unsafe extern "C-unwind" fn skip_sep(mut ls: *mut LexState) -> size_t {
    let mut count: size_t = 0 as size_t;
    let mut s: i32 = (*ls).current;
    save(ls, (*ls).current);
    let fresh27 = (*(*ls).z).n;
    (*(*ls).z).n = ((*(*ls).z).n).wrapping_sub(1);
    (*ls).current = (if fresh27 > 0 as size_t {
        let fresh28 = (*(*ls).z).p;
        (*(*ls).z).p = ((*(*ls).z).p).offset(1);
        *fresh28 as u8 as i32
    } else {
        luaZ_fill((*ls).z)
    });
    while (*ls).current == '=' as i32 {
        save(ls, (*ls).current);
        let fresh29 = (*(*ls).z).n;
        (*(*ls).z).n = ((*(*ls).z).n).wrapping_sub(1);
        (*ls).current = (if fresh29 > 0 as size_t {
            let fresh30 = (*(*ls).z).p;
            (*(*ls).z).p = ((*(*ls).z).p).offset(1);
            *fresh30 as u8 as i32
        } else {
            luaZ_fill((*ls).z)
        });
        count = count.wrapping_add(1);
        count;
    }
    return if (*ls).current == s {
        count.wrapping_add(2 as i32 as size_t)
    } else {
        (if count == 0 as size_t { 1 as i32 } else { 0 }) as size_t
    };
}
unsafe extern "C-unwind" fn read_long_string(
    mut ls: *mut LexState,
    mut seminfo: *mut SemInfo,
    mut sep: size_t,
) {
    let mut line: i32 = (*ls).linenumber;
    save(ls, (*ls).current);
    let fresh31 = (*(*ls).z).n;
    (*(*ls).z).n = ((*(*ls).z).n).wrapping_sub(1);
    (*ls).current = (if fresh31 > 0 as size_t {
        let fresh32 = (*(*ls).z).p;
        (*(*ls).z).p = ((*(*ls).z).p).offset(1);
        *fresh32 as u8 as i32
    } else {
        luaZ_fill((*ls).z)
    });
    if (*ls).current == '\n' as i32 || (*ls).current == '\r' as i32 {
        inclinenumber(ls);
    }
    loop {
        match (*ls).current {
            -1 => {
                let mut what: *const std::ffi::c_char = if !seminfo.is_null() {
                    c"string".as_ptr()
                } else {
                    c"comment".as_ptr()
                };
                let mut msg: *const std::ffi::c_char = luaO_pushfstring(
                    (*ls).L,
                    c"unfinished long %s (starting at line %d)".as_ptr(),
                    what,
                    line,
                );
                lexerror(ls, msg, TK_EOS as i32);
            }
            93 => {
                if !(skip_sep(ls) == sep) {
                    continue;
                }
                save(ls, (*ls).current);
                let fresh33 = (*(*ls).z).n;
                (*(*ls).z).n = ((*(*ls).z).n).wrapping_sub(1);
                (*ls).current = (if fresh33 > 0 as size_t {
                    let fresh34 = (*(*ls).z).p;
                    (*(*ls).z).p = ((*(*ls).z).p).offset(1);
                    *fresh34 as u8 as i32
                } else {
                    luaZ_fill((*ls).z)
                });
                break;
            }
            10 | 13 => {
                save(ls, '\n' as i32);
                inclinenumber(ls);
                if seminfo.is_null() {
                    (*(*ls).buff).n = 0 as size_t;
                }
            }
            _ => {
                if !seminfo.is_null() {
                    save(ls, (*ls).current);
                    let fresh35 = (*(*ls).z).n;
                    (*(*ls).z).n = ((*(*ls).z).n).wrapping_sub(1);
                    (*ls).current = (if fresh35 > 0 as size_t {
                        let fresh36 = (*(*ls).z).p;
                        (*(*ls).z).p = ((*(*ls).z).p).offset(1);
                        *fresh36 as u8 as i32
                    } else {
                        luaZ_fill((*ls).z)
                    });
                } else {
                    let fresh37 = (*(*ls).z).n;
                    (*(*ls).z).n = ((*(*ls).z).n).wrapping_sub(1);
                    (*ls).current = if fresh37 > 0 as size_t {
                        let fresh38 = (*(*ls).z).p;
                        (*(*ls).z).p = ((*(*ls).z).p).offset(1);
                        *fresh38 as u8 as i32
                    } else {
                        luaZ_fill((*ls).z)
                    };
                }
            }
        }
    }
    if !seminfo.is_null() {
        (*seminfo).ts = luaX_newstring(
            ls,
            ((*(*ls).buff).buffer).offset(sep as isize),
            ((*(*ls).buff).n).wrapping_sub(2 as i32 as size_t * sep),
        );
    }
}
unsafe extern "C-unwind" fn esccheck(
    mut ls: *mut LexState,
    mut c: i32,
    mut msg: *const std::ffi::c_char,
) {
    if c == 0 {
        if (*ls).current != -(1 as i32) {
            save(ls, (*ls).current);
            let fresh39 = (*(*ls).z).n;
            (*(*ls).z).n = ((*(*ls).z).n).wrapping_sub(1);
            (*ls).current = (if fresh39 > 0 as size_t {
                let fresh40 = (*(*ls).z).p;
                (*(*ls).z).p = ((*(*ls).z).p).offset(1);
                *fresh40 as u8 as i32
            } else {
                luaZ_fill((*ls).z)
            });
        }
        lexerror(ls, msg, TK_STRING as i32);
    }
}
unsafe extern "C-unwind" fn gethexa(mut ls: *mut LexState) -> i32 {
    save(ls, (*ls).current);
    let fresh41 = (*(*ls).z).n;
    (*(*ls).z).n = ((*(*ls).z).n).wrapping_sub(1);
    (*ls).current = (if fresh41 > 0 as size_t {
        let fresh42 = (*(*ls).z).p;
        (*(*ls).z).p = ((*(*ls).z).p).offset(1);
        *fresh42 as u8 as i32
    } else {
        luaZ_fill((*ls).z)
    });
    esccheck(
        ls,
        luai_ctype_[((*ls).current + 1 as i32) as usize] as i32 & (1 as i32) << 4 as i32,
        c"hexadecimal digit expected".as_ptr(),
    );
    return luaO_hexavalue((*ls).current);
}
unsafe extern "C-unwind" fn readhexaesc(mut ls: *mut LexState) -> i32 {
    let mut r: i32 = gethexa(ls);
    r = (r << 4 as i32) + gethexa(ls);
    (*(*ls).buff).n = ((*(*ls).buff).n).wrapping_sub(2 as i32 as size_t);
    return r;
}
unsafe extern "C-unwind" fn readutf8esc(mut ls: *mut LexState) -> usize {
    let mut r: usize = 0;
    let mut i: i32 = 4 as i32;
    save(ls, (*ls).current);
    let fresh43 = (*(*ls).z).n;
    (*(*ls).z).n = ((*(*ls).z).n).wrapping_sub(1);
    (*ls).current = (if fresh43 > 0 as size_t {
        let fresh44 = (*(*ls).z).p;
        (*(*ls).z).p = ((*(*ls).z).p).offset(1);
        *fresh44 as u8 as i32
    } else {
        luaZ_fill((*ls).z)
    });
    esccheck(
        ls,
        ((*ls).current == '{' as i32) as i32,
        c"missing '{'".as_ptr(),
    );
    r = gethexa(ls) as usize;
    loop {
        save(ls, (*ls).current);
        let fresh45 = (*(*ls).z).n;
        (*(*ls).z).n = ((*(*ls).z).n).wrapping_sub(1);
        (*ls).current = (if fresh45 > 0 as size_t {
            let fresh46 = (*(*ls).z).p;
            (*(*ls).z).p = ((*(*ls).z).p).offset(1);
            *fresh46 as u8 as i32
        } else {
            luaZ_fill((*ls).z)
        });
        if !(luai_ctype_[((*ls).current + 1 as i32) as usize] as i32 & (1 as i32) << 4 as i32 != 0)
        {
            break;
        }
        i += 1;
        i;
        esccheck(
            ls,
            (r <= (0x7fffffff as u32 >> 4 as i32) as usize) as i32,
            c"UTF-8 value too large".as_ptr(),
        );
        r = (r << 4 as i32).wrapping_add(luaO_hexavalue((*ls).current) as usize);
    }
    esccheck(
        ls,
        ((*ls).current == '}' as i32) as i32,
        c"missing '}'".as_ptr(),
    );
    let fresh47 = (*(*ls).z).n;
    (*(*ls).z).n = ((*(*ls).z).n).wrapping_sub(1);
    (*ls).current = if fresh47 > 0 as size_t {
        let fresh48 = (*(*ls).z).p;
        (*(*ls).z).p = ((*(*ls).z).p).offset(1);
        *fresh48 as u8 as i32
    } else {
        luaZ_fill((*ls).z)
    };
    (*(*ls).buff).n = ((*(*ls).buff).n).wrapping_sub(i as size_t);
    return r;
}
unsafe extern "C-unwind" fn utf8esc(mut ls: *mut LexState) {
    let mut buff: [std::ffi::c_char; 8] = [0; 8];
    let mut n: i32 = luaO_utf8esc(buff.as_mut_ptr(), readutf8esc(ls));
    while n > 0 {
        save(ls, buff[(8 as i32 - n) as usize] as i32);
        n -= 1;
        n;
    }
}
unsafe extern "C-unwind" fn readdecesc(mut ls: *mut LexState) -> i32 {
    let mut i: i32 = 0;
    let mut r: i32 = 0;
    i = 0;
    while i < 3 as i32
        && luai_ctype_[((*ls).current + 1 as i32) as usize] as i32 & (1 as i32) << 1 as i32 != 0
    {
        r = 10 * r + (*ls).current - '0' as i32;
        save(ls, (*ls).current);
        let fresh49 = (*(*ls).z).n;
        (*(*ls).z).n = ((*(*ls).z).n).wrapping_sub(1);
        (*ls).current = (if fresh49 > 0 as size_t {
            let fresh50 = (*(*ls).z).p;
            (*(*ls).z).p = ((*(*ls).z).p).offset(1);
            *fresh50 as u8 as i32
        } else {
            luaZ_fill((*ls).z)
        });
        i += 1;
        i;
    }
    esccheck(
        ls,
        (r <= 127 as i32 * 2 as i32 + 1 as i32) as i32,
        c"decimal escape too large".as_ptr(),
    );
    (*(*ls).buff).n = ((*(*ls).buff).n).wrapping_sub(i as size_t);
    return r;
}
unsafe extern "C-unwind" fn read_string(
    mut ls: *mut LexState,
    mut del: i32,
    mut seminfo: *mut SemInfo,
) {
    let mut current_block: u64;
    save(ls, (*ls).current);
    let fresh51 = (*(*ls).z).n;
    (*(*ls).z).n = ((*(*ls).z).n).wrapping_sub(1);
    (*ls).current = (if fresh51 > 0 as size_t {
        let fresh52 = (*(*ls).z).p;
        (*(*ls).z).p = ((*(*ls).z).p).offset(1);
        *fresh52 as u8 as i32
    } else {
        luaZ_fill((*ls).z)
    });
    while (*ls).current != del {
        match (*ls).current {
            -1 => {
                lexerror(ls, c"unfinished string".as_ptr(), TK_EOS as i32);
            }
            10 | 13 => {
                lexerror(ls, c"unfinished string".as_ptr(), TK_STRING as i32);
            }
            92 => {
                let mut c: i32 = 0;
                save(ls, (*ls).current);
                let fresh53 = (*(*ls).z).n;
                (*(*ls).z).n = ((*(*ls).z).n).wrapping_sub(1);
                (*ls).current = (if fresh53 > 0 as size_t {
                    let fresh54 = (*(*ls).z).p;
                    (*(*ls).z).p = ((*(*ls).z).p).offset(1);
                    *fresh54 as u8 as i32
                } else {
                    luaZ_fill((*ls).z)
                });
                match (*ls).current {
                    97 => {
                        c = '\u{7}' as i32;
                        current_block = 8623330865402145409;
                    }
                    98 => {
                        c = '\u{8}' as i32;
                        current_block = 8623330865402145409;
                    }
                    102 => {
                        c = '\u{c}' as i32;
                        current_block = 8623330865402145409;
                    }
                    110 => {
                        c = '\n' as i32;
                        current_block = 8623330865402145409;
                    }
                    114 => {
                        c = '\r' as i32;
                        current_block = 8623330865402145409;
                    }
                    116 => {
                        c = '\t' as i32;
                        current_block = 8623330865402145409;
                    }
                    118 => {
                        c = '\u{b}' as i32;
                        current_block = 8623330865402145409;
                    }
                    120 => {
                        c = readhexaesc(ls);
                        current_block = 8623330865402145409;
                    }
                    117 => {
                        utf8esc(ls);
                        continue;
                    }
                    10 | 13 => {
                        inclinenumber(ls);
                        c = '\n' as i32;
                        current_block = 14094486417619109786;
                    }
                    92 | 34 | 39 => {
                        c = (*ls).current;
                        current_block = 8623330865402145409;
                    }
                    -1 => {
                        continue;
                    }
                    122 => {
                        (*(*ls).buff).n = ((*(*ls).buff).n).wrapping_sub(1 as i32 as size_t);
                        let fresh55 = (*(*ls).z).n;
                        (*(*ls).z).n = ((*(*ls).z).n).wrapping_sub(1);
                        (*ls).current = if fresh55 > 0 as size_t {
                            let fresh56 = (*(*ls).z).p;
                            (*(*ls).z).p = ((*(*ls).z).p).offset(1);
                            *fresh56 as u8 as i32
                        } else {
                            luaZ_fill((*ls).z)
                        };
                        while luai_ctype_[((*ls).current + 1 as i32) as usize] as i32
                            & (1 as i32) << 3 as i32
                            != 0
                        {
                            if (*ls).current == '\n' as i32 || (*ls).current == '\r' as i32 {
                                inclinenumber(ls);
                            } else {
                                let fresh57 = (*(*ls).z).n;
                                (*(*ls).z).n = ((*(*ls).z).n).wrapping_sub(1);
                                (*ls).current = if fresh57 > 0 as size_t {
                                    let fresh58 = (*(*ls).z).p;
                                    (*(*ls).z).p = ((*(*ls).z).p).offset(1);
                                    *fresh58 as u8 as i32
                                } else {
                                    luaZ_fill((*ls).z)
                                };
                            }
                        }
                        continue;
                    }
                    _ => {
                        esccheck(
                            ls,
                            luai_ctype_[((*ls).current + 1 as i32) as usize] as i32
                                & (1 as i32) << 1 as i32,
                            c"invalid escape sequence".as_ptr(),
                        );
                        c = readdecesc(ls);
                        current_block = 14094486417619109786;
                    }
                }
                match current_block {
                    8623330865402145409 => {
                        let fresh59 = (*(*ls).z).n;
                        (*(*ls).z).n = ((*(*ls).z).n).wrapping_sub(1);
                        (*ls).current = if fresh59 > 0 as size_t {
                            let fresh60 = (*(*ls).z).p;
                            (*(*ls).z).p = ((*(*ls).z).p).offset(1);
                            *fresh60 as u8 as i32
                        } else {
                            luaZ_fill((*ls).z)
                        };
                    }
                    _ => {}
                }
                (*(*ls).buff).n = ((*(*ls).buff).n).wrapping_sub(1 as i32 as size_t);
                save(ls, c);
            }
            _ => {
                save(ls, (*ls).current);
                let fresh61 = (*(*ls).z).n;
                (*(*ls).z).n = ((*(*ls).z).n).wrapping_sub(1);
                (*ls).current = (if fresh61 > 0 as size_t {
                    let fresh62 = (*(*ls).z).p;
                    (*(*ls).z).p = ((*(*ls).z).p).offset(1);
                    *fresh62 as u8 as i32
                } else {
                    luaZ_fill((*ls).z)
                });
            }
        }
    }
    save(ls, (*ls).current);
    let fresh63 = (*(*ls).z).n;
    (*(*ls).z).n = ((*(*ls).z).n).wrapping_sub(1);
    (*ls).current = (if fresh63 > 0 as size_t {
        let fresh64 = (*(*ls).z).p;
        (*(*ls).z).p = ((*(*ls).z).p).offset(1);
        *fresh64 as u8 as i32
    } else {
        luaZ_fill((*ls).z)
    });
    (*seminfo).ts = luaX_newstring(
        ls,
        ((*(*ls).buff).buffer).offset(1),
        ((*(*ls).buff).n).wrapping_sub(2 as i32 as size_t),
    );
}
unsafe extern "C-unwind" fn llex(mut ls: *mut LexState, mut seminfo: *mut SemInfo) -> i32 {
    (*(*ls).buff).n = 0 as size_t;
    loop {
        let mut current_block_85: u64;
        match (*ls).current {
            10 | 13 => {
                inclinenumber(ls);
            }
            32 | 12 | 9 | 11 => {
                let fresh65 = (*(*ls).z).n;
                (*(*ls).z).n = ((*(*ls).z).n).wrapping_sub(1);
                (*ls).current = if fresh65 > 0 as size_t {
                    let fresh66 = (*(*ls).z).p;
                    (*(*ls).z).p = ((*(*ls).z).p).offset(1);
                    *fresh66 as u8 as i32
                } else {
                    luaZ_fill((*ls).z)
                };
            }
            45 => {
                let fresh67 = (*(*ls).z).n;
                (*(*ls).z).n = ((*(*ls).z).n).wrapping_sub(1);
                (*ls).current = if fresh67 > 0 as size_t {
                    let fresh68 = (*(*ls).z).p;
                    (*(*ls).z).p = ((*(*ls).z).p).offset(1);
                    *fresh68 as u8 as i32
                } else {
                    luaZ_fill((*ls).z)
                };
                if (*ls).current != '-' as i32 {
                    return '-' as i32;
                }
                let fresh69 = (*(*ls).z).n;
                (*(*ls).z).n = ((*(*ls).z).n).wrapping_sub(1);
                (*ls).current = if fresh69 > 0 as size_t {
                    let fresh70 = (*(*ls).z).p;
                    (*(*ls).z).p = ((*(*ls).z).p).offset(1);
                    *fresh70 as u8 as i32
                } else {
                    luaZ_fill((*ls).z)
                };
                if (*ls).current == '[' as i32 {
                    let mut sep: size_t = skip_sep(ls);
                    (*(*ls).buff).n = 0 as size_t;
                    if sep >= 2 as i32 as size_t {
                        read_long_string(ls, 0 as *mut SemInfo, sep);
                        (*(*ls).buff).n = 0 as size_t;
                        current_block_85 = 10512632378975961025;
                    } else {
                        current_block_85 = 3512920355445576850;
                    }
                } else {
                    current_block_85 = 3512920355445576850;
                }
                match current_block_85 {
                    10512632378975961025 => {}
                    _ => {
                        while !((*ls).current == '\n' as i32 || (*ls).current == '\r' as i32)
                            && (*ls).current != -(1 as i32)
                        {
                            let fresh71 = (*(*ls).z).n;
                            (*(*ls).z).n = ((*(*ls).z).n).wrapping_sub(1);
                            (*ls).current = if fresh71 > 0 as size_t {
                                let fresh72 = (*(*ls).z).p;
                                (*(*ls).z).p = ((*(*ls).z).p).offset(1);
                                *fresh72 as u8 as i32
                            } else {
                                luaZ_fill((*ls).z)
                            };
                        }
                    }
                }
            }
            91 => {
                let mut sep_0: size_t = skip_sep(ls);
                if sep_0 >= 2 as i32 as size_t {
                    read_long_string(ls, seminfo, sep_0);
                    return TK_STRING as i32;
                } else if sep_0 == 0 as size_t {
                    lexerror(
                        ls,
                        c"invalid long string delimiter".as_ptr(),
                        TK_STRING as i32,
                    );
                }
                return '[' as i32;
            }
            61 => {
                let fresh73 = (*(*ls).z).n;
                (*(*ls).z).n = ((*(*ls).z).n).wrapping_sub(1);
                (*ls).current = if fresh73 > 0 as size_t {
                    let fresh74 = (*(*ls).z).p;
                    (*(*ls).z).p = ((*(*ls).z).p).offset(1);
                    *fresh74 as u8 as i32
                } else {
                    luaZ_fill((*ls).z)
                };
                if check_next1(ls, '=' as i32) != 0 {
                    return TK_EQ as i32;
                } else {
                    return '=' as i32;
                }
            }
            60 => {
                let fresh75 = (*(*ls).z).n;
                (*(*ls).z).n = ((*(*ls).z).n).wrapping_sub(1);
                (*ls).current = if fresh75 > 0 as size_t {
                    let fresh76 = (*(*ls).z).p;
                    (*(*ls).z).p = ((*(*ls).z).p).offset(1);
                    *fresh76 as u8 as i32
                } else {
                    luaZ_fill((*ls).z)
                };
                if check_next1(ls, '=' as i32) != 0 {
                    return TK_LE as i32;
                } else if check_next1(ls, '<' as i32) != 0 {
                    return TK_SHL as i32;
                } else {
                    return '<' as i32;
                }
            }
            62 => {
                let fresh77 = (*(*ls).z).n;
                (*(*ls).z).n = ((*(*ls).z).n).wrapping_sub(1);
                (*ls).current = if fresh77 > 0 as size_t {
                    let fresh78 = (*(*ls).z).p;
                    (*(*ls).z).p = ((*(*ls).z).p).offset(1);
                    *fresh78 as u8 as i32
                } else {
                    luaZ_fill((*ls).z)
                };
                if check_next1(ls, '=' as i32) != 0 {
                    return TK_GE as i32;
                } else if check_next1(ls, '>' as i32) != 0 {
                    return TK_SHR as i32;
                } else {
                    return '>' as i32;
                }
            }
            47 => {
                let fresh79 = (*(*ls).z).n;
                (*(*ls).z).n = ((*(*ls).z).n).wrapping_sub(1);
                (*ls).current = if fresh79 > 0 as size_t {
                    let fresh80 = (*(*ls).z).p;
                    (*(*ls).z).p = ((*(*ls).z).p).offset(1);
                    *fresh80 as u8 as i32
                } else {
                    luaZ_fill((*ls).z)
                };
                if check_next1(ls, '/' as i32) != 0 {
                    return TK_IDIV as i32;
                } else {
                    return '/' as i32;
                }
            }
            126 => {
                let fresh81 = (*(*ls).z).n;
                (*(*ls).z).n = ((*(*ls).z).n).wrapping_sub(1);
                (*ls).current = if fresh81 > 0 as size_t {
                    let fresh82 = (*(*ls).z).p;
                    (*(*ls).z).p = ((*(*ls).z).p).offset(1);
                    *fresh82 as u8 as i32
                } else {
                    luaZ_fill((*ls).z)
                };
                if check_next1(ls, '=' as i32) != 0 {
                    return TK_NE as i32;
                } else {
                    return '~' as i32;
                }
            }
            58 => {
                let fresh83 = (*(*ls).z).n;
                (*(*ls).z).n = ((*(*ls).z).n).wrapping_sub(1);
                (*ls).current = if fresh83 > 0 as size_t {
                    let fresh84 = (*(*ls).z).p;
                    (*(*ls).z).p = ((*(*ls).z).p).offset(1);
                    *fresh84 as u8 as i32
                } else {
                    luaZ_fill((*ls).z)
                };
                if check_next1(ls, ':' as i32) != 0 {
                    return TK_DBCOLON as i32;
                } else {
                    return ':' as i32;
                }
            }
            34 | 39 => {
                read_string(ls, (*ls).current, seminfo);
                return TK_STRING as i32;
            }
            46 => {
                save(ls, (*ls).current);
                let fresh85 = (*(*ls).z).n;
                (*(*ls).z).n = ((*(*ls).z).n).wrapping_sub(1);
                (*ls).current = (if fresh85 > 0 as size_t {
                    let fresh86 = (*(*ls).z).p;
                    (*(*ls).z).p = ((*(*ls).z).p).offset(1);
                    *fresh86 as u8 as i32
                } else {
                    luaZ_fill((*ls).z)
                });
                if check_next1(ls, '.' as i32) != 0 {
                    if check_next1(ls, '.' as i32) != 0 {
                        return TK_DOTS as i32;
                    } else {
                        return TK_CONCAT as i32;
                    }
                } else if luai_ctype_[((*ls).current + 1 as i32) as usize] as i32
                    & (1 as i32) << 1 as i32
                    == 0
                {
                    return '.' as i32;
                } else {
                    return read_numeral(ls, seminfo);
                }
            }
            48 | 49 | 50 | 51 | 52 | 53 | 54 | 55 | 56 | 57 => {
                return read_numeral(ls, seminfo);
            }
            -1 => return TK_EOS as i32,
            _ => {
                if luai_ctype_[((*ls).current + 1 as i32) as usize] as i32 & (1 as i32) << 0 != 0 {
                    let mut ts: *mut TString = 0 as *mut TString;
                    loop {
                        save(ls, (*ls).current);
                        let fresh87 = (*(*ls).z).n;
                        (*(*ls).z).n = ((*(*ls).z).n).wrapping_sub(1);
                        (*ls).current = (if fresh87 > 0 as size_t {
                            let fresh88 = (*(*ls).z).p;
                            (*(*ls).z).p = ((*(*ls).z).p).offset(1);
                            *fresh88 as u8 as i32
                        } else {
                            luaZ_fill((*ls).z)
                        });
                        if !(luai_ctype_[((*ls).current + 1 as i32) as usize] as i32
                            & ((1 as i32) << 0 | (1 as i32) << 1 as i32)
                            != 0)
                        {
                            break;
                        }
                    }
                    ts = luaX_newstring(ls, (*(*ls).buff).buffer, (*(*ls).buff).n);
                    (*seminfo).ts = ts;
                    if (*ts).tt as i32 == 4 as i32 | (0) << 4 as i32 && (*ts).extra as i32 > 0 {
                        return (*ts).extra as i32 - 1 as i32
                            + (127 as i32 * 2 as i32 + 1 as i32 + 1 as i32);
                    } else {
                        return TK_NAME as i32;
                    }
                } else {
                    let mut c: i32 = (*ls).current;
                    let fresh89 = (*(*ls).z).n;
                    (*(*ls).z).n = ((*(*ls).z).n).wrapping_sub(1);
                    (*ls).current = if fresh89 > 0 as size_t {
                        let fresh90 = (*(*ls).z).p;
                        (*(*ls).z).p = ((*(*ls).z).p).offset(1);
                        *fresh90 as u8 as i32
                    } else {
                        luaZ_fill((*ls).z)
                    };
                    return c;
                }
            }
        }
    }
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaX_next(mut ls: *mut LexState) {
    (*ls).lastline = (*ls).linenumber;
    if (*ls).lookahead.token != TK_EOS as i32 {
        (*ls).t = (*ls).lookahead;
        (*ls).lookahead.token = TK_EOS as i32;
    } else {
        (*ls).t.token = llex(ls, &mut (*ls).t.seminfo);
    };
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaX_lookahead(mut ls: *mut LexState) -> i32 {
    (*ls).lookahead.token = llex(ls, &mut (*ls).lookahead.seminfo);
    return (*ls).lookahead.token;
}

unsafe extern "C-unwind" fn currentpc(mut ci: *mut CallInfo) -> i32 {
    return ((*ci).u.l.savedpc)
        .offset_from((*(*&mut (*((*(*ci).func.p).val.value_.gc as *mut GCUnion)).cl.l).p).code)
        as std::ffi::c_long as i32
        - 1 as i32;
}
unsafe extern "C-unwind" fn getbaseline(
    mut f: *const Proto,
    mut pc: i32,
    mut basepc: *mut i32,
) -> i32 {
    if (*f).sizeabslineinfo == 0 || pc < (*((*f).abslineinfo).offset(0 as isize)).pc {
        *basepc = -(1 as i32);
        return (*f).linedefined;
    } else {
        let mut i: i32 = (pc as u32).wrapping_div(128).wrapping_sub(1) as i32;
        while (i + 1 as i32) < (*f).sizeabslineinfo
            && pc >= (*((*f).abslineinfo).offset((i + 1 as i32) as isize)).pc
        {
            i += 1;
            i;
        }
        *basepc = (*((*f).abslineinfo).offset(i as isize)).pc;
        return (*((*f).abslineinfo).offset(i as isize)).line;
    };
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaG_getfuncline(mut f: *const Proto, mut pc: i32) -> i32 {
    if ((*f).lineinfo).is_null() {
        return -(1 as i32);
    } else {
        let mut basepc: i32 = 0;
        let mut baseline: i32 = getbaseline(f, pc, &mut basepc);
        loop {
            let fresh111 = basepc;
            basepc = basepc + 1;
            if !(fresh111 < pc) {
                break;
            }
            baseline += *((*f).lineinfo).offset(basepc as isize) as i32;
        }
        return baseline;
    };
}
unsafe extern "C-unwind" fn getcurrentline(mut ci: *mut CallInfo) -> i32 {
    return luaG_getfuncline(
        (*&mut (*((*(*ci).func.p).val.value_.gc as *mut GCUnion)).cl.l).p,
        currentpc(ci),
    );
}
unsafe extern "C-unwind" fn settraps(mut ci: *mut CallInfo) {
    while !ci.is_null() {
        if (*ci).callstatus as i32 & (1 as i32) << 1 as i32 == 0 {
            ::core::ptr::write_volatile(&mut (*ci).u.l.trap as *mut sig_atomic_t, 1 as i32);
        }
        ci = (*ci).previous;
    }
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn lua_sethook(
    mut L: *mut lua_State,
    mut func: lua_Hook,
    mut mask: i32,
    mut count: i32,
) {
    if func.is_none() || mask == 0 {
        mask = 0;
        func = None;
    }
    ::core::ptr::write_volatile(&mut (*L).hook as *mut lua_Hook, func);
    (*L).basehookcount = count;
    (*L).hookcount = (*L).basehookcount;
    ::core::ptr::write_volatile(
        &mut (*L).hookmask as *mut sig_atomic_t,
        mask as lu_byte as sig_atomic_t,
    );
    if mask != 0 {
        settraps((*L).ci);
    }
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn lua_gethook(mut L: *mut lua_State) -> lua_Hook {
    return (*L).hook;
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn lua_gethookmask(mut L: *mut lua_State) -> i32 {
    return (*L).hookmask;
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn lua_gethookcount(mut L: *mut lua_State) -> i32 {
    return (*L).basehookcount;
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn lua_getstack(
    mut L: *mut lua_State,
    mut level: i32,
    mut ar: *mut lua_Debug,
) -> i32 {
    let mut status: i32 = 0;
    let mut ci: *mut CallInfo = 0 as *mut CallInfo;
    if level < 0 {
        return 0;
    }
    ci = (*L).ci;
    while level > 0 && ci != &mut (*L).base_ci as *mut CallInfo {
        level -= 1;
        level;
        ci = (*ci).previous;
    }
    if level == 0 && ci != &mut (*L).base_ci as *mut CallInfo {
        status = 1 as i32;
        (*ar).i_ci = ci;
    } else {
        status = 0;
    }
    return status;
}
unsafe extern "C-unwind" fn upvalname(mut p: *const Proto, mut uv: i32) -> *const std::ffi::c_char {
    let mut s: *mut TString = (*((*p).upvalues).offset(uv as isize)).name;
    if s.is_null() {
        return c"?".as_ptr();
    } else {
        return ((*s).contents).as_mut_ptr();
    };
}
unsafe extern "C-unwind" fn findvararg(
    mut ci: *mut CallInfo,
    mut n: i32,
    mut pos: *mut StkId,
) -> *const std::ffi::c_char {
    if (*(*(&mut (*((*(*ci).func.p).val.value_.gc as *mut GCUnion)).cl.l as *mut LClosure)).p)
        .is_vararg
        != 0
    {
        let mut nextra: i32 = (*ci).u.l.nextraargs;
        if n >= -nextra {
            *pos = ((*ci).func.p)
                .offset(-(nextra as isize))
                .offset(-((n + 1 as i32) as isize));
            return c"(vararg)".as_ptr();
        }
    }
    return 0 as *const std::ffi::c_char;
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaG_findlocal(
    mut L: *mut lua_State,
    mut ci: *mut CallInfo,
    mut n: i32,
    mut pos: *mut StkId,
) -> *const std::ffi::c_char {
    let mut base: StkId = ((*ci).func.p).offset(1);
    let mut name: *const std::ffi::c_char = 0 as *const std::ffi::c_char;
    if (*ci).callstatus as i32 & (1 as i32) << 1 as i32 == 0 {
        if n < 0 {
            return findvararg(ci, n, pos);
        } else {
            name = luaF_getlocalname(
                (*&mut (*((*(*ci).func.p).val.value_.gc as *mut GCUnion)).cl.l).p,
                n,
                currentpc(ci),
            );
        }
    }
    if name.is_null() {
        let mut limit: StkId = if ci == (*L).ci {
            (*L).top.p
        } else {
            (*(*ci).next).func.p
        };
        if limit.offset_from(base) as std::ffi::c_long >= n as std::ffi::c_long && n > 0 {
            name = if (*ci).callstatus as i32 & (1 as i32) << 1 as i32 == 0 {
                c"(temporary)".as_ptr()
            } else {
                c"(C temporary)".as_ptr()
            };
        } else {
            return 0 as *const std::ffi::c_char;
        }
    }
    if !pos.is_null() {
        *pos = base.offset((n - 1 as i32) as isize);
    }
    return name;
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn lua_getlocal(
    mut L: *mut lua_State,
    mut ar: *const lua_Debug,
    mut n: i32,
) -> *const std::ffi::c_char {
    let mut name: *const std::ffi::c_char = 0 as *const std::ffi::c_char;
    if ar.is_null() {
        if !((*((*L).top.p).offset(-(1))).val.tt_ as i32
            == 6 as i32 | (0) << 4 as i32 | (1 as i32) << 6 as i32)
        {
            name = 0 as *const std::ffi::c_char;
        } else {
            name = luaF_getlocalname(
                (*&mut (*((*((*L).top.p).offset(-(1))).val.value_.gc as *mut GCUnion))
                    .cl
                    .l)
                    .p,
                n,
                0,
            );
        }
    } else {
        let mut pos: StkId = 0 as StkId;
        name = luaG_findlocal(L, (*ar).i_ci, n, &mut pos);
        if !name.is_null() {
            let mut io1: *mut TValue = &mut (*(*L).top.p).val;
            let mut io2: *const TValue = &mut (*pos).val;
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
            (*L).top.p = ((*L).top.p).offset(1);
            (*L).top.p;
        }
    }
    return name;
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn lua_setlocal(
    mut L: *mut lua_State,
    mut ar: *const lua_Debug,
    mut n: i32,
) -> *const std::ffi::c_char {
    let mut pos: StkId = 0 as StkId;
    let mut name: *const std::ffi::c_char = 0 as *const std::ffi::c_char;
    name = luaG_findlocal(L, (*ar).i_ci, n, &mut pos);
    if !name.is_null() {
        let mut io1: *mut TValue = &mut (*pos).val;
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
        (*L).top.p = ((*L).top.p).offset(-1);
        (*L).top.p;
    }
    return name;
}
unsafe extern "C-unwind" fn funcinfo(mut ar: *mut lua_Debug, mut cl: *mut Closure) {
    if !(!cl.is_null() && (*cl).c.tt as i32 == 6 as i32 | (0) << 4 as i32) {
        (*ar).source = c"=[C]".as_ptr();
        (*ar).srclen = (size_of::<[std::ffi::c_char; 5]>() as usize)
            .wrapping_div(size_of::<std::ffi::c_char>() as usize)
            .wrapping_sub(1);
        (*ar).linedefined = -(1 as i32);
        (*ar).lastlinedefined = -(1 as i32);
        (*ar).what = c"C".as_ptr();
    } else {
        let mut p: *const Proto = (*cl).l.p;
        if !((*p).source).is_null() {
            (*ar).source = ((*(*p).source).contents).as_mut_ptr();
            (*ar).srclen = if (*(*p).source).shrlen as i32 != 0xff as i32 {
                (*(*p).source).shrlen as size_t
            } else {
                (*(*p).source).u.lnglen
            };
        } else {
            (*ar).source = c"=?".as_ptr();
            (*ar).srclen = (size_of::<[std::ffi::c_char; 3]>() as usize)
                .wrapping_div(size_of::<std::ffi::c_char>() as usize)
                .wrapping_sub(1);
        }
        (*ar).linedefined = (*p).linedefined;
        (*ar).lastlinedefined = (*p).lastlinedefined;
        (*ar).what = if (*ar).linedefined == 0 {
            c"main".as_ptr()
        } else {
            c"Lua".as_ptr()
        };
    }
    luaO_chunkid(((*ar).short_src).as_mut_ptr(), (*ar).source, (*ar).srclen);
}
unsafe extern "C-unwind" fn nextline(
    mut p: *const Proto,
    mut currentline: i32,
    mut pc: i32,
) -> i32 {
    if *((*p).lineinfo).offset(pc as isize) as i32 != -(0x80) {
        return currentline + *((*p).lineinfo).offset(pc as isize) as i32;
    } else {
        return luaG_getfuncline(p, pc);
    };
}
unsafe extern "C-unwind" fn collectvalidlines(mut L: *mut lua_State, mut f: *mut Closure) {
    if !(!f.is_null() && (*f).c.tt as i32 == 6 as i32 | (0) << 4 as i32) {
        (*(*L).top.p).val.tt_ = (0 | (0) << 4 as i32) as lu_byte;
        (*L).top.p = ((*L).top.p).offset(1);
        (*L).top.p;
    } else {
        let mut p: *const Proto = (*f).l.p;
        let mut currentline: i32 = (*p).linedefined;
        let mut t: *mut Table = luaH_new(L);
        let mut io: *mut TValue = &mut (*(*L).top.p).val;
        let mut x_: *mut Table = t;
        (*io).value_.gc = &mut (*(x_ as *mut GCUnion)).gc;
        (*io).tt_ = (5 as i32 | (0) << 4 as i32 | (1 as i32) << 6 as i32) as lu_byte;
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
        if !((*p).lineinfo).is_null() {
            let mut i: i32 = 0;
            let mut v: TValue = TValue {
                value_: Value {
                    gc: 0 as *mut GCObject,
                },
                tt_: 0,
            };
            v.tt_ = (1 as i32 | (1 as i32) << 4 as i32) as lu_byte;
            if (*p).is_vararg == 0 {
                i = 0;
            } else {
                currentline = nextline(p, currentline, 0);
                i = 1 as i32;
            }
            while i < (*p).sizelineinfo {
                currentline = nextline(p, currentline, i);
                luaH_setint(L, t, currentline as lua_Integer, &mut v);
                i += 1;
                i;
            }
        }
    };
}
unsafe extern "C-unwind" fn getfuncname(
    mut L: *mut lua_State,
    mut ci: *mut CallInfo,
    mut name: *mut *const std::ffi::c_char,
) -> *const std::ffi::c_char {
    if !ci.is_null() && (*ci).callstatus as i32 & (1 as i32) << 5 as i32 == 0 {
        return funcnamefromcall(L, (*ci).previous, name);
    } else {
        return 0 as *const std::ffi::c_char;
    };
}
unsafe extern "C-unwind" fn auxgetinfo(
    mut L: *mut lua_State,
    mut what: *const std::ffi::c_char,
    mut ar: *mut lua_Debug,
    mut f: *mut Closure,
    mut ci: *mut CallInfo,
) -> i32 {
    let mut status: i32 = 1 as i32;
    while *what != 0 {
        match *what as i32 {
            83 => {
                funcinfo(ar, f);
            }
            108 => {
                (*ar).currentline =
                    if !ci.is_null() && (*ci).callstatus as i32 & (1 as i32) << 1 as i32 == 0 {
                        getcurrentline(ci)
                    } else {
                        -(1 as i32)
                    };
            }
            117 => {
                (*ar).nups = (if f.is_null() {
                    0
                } else {
                    (*f).c.nupvalues as i32
                }) as u8;
                if !(!f.is_null() && (*f).c.tt as i32 == 6 as i32 | (0) << 4 as i32) {
                    (*ar).isvararg = 1 as i32 as std::ffi::c_char;
                    (*ar).nparams = 0 as u8;
                } else {
                    (*ar).isvararg = (*(*f).l.p).is_vararg as std::ffi::c_char;
                    (*ar).nparams = (*(*f).l.p).numparams;
                }
            }
            116 => {
                (*ar).istailcall = (if !ci.is_null() {
                    (*ci).callstatus as i32 & (1 as i32) << 5 as i32
                } else {
                    0
                }) as std::ffi::c_char;
            }
            110 => {
                (*ar).namewhat = getfuncname(L, ci, &mut (*ar).name);
                if ((*ar).namewhat).is_null() {
                    (*ar).namewhat = c"".as_ptr();
                    (*ar).name = 0 as *const std::ffi::c_char;
                }
            }
            114 => {
                if ci.is_null() || (*ci).callstatus as i32 & (1 as i32) << 8 as i32 == 0 {
                    (*ar).ntransfer = 0 as u16;
                    (*ar).ftransfer = (*ar).ntransfer;
                } else {
                    (*ar).ftransfer = (*ci).u2.transferinfo.ftransfer;
                    (*ar).ntransfer = (*ci).u2.transferinfo.ntransfer;
                }
            }
            76 | 102 => {}
            _ => {
                status = 0;
            }
        }
        what = what.offset(1);
        what;
    }
    return status;
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn lua_getinfo(
    mut L: *mut lua_State,
    mut what: *const std::ffi::c_char,
    mut ar: *mut lua_Debug,
) -> i32 {
    let mut status: i32 = 0;
    let mut cl: *mut Closure = 0 as *mut Closure;
    let mut ci: *mut CallInfo = 0 as *mut CallInfo;
    let mut func: *mut TValue = 0 as *mut TValue;
    if *what as i32 == '>' as i32 {
        ci = 0 as *mut CallInfo;
        func = &mut (*((*L).top.p).offset(-(1))).val;
        what = what.offset(1);
        what;
        (*L).top.p = ((*L).top.p).offset(-1);
        (*L).top.p;
    } else {
        ci = (*ar).i_ci;
        func = &mut (*(*ci).func.p).val;
    }
    cl = if (*func).tt_ as i32 == 6 as i32 | (0) << 4 as i32 | (1 as i32) << 6 as i32
        || (*func).tt_ as i32 == 6 as i32 | (2 as i32) << 4 as i32 | (1 as i32) << 6 as i32
    {
        &mut (*((*func).value_.gc as *mut GCUnion)).cl
    } else {
        0 as *mut Closure
    };
    status = auxgetinfo(L, what, ar, cl, ci);
    if !(strchr(what, 'f' as i32)).is_null() {
        let mut io1: *mut TValue = &mut (*(*L).top.p).val;
        let mut io2: *const TValue = func;
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
        (*L).top.p = ((*L).top.p).offset(1);
        (*L).top.p;
    }
    if !(strchr(what, 'L' as i32)).is_null() {
        collectvalidlines(L, cl);
    }
    return status;
}
unsafe extern "C-unwind" fn filterpc(mut pc: i32, mut jmptarget: i32) -> i32 {
    if pc < jmptarget {
        return -(1 as i32);
    } else {
        return pc;
    };
}
unsafe extern "C-unwind" fn findsetreg(mut p: *const Proto, mut lastpc: i32, mut reg: i32) -> i32 {
    let mut pc: i32 = 0;
    let mut setreg: i32 = -(1 as i32);
    let mut jmptarget: i32 = 0;
    if luaP_opmodes[(*((*p).code).offset(lastpc as isize) >> 0
        & !(!(0 as Instruction) << 7 as i32) << 0) as OpCode as usize] as i32
        & (1 as i32) << 7 as i32
        != 0
    {
        lastpc -= 1;
        lastpc;
    }
    pc = 0;
    while pc < lastpc {
        let mut i: Instruction = *((*p).code).offset(pc as isize);
        let mut op: OpCode = (i >> 0 & !(!(0 as Instruction) << 7 as i32) << 0) as OpCode;
        let mut a: i32 = (i >> 0 + 7 as i32 & !(!(0 as Instruction) << 8 as i32) << 0) as i32;
        let mut change: i32 = 0;
        match op as u32 {
            8 => {
                let mut b: i32 = (i >> 0 + 7 as i32 + 8 as i32 + 1 as i32
                    & !(!(0 as Instruction) << 8 as i32) << 0)
                    as i32;
                change = (a <= reg && reg <= a + b) as i32;
            }
            76 => {
                change = (reg >= a + 2 as i32) as i32;
            }
            68 | 69 => {
                change = (reg >= a) as i32;
            }
            56 => {
                let mut b_0: i32 = (i >> 0 + 7 as i32
                    & !(!(0 as Instruction) << 8 as i32 + 8 as i32 + 1 as i32 + 8 as i32) << 0)
                    as i32
                    - (((1 as i32) << 8 as i32 + 8 as i32 + 1 as i32 + 8 as i32) - 1 as i32
                        >> 1 as i32);
                let mut dest: i32 = pc + 1 as i32 + b_0;
                if dest <= lastpc && dest > jmptarget {
                    jmptarget = dest;
                }
                change = 0;
            }
            _ => {
                change = (luaP_opmodes[op as usize] as i32 & (1 as i32) << 3 as i32 != 0
                    && reg == a) as i32;
            }
        }
        if change != 0 {
            setreg = filterpc(pc, jmptarget);
        }
        pc += 1;
        pc;
    }
    return setreg;
}
unsafe extern "C-unwind" fn kname(
    mut p: *const Proto,
    mut index: i32,
    mut name: *mut *const std::ffi::c_char,
) -> *const std::ffi::c_char {
    let mut kvalue: *mut TValue = &mut *((*p).k).offset(index as isize) as *mut TValue;
    if (*kvalue).tt_ as i32 & 0xf as i32 == 4 as i32 {
        *name = ((*&mut (*((*kvalue).value_.gc as *mut GCUnion)).ts).contents).as_mut_ptr();
        return c"constant".as_ptr();
    } else {
        *name = c"?".as_ptr();
        return 0 as *const std::ffi::c_char;
    };
}
unsafe extern "C-unwind" fn basicgetobjname(
    mut p: *const Proto,
    mut ppc: *mut i32,
    mut reg: i32,
    mut name: *mut *const std::ffi::c_char,
) -> *const std::ffi::c_char {
    let mut pc: i32 = *ppc;
    *name = luaF_getlocalname(p, reg + 1 as i32, pc);
    if !(*name).is_null() {
        return c"local".as_ptr();
    }
    pc = findsetreg(p, pc, reg);
    *ppc = pc;
    if pc != -(1 as i32) {
        let mut i: Instruction = *((*p).code).offset(pc as isize);
        let mut op: OpCode = (i >> 0 & !(!(0 as Instruction) << 7 as i32) << 0) as OpCode;
        match op as u32 {
            0 => {
                let mut b: i32 = (i >> 0 + 7 as i32 + 8 as i32 + 1 as i32
                    & !(!(0 as Instruction) << 8 as i32) << 0)
                    as i32;
                if b < (i >> 0 + 7 as i32 & !(!(0 as Instruction) << 8 as i32) << 0) as i32 {
                    return basicgetobjname(p, ppc, b, name);
                }
            }
            9 => {
                *name = upvalname(
                    p,
                    (i >> 0 + 7 as i32 + 8 as i32 + 1 as i32
                        & !(!(0 as Instruction) << 8 as i32) << 0) as i32,
                );
                return c"upvalue".as_ptr();
            }
            3 => {
                return kname(
                    p,
                    (i >> 0 + 7 as i32 + 8 as i32
                        & !(!(0 as Instruction) << 8 as i32 + 8 as i32 + 1 as i32) << 0)
                        as i32,
                    name,
                );
            }
            4 => {
                return kname(
                    p,
                    (*((*p).code).offset((pc + 1 as i32) as isize) >> 0 + 7 as i32
                        & !(!(0 as Instruction) << 8 as i32 + 8 as i32 + 1 as i32 + 8 as i32) << 0)
                        as i32,
                    name,
                );
            }
            _ => {}
        }
    }
    return 0 as *const std::ffi::c_char;
}
unsafe extern "C-unwind" fn rname(
    mut p: *const Proto,
    mut pc: i32,
    mut c: i32,
    mut name: *mut *const std::ffi::c_char,
) {
    let mut what: *const std::ffi::c_char = basicgetobjname(p, &mut pc, c, name);
    if !(!what.is_null() && *what as i32 == 'c' as i32) {
        *name = c"?".as_ptr();
    }
}
unsafe extern "C-unwind" fn rkname(
    mut p: *const Proto,
    mut pc: i32,
    mut i: Instruction,
    mut name: *mut *const std::ffi::c_char,
) {
    let mut c: i32 = (i >> 0 + 7 as i32 + 8 as i32 + 1 as i32 + 8 as i32
        & !(!(0 as Instruction) << 8 as i32) << 0) as i32;
    if (i >> 0 + 7 as i32 + 8 as i32 & !(!(0 as Instruction) << 1 as i32) << 0) as i32 != 0 {
        kname(p, c, name);
    } else {
        rname(p, pc, c, name);
    };
}
unsafe extern "C-unwind" fn isEnv(
    mut p: *const Proto,
    mut pc: i32,
    mut i: Instruction,
    mut isup: i32,
) -> *const std::ffi::c_char {
    let mut t: i32 =
        (i >> 0 + 7 as i32 + 8 as i32 + 1 as i32 & !(!(0 as Instruction) << 8 as i32) << 0) as i32;
    let mut name: *const std::ffi::c_char = 0 as *const std::ffi::c_char;
    if isup != 0 {
        name = upvalname(p, t);
    } else {
        basicgetobjname(p, &mut pc, t, &mut name);
    }
    return if !name.is_null() && strcmp(name, c"_ENV".as_ptr()) == 0 {
        c"global".as_ptr()
    } else {
        c"field".as_ptr()
    };
}
unsafe extern "C-unwind" fn getobjname(
    mut p: *const Proto,
    mut lastpc: i32,
    mut reg: i32,
    mut name: *mut *const std::ffi::c_char,
) -> *const std::ffi::c_char {
    let mut kind: *const std::ffi::c_char = basicgetobjname(p, &mut lastpc, reg, name);
    if !kind.is_null() {
        return kind;
    } else if lastpc != -(1 as i32) {
        let mut i: Instruction = *((*p).code).offset(lastpc as isize);
        let mut op: OpCode = (i >> 0 & !(!(0 as Instruction) << 7 as i32) << 0) as OpCode;
        match op as u32 {
            11 => {
                let mut k: i32 = (i >> 0 + 7 as i32 + 8 as i32 + 1 as i32 + 8 as i32
                    & !(!(0 as Instruction) << 8 as i32) << 0)
                    as i32;
                kname(p, k, name);
                return isEnv(p, lastpc, i, 1 as i32);
            }
            12 => {
                let mut k_0: i32 = (i >> 0 + 7 as i32 + 8 as i32 + 1 as i32 + 8 as i32
                    & !(!(0 as Instruction) << 8 as i32) << 0)
                    as i32;
                rname(p, lastpc, k_0, name);
                return isEnv(p, lastpc, i, 0);
            }
            13 => {
                *name = c"integer index".as_ptr();
                return c"field".as_ptr();
            }
            14 => {
                let mut k_1: i32 = (i >> 0 + 7 as i32 + 8 as i32 + 1 as i32 + 8 as i32
                    & !(!(0 as Instruction) << 8 as i32) << 0)
                    as i32;
                kname(p, k_1, name);
                return isEnv(p, lastpc, i, 0);
            }
            20 => {
                rkname(p, lastpc, i, name);
                return c"method".as_ptr();
            }
            _ => {}
        }
    }
    return 0 as *const std::ffi::c_char;
}
unsafe extern "C-unwind" fn funcnamefromcode(
    mut L: *mut lua_State,
    mut p: *const Proto,
    mut pc: i32,
    mut name: *mut *const std::ffi::c_char,
) -> *const std::ffi::c_char {
    let mut tm: TMS = TM_INDEX;
    let mut i: Instruction = *((*p).code).offset(pc as isize);
    match (i >> 0 & !(!(0 as Instruction) << 7 as i32) << 0) as OpCode as u32 {
        68 | 69 => {
            return getobjname(
                p,
                pc,
                (i >> 0 + 7 as i32 & !(!(0 as Instruction) << 8 as i32) << 0) as i32,
                name,
            );
        }
        76 => {
            *name = c"for iterator".as_ptr();
            return c"for iterator".as_ptr();
        }
        20 | 11 | 12 | 13 | 14 => {
            tm = TM_INDEX;
        }
        15 | 16 | 17 | 18 => {
            tm = TM_NEWINDEX;
        }
        46 | 47 | 48 => {
            tm = (i >> 0 + 7 as i32 + 8 as i32 + 1 as i32 + 8 as i32
                & !(!(0 as Instruction) << 8 as i32) << 0) as i32 as TMS;
        }
        49 => {
            tm = TM_UNM;
        }
        50 => {
            tm = TM_BNOT;
        }
        52 => {
            tm = TM_LEN;
        }
        53 => {
            tm = TM_CONCAT;
        }
        57 => {
            tm = TM_EQ;
        }
        58 | 62 | 64 => {
            tm = TM_LT;
        }
        59 | 63 | 65 => {
            tm = TM_LE;
        }
        54 | 70 => {
            tm = TM_CLOSE;
        }
        _ => return 0 as *const std::ffi::c_char,
    }
    *name = ((*(*(*L).l_G).tmname[tm as usize]).contents)
        .as_mut_ptr()
        .offset(2);
    return c"metamethod".as_ptr();
}
unsafe extern "C-unwind" fn funcnamefromcall(
    mut L: *mut lua_State,
    mut ci: *mut CallInfo,
    mut name: *mut *const std::ffi::c_char,
) -> *const std::ffi::c_char {
    if (*ci).callstatus as i32 & (1 as i32) << 3 as i32 != 0 {
        *name = c"?".as_ptr();
        return c"hook".as_ptr();
    } else if (*ci).callstatus as i32 & (1 as i32) << 7 as i32 != 0 {
        *name = c"__gc".as_ptr();
        return c"metamethod".as_ptr();
    } else if (*ci).callstatus as i32 & (1 as i32) << 1 as i32 == 0 {
        return funcnamefromcode(
            L,
            (*&mut (*((*(*ci).func.p).val.value_.gc as *mut GCUnion)).cl.l).p,
            currentpc(ci),
            name,
        );
    } else {
        return 0 as *const std::ffi::c_char;
    };
}
unsafe extern "C-unwind" fn instack(mut ci: *mut CallInfo, mut o: *const TValue) -> i32 {
    let mut pos: i32 = 0;
    let mut base: StkId = ((*ci).func.p).offset(1);
    pos = 0;
    while base.offset(pos as isize) < (*ci).top.p {
        if o == &mut (*base.offset(pos as isize)).val as *mut TValue as *const TValue {
            return pos;
        }
        pos += 1;
        pos;
    }
    return -(1 as i32);
}
unsafe extern "C-unwind" fn getupvalname(
    mut ci: *mut CallInfo,
    mut o: *const TValue,
    mut name: *mut *const std::ffi::c_char,
) -> *const std::ffi::c_char {
    let mut c: *mut LClosure = &mut (*((*(*ci).func.p).val.value_.gc as *mut GCUnion)).cl.l;
    let mut i: i32 = 0;
    i = 0;
    while i < (*c).nupvalues as i32 {
        if (**((*c).upvals).as_mut_ptr().offset(i as isize)).v.p == o as *mut TValue {
            *name = upvalname((*c).p, i);
            return c"upvalue".as_ptr();
        }
        i += 1;
        i;
    }
    return 0 as *const std::ffi::c_char;
}
unsafe extern "C-unwind" fn formatvarinfo(
    mut L: *mut lua_State,
    mut kind: *const std::ffi::c_char,
    mut name: *const std::ffi::c_char,
) -> *const std::ffi::c_char {
    if kind.is_null() {
        return c"".as_ptr();
    } else {
        return luaO_pushfstring(L, c" (%s '%s')".as_ptr(), kind, name);
    };
}
unsafe extern "C-unwind" fn varinfo(
    mut L: *mut lua_State,
    mut o: *const TValue,
) -> *const std::ffi::c_char {
    let mut ci: *mut CallInfo = (*L).ci;
    let mut name: *const std::ffi::c_char = 0 as *const std::ffi::c_char;
    let mut kind: *const std::ffi::c_char = 0 as *const std::ffi::c_char;
    if (*ci).callstatus as i32 & (1 as i32) << 1 as i32 == 0 {
        kind = getupvalname(ci, o, &mut name);
        if kind.is_null() {
            let mut reg: i32 = instack(ci, o);
            if reg >= 0 {
                kind = getobjname(
                    (*&mut (*((*(*ci).func.p).val.value_.gc as *mut GCUnion)).cl.l).p,
                    currentpc(ci),
                    reg,
                    &mut name,
                );
            }
        }
    }
    return formatvarinfo(L, kind, name);
}
unsafe extern "C-unwind" fn typeerror(
    mut L: *mut lua_State,
    mut o: *const TValue,
    mut op: *const std::ffi::c_char,
    mut extra: *const std::ffi::c_char,
) -> ! {
    let mut t: *const std::ffi::c_char = luaT_objtypename(L, o);
    luaG_runerror(L, c"attempt to %s a %s value%s".as_ptr(), op, t, extra);
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaG_typeerror(
    mut L: *mut lua_State,
    mut o: *const TValue,
    mut op: *const std::ffi::c_char,
) -> ! {
    typeerror(L, o, op, varinfo(L, o));
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaG_callerror(mut L: *mut lua_State, mut o: *const TValue) -> ! {
    let mut ci: *mut CallInfo = (*L).ci;
    let mut name: *const std::ffi::c_char = 0 as *const std::ffi::c_char;
    let mut kind: *const std::ffi::c_char = funcnamefromcall(L, ci, &mut name);
    let mut extra: *const std::ffi::c_char = if !kind.is_null() {
        formatvarinfo(L, kind, name)
    } else {
        varinfo(L, o)
    };
    typeerror(L, o, c"call".as_ptr(), extra);
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaG_forerror(
    mut L: *mut lua_State,
    mut o: *const TValue,
    mut what: *const std::ffi::c_char,
) -> ! {
    luaG_runerror(
        L,
        c"bad 'for' %s (number expected, got %s)".as_ptr(),
        what,
        luaT_objtypename(L, o),
    );
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaG_concaterror(
    mut L: *mut lua_State,
    mut p1: *const TValue,
    mut p2: *const TValue,
) -> ! {
    if (*p1).tt_ as i32 & 0xf as i32 == 4 as i32 || (*p1).tt_ as i32 & 0xf as i32 == 3 as i32 {
        p1 = p2;
    }
    luaG_typeerror(L, p1, c"concatenate".as_ptr());
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaG_opinterror(
    mut L: *mut lua_State,
    mut p1: *const TValue,
    mut p2: *const TValue,
    mut msg: *const std::ffi::c_char,
) -> ! {
    if !((*p1).tt_ as i32 & 0xf as i32 == 3 as i32) {
        p2 = p1;
    }
    luaG_typeerror(L, p2, msg);
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaG_tointerror(
    mut L: *mut lua_State,
    mut p1: *const TValue,
    mut p2: *const TValue,
) -> ! {
    let mut temp: lua_Integer = 0;
    if luaV_tointegerns::<F2Ieq>(p1, &mut temp) == 0 {
        p2 = p1;
    }
    luaG_runerror(
        L,
        c"number%s has no integer representation".as_ptr(),
        varinfo(L, p2),
    );
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaG_ordererror(
    mut L: *mut lua_State,
    mut p1: *const TValue,
    mut p2: *const TValue,
) -> ! {
    let mut t1: *const std::ffi::c_char = luaT_objtypename(L, p1);
    let mut t2: *const std::ffi::c_char = luaT_objtypename(L, p2);
    if strcmp(t1, t2) == 0 {
        luaG_runerror(L, c"attempt to compare two %s values".as_ptr(), t1);
    } else {
        luaG_runerror(L, c"attempt to compare %s with %s".as_ptr(), t1, t2);
    };
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaG_addinfo(
    mut L: *mut lua_State,
    mut msg: *const std::ffi::c_char,
    mut src: *mut TString,
    mut line: i32,
) -> *const std::ffi::c_char {
    let mut buff: [std::ffi::c_char; 60] = [0; 60];
    if !src.is_null() {
        luaO_chunkid(
            buff.as_mut_ptr(),
            ((*src).contents).as_mut_ptr(),
            if (*src).shrlen as i32 != 0xff as i32 {
                (*src).shrlen as size_t
            } else {
                (*src).u.lnglen
            },
        );
    } else {
        buff[0 as usize] = '?' as i32 as std::ffi::c_char;
        buff[1] = '\0' as i32 as std::ffi::c_char;
    }
    return luaO_pushfstring(L, c"%s:%d: %s".as_ptr(), buff.as_mut_ptr(), line, msg);
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaG_errormsg(mut L: *mut lua_State) -> ! {
    if (*L).errfunc != 0 as ptrdiff_t {
        let mut errfunc: StkId =
            ((*L).stack.p as *mut std::ffi::c_char).offset((*L).errfunc as isize) as StkId;
        let mut io1: *mut TValue = &mut (*(*L).top.p).val;
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
        let mut io1_0: *mut TValue = &mut (*((*L).top.p).offset(-(1))).val;
        let mut io2_0: *const TValue = &mut (*errfunc).val;
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
        (*L).top.p = ((*L).top.p).offset(1);
        (*L).top.p;
        luaD_callnoyield(L, ((*L).top.p).offset(-(2)), 1 as i32);
    }
    luaD_throw(L, 2 as i32);
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaG_runerror(
    mut L: *mut lua_State,
    mut fmt: *const std::ffi::c_char,
    mut args: ...
) -> ! {
    let mut ci: *mut CallInfo = (*L).ci;
    let mut msg: *const std::ffi::c_char = 0 as *const std::ffi::c_char;
    let mut argp: ::core::ffi::VaListImpl;
    if (*(*L).l_G).GCdebt > 0 as l_mem {
        luaC_step(L);
    }
    argp = args.clone();
    msg = luaO_pushvfstring(L, fmt, argp.as_va_list());
    if (*ci).callstatus as i32 & (1 as i32) << 1 as i32 == 0 {
        luaG_addinfo(
            L,
            msg,
            (*(*&mut (*((*(*ci).func.p).val.value_.gc as *mut GCUnion)).cl.l).p).source,
            getcurrentline(ci),
        );
        let mut io1: *mut TValue = &mut (*((*L).top.p).offset(-(2))).val;
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
        (*L).top.p = ((*L).top.p).offset(-1);
        (*L).top.p;
    }
    luaG_errormsg(L);
}
unsafe extern "C-unwind" fn changedline(
    mut p: *const Proto,
    mut oldpc: i32,
    mut newpc: i32,
) -> i32 {
    if ((*p).lineinfo).is_null() {
        return 0;
    }
    if newpc - oldpc < 128 as i32 / 2 as i32 {
        let mut delta: i32 = 0;
        let mut pc: i32 = oldpc;
        loop {
            pc += 1;
            let mut lineinfo: i32 = *((*p).lineinfo).offset(pc as isize) as i32;
            if lineinfo == -(0x80) {
                break;
            }
            delta += lineinfo;
            if pc == newpc {
                return (delta != 0) as i32;
            }
        }
    }
    return (luaG_getfuncline(p, oldpc) != luaG_getfuncline(p, newpc)) as i32;
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaG_tracecall(mut L: *mut lua_State) -> i32 {
    let mut ci: *mut CallInfo = (*L).ci;
    let mut p: *mut Proto = (*&mut (*((*(*ci).func.p).val.value_.gc as *mut GCUnion)).cl.l).p;
    ::core::ptr::write_volatile(&mut (*ci).u.l.trap as *mut sig_atomic_t, 1 as i32);
    if (*ci).u.l.savedpc == (*p).code as *const Instruction {
        if (*p).is_vararg != 0 {
            return 0;
        } else if (*ci).callstatus as i32 & (1 as i32) << 6 as i32 == 0 {
            luaD_hookcall(L, ci);
        }
    }
    return 1 as i32;
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaG_traceexec(
    mut L: *mut lua_State,
    mut pc: *const Instruction,
) -> i32 {
    let mut ci: *mut CallInfo = (*L).ci;
    let mut mask: lu_byte = (*L).hookmask as lu_byte;
    let mut p: *const Proto = (*&mut (*((*(*ci).func.p).val.value_.gc as *mut GCUnion)).cl.l).p;
    let mut counthook: i32 = 0;
    if mask as i32 & ((1 as i32) << 2 as i32 | (1 as i32) << 3 as i32) == 0 {
        ::core::ptr::write_volatile(&mut (*ci).u.l.trap as *mut sig_atomic_t, 0);
        return 0;
    }
    pc = pc.offset(1);
    pc;
    (*ci).u.l.savedpc = pc;
    counthook = (mask as i32 & (1 as i32) << 3 as i32 != 0 && {
        (*L).hookcount -= 1;
        (*L).hookcount == 0
    }) as i32;
    if counthook != 0 {
        (*L).hookcount = (*L).basehookcount;
    } else if mask as i32 & (1 as i32) << 2 as i32 == 0 {
        return 1 as i32;
    }
    if (*ci).callstatus as i32 & (1 as i32) << 6 as i32 != 0 {
        (*ci).callstatus = ((*ci).callstatus as i32 & !((1 as i32) << 6 as i32)) as u16;
        return 1 as i32;
    }
    if !(luaP_opmodes[(*((*ci).u.l.savedpc).offset(-(1)) >> 0
        & !(!(0 as Instruction) << 7 as i32) << 0) as OpCode as usize] as i32
        & (1 as i32) << 5 as i32
        != 0
        && (*((*ci).u.l.savedpc).offset(-(1)) >> 0 + 7 as i32 + 8 as i32 + 1 as i32
            & !(!(0 as Instruction) << 8 as i32) << 0) as i32
            == 0)
    {
        (*L).top.p = (*ci).top.p;
    }
    if counthook != 0 {
        luaD_hook(L, 3 as i32, -(1 as i32), 0, 0);
    }
    if mask as i32 & (1 as i32) << 2 as i32 != 0 {
        let mut oldpc: i32 = if (*L).oldpc < (*p).sizecode {
            (*L).oldpc
        } else {
            0
        };
        let mut npci: i32 = pc.offset_from((*p).code) as std::ffi::c_long as i32 - 1 as i32;
        if npci <= oldpc || changedline(p, oldpc, npci) != 0 {
            let mut newline: i32 = luaG_getfuncline(p, npci);
            luaD_hook(L, 2 as i32, newline, 0, 0);
        }
        (*L).oldpc = npci;
    }
    if (*L).status as i32 == 1 as i32 {
        if counthook != 0 {
            (*L).hookcount = 1 as i32;
        }
        (*ci).callstatus = ((*ci).callstatus as i32 | (1 as i32) << 6 as i32) as u16;
        luaD_throw(L, 1 as i32);
    }
    return 1 as i32;
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaF_newCclosure(
    mut L: *mut lua_State,
    mut nupvals: i32,
) -> *mut CClosure {
    let mut o: *mut GCObject = luaC_newobj(
        L,
        6 as i32 | (2 as i32) << 4 as i32,
        (32 as usize as i32 + size_of::<TValue>() as usize as i32 * nupvals) as size_t,
    );
    let mut c: *mut CClosure = &mut (*(o as *mut GCUnion)).cl.c;
    (*c).nupvalues = nupvals as lu_byte;
    return c;
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaF_newLclosure(
    mut L: *mut lua_State,
    mut nupvals: i32,
) -> *mut LClosure {
    let mut o: *mut GCObject = luaC_newobj(
        L,
        6 as i32 | (0) << 4 as i32,
        (32 as usize as i32 + size_of::<*mut TValue>() as usize as i32 * nupvals) as size_t,
    );
    let mut c: *mut LClosure = &mut (*(o as *mut GCUnion)).cl.l;
    (*c).p = 0 as *mut Proto;
    (*c).nupvalues = nupvals as lu_byte;
    loop {
        let fresh112 = nupvals;
        nupvals = nupvals - 1;
        if !(fresh112 != 0) {
            break;
        }
        let ref mut fresh113 = *((*c).upvals).as_mut_ptr().offset(nupvals as isize);
        *fresh113 = 0 as *mut UpVal;
    }
    return c;
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaF_initupvals(mut L: *mut lua_State, mut cl: *mut LClosure) {
    let mut i: i32 = 0;
    i = 0;
    while i < (*cl).nupvalues as i32 {
        let mut o: *mut GCObject =
            luaC_newobj(L, 9 as i32 | (0) << 4 as i32, size_of::<UpVal>() as usize);
        let mut uv: *mut UpVal = &mut (*(o as *mut GCUnion)).upv;
        (*uv).v.p = &mut (*uv).u.value;
        (*(*uv).v.p).tt_ = (0 | (0) << 4 as i32) as lu_byte;
        let ref mut fresh114 = *((*cl).upvals).as_mut_ptr().offset(i as isize);
        *fresh114 = uv;
        if (*cl).marked as i32 & (1 as i32) << 5 as i32 != 0
            && (*uv).marked as i32 & ((1 as i32) << 3 as i32 | (1 as i32) << 4 as i32) != 0
        {
            luaC_barrier_(
                L,
                &mut (*(cl as *mut GCUnion)).gc,
                &mut (*(uv as *mut GCUnion)).gc,
            );
        } else {
        };
        i += 1;
        i;
    }
}
unsafe extern "C-unwind" fn newupval(
    mut L: *mut lua_State,
    mut level: StkId,
    mut prev: *mut *mut UpVal,
) -> *mut UpVal {
    let mut o: *mut GCObject =
        luaC_newobj(L, 9 as i32 | (0) << 4 as i32, size_of::<UpVal>() as usize);
    let mut uv: *mut UpVal = &mut (*(o as *mut GCUnion)).upv;
    let mut next: *mut UpVal = *prev;
    (*uv).v.p = &mut (*level).val;
    (*uv).u.open.next = next;
    (*uv).u.open.previous = prev;
    if !next.is_null() {
        (*next).u.open.previous = &mut (*uv).u.open.next;
    }
    *prev = uv;
    if !((*L).twups != L) {
        (*L).twups = (*(*L).l_G).twups;
        (*(*L).l_G).twups = L;
    }
    return uv;
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaF_findupval(
    mut L: *mut lua_State,
    mut level: StkId,
) -> *mut UpVal {
    let mut pp: *mut *mut UpVal = &mut (*L).openupval;
    let mut p: *mut UpVal = 0 as *mut UpVal;
    loop {
        p = *pp;
        if !(!p.is_null() && (*p).v.p as StkId >= level) {
            break;
        }
        if (*p).v.p as StkId == level {
            return p;
        }
        pp = &mut (*p).u.open.next;
    }
    return newupval(L, level, pp);
}
unsafe extern "C-unwind" fn callclosemethod(
    mut L: *mut lua_State,
    mut obj: *mut TValue,
    mut err: *mut TValue,
    mut yy: i32,
) {
    let mut top: StkId = (*L).top.p;
    let mut tm: *const TValue = luaT_gettmbyobj(L, obj, TM_CLOSE);
    let mut io1: *mut TValue = &mut (*top).val;
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
    let mut io1_0: *mut TValue = &mut (*top.offset(1)).val;
    let mut io2_0: *const TValue = obj;
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
    let mut io1_1: *mut TValue = &mut (*top.offset(2)).val;
    let mut io2_1: *const TValue = err;
    (*io1_1).value_ = (*io2_1).value_;
    (*io1_1).tt_ = (*io2_1).tt_;
    if (*io1_1).tt_ as i32 & (1 as i32) << 6 as i32 == 0
        || (*io1_1).tt_ as i32 & 0x3f as i32 == (*(*io1_1).value_.gc).tt as i32
            && (L.is_null()
                || (*(*io1_1).value_.gc).marked as i32
                    & ((*(*L).l_G).currentwhite as i32
                        ^ ((1 as i32) << 3 as i32 | (1 as i32) << 4 as i32))
                    == 0)
    {
    } else {
    };
    (*L).top.p = top.offset(3);
    if yy != 0 {
        luaD_call(L, top, 0);
    } else {
        luaD_callnoyield(L, top, 0);
    };
}
unsafe extern "C-unwind" fn checkclosemth(mut L: *mut lua_State, mut level: StkId) {
    let mut tm: *const TValue = luaT_gettmbyobj(L, &mut (*level).val, TM_CLOSE);
    if (*tm).tt_ as i32 & 0xf as i32 == 0 {
        let mut idx: i32 = level.offset_from((*(*L).ci).func.p) as std::ffi::c_long as i32;
        let mut vname: *const std::ffi::c_char = luaG_findlocal(L, (*L).ci, idx, 0 as *mut StkId);
        if vname.is_null() {
            vname = c"?".as_ptr();
        }
        luaG_runerror(L, c"variable '%s' got a non-closable value".as_ptr(), vname);
    }
}
unsafe extern "C-unwind" fn prepcallclosemth(
    mut L: *mut lua_State,
    mut level: StkId,
    mut status: i32,
    mut yy: i32,
) {
    let mut uv: *mut TValue = &mut (*level).val;
    let mut errobj: *mut TValue = 0 as *mut TValue;
    if status == -(1 as i32) {
        errobj = &mut (*(*L).l_G).nilvalue;
    } else {
        errobj = &mut (*level.offset(1)).val;
        luaD_seterrorobj(L, status, level.offset(1));
    }
    callclosemethod(L, uv, errobj, yy);
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaF_newtbcupval(mut L: *mut lua_State, mut level: StkId) {
    if (*level).val.tt_ as i32 == 1 as i32 | (0) << 4 as i32
        || (*level).val.tt_ as i32 & 0xf as i32 == 0
    {
        return;
    }
    checkclosemth(L, level);
    while level.offset_from((*L).tbclist.p) as std::ffi::c_long as u32 as usize
        > ((256 as usize) << (size_of::<u16>() as usize).wrapping_sub(1).wrapping_mul(8))
            .wrapping_sub(1)
    {
        (*L).tbclist.p = ((*L).tbclist.p).offset(
            ((256 as usize) << (size_of::<u16>() as usize).wrapping_sub(1).wrapping_mul(8))
                .wrapping_sub(1) as isize,
        );
        (*(*L).tbclist.p).tbclist.delta = 0 as u16;
    }
    (*level).tbclist.delta = level.offset_from((*L).tbclist.p) as std::ffi::c_long as u16;
    (*L).tbclist.p = level;
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaF_unlinkupval(mut uv: *mut UpVal) {
    *(*uv).u.open.previous = (*uv).u.open.next;
    if !((*uv).u.open.next).is_null() {
        (*(*uv).u.open.next).u.open.previous = (*uv).u.open.previous;
    }
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaF_closeupval(mut L: *mut lua_State, mut level: StkId) {
    let mut uv: *mut UpVal = 0 as *mut UpVal;
    let mut upl: StkId = 0 as *mut StackValue;
    loop {
        uv = (*L).openupval;
        if !(!uv.is_null() && {
            upl = (*uv).v.p as StkId;
            upl >= level
        }) {
            break;
        }
        let mut slot: *mut TValue = &mut (*uv).u.value;
        luaF_unlinkupval(uv);
        let mut io1: *mut TValue = slot;
        let mut io2: *const TValue = (*uv).v.p;
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
        (*uv).v.p = slot;
        if (*uv).marked as i32 & ((1 as i32) << 3 as i32 | (1 as i32) << 4 as i32) == 0 {
            (*uv).marked = ((*uv).marked as i32 | (1 as i32) << 5 as i32) as lu_byte;
            if (*slot).tt_ as i32 & (1 as i32) << 6 as i32 != 0 {
                if (*uv).marked as i32 & (1 as i32) << 5 as i32 != 0
                    && (*(*slot).value_.gc).marked as i32
                        & ((1 as i32) << 3 as i32 | (1 as i32) << 4 as i32)
                        != 0
                {
                    luaC_barrier_(
                        L,
                        &mut (*(uv as *mut GCUnion)).gc,
                        &mut (*((*slot).value_.gc as *mut GCUnion)).gc,
                    );
                } else {
                };
            } else {
            };
        }
    }
}
unsafe extern "C-unwind" fn poptbclist(mut L: *mut lua_State) {
    let mut tbc: StkId = (*L).tbclist.p;
    tbc = tbc.offset(-((*tbc).tbclist.delta as i32 as isize));
    while tbc > (*L).stack.p && (*tbc).tbclist.delta as i32 == 0 {
        tbc = tbc.offset(
            -(((256 as usize) << (size_of::<u16>() as usize).wrapping_sub(1).wrapping_mul(8))
                .wrapping_sub(1) as isize),
        );
    }
    (*L).tbclist.p = tbc;
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaF_close(
    mut L: *mut lua_State,
    mut level: StkId,
    mut status: i32,
    mut yy: i32,
) -> StkId {
    let mut levelrel: ptrdiff_t =
        (level as *mut std::ffi::c_char).offset_from((*L).stack.p as *mut std::ffi::c_char);
    luaF_closeupval(L, level);
    while (*L).tbclist.p >= level {
        let mut tbc: StkId = (*L).tbclist.p;
        poptbclist(L);
        prepcallclosemth(L, tbc, status, yy);
        level = ((*L).stack.p as *mut std::ffi::c_char).offset(levelrel as isize) as StkId;
    }
    return level;
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaF_newproto(mut L: *mut lua_State) -> *mut Proto {
    let mut o: *mut GCObject = luaC_newobj(
        L,
        9 as i32 + 1 as i32 | (0) << 4 as i32,
        size_of::<Proto>() as usize,
    );
    let mut f: *mut Proto = &mut (*(o as *mut GCUnion)).p;
    (*f).k = 0 as *mut TValue;
    (*f).sizek = 0;
    (*f).p = 0 as *mut *mut Proto;
    (*f).sizep = 0;
    (*f).code = 0 as *mut Instruction;
    (*f).sizecode = 0;
    (*f).lineinfo = 0 as *mut ls_byte;
    (*f).sizelineinfo = 0;
    (*f).abslineinfo = 0 as *mut AbsLineInfo;
    (*f).sizeabslineinfo = 0;
    (*f).upvalues = 0 as *mut Upvaldesc;
    (*f).sizeupvalues = 0;
    (*f).numparams = 0 as lu_byte;
    (*f).is_vararg = 0 as lu_byte;
    (*f).maxstacksize = 0 as lu_byte;
    (*f).locvars = 0 as *mut LocVar;
    (*f).sizelocvars = 0;
    (*f).linedefined = 0;
    (*f).lastlinedefined = 0;
    (*f).source = 0 as *mut TString;
    return f;
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaF_freeproto(mut L: *mut lua_State, mut f: *mut Proto) {
    luaM_free_(
        L,
        (*f).code as *mut c_void,
        ((*f).sizecode as usize).wrapping_mul(size_of::<Instruction>() as usize),
    );
    luaM_free_(
        L,
        (*f).p as *mut c_void,
        ((*f).sizep as usize).wrapping_mul(size_of::<*mut Proto>() as usize),
    );
    luaM_free_(
        L,
        (*f).k as *mut c_void,
        ((*f).sizek as usize).wrapping_mul(size_of::<TValue>() as usize),
    );
    luaM_free_(
        L,
        (*f).lineinfo as *mut c_void,
        ((*f).sizelineinfo as usize).wrapping_mul(size_of::<ls_byte>() as usize),
    );
    luaM_free_(
        L,
        (*f).abslineinfo as *mut c_void,
        ((*f).sizeabslineinfo as usize).wrapping_mul(size_of::<AbsLineInfo>() as usize),
    );
    luaM_free_(
        L,
        (*f).locvars as *mut c_void,
        ((*f).sizelocvars as usize).wrapping_mul(size_of::<LocVar>() as usize),
    );
    luaM_free_(
        L,
        (*f).upvalues as *mut c_void,
        ((*f).sizeupvalues as usize).wrapping_mul(size_of::<Upvaldesc>() as usize),
    );
    if let Some(ptr) = NonNull::new(ptr::slice_from_raw_parts_mut(
        (*f).loop_cnts,
        (*f).size_loop_cnts as usize,
    )) {
        // for lc in ptr.as_ref() {
        //     eprintln!("Loop Count {}: {}", lc.pc, lc.count);
        // }

        (&*(*L).l_G).dealloc(ptr);
    }
    luaM_free_(L, f as *mut c_void, size_of::<Proto>() as usize);
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaF_getlocalname(
    mut f: *const Proto,
    mut local_number: i32,
    mut pc: i32,
) -> *const std::ffi::c_char {
    let mut i: i32 = 0;
    i = 0;
    while i < (*f).sizelocvars && (*((*f).locvars).offset(i as isize)).startpc <= pc {
        if pc < (*((*f).locvars).offset(i as isize)).endpc {
            local_number -= 1;
            local_number;
            if local_number == 0 {
                return ((*(*((*f).locvars).offset(i as isize)).varname).contents).as_mut_ptr();
            }
        }
        i += 1;
        i;
    }
    return 0 as *const std::ffi::c_char;
}

static mut udatatypename: [std::ffi::c_char; 9] =
    unsafe { *::core::mem::transmute::<&[u8; 9], &[std::ffi::c_char; 9]>(b"userdata\0") };
#[unsafe(no_mangle)]
pub static mut luaT_typenames_: [*const std::ffi::c_char; 12] = unsafe {
    [
        c"no value".as_ptr(),
        c"nil".as_ptr(),
        c"boolean".as_ptr(),
        c"userdata".as_ptr(),
        c"number".as_ptr(),
        c"string".as_ptr(),
        c"table".as_ptr(),
        c"function".as_ptr(),
        c"userdata".as_ptr(),
        c"thread".as_ptr(),
        c"upvalue".as_ptr(),
        c"proto".as_ptr(),
    ]
};
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaT_init(mut L: *mut lua_State) {
    static mut luaT_eventname: [*const std::ffi::c_char; 25] = [
        c"__index".as_ptr(),
        c"__newindex".as_ptr(),
        c"__gc".as_ptr(),
        c"__mode".as_ptr(),
        c"__len".as_ptr(),
        c"__eq".as_ptr(),
        c"__add".as_ptr(),
        c"__sub".as_ptr(),
        c"__mul".as_ptr(),
        c"__mod".as_ptr(),
        c"__pow".as_ptr(),
        c"__div".as_ptr(),
        c"__idiv".as_ptr(),
        c"__band".as_ptr(),
        c"__bor".as_ptr(),
        c"__bxor".as_ptr(),
        c"__shl".as_ptr(),
        c"__shr".as_ptr(),
        c"__unm".as_ptr(),
        c"__bnot".as_ptr(),
        c"__lt".as_ptr(),
        c"__le".as_ptr(),
        c"__concat".as_ptr(),
        c"__call".as_ptr(),
        c"__close".as_ptr(),
    ];
    let mut i: i32 = 0;
    i = 0;
    while i < TM_N as i32 {
        (*(*L).l_G).tmname[i as usize] = luaS_new(L, luaT_eventname[i as usize]);
        luaC_fix(
            L,
            &mut (*(*((*(*L).l_G).tmname).as_mut_ptr().offset(i as isize) as *mut GCUnion)).gc,
        );
        i += 1;
        i;
    }
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaT_gettm(
    mut events: *mut Table,
    mut event: TMS,
    mut ename: *mut TString,
) -> *const TValue {
    let mut tm: *const TValue = luaH_getshortstr(events, ename);
    if (*tm).tt_ as i32 & 0xf as i32 == 0 {
        (*events).flags =
            ((*events).flags as i32 | ((1 as u32) << event as u32) as lu_byte as i32) as lu_byte;
        return 0 as *const TValue;
    } else {
        return tm;
    };
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaT_gettmbyobj(
    mut L: *mut lua_State,
    mut o: *const TValue,
    mut event: TMS,
) -> *const TValue {
    let mut mt: *mut Table = 0 as *mut Table;
    match (*o).tt_ as i32 & 0xf as i32 {
        5 => {
            mt = (*&mut (*((*o).value_.gc as *mut GCUnion)).h).metatable;
        }
        7 => {
            mt = (*&mut (*((*o).value_.gc as *mut GCUnion)).u).metatable;
        }
        _ => {
            mt = (*(*L).l_G).mt[((*o).tt_ as i32 & 0xf as i32) as usize];
        }
    }
    return if !mt.is_null() {
        luaH_getshortstr(mt, (*(*L).l_G).tmname[event as usize])
    } else {
        &mut (*(*L).l_G).nilvalue as *mut TValue as *const TValue
    };
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaT_objtypename(
    mut L: *mut lua_State,
    mut o: *const TValue,
) -> *const std::ffi::c_char {
    let mut mt: *mut Table = 0 as *mut Table;
    if (*o).tt_ as i32 == 5 as i32 | (0) << 4 as i32 | (1 as i32) << 6 as i32 && {
        mt = (*(&mut (*((*o).value_.gc as *mut GCUnion)).h as *mut Table)).metatable;
        !mt.is_null()
    } || (*o).tt_ as i32 == 7 as i32 | (0) << 4 as i32 | (1 as i32) << 6 as i32 && {
        mt = (*(&mut (*((*o).value_.gc as *mut GCUnion)).u as *mut Udata)).metatable;
        !mt.is_null()
    } {
        let mut name: *const TValue = luaH_getshortstr(mt, luaS_new(L, c"__name".as_ptr()));
        if (*name).tt_ as i32 & 0xf as i32 == 4 as i32 {
            return ((*&mut (*((*name).value_.gc as *mut GCUnion)).ts).contents).as_mut_ptr();
        }
    }
    return luaT_typenames_[(((*o).tt_ as i32 & 0xf as i32) + 1 as i32) as usize];
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaT_callTM(
    mut L: *mut lua_State,
    mut f: *const TValue,
    mut p1: *const TValue,
    mut p2: *const TValue,
    mut p3: *const TValue,
) {
    let mut func: StkId = (*L).top.p;
    let mut io1: *mut TValue = &mut (*func).val;
    let mut io2: *const TValue = f;
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
    let mut io1_0: *mut TValue = &mut (*func.offset(1)).val;
    let mut io2_0: *const TValue = p1;
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
    let mut io1_1: *mut TValue = &mut (*func.offset(2)).val;
    let mut io2_1: *const TValue = p2;
    (*io1_1).value_ = (*io2_1).value_;
    (*io1_1).tt_ = (*io2_1).tt_;
    if (*io1_1).tt_ as i32 & (1 as i32) << 6 as i32 == 0
        || (*io1_1).tt_ as i32 & 0x3f as i32 == (*(*io1_1).value_.gc).tt as i32
            && (L.is_null()
                || (*(*io1_1).value_.gc).marked as i32
                    & ((*(*L).l_G).currentwhite as i32
                        ^ ((1 as i32) << 3 as i32 | (1 as i32) << 4 as i32))
                    == 0)
    {
    } else {
    };
    let mut io1_2: *mut TValue = &mut (*func.offset(3)).val;
    let mut io2_2: *const TValue = p3;
    (*io1_2).value_ = (*io2_2).value_;
    (*io1_2).tt_ = (*io2_2).tt_;
    if (*io1_2).tt_ as i32 & (1 as i32) << 6 as i32 == 0
        || (*io1_2).tt_ as i32 & 0x3f as i32 == (*(*io1_2).value_.gc).tt as i32
            && (L.is_null()
                || (*(*io1_2).value_.gc).marked as i32
                    & ((*(*L).l_G).currentwhite as i32
                        ^ ((1 as i32) << 3 as i32 | (1 as i32) << 4 as i32))
                    == 0)
    {
    } else {
    };
    (*L).top.p = func.offset(4);
    if (*(*L).ci).callstatus as i32 & ((1 as i32) << 1 as i32 | (1 as i32) << 3 as i32) == 0 {
        luaD_call(L, func, 0);
    } else {
        luaD_callnoyield(L, func, 0);
    };
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaT_callTMres(
    mut L: *mut lua_State,
    mut f: *const TValue,
    mut p1: *const TValue,
    mut p2: *const TValue,
    mut res: StkId,
) {
    let mut result: ptrdiff_t =
        (res as *mut std::ffi::c_char).offset_from((*L).stack.p as *mut std::ffi::c_char);
    let mut func: StkId = (*L).top.p;
    let mut io1: *mut TValue = &mut (*func).val;
    let mut io2: *const TValue = f;
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
    let mut io1_0: *mut TValue = &mut (*func.offset(1)).val;
    let mut io2_0: *const TValue = p1;
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
    let mut io1_1: *mut TValue = &mut (*func.offset(2)).val;
    let mut io2_1: *const TValue = p2;
    (*io1_1).value_ = (*io2_1).value_;
    (*io1_1).tt_ = (*io2_1).tt_;
    if (*io1_1).tt_ as i32 & (1 as i32) << 6 as i32 == 0
        || (*io1_1).tt_ as i32 & 0x3f as i32 == (*(*io1_1).value_.gc).tt as i32
            && (L.is_null()
                || (*(*io1_1).value_.gc).marked as i32
                    & ((*(*L).l_G).currentwhite as i32
                        ^ ((1 as i32) << 3 as i32 | (1 as i32) << 4 as i32))
                    == 0)
    {
    } else {
    };
    (*L).top.p = ((*L).top.p).offset(3);
    if (*(*L).ci).callstatus as i32 & ((1 as i32) << 1 as i32 | (1 as i32) << 3 as i32) == 0 {
        luaD_call(L, func, 1 as i32);
    } else {
        luaD_callnoyield(L, func, 1 as i32);
    }
    res = ((*L).stack.p as *mut std::ffi::c_char).offset(result as isize) as StkId;
    let mut io1_2: *mut TValue = &mut (*res).val;
    (*L).top.p = ((*L).top.p).offset(-1);
    let mut io2_2: *const TValue = &mut (*(*L).top.p).val;
    (*io1_2).value_ = (*io2_2).value_;
    (*io1_2).tt_ = (*io2_2).tt_;
    if (*io1_2).tt_ as i32 & (1 as i32) << 6 as i32 == 0
        || (*io1_2).tt_ as i32 & 0x3f as i32 == (*(*io1_2).value_.gc).tt as i32
            && (L.is_null()
                || (*(*io1_2).value_.gc).marked as i32
                    & ((*(*L).l_G).currentwhite as i32
                        ^ ((1 as i32) << 3 as i32 | (1 as i32) << 4 as i32))
                    == 0)
    {
    } else {
    };
}
unsafe extern "C-unwind" fn callbinTM(
    mut L: *mut lua_State,
    mut p1: *const TValue,
    mut p2: *const TValue,
    mut res: StkId,
    mut event: TMS,
) -> i32 {
    let mut tm: *const TValue = luaT_gettmbyobj(L, p1, event);
    if (*tm).tt_ as i32 & 0xf as i32 == 0 {
        tm = luaT_gettmbyobj(L, p2, event);
    }
    if (*tm).tt_ as i32 & 0xf as i32 == 0 {
        return 0;
    }
    luaT_callTMres(L, tm, p1, p2, res);
    return 1 as i32;
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaT_trybinTM(
    mut L: *mut lua_State,
    mut p1: *const TValue,
    mut p2: *const TValue,
    mut res: StkId,
    mut event: TMS,
) {
    if ((callbinTM(L, p1, p2, res, event) == 0) as i32 != 0) as i32 as std::ffi::c_long != 0 {
        match event as u32 {
            13 | 14 | 15 | 16 | 17 | 19 => {
                if (*p1).tt_ as i32 & 0xf as i32 == 3 as i32
                    && (*p2).tt_ as i32 & 0xf as i32 == 3 as i32
                {
                    luaG_tointerror(L, p1, p2);
                } else {
                    luaG_opinterror(L, p1, p2, c"perform bitwise operation on".as_ptr());
                }
            }
            _ => {
                luaG_opinterror(L, p1, p2, c"perform arithmetic on".as_ptr());
            }
        }
    }
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaT_tryconcatTM(mut L: *mut lua_State) {
    let mut top: StkId = (*L).top.p;
    if ((callbinTM(
        L,
        &mut (*top.offset(-(2))).val,
        &mut (*top.offset(-(1))).val,
        top.offset(-(2)),
        TM_CONCAT,
    ) == 0) as i32
        != 0) as i32 as std::ffi::c_long
        != 0
    {
        luaG_concaterror(
            L,
            &mut (*top.offset(-(2))).val,
            &mut (*top.offset(-(1))).val,
        );
    }
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaT_trybinassocTM(
    mut L: *mut lua_State,
    mut p1: *const TValue,
    mut p2: *const TValue,
    mut flip: i32,
    mut res: StkId,
    mut event: TMS,
) {
    if flip != 0 {
        luaT_trybinTM(L, p2, p1, res, event);
    } else {
        luaT_trybinTM(L, p1, p2, res, event);
    };
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaT_trybiniTM(
    mut L: *mut lua_State,
    mut p1: *const TValue,
    mut i2: lua_Integer,
    mut flip: i32,
    mut res: StkId,
    mut event: TMS,
) {
    let mut aux: TValue = TValue {
        value_: Value {
            gc: 0 as *mut GCObject,
        },
        tt_: 0,
    };
    let mut io: *mut TValue = &mut aux;
    (*io).value_.i = i2;
    (*io).tt_ = (3 as i32 | (0) << 4 as i32) as lu_byte;
    luaT_trybinassocTM(L, p1, &mut aux, flip, res, event);
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaT_callorderTM(
    mut L: *mut lua_State,
    mut p1: *const TValue,
    mut p2: *const TValue,
    mut event: TMS,
) -> i32 {
    if callbinTM(L, p1, p2, (*L).top.p, event) != 0 {
        return !((*(*L).top.p).val.tt_ as i32 == 1 as i32 | (0) << 4 as i32
            || (*(*L).top.p).val.tt_ as i32 & 0xf as i32 == 0) as i32;
    }
    luaG_ordererror(L, p1, p2);
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaT_callorderiTM(
    mut L: *mut lua_State,
    mut p1: *const TValue,
    mut v2: i32,
    mut flip: i32,
    mut isfloat: i32,
    mut event: TMS,
) -> i32 {
    let mut aux: TValue = TValue {
        value_: Value {
            gc: 0 as *mut GCObject,
        },
        tt_: 0,
    };
    let mut p2: *const TValue = 0 as *const TValue;
    if isfloat != 0 {
        let mut io: *mut TValue = &mut aux;
        (*io).value_.n = v2 as lua_Number;
        (*io).tt_ = (3 as i32 | (1 as i32) << 4 as i32) as lu_byte;
    } else {
        let mut io_0: *mut TValue = &mut aux;
        (*io_0).value_.i = v2 as lua_Integer;
        (*io_0).tt_ = (3 as i32 | (0) << 4 as i32) as lu_byte;
    }
    if flip != 0 {
        p2 = p1;
        p1 = &mut aux;
    } else {
        p2 = &mut aux;
    }
    return luaT_callorderTM(L, p1, p2, event);
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaT_adjustvarargs(
    mut L: *mut lua_State,
    mut nfixparams: i32,
    mut ci: *mut CallInfo,
    mut p: *const Proto,
) {
    let mut i: i32 = 0;
    let mut actual: i32 =
        ((*L).top.p).offset_from((*ci).func.p) as std::ffi::c_long as i32 - 1 as i32;
    let mut nextra: i32 = actual - nfixparams;
    (*ci).u.l.nextraargs = nextra;
    if ((((*L).stack_last.p).offset_from((*L).top.p) as std::ffi::c_long
        <= ((*p).maxstacksize as i32 + 1 as i32) as std::ffi::c_long) as i32
        != 0) as i32 as std::ffi::c_long
        != 0
    {
        luaD_growstack(L, (*p).maxstacksize as i32 + 1 as i32, 1 as i32);
    }
    let fresh118 = (*L).top.p;
    (*L).top.p = ((*L).top.p).offset(1);
    let mut io1: *mut TValue = &mut (*fresh118).val;
    let mut io2: *const TValue = &mut (*(*ci).func.p).val;
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
    i = 1 as i32;
    while i <= nfixparams {
        let fresh119 = (*L).top.p;
        (*L).top.p = ((*L).top.p).offset(1);
        let mut io1_0: *mut TValue = &mut (*fresh119).val;
        let mut io2_0: *const TValue = &mut (*((*ci).func.p).offset(i as isize)).val;
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
        (*((*ci).func.p).offset(i as isize)).val.tt_ = (0 | (0) << 4 as i32) as lu_byte;
        i += 1;
        i;
    }
    (*ci).func.p = ((*ci).func.p).offset((actual + 1 as i32) as isize);
    (*ci).top.p = ((*ci).top.p).offset((actual + 1 as i32) as isize);
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaT_getvarargs(
    mut L: *mut lua_State,
    mut ci: *mut CallInfo,
    mut where_0: StkId,
    mut wanted: i32,
) {
    let mut i: i32 = 0;
    let mut nextra: i32 = (*ci).u.l.nextraargs;
    if wanted < 0 {
        wanted = nextra;
        if ((((*L).stack_last.p).offset_from((*L).top.p) as std::ffi::c_long
            <= nextra as std::ffi::c_long) as i32
            != 0) as i32 as std::ffi::c_long
            != 0
        {
            let mut t__: ptrdiff_t = (where_0 as *mut std::ffi::c_char)
                .offset_from((*L).stack.p as *mut std::ffi::c_char);
            if (*(*L).l_G).GCdebt > 0 as l_mem {
                luaC_step(L);
            }
            luaD_growstack(L, nextra, 1 as i32);
            where_0 = ((*L).stack.p as *mut std::ffi::c_char).offset(t__ as isize) as StkId;
        }
        (*L).top.p = where_0.offset(nextra as isize);
    }
    i = 0;
    while i < wanted && i < nextra {
        let mut io1: *mut TValue = &mut (*where_0.offset(i as isize)).val;
        let mut io2: *const TValue =
            &mut (*((*ci).func.p).offset(-(nextra as isize)).offset(i as isize)).val;
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
    while i < wanted {
        (*where_0.offset(i as isize)).val.tt_ = (0 | (0) << 4 as i32) as lu_byte;
        i += 1;
        i;
    }
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaS_eqlngstr(mut a: *mut TString, mut b: *mut TString) -> i32 {
    let mut len: size_t = (*a).u.lnglen;
    return (a == b
        || len == (*b).u.lnglen
            && memcmp(
                ((*a).contents).as_mut_ptr() as *const c_void,
                ((*b).contents).as_mut_ptr() as *const c_void,
                len,
            ) == 0) as i32;
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaS_hash(
    mut str: *const std::ffi::c_char,
    mut l: size_t,
    mut seed: u32,
) -> u32 {
    let mut h: u32 = seed ^ l as u32;
    while l > 0 as size_t {
        h ^= (h << 5 as i32).wrapping_add(h >> 2 as i32).wrapping_add(
            *str.offset(l.wrapping_sub(1 as i32 as size_t) as isize) as lu_byte as u32,
        );
        l = l.wrapping_sub(1);
        l;
    }
    return h;
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaS_hashlongstr(mut ts: *mut TString) -> u32 {
    if (*ts).extra as i32 == 0 {
        let mut len: size_t = (*ts).u.lnglen;
        (*ts).hash = luaS_hash(((*ts).contents).as_mut_ptr(), len, (*ts).hash);
        (*ts).extra = 1 as i32 as lu_byte;
    }
    return (*ts).hash;
}
unsafe extern "C-unwind" fn tablerehash(
    mut vect: *mut *mut TString,
    mut osize: i32,
    mut nsize: i32,
) {
    let mut i: i32 = 0;
    i = osize;
    while i < nsize {
        let ref mut fresh120 = *vect.offset(i as isize);
        *fresh120 = 0 as *mut TString;
        i += 1;
        i;
    }
    i = 0;
    while i < osize {
        let mut p: *mut TString = *vect.offset(i as isize);
        let ref mut fresh121 = *vect.offset(i as isize);
        *fresh121 = 0 as *mut TString;
        while !p.is_null() {
            let mut hnext: *mut TString = (*p).u.hnext;
            let mut h: u32 = ((*p).hash & (nsize - 1 as i32) as u32) as i32 as u32;
            (*p).u.hnext = *vect.offset(h as isize);
            let ref mut fresh122 = *vect.offset(h as isize);
            *fresh122 = p;
            p = hnext;
        }
        i += 1;
        i;
    }
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaS_resize(mut L: *mut lua_State, mut nsize: i32) {
    let mut tb: *mut stringtable = &mut (*(*L).l_G).strt;
    let mut osize: i32 = (*tb).size;
    let mut newvect: *mut *mut TString = 0 as *mut *mut TString;
    if nsize < osize {
        tablerehash((*tb).hash, osize, nsize);
    }
    newvect = luaM_realloc_(
        L,
        (*tb).hash as *mut c_void,
        (osize as size_t).wrapping_mul(size_of::<*mut TString>() as usize),
        (nsize as size_t).wrapping_mul(size_of::<*mut TString>() as usize),
    ) as *mut *mut TString;
    if ((newvect == 0 as *mut c_void as *mut *mut TString) as i32 != 0) as i32 as std::ffi::c_long
        != 0
    {
        if nsize < osize {
            tablerehash((*tb).hash, nsize, osize);
        }
    } else {
        (*tb).hash = newvect;
        (*tb).size = nsize;
        if nsize > osize {
            tablerehash(newvect, osize, nsize);
        }
    };
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaS_clearcache(mut g: *mut global_State) {
    let mut i: i32 = 0;
    let mut j: i32 = 0;
    i = 0;
    while i < 53 as i32 {
        j = 0;
        while j < 2 as i32 {
            if (*(*g).strcache[i as usize][j as usize]).marked as i32
                & ((1 as i32) << 3 as i32 | (1 as i32) << 4 as i32)
                != 0
            {
                (*g).strcache[i as usize][j as usize] = (*g).memerrmsg;
            }
            j += 1;
            j;
        }
        i += 1;
        i;
    }
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaS_init(mut L: *mut lua_State) {
    let mut g: *mut global_State = (*L).l_G;
    let mut i: i32 = 0;
    let mut j: i32 = 0;
    let mut tb: *mut stringtable = &mut (*(*L).l_G).strt;
    (*tb).hash = luaM_malloc_(
        L,
        (128usize).wrapping_mul(size_of::<*mut TString>() as usize),
        0,
    ) as *mut *mut TString;
    tablerehash((*tb).hash, 0, 128 as i32);
    (*tb).size = 128 as i32;
    (*g).memerrmsg = luaS_newlstr(
        L,
        c"not enough memory".as_ptr(),
        (size_of::<[std::ffi::c_char; 18]>() as usize)
            .wrapping_div(size_of::<std::ffi::c_char>() as usize)
            .wrapping_sub(1),
    );
    luaC_fix(L, &mut (*((*g).memerrmsg as *mut GCUnion)).gc);
    i = 0;
    while i < 53 as i32 {
        j = 0;
        while j < 2 as i32 {
            (*g).strcache[i as usize][j as usize] = (*g).memerrmsg;
            j += 1;
            j;
        }
        i += 1;
        i;
    }
}
unsafe extern "C-unwind" fn createstrobj(
    mut L: *mut lua_State,
    mut l: size_t,
    mut tag: i32,
    mut h: u32,
) -> *mut TString {
    let mut ts: *mut TString = 0 as *mut TString;
    let mut o: *mut GCObject = 0 as *mut GCObject;
    let mut totalsize: size_t = 0;
    totalsize = (24 as usize).wrapping_add(
        l.wrapping_add(1 as i32 as size_t)
            .wrapping_mul(size_of::<std::ffi::c_char>() as usize),
    );
    o = luaC_newobj(L, tag, totalsize);
    ts = &mut (*(o as *mut GCUnion)).ts;
    (*ts).hash = h;
    (*ts).extra = 0 as lu_byte;
    *((*ts).contents).as_mut_ptr().offset(l as isize) = '\0' as i32 as std::ffi::c_char;
    return ts;
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaS_createlngstrobj(
    mut L: *mut lua_State,
    mut l: size_t,
) -> *mut TString {
    let mut ts: *mut TString =
        createstrobj(L, l, 4 as i32 | (1 as i32) << 4 as i32, (*(*L).l_G).seed);
    (*ts).u.lnglen = l;
    (*ts).shrlen = 0xff as i32 as lu_byte;
    return ts;
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaS_remove(mut L: *mut lua_State, mut ts: *mut TString) {
    let mut tb: *mut stringtable = &mut (*(*L).l_G).strt;
    let mut p: *mut *mut TString = &mut *((*tb).hash)
        .offset(((*ts).hash & ((*tb).size - 1 as i32) as u32) as i32 as isize)
        as *mut *mut TString;
    while *p != ts {
        p = &mut (**p).u.hnext;
    }
    *p = (**p).u.hnext;
    (*tb).nuse -= 1;
    (*tb).nuse;
}
unsafe extern "C-unwind" fn growstrtab(mut L: *mut lua_State, mut tb: *mut stringtable) {
    if (((*tb).nuse == 2147483647 as i32) as i32 != 0) as i32 as std::ffi::c_long != 0 {
        luaC_fullgc(L, 1 as i32);
        if (*tb).nuse == 2147483647 as i32 {
            luaD_throw(L, 4 as i32);
        }
    }
    if (*tb).size
        <= (if 2147483647 as i32 as size_t
            <= (!(0 as size_t)).wrapping_div(size_of::<*mut TString>() as usize)
        {
            2147483647
        } else {
            (!(0 as size_t)).wrapping_div(size_of::<*mut TString>() as usize) as u32
        }) as i32
            / 2 as i32
    {
        luaS_resize(L, (*tb).size * 2 as i32);
    }
}
unsafe extern "C-unwind" fn internshrstr(
    mut L: *mut lua_State,
    mut str: *const std::ffi::c_char,
    mut l: size_t,
) -> *mut TString {
    let mut ts: *mut TString = 0 as *mut TString;
    let mut g: *mut global_State = (*L).l_G;
    let mut tb: *mut stringtable = &mut (*g).strt;
    let mut h: u32 = luaS_hash(str, l, (*g).seed);
    let mut list: *mut *mut TString = &mut *((*tb).hash)
        .offset((h & ((*tb).size - 1 as i32) as u32) as i32 as isize)
        as *mut *mut TString;
    ts = *list;
    while !ts.is_null() {
        if l == (*ts).shrlen as size_t
            && memcmp(
                str as *const c_void,
                ((*ts).contents).as_mut_ptr() as *const c_void,
                l.wrapping_mul(size_of::<std::ffi::c_char>() as usize),
            ) == 0
        {
            if (*ts).marked as i32
                & ((*g).currentwhite as i32 ^ ((1 as i32) << 3 as i32 | (1 as i32) << 4 as i32))
                != 0
            {
                (*ts).marked = ((*ts).marked as i32
                    ^ ((1 as i32) << 3 as i32 | (1 as i32) << 4 as i32))
                    as lu_byte;
            }
            return ts;
        }
        ts = (*ts).u.hnext;
    }
    if (*tb).nuse >= (*tb).size {
        growstrtab(L, tb);
        list = &mut *((*tb).hash).offset((h & ((*tb).size - 1 as i32) as u32) as i32 as isize)
            as *mut *mut TString;
    }
    ts = createstrobj(L, l, 4 as i32 | (0) << 4 as i32, h);
    (*ts).shrlen = l as lu_byte;
    memcpy(
        ((*ts).contents).as_mut_ptr() as *mut c_void,
        str as *const c_void,
        l.wrapping_mul(size_of::<std::ffi::c_char>() as usize),
    );
    (*ts).u.hnext = *list;
    *list = ts;
    (*tb).nuse += 1;
    (*tb).nuse;
    return ts;
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaS_newlstr(
    mut L: *mut lua_State,
    mut str: *const std::ffi::c_char,
    mut l: size_t,
) -> *mut TString {
    if l <= 40 as size_t {
        return internshrstr(L, str, l);
    } else {
        let mut ts: *mut TString = 0 as *mut TString;
        if ((l.wrapping_mul(size_of::<std::ffi::c_char>() as usize)
            >= (if (size_of::<size_t>() as usize) < size_of::<lua_Integer>() as usize {
                !(0 as size_t)
            } else {
                9223372036854775807 as std::ffi::c_longlong as size_t
            })
            .wrapping_sub(size_of::<TString>() as usize)) as i32
            != 0) as i32 as std::ffi::c_long
            != 0
        {
            luaM_toobig(L);
        }
        ts = luaS_createlngstrobj(L, l);
        memcpy(
            ((*ts).contents).as_mut_ptr() as *mut c_void,
            str as *const c_void,
            l.wrapping_mul(size_of::<std::ffi::c_char>() as usize),
        );
        return ts;
    };
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaS_new(
    mut L: *mut lua_State,
    mut str: *const std::ffi::c_char,
) -> *mut TString {
    let mut i: u32 = ((str as uintptr_t
        & (2147483647u32)
            .wrapping_mul(2 as u32)
            .wrapping_add(1 as u32) as uintptr_t) as u32)
        .wrapping_rem(53);
    let mut j: i32 = 0;
    let mut p: *mut *mut TString = ((*(*L).l_G).strcache[i as usize]).as_mut_ptr();
    j = 0;
    while j < 2 as i32 {
        if strcmp(str, ((**p.offset(j as isize)).contents).as_mut_ptr()) == 0 {
            return *p.offset(j as isize);
        }
        j += 1;
        j;
    }
    j = 2 as i32 - 1 as i32;
    while j > 0 {
        let ref mut fresh123 = *p.offset(j as isize);
        *fresh123 = *p.offset((j - 1 as i32) as isize);
        j -= 1;
        j;
    }
    let ref mut fresh124 = *p.offset(0 as isize);
    *fresh124 = luaS_newlstr(L, str, strlen(str));
    return *p.offset(0 as isize);
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaS_newudata(
    mut L: *mut lua_State,
    mut s: size_t,
    mut nuvalue: i32,
) -> *mut Udata {
    let mut u: *mut Udata = 0 as *mut Udata;
    let mut i: i32 = 0;
    let mut o: *mut GCObject = 0 as *mut GCObject;
    if ((s
        > (if (size_of::<size_t>() as usize) < size_of::<lua_Integer>() as usize {
            !(0 as size_t)
        } else {
            9223372036854775807 as std::ffi::c_longlong as size_t
        })
        .wrapping_sub(
            (if nuvalue == 0 {
                32 as usize
            } else {
                (40 as usize)
                    .wrapping_add((size_of::<UValue>() as usize).wrapping_mul(nuvalue as usize))
            }),
        )) as i32
        != 0) as i32 as std::ffi::c_long
        != 0
    {
        luaM_toobig(L);
    }
    o = luaC_newobj(
        L,
        7 as i32 | (0) << 4 as i32,
        (if nuvalue == 0 {
            32 as usize
        } else {
            (40 as usize)
                .wrapping_add((size_of::<UValue>() as usize).wrapping_mul(nuvalue as usize))
        })
        .wrapping_add(s),
    );
    u = &mut (*(o as *mut GCUnion)).u;
    (*u).len = s;
    (*u).nuvalue = nuvalue as u16;
    (*u).metatable = 0 as *mut Table;
    i = 0;
    while i < nuvalue {
        (*((*u).uv).as_mut_ptr().offset(i as isize)).uv.tt_ = (0 | (0) << 4 as i32) as lu_byte;
        i += 1;
        i;
    }
    return u;
}

#[unsafe(no_mangle)]
pub static mut lua_ident: [std::ffi::c_char; 129] = unsafe {
    *::core::mem::transmute::<
        &[u8; 129],
        &[std::ffi::c_char; 129],
    >(
        b"$LuaVersion: Lua 5.4.7  Copyright (C) 1994-2024 Lua.org, PUC-Rio $$LuaAuthors: R. Ierusalimschy, L. H. de Figueiredo, W. Celes $\0",
    )
};
unsafe extern "C-unwind" fn index2value(mut L: *mut lua_State, mut idx: i32) -> *mut TValue {
    let mut ci: *mut CallInfo = (*L).ci;
    if idx > 0 {
        let mut o: StkId = ((*ci).func.p).offset(idx as isize);
        if o >= (*L).top.p {
            return &mut (*(*L).l_G).nilvalue;
        } else {
            return &mut (*o).val;
        }
    } else if !(idx <= -(1000000) - 1000) {
        return &mut (*((*L).top.p).offset(idx as isize)).val;
    } else if idx == -(1000000) - 1000 {
        return &mut (*(*L).l_G).l_registry;
    } else {
        idx = -(1000000) - 1000 - idx;
        if (*(*ci).func.p).val.tt_ as i32
            == 6 as i32 | (2 as i32) << 4 as i32 | (1 as i32) << 6 as i32
        {
            let mut func: *mut CClosure =
                &mut (*((*(*ci).func.p).val.value_.gc as *mut GCUnion)).cl.c;
            return if idx <= (*func).nupvalues as i32 {
                &mut *((*func).upvalue)
                    .as_mut_ptr()
                    .offset((idx - 1 as i32) as isize) as *mut TValue
            } else {
                &mut (*(*L).l_G).nilvalue
            };
        } else {
            return &mut (*(*L).l_G).nilvalue;
        }
    };
}
#[inline]
unsafe extern "C-unwind" fn index2stack(mut L: *mut lua_State, mut idx: i32) -> StkId {
    let mut ci: *mut CallInfo = (*L).ci;
    if idx > 0 {
        let mut o: StkId = ((*ci).func.p).offset(idx as isize);
        return o;
    } else {
        return ((*L).top.p).offset(idx as isize);
    };
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn lua_checkstack(mut L: *mut lua_State, mut n: i32) -> i32 {
    let mut res: i32 = 0;
    let mut ci: *mut CallInfo = 0 as *mut CallInfo;
    ci = (*L).ci;
    if ((*L).stack_last.p).offset_from((*L).top.p) as std::ffi::c_long > n as std::ffi::c_long {
        res = 1 as i32;
    } else {
        res = luaD_growstack(L, n, 0);
    }
    if res != 0 && (*ci).top.p < ((*L).top.p).offset(n as isize) {
        (*ci).top.p = ((*L).top.p).offset(n as isize);
    }
    return res;
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn lua_xmove(
    mut from: *mut lua_State,
    mut to: *mut lua_State,
    mut n: i32,
) {
    let mut i: i32 = 0;
    if from == to {
        return;
    }
    (*from).top.p = ((*from).top.p).offset(-(n as isize));
    i = 0;
    while i < n {
        let mut io1: *mut TValue = &mut (*(*to).top.p).val;
        let mut io2: *const TValue = &mut (*((*from).top.p).offset(i as isize)).val;
        (*io1).value_ = (*io2).value_;
        (*io1).tt_ = (*io2).tt_;
        if (*io1).tt_ as i32 & (1 as i32) << 6 as i32 == 0
            || (*io1).tt_ as i32 & 0x3f as i32 == (*(*io1).value_.gc).tt as i32
                && (to.is_null()
                    || (*(*io1).value_.gc).marked as i32
                        & ((*(*to).l_G).currentwhite as i32
                            ^ ((1 as i32) << 3 as i32 | (1 as i32) << 4 as i32))
                        == 0)
        {
        } else {
        };
        (*to).top.p = ((*to).top.p).offset(1);
        (*to).top.p;
        i += 1;
        i;
    }
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn lua_atpanic(
    mut L: *mut lua_State,
    mut panicf: lua_CFunction,
) -> lua_CFunction {
    let mut old: lua_CFunction = None;
    old = (*(*L).l_G).panic;
    (*(*L).l_G).panic = panicf;
    return old;
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn lua_version(mut L: *mut lua_State) -> lua_Number {
    return 504 as i32 as lua_Number;
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn lua_absindex(mut L: *mut lua_State, mut idx: i32) -> i32 {
    return if idx > 0 || idx <= -(1000000) - 1000 {
        idx
    } else {
        ((*L).top.p).offset_from((*(*L).ci).func.p) as std::ffi::c_long as i32 + idx
    };
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn lua_gettop(mut L: *mut lua_State) -> i32 {
    return ((*L).top.p).offset_from(((*(*L).ci).func.p).offset(1)) as std::ffi::c_long as i32;
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn lua_settop(mut L: *mut lua_State, mut idx: i32) {
    let mut ci: *mut CallInfo = 0 as *mut CallInfo;
    let mut func: StkId = 0 as *mut StackValue;
    let mut newtop: StkId = 0 as *mut StackValue;
    let mut diff: ptrdiff_t = 0;
    ci = (*L).ci;
    func = (*ci).func.p;
    if idx >= 0 {
        diff = func.offset(1).offset(idx as isize).offset_from((*L).top.p);
        while diff > 0 as ptrdiff_t {
            let fresh141 = (*L).top.p;
            (*L).top.p = ((*L).top.p).offset(1);
            (*fresh141).val.tt_ = (0 | (0) << 4 as i32) as lu_byte;
            diff -= 1;
            diff;
        }
    } else {
        diff = (idx + 1 as i32) as ptrdiff_t;
    }
    newtop = ((*L).top.p).offset(diff as isize);
    if diff < 0 as ptrdiff_t && (*L).tbclist.p >= newtop {
        newtop = luaF_close(L, newtop, -(1 as i32), 0);
    }
    (*L).top.p = newtop;
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn lua_closeslot(mut L: *mut lua_State, mut idx: i32) {
    let mut level: StkId = 0 as *mut StackValue;
    level = index2stack(L, idx);
    level = luaF_close(L, level, -(1 as i32), 0);
    (*level).val.tt_ = (0 | (0) << 4 as i32) as lu_byte;
}
#[inline]
unsafe extern "C-unwind" fn reverse(mut L: *mut lua_State, mut from: StkId, mut to: StkId) {
    while from < to {
        let mut temp: TValue = TValue {
            value_: Value {
                gc: 0 as *mut GCObject,
            },
            tt_: 0,
        };
        let mut io1: *mut TValue = &mut temp;
        let mut io2: *const TValue = &mut (*from).val;
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
        let mut io1_0: *mut TValue = &mut (*from).val;
        let mut io2_0: *const TValue = &mut (*to).val;
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
        let mut io1_1: *mut TValue = &mut (*to).val;
        let mut io2_1: *const TValue = &mut temp;
        (*io1_1).value_ = (*io2_1).value_;
        (*io1_1).tt_ = (*io2_1).tt_;
        if (*io1_1).tt_ as i32 & (1 as i32) << 6 as i32 == 0
            || (*io1_1).tt_ as i32 & 0x3f as i32 == (*(*io1_1).value_.gc).tt as i32
                && (L.is_null()
                    || (*(*io1_1).value_.gc).marked as i32
                        & ((*(*L).l_G).currentwhite as i32
                            ^ ((1 as i32) << 3 as i32 | (1 as i32) << 4 as i32))
                        == 0)
        {
        } else {
        };
        from = from.offset(1);
        from;
        to = to.offset(-1);
        to;
    }
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn lua_rotate(mut L: *mut lua_State, mut idx: i32, mut n: i32) {
    let mut p: StkId = 0 as *mut StackValue;
    let mut t: StkId = 0 as *mut StackValue;
    let mut m: StkId = 0 as *mut StackValue;
    t = ((*L).top.p).offset(-(1));
    p = index2stack(L, idx);
    m = if n >= 0 {
        t.offset(-(n as isize))
    } else {
        p.offset(-(n as isize)).offset(-(1))
    };
    reverse(L, p, m);
    reverse(L, m.offset(1), t);
    reverse(L, p, t);
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn lua_copy(mut L: *mut lua_State, mut fromidx: i32, mut toidx: i32) {
    let mut fr: *mut TValue = 0 as *mut TValue;
    let mut to: *mut TValue = 0 as *mut TValue;
    fr = index2value(L, fromidx);
    to = index2value(L, toidx);
    let mut io1: *mut TValue = to;
    let mut io2: *const TValue = fr;
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
    if toidx < -(1000000) - 1000 {
        if (*fr).tt_ as i32 & (1 as i32) << 6 as i32 != 0 {
            if (*(&mut (*((*(*(*L).ci).func.p).val.value_.gc as *mut GCUnion)).cl.c
                as *mut CClosure))
                .marked as i32
                & (1 as i32) << 5 as i32
                != 0
                && (*(*fr).value_.gc).marked as i32
                    & ((1 as i32) << 3 as i32 | (1 as i32) << 4 as i32)
                    != 0
            {
                luaC_barrier_(
                    L,
                    &mut (*(&mut (*((*(*(*L).ci).func.p).val.value_.gc as *mut GCUnion)).cl.c
                        as *mut CClosure as *mut GCUnion))
                        .gc,
                    &mut (*((*fr).value_.gc as *mut GCUnion)).gc,
                );
            } else {
            };
        } else {
        };
    }
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn lua_pushvalue(mut L: *mut lua_State, mut idx: i32) {
    let mut io1: *mut TValue = &mut (*(*L).top.p).val;
    let mut io2: *const TValue = index2value(L, idx);
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
    (*L).top.p = ((*L).top.p).offset(1);
    (*L).top.p;
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn lua_type(mut L: *mut lua_State, mut idx: i32) -> i32 {
    let mut o: *const TValue = index2value(L, idx);
    return if !((*o).tt_ as i32 & 0xf as i32 == 0)
        || o != &mut (*(*L).l_G).nilvalue as *mut TValue as *const TValue
    {
        (*o).tt_ as i32 & 0xf as i32
    } else {
        -(1 as i32)
    };
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn lua_typename(
    mut L: *mut lua_State,
    mut t: i32,
) -> *const std::ffi::c_char {
    return luaT_typenames_[(t + 1 as i32) as usize];
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn lua_iscfunction(mut L: *mut lua_State, mut idx: i32) -> i32 {
    let mut o: *const TValue = index2value(L, idx);
    return ((*o).tt_ as i32 == 6 as i32 | (1 as i32) << 4 as i32
        || (*o).tt_ as i32 == 6 as i32 | (2 as i32) << 4 as i32 | (1 as i32) << 6 as i32)
        as i32;
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn lua_isinteger(mut L: *mut lua_State, mut idx: i32) -> i32 {
    let mut o: *const TValue = index2value(L, idx);
    return ((*o).tt_ as i32 == 3 as i32 | (0) << 4 as i32) as i32;
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn lua_isnumber(mut L: *mut lua_State, mut idx: i32) -> i32 {
    let mut n: lua_Number = 0.;
    let mut o: *const TValue = index2value(L, idx);
    return if (*o).tt_ as i32 == 3 as i32 | (1 as i32) << 4 as i32 {
        n = (*o).value_.n;
        1 as i32
    } else {
        luaV_tonumber_(o, &mut n)
    };
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn lua_isstring(mut L: *mut lua_State, mut idx: i32) -> i32 {
    let mut o: *const TValue = index2value(L, idx);
    return ((*o).tt_ as i32 & 0xf as i32 == 4 as i32 || (*o).tt_ as i32 & 0xf as i32 == 3 as i32)
        as i32;
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn lua_isuserdata(mut L: *mut lua_State, mut idx: i32) -> i32 {
    let mut o: *const TValue = index2value(L, idx);
    return ((*o).tt_ as i32 == 7 as i32 | (0) << 4 as i32 | (1 as i32) << 6 as i32
        || (*o).tt_ as i32 == 2 as i32 | (0) << 4 as i32) as i32;
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn lua_rawequal(
    mut L: *mut lua_State,
    mut index1: i32,
    mut index2: i32,
) -> i32 {
    let mut o1: *const TValue = index2value(L, index1);
    let mut o2: *const TValue = index2value(L, index2);
    return if (!((*o1).tt_ as i32 & 0xf as i32 == 0)
        || o1 != &mut (*(*L).l_G).nilvalue as *mut TValue as *const TValue)
        && (!((*o2).tt_ as i32 & 0xf as i32 == 0)
            || o2 != &mut (*(*L).l_G).nilvalue as *mut TValue as *const TValue)
    {
        luaV_equalobj(0 as *mut lua_State, o1, o2)
    } else {
        0
    };
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn lua_arith(mut L: *mut lua_State, mut op: i32) {
    if !(op != 12 as i32 && op != 13 as i32) {
        let mut io1: *mut TValue = &mut (*(*L).top.p).val;
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
        (*L).top.p = ((*L).top.p).offset(1);
        (*L).top.p;
    }
    luaO_arith(
        L,
        op,
        &mut (*((*L).top.p).offset(-(2))).val,
        &mut (*((*L).top.p).offset(-(1))).val,
        ((*L).top.p).offset(-(2)),
    );
    (*L).top.p = ((*L).top.p).offset(-1);
    (*L).top.p;
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn lua_compare(
    mut L: *mut lua_State,
    mut index1: i32,
    mut index2: i32,
    mut op: i32,
) -> i32 {
    let mut o1: *const TValue = 0 as *const TValue;
    let mut o2: *const TValue = 0 as *const TValue;
    let mut i: i32 = 0;
    o1 = index2value(L, index1);
    o2 = index2value(L, index2);
    if (!((*o1).tt_ as i32 & 0xf as i32 == 0)
        || o1 != &mut (*(*L).l_G).nilvalue as *mut TValue as *const TValue)
        && (!((*o2).tt_ as i32 & 0xf as i32 == 0)
            || o2 != &mut (*(*L).l_G).nilvalue as *mut TValue as *const TValue)
    {
        match op {
            0 => {
                i = luaV_equalobj(L, o1, o2);
            }
            1 => {
                i = luaV_lessthan(L, o1, o2);
            }
            2 => {
                i = luaV_lessequal(L, o1, o2);
            }
            _ => {}
        }
    }
    return i;
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn lua_stringtonumber(
    mut L: *mut lua_State,
    mut s: *const std::ffi::c_char,
) -> size_t {
    let mut sz: size_t = luaO_str2num(s, &mut (*(*L).top.p).val);
    if sz != 0 as size_t {
        (*L).top.p = ((*L).top.p).offset(1);
        (*L).top.p;
    }
    return sz;
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn lua_tonumberx(
    mut L: *mut lua_State,
    mut idx: i32,
    mut pisnum: *mut i32,
) -> lua_Number {
    let mut n: lua_Number = 0 as lua_Number;
    let mut o: *const TValue = index2value(L, idx);
    let mut isnum: i32 = if (*o).tt_ as i32 == 3 as i32 | (1 as i32) << 4 as i32 {
        n = (*o).value_.n;
        1 as i32
    } else {
        luaV_tonumber_(o, &mut n)
    };
    if !pisnum.is_null() {
        *pisnum = isnum;
    }
    return n;
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn lua_tointegerx(
    mut L: *mut lua_State,
    mut idx: i32,
    mut pisnum: *mut i32,
) -> lua_Integer {
    let mut res: lua_Integer = 0 as lua_Integer;
    let mut o: *const TValue = index2value(L, idx);
    let mut isnum: i32 = if (((*o).tt_ as i32 == 3 as i32 | (0) << 4 as i32) as i32 != 0) as i32
        as std::ffi::c_long
        != 0
    {
        res = (*o).value_.i;
        1 as i32
    } else {
        luaV_tointeger::<F2Ieq>(o, &mut res)
    };
    if !pisnum.is_null() {
        *pisnum = isnum;
    }
    return res;
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn lua_toboolean(mut L: *mut lua_State, mut idx: i32) -> i32 {
    let mut o: *const TValue = index2value(L, idx);
    return !((*o).tt_ as i32 == 1 as i32 | (0) << 4 as i32 || (*o).tt_ as i32 & 0xf as i32 == 0)
        as i32;
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn lua_tolstring(
    mut L: *mut lua_State,
    mut idx: i32,
    mut len: *mut size_t,
) -> *const std::ffi::c_char {
    let mut o: *mut TValue = 0 as *mut TValue;
    o = index2value(L, idx);
    if !((*o).tt_ as i32 & 0xf as i32 == 4 as i32) {
        if !((*o).tt_ as i32 & 0xf as i32 == 3 as i32) {
            if !len.is_null() {
                *len = 0 as size_t;
            }
            return 0 as *const std::ffi::c_char;
        }
        luaO_tostring(L, o);
        if (*(*L).l_G).GCdebt > 0 as l_mem {
            luaC_step(L);
        }
        o = index2value(L, idx);
    }
    if !len.is_null() {
        *len = if (*(&mut (*((*o).value_.gc as *mut GCUnion)).ts as *mut TString)).shrlen as i32
            != 0xff as i32
        {
            (*&mut (*((*o).value_.gc as *mut GCUnion)).ts).shrlen as size_t
        } else {
            (*&mut (*((*o).value_.gc as *mut GCUnion)).ts).u.lnglen
        };
    }
    return ((*&mut (*((*o).value_.gc as *mut GCUnion)).ts).contents).as_mut_ptr();
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn lua_rawlen(mut L: *mut lua_State, mut idx: i32) -> lua_Unsigned {
    let mut o: *const TValue = index2value(L, idx);
    match (*o).tt_ as i32 & 0x3f as i32 {
        4 => return (*&mut (*((*o).value_.gc as *mut GCUnion)).ts).shrlen as lua_Unsigned,
        20 => {
            return (*&mut (*((*o).value_.gc as *mut GCUnion)).ts).u.lnglen as lua_Unsigned;
        }
        7 => return (*&mut (*((*o).value_.gc as *mut GCUnion)).u).len as lua_Unsigned,
        5 => return luaH_getn(&mut (*((*o).value_.gc as *mut GCUnion)).h),
        _ => return 0 as lua_Unsigned,
    };
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn lua_tocfunction(
    mut L: *mut lua_State,
    mut idx: i32,
) -> lua_CFunction {
    let mut o: *const TValue = index2value(L, idx);
    if (*o).tt_ as i32 == 6 as i32 | (1 as i32) << 4 as i32 {
        return (*o).value_.f;
    } else if (*o).tt_ as i32 == 6 as i32 | (2 as i32) << 4 as i32 | (1 as i32) << 6 as i32 {
        return (*&mut (*((*o).value_.gc as *mut GCUnion)).cl.c).f;
    } else {
        return None;
    };
}
#[inline]
unsafe extern "C-unwind" fn touserdata(mut o: *const TValue) -> *mut c_void {
    match (*o).tt_ as i32 & 0xf as i32 {
        7 => {
            return (&mut (*((*o).value_.gc as *mut GCUnion)).u as *mut Udata
                as *mut std::ffi::c_char)
                .offset(
                    (if (*(&mut (*((*o).value_.gc as *mut GCUnion)).u as *mut Udata)).nuvalue as i32
                        == 0
                    {
                        32 as usize
                    } else {
                        (40 as usize).wrapping_add((size_of::<UValue>() as usize).wrapping_mul(
                            (*(&mut (*((*o).value_.gc as *mut GCUnion)).u as *mut Udata)).nuvalue
                                as usize,
                        ))
                    }) as isize,
                ) as *mut c_void;
        }
        2 => return (*o).value_.p,
        _ => return 0 as *mut c_void,
    };
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn lua_touserdata(mut L: *mut lua_State, mut idx: i32) -> *mut c_void {
    let mut o: *const TValue = index2value(L, idx);
    return touserdata(o);
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn lua_tothread(
    mut L: *mut lua_State,
    mut idx: i32,
) -> *mut lua_State {
    let mut o: *const TValue = index2value(L, idx);
    return if !((*o).tt_ as i32 == 8 as i32 | (0) << 4 as i32 | (1 as i32) << 6 as i32) {
        0 as *mut lua_State
    } else {
        &mut (*((*o).value_.gc as *mut GCUnion)).th
    };
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn lua_topointer(
    mut L: *mut lua_State,
    mut idx: i32,
) -> *const c_void {
    let mut o: *const TValue = index2value(L, idx);
    match (*o).tt_ as i32 & 0x3f as i32 {
        22 => {
            return ::core::mem::transmute::<lua_CFunction, size_t>((*o).value_.f) as *mut c_void;
        }
        7 | 2 => return touserdata(o),
        _ => {
            if (*o).tt_ as i32 & (1 as i32) << 6 as i32 != 0 {
                return (*o).value_.gc as *const c_void;
            } else {
                return 0 as *const c_void;
            }
        }
    };
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn lua_pushnil(mut L: *mut lua_State) {
    (*(*L).top.p).val.tt_ = (0 | (0) << 4 as i32) as lu_byte;
    (*L).top.p = ((*L).top.p).offset(1);
    (*L).top.p;
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn lua_pushnumber(mut L: *mut lua_State, mut n: lua_Number) {
    let mut io: *mut TValue = &mut (*(*L).top.p).val;
    (*io).value_.n = n;
    (*io).tt_ = (3 as i32 | (1 as i32) << 4 as i32) as lu_byte;
    (*L).top.p = ((*L).top.p).offset(1);
    (*L).top.p;
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn lua_pushinteger(mut L: *mut lua_State, mut n: lua_Integer) {
    let mut io: *mut TValue = &mut (*(*L).top.p).val;
    (*io).value_.i = n;
    (*io).tt_ = (3 as i32 | (0) << 4 as i32) as lu_byte;
    (*L).top.p = ((*L).top.p).offset(1);
    (*L).top.p;
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn lua_pushlstring(
    mut L: *mut lua_State,
    mut s: *const std::ffi::c_char,
    mut len: size_t,
) -> *const std::ffi::c_char {
    let mut ts: *mut TString = 0 as *mut TString;
    ts = if len == 0 as size_t {
        luaS_new(L, c"".as_ptr())
    } else {
        luaS_newlstr(L, s, len)
    };
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
    (*L).top.p = ((*L).top.p).offset(1);
    (*L).top.p;
    if (*(*L).l_G).GCdebt > 0 as l_mem {
        luaC_step(L);
    }
    return ((*ts).contents).as_mut_ptr();
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn lua_pushstring(
    mut L: *mut lua_State,
    mut s: *const std::ffi::c_char,
) -> *const std::ffi::c_char {
    if s.is_null() {
        (*(*L).top.p).val.tt_ = (0 | (0) << 4 as i32) as lu_byte;
    } else {
        let mut ts: *mut TString = 0 as *mut TString;
        ts = luaS_new(L, s);
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
        s = ((*ts).contents).as_mut_ptr();
    }
    (*L).top.p = ((*L).top.p).offset(1);
    (*L).top.p;
    if (*(*L).l_G).GCdebt > 0 as l_mem {
        luaC_step(L);
    }
    return s;
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn lua_pushvfstring(
    mut L: *mut lua_State,
    mut fmt: *const std::ffi::c_char,
    mut argp: ::core::ffi::VaList,
) -> *const std::ffi::c_char {
    let mut ret: *const std::ffi::c_char = 0 as *const std::ffi::c_char;
    ret = luaO_pushvfstring(L, fmt, argp.as_va_list());
    if (*(*L).l_G).GCdebt > 0 as l_mem {
        luaC_step(L);
    }
    return ret;
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn lua_pushfstring(
    mut L: *mut lua_State,
    mut fmt: *const std::ffi::c_char,
    mut args: ...
) -> *const std::ffi::c_char {
    let mut ret: *const std::ffi::c_char = 0 as *const std::ffi::c_char;
    let mut argp: ::core::ffi::VaListImpl;
    argp = args.clone();
    ret = luaO_pushvfstring(L, fmt, argp.as_va_list());
    if (*(*L).l_G).GCdebt > 0 as l_mem {
        luaC_step(L);
    }
    return ret;
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn lua_pushcclosure(
    mut L: *mut lua_State,
    mut fn_0: lua_CFunction,
    mut n: i32,
) {
    if n == 0 {
        let mut io: *mut TValue = &mut (*(*L).top.p).val;
        (*io).value_.f = fn_0;
        (*io).tt_ = (6 as i32 | (1 as i32) << 4 as i32) as lu_byte;
        (*L).top.p = ((*L).top.p).offset(1);
        (*L).top.p;
    } else {
        let mut cl: *mut CClosure = 0 as *mut CClosure;
        cl = luaF_newCclosure(L, n);
        (*cl).f = fn_0;
        (*L).top.p = ((*L).top.p).offset(-(n as isize));
        loop {
            let fresh142 = n;
            n = n - 1;
            if !(fresh142 != 0) {
                break;
            }
            let mut io1: *mut TValue =
                &mut *((*cl).upvalue).as_mut_ptr().offset(n as isize) as *mut TValue;
            let mut io2: *const TValue = &mut (*((*L).top.p).offset(n as isize)).val;
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
        let mut io_0: *mut TValue = &mut (*(*L).top.p).val;
        let mut x_: *mut CClosure = cl;
        (*io_0).value_.gc = &mut (*(x_ as *mut GCUnion)).gc;
        (*io_0).tt_ = (6 as i32 | (2 as i32) << 4 as i32 | (1 as i32) << 6 as i32) as lu_byte;
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
        (*L).top.p = ((*L).top.p).offset(1);
        (*L).top.p;
        if (*(*L).l_G).GCdebt > 0 as l_mem {
            luaC_step(L);
        }
    };
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn lua_pushboolean(mut L: *mut lua_State, mut b: i32) {
    if b != 0 {
        (*(*L).top.p).val.tt_ = (1 as i32 | (1 as i32) << 4 as i32) as lu_byte;
    } else {
        (*(*L).top.p).val.tt_ = (1 as i32 | (0) << 4 as i32) as lu_byte;
    }
    (*L).top.p = ((*L).top.p).offset(1);
    (*L).top.p;
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn lua_pushlightuserdata(mut L: *mut lua_State, mut p: *mut c_void) {
    let mut io: *mut TValue = &mut (*(*L).top.p).val;
    (*io).value_.p = p;
    (*io).tt_ = (2 as i32 | (0) << 4 as i32) as lu_byte;
    (*L).top.p = ((*L).top.p).offset(1);
    (*L).top.p;
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn lua_pushthread(mut L: *mut lua_State) -> i32 {
    let mut io: *mut TValue = &mut (*(*L).top.p).val;
    let mut x_: *mut lua_State = L;
    (*io).value_.gc = &mut (*(x_ as *mut GCUnion)).gc;
    (*io).tt_ = (8 as i32 | (0) << 4 as i32 | (1 as i32) << 6 as i32) as lu_byte;
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
    return ((*(*L).l_G).mainthread == L) as i32;
}
#[inline]
unsafe extern "C-unwind" fn auxgetstr(
    mut L: *mut lua_State,
    mut t: *const TValue,
    mut k: *const std::ffi::c_char,
) -> i32 {
    let mut slot: *const TValue = 0 as *const TValue;
    let mut str: *mut TString = luaS_new(L, k);
    if if !((*t).tt_ as i32 == 5 as i32 | (0) << 4 as i32 | (1 as i32) << 6 as i32) {
        slot = 0 as *const TValue;
        0
    } else {
        slot = luaH_getstr(&mut (*((*t).value_.gc as *mut GCUnion)).h, str);
        !((*slot).tt_ as i32 & 0xf as i32 == 0) as i32
    } != 0
    {
        let mut io1: *mut TValue = &mut (*(*L).top.p).val;
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
        (*L).top.p = ((*L).top.p).offset(1);
        (*L).top.p;
    } else {
        let mut io: *mut TValue = &mut (*(*L).top.p).val;
        let mut x_: *mut TString = str;
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
        luaV_finishget(
            L,
            t,
            &mut (*((*L).top.p).offset(-(1))).val,
            ((*L).top.p).offset(-(1)),
            slot,
        );
    }
    return (*((*L).top.p).offset(-(1))).val.tt_ as i32 & 0xf as i32;
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn lua_getglobal(
    mut L: *mut lua_State,
    mut name: *const std::ffi::c_char,
) -> i32 {
    let mut G: *const TValue = 0 as *const TValue;
    G = &mut *((*(&mut (*((*(*L).l_G).l_registry.value_.gc as *mut GCUnion)).h as *mut Table))
        .array)
        .offset((2 as i32 - 1 as i32) as isize) as *mut TValue;
    return auxgetstr(L, G, name);
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn lua_gettable(mut L: *mut lua_State, mut idx: i32) -> i32 {
    let mut slot: *const TValue = 0 as *const TValue;
    let mut t: *mut TValue = 0 as *mut TValue;
    t = index2value(L, idx);
    if if !((*t).tt_ as i32 == 5 as i32 | (0) << 4 as i32 | (1 as i32) << 6 as i32) {
        slot = 0 as *const TValue;
        0
    } else {
        slot = luaH_get(
            &mut (*((*t).value_.gc as *mut GCUnion)).h,
            &mut (*((*L).top.p).offset(-(1))).val,
        );
        !((*slot).tt_ as i32 & 0xf as i32 == 0) as i32
    } != 0
    {
        let mut io1: *mut TValue = &mut (*((*L).top.p).offset(-(1))).val;
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
    } else {
        luaV_finishget(
            L,
            t,
            &mut (*((*L).top.p).offset(-(1))).val,
            ((*L).top.p).offset(-(1)),
            slot,
        );
    }
    return (*((*L).top.p).offset(-(1))).val.tt_ as i32 & 0xf as i32;
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn lua_getfield(
    mut L: *mut lua_State,
    mut idx: i32,
    mut k: *const std::ffi::c_char,
) -> i32 {
    return auxgetstr(L, index2value(L, idx), k);
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn lua_geti(
    mut L: *mut lua_State,
    mut idx: i32,
    mut n: lua_Integer,
) -> i32 {
    let mut t: *mut TValue = 0 as *mut TValue;
    let mut slot: *const TValue = 0 as *const TValue;
    t = index2value(L, idx);
    if if !((*t).tt_ as i32 == 5 as i32 | (0) << 4 as i32 | (1 as i32) << 6 as i32) {
        slot = 0 as *const TValue;
        0
    } else {
        slot = (if (n as lua_Unsigned).wrapping_sub(1 as u32 as lua_Unsigned)
            < (*(&mut (*((*t).value_.gc as *mut GCUnion)).h as *mut Table)).alimit as lua_Unsigned
        {
            &mut *((*(&mut (*((*t).value_.gc as *mut GCUnion)).h as *mut Table)).array)
                .offset((n - 1 as i32 as lua_Integer) as isize) as *mut TValue
                as *const TValue
        } else {
            luaH_getint(&mut (*((*t).value_.gc as *mut GCUnion)).h, n)
        });
        !((*slot).tt_ as i32 & 0xf as i32 == 0) as i32
    } != 0
    {
        let mut io1: *mut TValue = &mut (*(*L).top.p).val;
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
    } else {
        let mut aux: TValue = TValue {
            value_: Value {
                gc: 0 as *mut GCObject,
            },
            tt_: 0,
        };
        let mut io: *mut TValue = &mut aux;
        (*io).value_.i = n;
        (*io).tt_ = (3 as i32 | (0) << 4 as i32) as lu_byte;
        luaV_finishget(L, t, &mut aux, (*L).top.p, slot);
    }
    (*L).top.p = ((*L).top.p).offset(1);
    (*L).top.p;
    return (*((*L).top.p).offset(-(1))).val.tt_ as i32 & 0xf as i32;
}
#[inline]
unsafe extern "C-unwind" fn finishrawget(mut L: *mut lua_State, mut val: *const TValue) -> i32 {
    if (*val).tt_ as i32 & 0xf as i32 == 0 {
        (*(*L).top.p).val.tt_ = (0 | (0) << 4 as i32) as lu_byte;
    } else {
        let mut io1: *mut TValue = &mut (*(*L).top.p).val;
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
    }
    (*L).top.p = ((*L).top.p).offset(1);
    (*L).top.p;
    return (*((*L).top.p).offset(-(1))).val.tt_ as i32 & 0xf as i32;
}
unsafe extern "C-unwind" fn gettable(mut L: *mut lua_State, mut idx: i32) -> *mut Table {
    let mut t: *mut TValue = index2value(L, idx);
    return &mut (*((*t).value_.gc as *mut GCUnion)).h;
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn lua_rawget(mut L: *mut lua_State, mut idx: i32) -> i32 {
    let mut t: *mut Table = 0 as *mut Table;
    let mut val: *const TValue = 0 as *const TValue;
    t = gettable(L, idx);
    val = luaH_get(t, &mut (*((*L).top.p).offset(-(1))).val);
    (*L).top.p = ((*L).top.p).offset(-1);
    (*L).top.p;
    return finishrawget(L, val);
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn lua_rawgeti(
    mut L: *mut lua_State,
    mut idx: i32,
    mut n: lua_Integer,
) -> i32 {
    let mut t: *mut Table = 0 as *mut Table;
    t = gettable(L, idx);
    return finishrawget(L, luaH_getint(t, n));
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn lua_rawgetp(
    mut L: *mut lua_State,
    mut idx: i32,
    mut p: *const c_void,
) -> i32 {
    let mut t: *mut Table = 0 as *mut Table;
    let mut k: TValue = TValue {
        value_: Value {
            gc: 0 as *mut GCObject,
        },
        tt_: 0,
    };
    t = gettable(L, idx);
    let mut io: *mut TValue = &mut k;
    (*io).value_.p = p as *mut c_void;
    (*io).tt_ = (2 as i32 | (0) << 4 as i32) as lu_byte;
    return finishrawget(L, luaH_get(t, &mut k));
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn lua_createtable(
    mut L: *mut lua_State,
    mut narray: i32,
    mut nrec: i32,
) {
    let mut t: *mut Table = 0 as *mut Table;
    t = luaH_new(L);
    let mut io: *mut TValue = &mut (*(*L).top.p).val;
    let mut x_: *mut Table = t;
    (*io).value_.gc = &mut (*(x_ as *mut GCUnion)).gc;
    (*io).tt_ = (5 as i32 | (0) << 4 as i32 | (1 as i32) << 6 as i32) as lu_byte;
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
    if narray > 0 || nrec > 0 {
        luaH_resize(L, t, narray as u32, nrec as u32);
    }
    if (*(*L).l_G).GCdebt > 0 as l_mem {
        luaC_step(L);
    }
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn lua_getmetatable(mut L: *mut lua_State, mut objindex: i32) -> i32 {
    let mut obj: *const TValue = 0 as *const TValue;
    let mut mt: *mut Table = 0 as *mut Table;
    let mut res: i32 = 0;
    obj = index2value(L, objindex);
    match (*obj).tt_ as i32 & 0xf as i32 {
        5 => {
            mt = (*&mut (*((*obj).value_.gc as *mut GCUnion)).h).metatable;
        }
        7 => {
            mt = (*&mut (*((*obj).value_.gc as *mut GCUnion)).u).metatable;
        }
        _ => {
            mt = (*(*L).l_G).mt[((*obj).tt_ as i32 & 0xf as i32) as usize];
        }
    }
    if !mt.is_null() {
        let mut io: *mut TValue = &mut (*(*L).top.p).val;
        let mut x_: *mut Table = mt;
        (*io).value_.gc = &mut (*(x_ as *mut GCUnion)).gc;
        (*io).tt_ = (5 as i32 | (0) << 4 as i32 | (1 as i32) << 6 as i32) as lu_byte;
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
        res = 1 as i32;
    }
    return res;
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn lua_getiuservalue(
    mut L: *mut lua_State,
    mut idx: i32,
    mut n: i32,
) -> i32 {
    let mut o: *mut TValue = 0 as *mut TValue;
    let mut t: i32 = 0;
    o = index2value(L, idx);
    if n <= 0 || n > (*(&mut (*((*o).value_.gc as *mut GCUnion)).u as *mut Udata)).nuvalue as i32 {
        (*(*L).top.p).val.tt_ = (0 | (0) << 4 as i32) as lu_byte;
        t = -(1 as i32);
    } else {
        let mut io1: *mut TValue = &mut (*(*L).top.p).val;
        let mut io2: *const TValue =
            &mut (*((*(&mut (*((*o).value_.gc as *mut GCUnion)).u as *mut Udata)).uv)
                .as_mut_ptr()
                .offset((n - 1 as i32) as isize))
            .uv;
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
        t = (*(*L).top.p).val.tt_ as i32 & 0xf as i32;
    }
    (*L).top.p = ((*L).top.p).offset(1);
    (*L).top.p;
    return t;
}
unsafe extern "C-unwind" fn auxsetstr(
    mut L: *mut lua_State,
    mut t: *const TValue,
    mut k: *const std::ffi::c_char,
) {
    let mut slot: *const TValue = 0 as *const TValue;
    let mut str: *mut TString = luaS_new(L, k);
    if if !((*t).tt_ as i32 == 5 as i32 | (0) << 4 as i32 | (1 as i32) << 6 as i32) {
        slot = 0 as *const TValue;
        0
    } else {
        slot = luaH_getstr(&mut (*((*t).value_.gc as *mut GCUnion)).h, str);
        !((*slot).tt_ as i32 & 0xf as i32 == 0) as i32
    } != 0
    {
        let mut io1: *mut TValue = slot as *mut TValue;
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
        if (*((*L).top.p).offset(-(1))).val.tt_ as i32 & (1 as i32) << 6 as i32 != 0 {
            if (*(*t).value_.gc).marked as i32 & (1 as i32) << 5 as i32 != 0
                && (*(*((*L).top.p).offset(-(1))).val.value_.gc).marked as i32
                    & ((1 as i32) << 3 as i32 | (1 as i32) << 4 as i32)
                    != 0
            {
                luaC_barrierback_(L, (*t).value_.gc);
            } else {
            };
        } else {
        };
        (*L).top.p = ((*L).top.p).offset(-1);
        (*L).top.p;
    } else {
        let mut io: *mut TValue = &mut (*(*L).top.p).val;
        let mut x_: *mut TString = str;
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
        luaV_finishset(
            L,
            t,
            &mut (*((*L).top.p).offset(-(1))).val,
            &mut (*((*L).top.p).offset(-(2))).val,
            slot,
        );
        (*L).top.p = ((*L).top.p).offset(-(2));
    };
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn lua_setglobal(
    mut L: *mut lua_State,
    mut name: *const std::ffi::c_char,
) {
    let mut G: *const TValue = 0 as *const TValue;
    G = &mut *((*(&mut (*((*(*L).l_G).l_registry.value_.gc as *mut GCUnion)).h as *mut Table))
        .array)
        .offset((2 as i32 - 1 as i32) as isize) as *mut TValue;
    auxsetstr(L, G, name);
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn lua_settable(mut L: *mut lua_State, mut idx: i32) {
    let mut t: *mut TValue = 0 as *mut TValue;
    let mut slot: *const TValue = 0 as *const TValue;
    t = index2value(L, idx);
    if if !((*t).tt_ as i32 == 5 as i32 | (0) << 4 as i32 | (1 as i32) << 6 as i32) {
        slot = 0 as *const TValue;
        0
    } else {
        slot = luaH_get(
            &mut (*((*t).value_.gc as *mut GCUnion)).h,
            &mut (*((*L).top.p).offset(-(2))).val,
        );
        !((*slot).tt_ as i32 & 0xf as i32 == 0) as i32
    } != 0
    {
        let mut io1: *mut TValue = slot as *mut TValue;
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
        if (*((*L).top.p).offset(-(1))).val.tt_ as i32 & (1 as i32) << 6 as i32 != 0 {
            if (*(*t).value_.gc).marked as i32 & (1 as i32) << 5 as i32 != 0
                && (*(*((*L).top.p).offset(-(1))).val.value_.gc).marked as i32
                    & ((1 as i32) << 3 as i32 | (1 as i32) << 4 as i32)
                    != 0
            {
                luaC_barrierback_(L, (*t).value_.gc);
            } else {
            };
        } else {
        };
    } else {
        luaV_finishset(
            L,
            t,
            &mut (*((*L).top.p).offset(-(2))).val,
            &mut (*((*L).top.p).offset(-(1))).val,
            slot,
        );
    }
    (*L).top.p = ((*L).top.p).offset(-(2));
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn lua_setfield(
    mut L: *mut lua_State,
    mut idx: i32,
    mut k: *const std::ffi::c_char,
) {
    auxsetstr(L, index2value(L, idx), k);
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn lua_seti(mut L: *mut lua_State, mut idx: i32, mut n: lua_Integer) {
    let mut t: *mut TValue = 0 as *mut TValue;
    let mut slot: *const TValue = 0 as *const TValue;
    t = index2value(L, idx);
    if if !((*t).tt_ as i32 == 5 as i32 | (0) << 4 as i32 | (1 as i32) << 6 as i32) {
        slot = 0 as *const TValue;
        0
    } else {
        slot = (if (n as lua_Unsigned).wrapping_sub(1 as u32 as lua_Unsigned)
            < (*(&mut (*((*t).value_.gc as *mut GCUnion)).h as *mut Table)).alimit as lua_Unsigned
        {
            &mut *((*(&mut (*((*t).value_.gc as *mut GCUnion)).h as *mut Table)).array)
                .offset((n - 1 as i32 as lua_Integer) as isize) as *mut TValue
                as *const TValue
        } else {
            luaH_getint(&mut (*((*t).value_.gc as *mut GCUnion)).h, n)
        });
        !((*slot).tt_ as i32 & 0xf as i32 == 0) as i32
    } != 0
    {
        let mut io1: *mut TValue = slot as *mut TValue;
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
        if (*((*L).top.p).offset(-(1))).val.tt_ as i32 & (1 as i32) << 6 as i32 != 0 {
            if (*(*t).value_.gc).marked as i32 & (1 as i32) << 5 as i32 != 0
                && (*(*((*L).top.p).offset(-(1))).val.value_.gc).marked as i32
                    & ((1 as i32) << 3 as i32 | (1 as i32) << 4 as i32)
                    != 0
            {
                luaC_barrierback_(L, (*t).value_.gc);
            } else {
            };
        } else {
        };
    } else {
        let mut aux: TValue = TValue {
            value_: Value {
                gc: 0 as *mut GCObject,
            },
            tt_: 0,
        };
        let mut io: *mut TValue = &mut aux;
        (*io).value_.i = n;
        (*io).tt_ = (3 as i32 | (0) << 4 as i32) as lu_byte;
        luaV_finishset(L, t, &mut aux, &mut (*((*L).top.p).offset(-(1))).val, slot);
    }
    (*L).top.p = ((*L).top.p).offset(-1);
    (*L).top.p;
}
unsafe extern "C-unwind" fn aux_rawset(
    mut L: *mut lua_State,
    mut idx: i32,
    mut key: *mut TValue,
    mut n: i32,
) {
    let mut t: *mut Table = 0 as *mut Table;
    t = gettable(L, idx);
    luaH_set(L, t, key, &mut (*((*L).top.p).offset(-(1))).val);
    (*t).flags = ((*t).flags as u32 & !!(!(0 as u32) << TM_EQ as i32 + 1 as i32)) as lu_byte;
    if (*((*L).top.p).offset(-(1))).val.tt_ as i32 & (1 as i32) << 6 as i32 != 0 {
        if (*(&mut (*(t as *mut GCUnion)).gc as *mut GCObject)).marked as i32
            & (1 as i32) << 5 as i32
            != 0
            && (*(*((*L).top.p).offset(-(1))).val.value_.gc).marked as i32
                & ((1 as i32) << 3 as i32 | (1 as i32) << 4 as i32)
                != 0
        {
            luaC_barrierback_(L, &mut (*(t as *mut GCUnion)).gc);
        } else {
        };
    } else {
    };
    (*L).top.p = ((*L).top.p).offset(-(n as isize));
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn lua_rawset(mut L: *mut lua_State, mut idx: i32) {
    aux_rawset(L, idx, &mut (*((*L).top.p).offset(-(2))).val, 2 as i32);
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn lua_rawsetp(
    mut L: *mut lua_State,
    mut idx: i32,
    mut p: *const c_void,
) {
    let mut k: TValue = TValue {
        value_: Value {
            gc: 0 as *mut GCObject,
        },
        tt_: 0,
    };
    let mut io: *mut TValue = &mut k;
    (*io).value_.p = p as *mut c_void;
    (*io).tt_ = (2 as i32 | (0) << 4 as i32) as lu_byte;
    aux_rawset(L, idx, &mut k, 1 as i32);
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn lua_rawseti(
    mut L: *mut lua_State,
    mut idx: i32,
    mut n: lua_Integer,
) {
    let mut t: *mut Table = 0 as *mut Table;
    t = gettable(L, idx);
    luaH_setint(L, t, n, &mut (*((*L).top.p).offset(-(1))).val);
    if (*((*L).top.p).offset(-(1))).val.tt_ as i32 & (1 as i32) << 6 as i32 != 0 {
        if (*(&mut (*(t as *mut GCUnion)).gc as *mut GCObject)).marked as i32
            & (1 as i32) << 5 as i32
            != 0
            && (*(*((*L).top.p).offset(-(1))).val.value_.gc).marked as i32
                & ((1 as i32) << 3 as i32 | (1 as i32) << 4 as i32)
                != 0
        {
            luaC_barrierback_(L, &mut (*(t as *mut GCUnion)).gc);
        } else {
        };
    } else {
    };
    (*L).top.p = ((*L).top.p).offset(-1);
    (*L).top.p;
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn lua_setmetatable(mut L: *mut lua_State, mut objindex: i32) -> i32 {
    let mut obj: *mut TValue = 0 as *mut TValue;
    let mut mt: *mut Table = 0 as *mut Table;
    obj = index2value(L, objindex);
    if (*((*L).top.p).offset(-(1))).val.tt_ as i32 & 0xf as i32 == 0 {
        mt = 0 as *mut Table;
    } else {
        mt = &mut (*((*((*L).top.p).offset(-(1))).val.value_.gc as *mut GCUnion)).h;
    }
    match (*obj).tt_ as i32 & 0xf as i32 {
        5 => {
            let ref mut fresh143 = (*&mut (*((*obj).value_.gc as *mut GCUnion)).h).metatable;
            *fresh143 = mt;
            if !mt.is_null() {
                if (*(*obj).value_.gc).marked as i32 & (1 as i32) << 5 as i32 != 0
                    && (*mt).marked as i32 & ((1 as i32) << 3 as i32 | (1 as i32) << 4 as i32) != 0
                {
                    luaC_barrier_(
                        L,
                        &mut (*((*obj).value_.gc as *mut GCUnion)).gc,
                        &mut (*(mt as *mut GCUnion)).gc,
                    );
                } else {
                };
                luaC_checkfinalizer(L, (*obj).value_.gc, mt);
            }
        }
        7 => {
            let ref mut fresh144 = (*&mut (*((*obj).value_.gc as *mut GCUnion)).u).metatable;
            *fresh144 = mt;
            if !mt.is_null() {
                if (*(&mut (*((*obj).value_.gc as *mut GCUnion)).u as *mut Udata)).marked as i32
                    & (1 as i32) << 5 as i32
                    != 0
                    && (*mt).marked as i32 & ((1 as i32) << 3 as i32 | (1 as i32) << 4 as i32) != 0
                {
                    luaC_barrier_(
                        L,
                        &mut (*(&mut (*((*obj).value_.gc as *mut GCUnion)).u as *mut Udata
                            as *mut GCUnion))
                            .gc,
                        &mut (*(mt as *mut GCUnion)).gc,
                    );
                } else {
                };
                luaC_checkfinalizer(L, (*obj).value_.gc, mt);
            }
        }
        _ => {
            (*(*L).l_G).mt[((*obj).tt_ as i32 & 0xf as i32) as usize] = mt;
        }
    }
    (*L).top.p = ((*L).top.p).offset(-1);
    (*L).top.p;
    return 1 as i32;
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn lua_setiuservalue(
    mut L: *mut lua_State,
    mut idx: i32,
    mut n: i32,
) -> i32 {
    let mut o: *mut TValue = 0 as *mut TValue;
    let mut res: i32 = 0;
    o = index2value(L, idx);
    if !((n as u32).wrapping_sub(1 as u32)
        < (*(&mut (*((*o).value_.gc as *mut GCUnion)).u as *mut Udata)).nuvalue as u32)
    {
        res = 0;
    } else {
        let mut io1: *mut TValue =
            &mut (*((*(&mut (*((*o).value_.gc as *mut GCUnion)).u as *mut Udata)).uv)
                .as_mut_ptr()
                .offset((n - 1 as i32) as isize))
            .uv;
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
        if (*((*L).top.p).offset(-(1))).val.tt_ as i32 & (1 as i32) << 6 as i32 != 0 {
            if (*(*o).value_.gc).marked as i32 & (1 as i32) << 5 as i32 != 0
                && (*(*((*L).top.p).offset(-(1))).val.value_.gc).marked as i32
                    & ((1 as i32) << 3 as i32 | (1 as i32) << 4 as i32)
                    != 0
            {
                luaC_barrierback_(L, (*o).value_.gc);
            } else {
            };
        } else {
        };
        res = 1 as i32;
    }
    (*L).top.p = ((*L).top.p).offset(-1);
    (*L).top.p;
    return res;
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn lua_callk(
    mut L: *mut lua_State,
    mut nargs: i32,
    mut nresults: i32,
    mut ctx: lua_KContext,
    mut k: lua_KFunction,
) {
    let mut func: StkId = 0 as *mut StackValue;
    func = ((*L).top.p).offset(-((nargs + 1 as i32) as isize));
    if k.is_some() && (*L).nCcalls & 0xffff0000 as u32 == 0 as u32 {
        (*(*L).ci).u.c.k = k;
        (*(*L).ci).u.c.ctx = ctx;
        luaD_call(L, func, nresults);
    } else {
        luaD_callnoyield(L, func, nresults);
    }
    if nresults <= -(1 as i32) && (*(*L).ci).top.p < (*L).top.p {
        (*(*L).ci).top.p = (*L).top.p;
    }
}
unsafe extern "C-unwind" fn f_call(mut L: *mut lua_State, mut ud: *mut c_void) {
    let mut c: *mut CallS = ud as *mut CallS;
    luaD_callnoyield(L, (*c).func, (*c).nresults);
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn lua_pcallk(
    mut L: *mut lua_State,
    mut nargs: i32,
    mut nresults: i32,
    mut errfunc: i32,
    mut ctx: lua_KContext,
    mut k: lua_KFunction,
) -> i32 {
    let mut c: CallS = CallS {
        func: 0 as *mut StackValue,
        nresults: 0,
    };
    let mut status: i32 = 0;
    let mut func: ptrdiff_t = 0;
    if errfunc == 0 {
        func = 0 as ptrdiff_t;
    } else {
        let mut o: StkId = index2stack(L, errfunc);
        func = (o as *mut std::ffi::c_char).offset_from((*L).stack.p as *mut std::ffi::c_char);
    }
    c.func = ((*L).top.p).offset(-((nargs + 1 as i32) as isize));
    if k.is_none() || !((*L).nCcalls & 0xffff0000 as u32 == 0 as u32) {
        c.nresults = nresults;
        status = luaD_pcall(
            L,
            Some(f_call as unsafe extern "C-unwind" fn(*mut lua_State, *mut c_void) -> ()),
            &mut c as *mut CallS as *mut c_void,
            (c.func as *mut std::ffi::c_char).offset_from((*L).stack.p as *mut std::ffi::c_char),
            func,
        );
    } else {
        let mut ci: *mut CallInfo = (*L).ci;
        (*ci).u.c.k = k;
        (*ci).u.c.ctx = ctx;
        (*ci).u2.funcidx = (c.func as *mut std::ffi::c_char)
            .offset_from((*L).stack.p as *mut std::ffi::c_char)
            as std::ffi::c_long as i32;
        (*ci).u.c.old_errfunc = (*L).errfunc;
        (*L).errfunc = func;
        (*ci).callstatus =
            ((*ci).callstatus as i32 & !((1 as i32) << 0) | (*L).allowhook as i32) as u16;
        (*ci).callstatus = ((*ci).callstatus as i32 | (1 as i32) << 4 as i32) as u16;
        luaD_call(L, c.func, nresults);
        (*ci).callstatus = ((*ci).callstatus as i32 & !((1 as i32) << 4 as i32)) as u16;
        (*L).errfunc = (*ci).u.c.old_errfunc;
        status = 0;
    }
    if nresults <= -(1 as i32) && (*(*L).ci).top.p < (*L).top.p {
        (*(*L).ci).top.p = (*L).top.p;
    }
    return status;
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn lua_load(
    mut L: *mut lua_State,
    mut reader: lua_Reader,
    mut data: *mut c_void,
    mut chunkname: *const std::ffi::c_char,
    mut mode: *const std::ffi::c_char,
) -> i32 {
    let mut z: ZIO = Zio {
        n: 0,
        p: 0 as *const std::ffi::c_char,
        reader: None,
        data: 0 as *mut c_void,
        L: 0 as *mut lua_State,
    };
    let mut status: i32 = 0;
    if chunkname.is_null() {
        chunkname = c"?".as_ptr();
    }
    luaZ_init(L, &mut z, reader, data);
    status = luaD_protectedparser(L, &mut z, chunkname, mode);
    if status == 0 {
        let mut f: *mut LClosure = &mut (*((*((*L).top.p).offset(-(1))).val.value_.gc
            as *mut GCUnion))
            .cl
            .l;
        if (*f).nupvalues as i32 >= 1 as i32 {
            let mut gt: *const TValue =
                &mut *((*(&mut (*((*(*L).l_G).l_registry.value_.gc as *mut GCUnion)).h
                    as *mut Table))
                    .array)
                    .offset((2 as i32 - 1 as i32) as isize) as *mut TValue;
            let mut io1: *mut TValue = (**((*f).upvals).as_mut_ptr().offset(0 as isize)).v.p;
            let mut io2: *const TValue = gt;
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
            if (*gt).tt_ as i32 & (1 as i32) << 6 as i32 != 0 {
                if (**((*f).upvals).as_mut_ptr().offset(0 as isize)).marked as i32
                    & (1 as i32) << 5 as i32
                    != 0
                    && (*(*gt).value_.gc).marked as i32
                        & ((1 as i32) << 3 as i32 | (1 as i32) << 4 as i32)
                        != 0
                {
                    luaC_barrier_(
                        L,
                        &mut (*(*((*f).upvals).as_mut_ptr().offset(0 as isize) as *mut GCUnion)).gc,
                        &mut (*((*gt).value_.gc as *mut GCUnion)).gc,
                    );
                } else {
                };
            } else {
            };
        }
    }
    return status;
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn lua_dump(
    mut L: *mut lua_State,
    mut writer_0: lua_Writer,
    mut data: *mut c_void,
    mut strip: i32,
) -> i32 {
    let mut status: i32 = 0;
    let mut o: *mut TValue = 0 as *mut TValue;
    o = &mut (*((*L).top.p).offset(-(1))).val;
    if (*o).tt_ as i32 == 6 as i32 | (0) << 4 as i32 | (1 as i32) << 6 as i32 {
        status = luaU_dump(
            L,
            (*&mut (*((*o).value_.gc as *mut GCUnion)).cl.l).p,
            writer_0,
            data,
            strip,
        );
    } else {
        status = 1 as i32;
    }
    return status;
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn lua_status(mut L: *mut lua_State) -> i32 {
    return (*L).status as i32;
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn lua_gc(mut L: *mut lua_State, mut what: i32, mut args: ...) -> i32 {
    let mut argp: ::core::ffi::VaListImpl;
    let mut res: i32 = 0;
    let mut g: *mut global_State = (*L).l_G;
    if (*g).gcstp as i32 & 2 as i32 != 0 {
        return -(1 as i32);
    }
    argp = args.clone();
    match what {
        0 => {
            (*g).gcstp = 1 as i32 as lu_byte;
        }
        1 => {
            luaE_setdebt(g, 0 as l_mem);
            (*g).gcstp = 0 as lu_byte;
        }
        2 => {
            luaC_fullgc(L, 0);
        }
        3 => {
            res = (((*g).totalbytes + (*g).GCdebt) as lu_mem >> 10) as i32;
        }
        4 => {
            res = (((*g).totalbytes + (*g).GCdebt) as lu_mem & 0x3ff as i32 as lu_mem) as i32;
        }
        5 => {
            let mut data: i32 = argp.arg::<i32>();
            let mut debt: l_mem = 1 as i32 as l_mem;
            let mut oldstp: lu_byte = (*g).gcstp;
            (*g).gcstp = 0 as lu_byte;
            if data == 0 {
                luaE_setdebt(g, 0 as l_mem);
                luaC_step(L);
            } else {
                debt = data as l_mem * 1024 as i32 as l_mem + (*g).GCdebt;
                luaE_setdebt(g, debt);
                if (*(*L).l_G).GCdebt > 0 as l_mem {
                    luaC_step(L);
                }
            }
            (*g).gcstp = oldstp;
            if debt > 0 as l_mem && (*g).gcstate as i32 == 8 as i32 {
                res = 1 as i32;
            }
        }
        6 => {
            let mut data_0: i32 = argp.arg::<i32>();
            res = (*g).gcpause as i32 * 4 as i32;
            (*g).gcpause = (data_0 / 4 as i32) as lu_byte;
        }
        7 => {
            let mut data_1: i32 = argp.arg::<i32>();
            res = (*g).gcstepmul as i32 * 4 as i32;
            (*g).gcstepmul = (data_1 / 4 as i32) as lu_byte;
        }
        9 => {
            res = ((*g).gcstp as i32 == 0) as i32;
        }
        10 => {
            let mut minormul: i32 = argp.arg::<i32>();
            let mut majormul: i32 = argp.arg::<i32>();
            res = if (*g).gckind as i32 == 1 as i32 || (*g).lastatomic != 0 as lu_mem {
                10
            } else {
                11 as i32
            };
            if minormul != 0 {
                (*g).genminormul = minormul as lu_byte;
            }
            if majormul != 0 {
                (*g).genmajormul = (majormul / 4 as i32) as lu_byte;
            }
            luaC_changemode(L, 1 as i32);
        }
        11 => {
            let mut pause: i32 = argp.arg::<i32>();
            let mut stepmul: i32 = argp.arg::<i32>();
            let mut stepsize: i32 = argp.arg::<i32>();
            res = if (*g).gckind as i32 == 1 as i32 || (*g).lastatomic != 0 as lu_mem {
                10
            } else {
                11 as i32
            };
            if pause != 0 {
                (*g).gcpause = (pause / 4 as i32) as lu_byte;
            }
            if stepmul != 0 {
                (*g).gcstepmul = (stepmul / 4 as i32) as lu_byte;
            }
            if stepsize != 0 {
                (*g).gcstepsize = stepsize as lu_byte;
            }
            luaC_changemode(L, 0);
        }
        _ => {
            res = -(1 as i32);
        }
    }
    return res;
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn lua_error(mut L: *mut lua_State) -> i32 {
    let mut errobj: *mut TValue = 0 as *mut TValue;
    errobj = &mut (*((*L).top.p).offset(-(1))).val;
    if (*errobj).tt_ as i32 == 4 as i32 | (0) << 4 as i32 | (1 as i32) << 6 as i32
        && &mut (*((*errobj).value_.gc as *mut GCUnion)).ts as *mut TString == (*(*L).l_G).memerrmsg
    {
        luaD_throw(L, 4 as i32);
    } else {
        luaG_errormsg(L);
    };
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn lua_next(mut L: *mut lua_State, mut idx: i32) -> i32 {
    let mut t: *mut Table = 0 as *mut Table;
    let mut more: i32 = 0;
    t = gettable(L, idx);
    more = luaH_next(L, t, ((*L).top.p).offset(-(1)));
    if more != 0 {
        (*L).top.p = ((*L).top.p).offset(1);
        (*L).top.p;
    } else {
        (*L).top.p = ((*L).top.p).offset(-(1));
    }
    return more;
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn lua_toclose(mut L: *mut lua_State, mut idx: i32) {
    let mut nresults: i32 = 0;
    let mut o: StkId = 0 as *mut StackValue;
    o = index2stack(L, idx);
    nresults = (*(*L).ci).nresults as i32;
    luaF_newtbcupval(L, o);
    if !(nresults < -(1 as i32)) {
        (*(*L).ci).nresults = (-nresults - 3 as i32) as i16;
    }
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn lua_concat(mut L: *mut lua_State, mut n: i32) {
    if n > 0 {
        luaV_concat(L, n);
    } else {
        let mut io: *mut TValue = &mut (*(*L).top.p).val;
        let mut x_: *mut TString = luaS_newlstr(L, c"".as_ptr(), 0 as size_t);
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
    }
    if (*(*L).l_G).GCdebt > 0 as l_mem {
        luaC_step(L);
    }
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn lua_len(mut L: *mut lua_State, mut idx: i32) {
    let mut t: *mut TValue = 0 as *mut TValue;
    t = index2value(L, idx);
    luaV_objlen(L, (*L).top.p, t);
    (*L).top.p = ((*L).top.p).offset(1);
    (*L).top.p;
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn lua_getallocf(
    mut L: *mut lua_State,
    mut ud: *mut *mut c_void,
) -> lua_Alloc {
    let mut f: lua_Alloc = None;
    if !ud.is_null() {
        *ud = (*(*L).l_G).ud;
    }
    f = (*(*L).l_G).frealloc;
    return f;
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn lua_setallocf(
    mut L: *mut lua_State,
    mut f: lua_Alloc,
    mut ud: *mut c_void,
) {
    (*(*L).l_G).ud = ud;
    (*(*L).l_G).frealloc = f;
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn lua_setwarnf(
    mut L: *mut lua_State,
    mut f: lua_WarnFunction,
    mut ud: *mut c_void,
) {
    (*(*L).l_G).ud_warn = ud;
    (*(*L).l_G).warnf = f;
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn lua_warning(
    mut L: *mut lua_State,
    mut msg: *const std::ffi::c_char,
    mut tocont: i32,
) {
    luaE_warning(L, msg, tocont);
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn lua_newuserdatauv(
    mut L: *mut lua_State,
    mut size: size_t,
    mut nuvalue: i32,
) -> *mut c_void {
    let mut u: *mut Udata = 0 as *mut Udata;
    u = luaS_newudata(L, size, nuvalue);
    let mut io: *mut TValue = &mut (*(*L).top.p).val;
    let mut x_: *mut Udata = u;
    (*io).value_.gc = &mut (*(x_ as *mut GCUnion)).gc;
    (*io).tt_ = (7 as i32 | (0) << 4 as i32 | (1 as i32) << 6 as i32) as lu_byte;
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
    if (*(*L).l_G).GCdebt > 0 as l_mem {
        luaC_step(L);
    }
    return (u as *mut std::ffi::c_char).offset(
        (if (*u).nuvalue as i32 == 0 {
            32 as usize
        } else {
            (40 as usize)
                .wrapping_add((size_of::<UValue>() as usize).wrapping_mul((*u).nuvalue as usize))
        }) as isize,
    ) as *mut c_void;
}
unsafe extern "C-unwind" fn aux_upvalue(
    mut fi: *mut TValue,
    mut n: i32,
    mut val: *mut *mut TValue,
    mut owner: *mut *mut GCObject,
) -> *const std::ffi::c_char {
    match (*fi).tt_ as i32 & 0x3f as i32 {
        38 => {
            let mut f: *mut CClosure = &mut (*((*fi).value_.gc as *mut GCUnion)).cl.c;
            if !((n as u32).wrapping_sub(1 as u32) < (*f).nupvalues as u32) {
                return 0 as *const std::ffi::c_char;
            }
            *val = &mut *((*f).upvalue).as_mut_ptr().offset((n - 1 as i32) as isize) as *mut TValue;
            if !owner.is_null() {
                *owner = &mut (*(f as *mut GCUnion)).gc;
            }
            return c"".as_ptr();
        }
        6 => {
            let mut f_0: *mut LClosure = &mut (*((*fi).value_.gc as *mut GCUnion)).cl.l;
            let mut name: *mut TString = 0 as *mut TString;
            let mut p: *mut Proto = (*f_0).p;
            if !((n as u32).wrapping_sub(1 as u32) < (*p).sizeupvalues as u32) {
                return 0 as *const std::ffi::c_char;
            }
            *val = (**((*f_0).upvals).as_mut_ptr().offset((n - 1 as i32) as isize))
                .v
                .p;
            if !owner.is_null() {
                *owner = &mut (*(*((*f_0).upvals).as_mut_ptr().offset((n - 1 as i32) as isize)
                    as *mut GCUnion))
                    .gc;
            }
            name = (*((*p).upvalues).offset((n - 1 as i32) as isize)).name;
            return if name.is_null() {
                c"(no name)".as_ptr()
            } else {
                ((*name).contents).as_mut_ptr() as *const std::ffi::c_char
            };
        }
        _ => return 0 as *const std::ffi::c_char,
    };
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn lua_getupvalue(
    mut L: *mut lua_State,
    mut funcindex: i32,
    mut n: i32,
) -> *const std::ffi::c_char {
    let mut name: *const std::ffi::c_char = 0 as *const std::ffi::c_char;
    let mut val: *mut TValue = 0 as *mut TValue;
    name = aux_upvalue(
        index2value(L, funcindex),
        n,
        &mut val,
        0 as *mut *mut GCObject,
    );
    if !name.is_null() {
        let mut io1: *mut TValue = &mut (*(*L).top.p).val;
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
        (*L).top.p = ((*L).top.p).offset(1);
        (*L).top.p;
    }
    return name;
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn lua_setupvalue(
    mut L: *mut lua_State,
    mut funcindex: i32,
    mut n: i32,
) -> *const std::ffi::c_char {
    let mut name: *const std::ffi::c_char = 0 as *const std::ffi::c_char;
    let mut val: *mut TValue = 0 as *mut TValue;
    let mut owner: *mut GCObject = 0 as *mut GCObject;
    let mut fi: *mut TValue = 0 as *mut TValue;
    fi = index2value(L, funcindex);
    name = aux_upvalue(fi, n, &mut val, &mut owner);
    if !name.is_null() {
        (*L).top.p = ((*L).top.p).offset(-1);
        (*L).top.p;
        let mut io1: *mut TValue = val;
        let mut io2: *const TValue = &mut (*(*L).top.p).val;
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
            if (*owner).marked as i32 & (1 as i32) << 5 as i32 != 0
                && (*(*val).value_.gc).marked as i32
                    & ((1 as i32) << 3 as i32 | (1 as i32) << 4 as i32)
                    != 0
            {
                luaC_barrier_(
                    L,
                    &mut (*(owner as *mut GCUnion)).gc,
                    &mut (*((*val).value_.gc as *mut GCUnion)).gc,
                );
            } else {
            };
        } else {
        };
    }
    return name;
}
unsafe extern "C-unwind" fn getupvalref(
    mut L: *mut lua_State,
    mut fidx: i32,
    mut n: i32,
    mut pf: *mut *mut LClosure,
) -> *mut *mut UpVal {
    static mut nullup: *const UpVal = 0 as *const UpVal;
    let mut f: *mut LClosure = 0 as *mut LClosure;
    let mut fi: *mut TValue = index2value(L, fidx);
    f = &mut (*((*fi).value_.gc as *mut GCUnion)).cl.l;
    if !pf.is_null() {
        *pf = f;
    }
    if 1 as i32 <= n && n <= (*(*f).p).sizeupvalues {
        return &mut *((*f).upvals).as_mut_ptr().offset((n - 1 as i32) as isize) as *mut *mut UpVal;
    } else {
        return &raw const nullup as *const *const UpVal as *mut *mut UpVal;
    };
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn lua_upvalueid(
    mut L: *mut lua_State,
    mut fidx: i32,
    mut n: i32,
) -> *mut c_void {
    let mut fi: *mut TValue = index2value(L, fidx);
    match (*fi).tt_ as i32 & 0x3f as i32 {
        6 => {
            return *getupvalref(L, fidx, n, 0 as *mut *mut LClosure) as *mut c_void;
        }
        38 => {
            let mut f: *mut CClosure = &mut (*((*fi).value_.gc as *mut GCUnion)).cl.c;
            if 1 as i32 <= n && n <= (*f).nupvalues as i32 {
                return &mut *((*f).upvalue).as_mut_ptr().offset((n - 1 as i32) as isize)
                    as *mut TValue as *mut c_void;
            }
        }
        22 => {}
        _ => return 0 as *mut c_void,
    }
    return 0 as *mut c_void;
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn lua_upvaluejoin(
    mut L: *mut lua_State,
    mut fidx1: i32,
    mut n1: i32,
    mut fidx2: i32,
    mut n2: i32,
) {
    let mut f1: *mut LClosure = 0 as *mut LClosure;
    let mut up1: *mut *mut UpVal = getupvalref(L, fidx1, n1, &mut f1);
    let mut up2: *mut *mut UpVal = getupvalref(L, fidx2, n2, 0 as *mut *mut LClosure);
    *up1 = *up2;
    if (*f1).marked as i32 & (1 as i32) << 5 as i32 != 0
        && (**up1).marked as i32 & ((1 as i32) << 3 as i32 | (1 as i32) << 4 as i32) != 0
    {
        luaC_barrier_(
            L,
            &mut (*(f1 as *mut GCUnion)).gc,
            &mut (*(*up1 as *mut GCUnion)).gc,
        );
    } else {
    };
}
unsafe extern "C-unwind" fn findfield(
    mut L: *mut lua_State,
    mut objidx: i32,
    mut level: i32,
) -> i32 {
    if level == 0 || !(lua_type(L, -(1 as i32)) == 5 as i32) {
        return 0;
    }
    lua_pushnil(L);
    while lua_next(L, -(2 as i32)) != 0 {
        if lua_type(L, -(2 as i32)) == 4 as i32 {
            if lua_rawequal(L, objidx, -(1 as i32)) != 0 {
                lua_settop(L, -(1 as i32) - 1 as i32);
                return 1 as i32;
            } else if findfield(L, objidx, level - 1 as i32) != 0 {
                lua_pushstring(L, c".".as_ptr());
                lua_copy(L, -(1 as i32), -(3 as i32));
                lua_settop(L, -(1 as i32) - 1 as i32);
                lua_concat(L, 3 as i32);
                return 1 as i32;
            }
        }
        lua_settop(L, -(1 as i32) - 1 as i32);
    }
    return 0;
}
unsafe extern "C-unwind" fn pushglobalfuncname(
    mut L: *mut lua_State,
    mut ar: *mut lua_Debug,
) -> i32 {
    let mut top: i32 = lua_gettop(L);
    lua_getinfo(L, c"f".as_ptr(), ar);
    lua_getfield(L, -(1000000) - 1000, c"_LOADED".as_ptr());
    luaL_checkstack(L, 6 as i32, c"not enough stack".as_ptr());
    if findfield(L, top + 1 as i32, 2 as i32) != 0 {
        let mut name: *const std::ffi::c_char = lua_tolstring(L, -(1 as i32), 0 as *mut size_t);
        if strncmp(name, c"_G.".as_ptr(), 3) == 0 {
            lua_pushstring(L, name.offset(3));
            lua_rotate(L, -(2 as i32), -(1 as i32));
            lua_settop(L, -(1 as i32) - 1 as i32);
        }
        lua_copy(L, -(1 as i32), top + 1 as i32);
        lua_settop(L, top + 1 as i32);
        return 1 as i32;
    } else {
        lua_settop(L, top);
        return 0;
    };
}
unsafe extern "C-unwind" fn pushfuncname(mut L: *mut lua_State, mut ar: *mut lua_Debug) {
    if pushglobalfuncname(L, ar) != 0 {
        lua_pushfstring(
            L,
            c"function '%s'".as_ptr(),
            lua_tolstring(L, -(1 as i32), 0 as *mut size_t),
        );
        lua_rotate(L, -(2 as i32), -(1 as i32));
        lua_settop(L, -(1 as i32) - 1 as i32);
    } else if *(*ar).namewhat as i32 != '\0' as i32 {
        lua_pushfstring(L, c"%s '%s'".as_ptr(), (*ar).namewhat, (*ar).name);
    } else if *(*ar).what as i32 == 'm' as i32 {
        lua_pushstring(L, c"main chunk".as_ptr());
    } else if *(*ar).what as i32 != 'C' as i32 {
        lua_pushfstring(
            L,
            c"function <%s:%d>".as_ptr(),
            ((*ar).short_src).as_mut_ptr(),
            (*ar).linedefined,
        );
    } else {
        lua_pushstring(L, c"?".as_ptr());
    };
}
unsafe extern "C-unwind" fn lastlevel(mut L: *mut lua_State) -> i32 {
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
    let mut li: i32 = 1 as i32;
    let mut le: i32 = 1 as i32;
    while lua_getstack(L, le, &mut ar) != 0 {
        li = le;
        le *= 2 as i32;
    }
    while li < le {
        let mut m: i32 = (li + le) / 2 as i32;
        if lua_getstack(L, m, &mut ar) != 0 {
            li = m + 1 as i32;
        } else {
            le = m;
        }
    }
    return le - 1 as i32;
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaL_traceback(
    mut L: *mut lua_State,
    mut L1: *mut lua_State,
    mut msg: *const std::ffi::c_char,
    mut level: i32,
) {
    let mut b: luaL_Buffer = luaL_Buffer {
        b: 0 as *mut std::ffi::c_char,
        size: 0,
        n: 0,
        L: 0 as *mut lua_State,
        init: C2RustUnnamed_15 { n: 0. },
    };
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
    let mut last: i32 = lastlevel(L1);
    let mut limit2show: i32 = if last - level > 10 + 11 as i32 {
        10
    } else {
        -(1 as i32)
    };
    luaL_buffinit(L, &mut b);
    if !msg.is_null() {
        luaL_addstring(&mut b, msg);
        (b.n < b.size || !(luaL_prepbuffsize(&mut b, 1 as i32 as size_t)).is_null()) as i32;
        let fresh145 = b.n;
        b.n = (b.n).wrapping_add(1);
        *(b.b).offset(fresh145 as isize) = '\n' as i32 as std::ffi::c_char;
    }
    luaL_addstring(&mut b, c"stack traceback:".as_ptr());
    loop {
        let fresh146 = level;
        level = level + 1;
        if !(lua_getstack(L1, fresh146, &mut ar) != 0) {
            break;
        }
        let fresh147 = limit2show;
        limit2show = limit2show - 1;
        if fresh147 == 0 {
            let mut n: i32 = last - level - 11 as i32 + 1 as i32;
            lua_pushfstring(L, c"\n\t...\t(skipping %d levels)".as_ptr(), n);
            luaL_addvalue(&mut b);
            level += n;
        } else {
            lua_getinfo(L1, c"Slnt".as_ptr(), &mut ar);
            if ar.currentline <= 0 {
                lua_pushfstring(L, c"\n\t%s: in ".as_ptr(), (ar.short_src).as_mut_ptr());
            } else {
                lua_pushfstring(
                    L,
                    c"\n\t%s:%d: in ".as_ptr(),
                    (ar.short_src).as_mut_ptr(),
                    ar.currentline,
                );
            }
            luaL_addvalue(&mut b);
            pushfuncname(L, &mut ar);
            luaL_addvalue(&mut b);
            if ar.istailcall != 0 {
                luaL_addstring(&mut b, c"\n\t(...tail calls...)".as_ptr());
            }
        }
    }
    luaL_pushresult(&mut b);
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaL_argerror(
    mut L: *mut lua_State,
    mut arg: i32,
    mut extramsg: *const std::ffi::c_char,
) -> i32 {
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
    if lua_getstack(L, 0, &mut ar) == 0 {
        return luaL_error(L, c"bad argument #%d (%s)".as_ptr(), arg, extramsg);
    }
    lua_getinfo(L, c"n".as_ptr(), &mut ar);
    if strcmp(ar.namewhat, c"method".as_ptr()) == 0 {
        arg -= 1;
        arg;
        if arg == 0 {
            return luaL_error(
                L,
                c"calling '%s' on bad self (%s)".as_ptr(),
                ar.name,
                extramsg,
            );
        }
    }
    if (ar.name).is_null() {
        ar.name = if pushglobalfuncname(L, &mut ar) != 0 {
            lua_tolstring(L, -(1 as i32), 0 as *mut size_t)
        } else {
            c"?".as_ptr()
        };
    }
    return luaL_error(
        L,
        c"bad argument #%d to '%s' (%s)".as_ptr(),
        arg,
        ar.name,
        extramsg,
    );
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaL_typeerror(
    mut L: *mut lua_State,
    mut arg: i32,
    mut tname: *const std::ffi::c_char,
) -> i32 {
    let mut msg: *const std::ffi::c_char = 0 as *const std::ffi::c_char;
    let mut typearg: *const std::ffi::c_char = 0 as *const std::ffi::c_char;
    if luaL_getmetafield(L, arg, c"__name".as_ptr()) == 4 as i32 {
        typearg = lua_tolstring(L, -(1 as i32), 0 as *mut size_t);
    } else if lua_type(L, arg) == 2 as i32 {
        typearg = c"light userdata".as_ptr();
    } else {
        typearg = lua_typename(L, lua_type(L, arg));
    }
    msg = lua_pushfstring(L, c"%s expected, got %s".as_ptr(), tname, typearg);
    return luaL_argerror(L, arg, msg);
}
unsafe extern "C-unwind" fn tag_error(mut L: *mut lua_State, mut arg: i32, mut tag: i32) {
    luaL_typeerror(L, arg, lua_typename(L, tag));
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaL_where(mut L: *mut lua_State, mut level: i32) {
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
    if lua_getstack(L, level, &mut ar) != 0 {
        lua_getinfo(L, c"Sl".as_ptr(), &mut ar);
        if ar.currentline > 0 {
            lua_pushfstring(
                L,
                c"%s:%d: ".as_ptr(),
                (ar.short_src).as_mut_ptr(),
                ar.currentline,
            );
            return;
        }
    }
    lua_pushfstring(L, c"".as_ptr());
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaL_error(
    mut L: *mut lua_State,
    mut fmt: *const std::ffi::c_char,
    mut args: ...
) -> i32 {
    let mut argp: ::core::ffi::VaListImpl;
    argp = args.clone();
    luaL_where(L, 1 as i32);
    lua_pushvfstring(L, fmt, argp.as_va_list());
    lua_concat(L, 2 as i32);
    return lua_error(L);
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaL_fileresult(
    mut L: *mut lua_State,
    mut stat: i32,
    mut fname: *const std::ffi::c_char,
) -> i32 {
    let mut en: i32 = *__errno_location();
    if stat != 0 {
        lua_pushboolean(L, 1 as i32);
        return 1 as i32;
    } else {
        let mut msg: *const std::ffi::c_char = 0 as *const std::ffi::c_char;
        lua_pushnil(L);
        msg = if en != 0 {
            strerror(en) as *const std::ffi::c_char
        } else {
            c"(no extra info)".as_ptr()
        };
        if !fname.is_null() {
            lua_pushfstring(L, c"%s: %s".as_ptr(), fname, msg);
        } else {
            lua_pushstring(L, msg);
        }
        lua_pushinteger(L, en as lua_Integer);
        return 3 as i32;
    };
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaL_execresult(mut L: *mut lua_State, mut stat: i32) -> i32 {
    if stat != 0 && *__errno_location() != 0 {
        return luaL_fileresult(L, 0, 0 as *const std::ffi::c_char);
    } else {
        let mut what: *const std::ffi::c_char = c"exit".as_ptr();
        if stat & 0x7f as i32 == 0 {
            stat = (stat & 0xff00) >> 8 as i32;
        } else if ((stat & 0x7f as i32) + 1 as i32) as std::ffi::c_schar as i32 >> 1 as i32 > 0 {
            stat = stat & 0x7f as i32;
            what = c"signal".as_ptr();
        }
        if *what as i32 == 'e' as i32 && stat == 0 {
            lua_pushboolean(L, 1 as i32);
        } else {
            lua_pushnil(L);
        }
        lua_pushstring(L, what);
        lua_pushinteger(L, stat as lua_Integer);
        return 3 as i32;
    };
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaL_newmetatable(
    mut L: *mut lua_State,
    mut tname: *const std::ffi::c_char,
) -> i32 {
    if lua_getfield(L, -(1000000) - 1000, tname) != 0 {
        return 0;
    }
    lua_settop(L, -(1 as i32) - 1 as i32);
    lua_createtable(L, 0, 2 as i32);
    lua_pushstring(L, tname);
    lua_setfield(L, -(2 as i32), c"__name".as_ptr());
    lua_pushvalue(L, -(1 as i32));
    lua_setfield(L, -(1000000) - 1000, tname);
    return 1 as i32;
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaL_setmetatable(
    mut L: *mut lua_State,
    mut tname: *const std::ffi::c_char,
) {
    lua_getfield(L, -(1000000) - 1000, tname);
    lua_setmetatable(L, -(2 as i32));
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaL_testudata(
    mut L: *mut lua_State,
    mut ud: i32,
    mut tname: *const std::ffi::c_char,
) -> *mut c_void {
    let mut p: *mut c_void = lua_touserdata(L, ud);
    if !p.is_null() {
        if lua_getmetatable(L, ud) != 0 {
            lua_getfield(L, -(1000000) - 1000, tname);
            if lua_rawequal(L, -(1 as i32), -(2 as i32)) == 0 {
                p = 0 as *mut c_void;
            }
            lua_settop(L, -(2 as i32) - 1 as i32);
            return p;
        }
    }
    return 0 as *mut c_void;
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaL_checkudata(
    mut L: *mut lua_State,
    mut ud: i32,
    mut tname: *const std::ffi::c_char,
) -> *mut c_void {
    let mut p: *mut c_void = luaL_testudata(L, ud, tname);
    (((p != 0 as *mut c_void) as i32 != 0) as i32 as std::ffi::c_long != 0
        || luaL_typeerror(L, ud, tname) != 0) as i32;
    return p;
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaL_checkoption(
    mut L: *mut lua_State,
    mut arg: i32,
    mut def: *const std::ffi::c_char,
    mut lst: *const *const std::ffi::c_char,
) -> i32 {
    let mut name: *const std::ffi::c_char = if !def.is_null() {
        luaL_optlstring(L, arg, def, 0 as *mut size_t)
    } else {
        luaL_checklstring(L, arg, 0 as *mut size_t)
    };
    let mut i: i32 = 0;
    i = 0;
    while !(*lst.offset(i as isize)).is_null() {
        if strcmp(*lst.offset(i as isize), name) == 0 {
            return i;
        }
        i += 1;
        i;
    }
    return luaL_argerror(
        L,
        arg,
        lua_pushfstring(L, c"invalid option '%s'".as_ptr(), name),
    );
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaL_checkstack(
    mut L: *mut lua_State,
    mut space: i32,
    mut msg: *const std::ffi::c_char,
) {
    if ((lua_checkstack(L, space) == 0) as i32 != 0) as i32 as std::ffi::c_long != 0 {
        if !msg.is_null() {
            luaL_error(L, c"stack overflow (%s)".as_ptr(), msg);
        } else {
            luaL_error(L, c"stack overflow".as_ptr());
        }
    }
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaL_checktype(mut L: *mut lua_State, mut arg: i32, mut t: i32) {
    if ((lua_type(L, arg) != t) as i32 != 0) as i32 as std::ffi::c_long != 0 {
        tag_error(L, arg, t);
    }
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaL_checkany(mut L: *mut lua_State, mut arg: i32) {
    if ((lua_type(L, arg) == -(1 as i32)) as i32 != 0) as i32 as std::ffi::c_long != 0 {
        luaL_argerror(L, arg, c"value expected".as_ptr());
    }
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaL_checklstring(
    mut L: *mut lua_State,
    mut arg: i32,
    mut len: *mut size_t,
) -> *const std::ffi::c_char {
    let mut s: *const std::ffi::c_char = lua_tolstring(L, arg, len);
    if (s.is_null() as i32 != 0) as i32 as std::ffi::c_long != 0 {
        tag_error(L, arg, 4 as i32);
    }
    return s;
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaL_optlstring(
    mut L: *mut lua_State,
    mut arg: i32,
    mut def: *const std::ffi::c_char,
    mut len: *mut size_t,
) -> *const std::ffi::c_char {
    if lua_type(L, arg) <= 0 {
        if !len.is_null() {
            *len = if !def.is_null() {
                strlen(def)
            } else {
                0 as usize
            };
        }
        return def;
    } else {
        return luaL_checklstring(L, arg, len);
    };
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaL_checknumber(
    mut L: *mut lua_State,
    mut arg: i32,
) -> lua_Number {
    let mut isnum: i32 = 0;
    let mut d: lua_Number = lua_tonumberx(L, arg, &mut isnum);
    if ((isnum == 0) as i32 != 0) as i32 as std::ffi::c_long != 0 {
        tag_error(L, arg, 3 as i32);
    }
    return d;
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaL_optnumber(
    mut L: *mut lua_State,
    mut arg: i32,
    mut def: lua_Number,
) -> lua_Number {
    return if lua_type(L, arg) <= 0 {
        def
    } else {
        luaL_checknumber(L, arg)
    };
}
unsafe extern "C-unwind" fn interror(mut L: *mut lua_State, mut arg: i32) {
    if lua_isnumber(L, arg) != 0 {
        luaL_argerror(L, arg, c"number has no integer representation".as_ptr());
    } else {
        tag_error(L, arg, 3 as i32);
    };
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaL_checkinteger(
    mut L: *mut lua_State,
    mut arg: i32,
) -> lua_Integer {
    let mut isnum: i32 = 0;
    let mut d: lua_Integer = lua_tointegerx(L, arg, &mut isnum);
    if ((isnum == 0) as i32 != 0) as i32 as std::ffi::c_long != 0 {
        interror(L, arg);
    }
    return d;
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaL_optinteger(
    mut L: *mut lua_State,
    mut arg: i32,
    mut def: lua_Integer,
) -> lua_Integer {
    return if lua_type(L, arg) <= 0 {
        def
    } else {
        luaL_checkinteger(L, arg)
    };
}
unsafe extern "C-unwind" fn resizebox(
    mut L: *mut lua_State,
    mut idx: i32,
    mut newsize: size_t,
) -> *mut c_void {
    let mut ud: *mut c_void = 0 as *mut c_void;
    let mut allocf: lua_Alloc = lua_getallocf(L, &mut ud);
    let mut box_0: *mut UBox = lua_touserdata(L, idx) as *mut UBox;
    let mut temp: *mut c_void =
        allocf.expect("non-null function pointer")(ud, (*box_0).box_0, (*box_0).bsize, newsize);
    if ((temp.is_null() && newsize > 0 as size_t) as i32 != 0) as i32 as std::ffi::c_long != 0 {
        lua_pushstring(L, c"not enough memory".as_ptr());
        lua_error(L);
    }
    (*box_0).box_0 = temp;
    (*box_0).bsize = newsize;
    return temp;
}
unsafe extern "C-unwind" fn boxgc(mut L: *mut lua_State) -> i32 {
    resizebox(L, 1 as i32, 0 as size_t);
    return 0;
}
static mut boxmt: [luaL_Reg; 3] = unsafe {
    [
        {
            let mut init = luaL_Reg {
                name: c"__gc".as_ptr(),
                func: Some(boxgc as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"__close".as_ptr(),
                func: Some(boxgc as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: 0 as *const std::ffi::c_char,
                func: None,
            };
            init
        },
    ]
};
unsafe extern "C-unwind" fn newbox(mut L: *mut lua_State) {
    let mut box_0: *mut UBox = lua_newuserdatauv(L, size_of::<UBox>() as usize, 0) as *mut UBox;
    (*box_0).box_0 = 0 as *mut c_void;
    (*box_0).bsize = 0 as size_t;
    if luaL_newmetatable(L, c"_UBOX*".as_ptr()) != 0 {
        luaL_setfuncs(L, (&raw const boxmt).cast(), 0);
    }
    lua_setmetatable(L, -(2 as i32));
}
unsafe extern "C-unwind" fn newbuffsize(mut B: *mut luaL_Buffer, mut sz: size_t) -> size_t {
    let mut newsize: size_t = (*B).size / 2 as i32 as size_t * 3 as i32 as size_t;
    if (((!(0 as size_t)).wrapping_sub(sz) < (*B).n) as i32 != 0) as i32 as std::ffi::c_long != 0 {
        return luaL_error((*B).L, c"buffer too large".as_ptr()) as size_t;
    }
    if newsize < ((*B).n).wrapping_add(sz) {
        newsize = ((*B).n).wrapping_add(sz);
    }
    return newsize;
}
unsafe extern "C-unwind" fn prepbuffsize(
    mut B: *mut luaL_Buffer,
    mut sz: size_t,
    mut boxidx: i32,
) -> *mut std::ffi::c_char {
    if ((*B).size).wrapping_sub((*B).n) >= sz {
        return ((*B).b).offset((*B).n as isize);
    } else {
        let mut L: *mut lua_State = (*B).L;
        let mut newbuff: *mut std::ffi::c_char = 0 as *mut std::ffi::c_char;
        let mut newsize: size_t = newbuffsize(B, sz);
        if (*B).b != ((*B).init.b).as_mut_ptr() {
            newbuff = resizebox(L, boxidx, newsize) as *mut std::ffi::c_char;
        } else {
            lua_rotate(L, boxidx, -(1 as i32));
            lua_settop(L, -(1 as i32) - 1 as i32);
            newbox(L);
            lua_rotate(L, boxidx, 1 as i32);
            lua_toclose(L, boxidx);
            newbuff = resizebox(L, boxidx, newsize) as *mut std::ffi::c_char;
            memcpy(
                newbuff as *mut c_void,
                (*B).b as *const c_void,
                ((*B).n).wrapping_mul(size_of::<std::ffi::c_char>() as usize),
            );
        }
        (*B).b = newbuff;
        (*B).size = newsize;
        return newbuff.offset((*B).n as isize);
    };
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaL_prepbuffsize(
    mut B: *mut luaL_Buffer,
    mut sz: size_t,
) -> *mut std::ffi::c_char {
    return prepbuffsize(B, sz, -(1 as i32));
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaL_addlstring(
    mut B: *mut luaL_Buffer,
    mut s: *const std::ffi::c_char,
    mut l: size_t,
) {
    if l > 0 as size_t {
        let mut b: *mut std::ffi::c_char = prepbuffsize(B, l, -(1 as i32));
        memcpy(
            b as *mut c_void,
            s as *const c_void,
            l.wrapping_mul(size_of::<std::ffi::c_char>() as usize),
        );
        (*B).n = ((*B).n).wrapping_add(l);
    }
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaL_addstring(
    mut B: *mut luaL_Buffer,
    mut s: *const std::ffi::c_char,
) {
    luaL_addlstring(B, s, strlen(s));
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaL_pushresult(mut B: *mut luaL_Buffer) {
    let mut L: *mut lua_State = (*B).L;
    lua_pushlstring(L, (*B).b, (*B).n);
    if (*B).b != ((*B).init.b).as_mut_ptr() {
        lua_closeslot(L, -(2 as i32));
    }
    lua_rotate(L, -(2 as i32), -(1 as i32));
    lua_settop(L, -(1 as i32) - 1 as i32);
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaL_pushresultsize(mut B: *mut luaL_Buffer, mut sz: size_t) {
    (*B).n = ((*B).n).wrapping_add(sz);
    luaL_pushresult(B);
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaL_addvalue(mut B: *mut luaL_Buffer) {
    let mut L: *mut lua_State = (*B).L;
    let mut len: size_t = 0;
    let mut s: *const std::ffi::c_char = lua_tolstring(L, -(1 as i32), &mut len);
    let mut b: *mut std::ffi::c_char = prepbuffsize(B, len, -(2 as i32));
    memcpy(
        b as *mut c_void,
        s as *const c_void,
        len.wrapping_mul(size_of::<std::ffi::c_char>() as usize),
    );
    (*B).n = ((*B).n).wrapping_add(len);
    lua_settop(L, -(1 as i32) - 1 as i32);
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaL_buffinit(mut L: *mut lua_State, mut B: *mut luaL_Buffer) {
    (*B).L = L;
    (*B).b = ((*B).init.b).as_mut_ptr();
    (*B).n = 0 as size_t;
    (*B).size = (16usize)
        .wrapping_mul(size_of::<*mut c_void>() as usize)
        .wrapping_mul(size_of::<lua_Number>() as usize) as i32 as size_t;
    lua_pushlightuserdata(L, B as *mut c_void);
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaL_buffinitsize(
    mut L: *mut lua_State,
    mut B: *mut luaL_Buffer,
    mut sz: size_t,
) -> *mut std::ffi::c_char {
    luaL_buffinit(L, B);
    return prepbuffsize(B, sz, -(1 as i32));
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaL_ref(mut L: *mut lua_State, mut t: i32) -> i32 {
    let mut ref_0: i32 = 0;
    if lua_type(L, -(1 as i32)) == 0 {
        lua_settop(L, -(1 as i32) - 1 as i32);
        return -(1 as i32);
    }
    t = lua_absindex(L, t);
    if lua_rawgeti(L, t, (2 as i32 + 1 as i32) as lua_Integer) == 0 {
        ref_0 = 0;
        lua_pushinteger(L, 0 as lua_Integer);
        lua_rawseti(L, t, (2 as i32 + 1 as i32) as lua_Integer);
    } else {
        ref_0 = lua_tointegerx(L, -(1 as i32), 0 as *mut i32) as i32;
    }
    lua_settop(L, -(1 as i32) - 1 as i32);
    if ref_0 != 0 {
        lua_rawgeti(L, t, ref_0 as lua_Integer);
        lua_rawseti(L, t, (2 as i32 + 1 as i32) as lua_Integer);
    } else {
        ref_0 = lua_rawlen(L, t) as i32 + 1 as i32;
    }
    lua_rawseti(L, t, ref_0 as lua_Integer);
    return ref_0;
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaL_unref(mut L: *mut lua_State, mut t: i32, mut ref_0: i32) {
    if ref_0 >= 0 {
        t = lua_absindex(L, t);
        lua_rawgeti(L, t, (2 as i32 + 1 as i32) as lua_Integer);
        lua_rawseti(L, t, ref_0 as lua_Integer);
        lua_pushinteger(L, ref_0 as lua_Integer);
        lua_rawseti(L, t, (2 as i32 + 1 as i32) as lua_Integer);
    }
}
unsafe extern "C-unwind" fn getF(
    mut L: *mut lua_State,
    mut ud: *mut c_void,
    mut size: *mut size_t,
) -> *const std::ffi::c_char {
    let mut lf: *mut LoadF = ud as *mut LoadF;
    if (*lf).n > 0 {
        *size = (*lf).n as size_t;
        (*lf).n = 0;
    } else {
        if feof((*lf).f) != 0 {
            return 0 as *const std::ffi::c_char;
        }
        *size = fread(
            ((*lf).buff).as_mut_ptr() as *mut c_void,
            1,
            size_of::<[std::ffi::c_char; 8192]>() as usize,
            (*lf).f,
        );
    }
    return ((*lf).buff).as_mut_ptr();
}
unsafe extern "C-unwind" fn errfile(
    mut L: *mut lua_State,
    mut what: *const std::ffi::c_char,
    mut fnameindex: i32,
) -> i32 {
    let mut err: i32 = *__errno_location();
    let mut filename: *const std::ffi::c_char =
        (lua_tolstring(L, fnameindex, 0 as *mut size_t)).offset(1);
    if err != 0 {
        lua_pushfstring(
            L,
            c"cannot %s %s: %s".as_ptr(),
            what,
            filename,
            strerror(err),
        );
    } else {
        lua_pushfstring(L, c"cannot %s %s".as_ptr(), what, filename);
    }
    lua_rotate(L, fnameindex, -(1 as i32));
    lua_settop(L, -(1 as i32) - 1 as i32);
    return 5 as i32 + 1 as i32;
}
unsafe extern "C-unwind" fn skipBOM(mut f: *mut FILE) -> i32 {
    let mut c: i32 = getc(f);
    if c == 0xef as i32 && getc(f) == 0xbb as i32 && getc(f) == 0xbf as i32 {
        return getc(f);
    } else {
        return c;
    };
}
unsafe extern "C-unwind" fn skipcomment(mut f: *mut FILE, mut cp: *mut i32) -> i32 {
    *cp = skipBOM(f);
    let mut c: i32 = *cp;
    if c == '#' as i32 {
        loop {
            c = getc(f);
            if !(c != -(1 as i32) && c != '\n' as i32) {
                break;
            }
        }
        *cp = getc(f);
        return 1 as i32;
    } else {
        return 0;
    };
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaL_loadfilex(
    mut L: *mut lua_State,
    mut filename: *const std::ffi::c_char,
    mut mode: *const std::ffi::c_char,
) -> i32 {
    let mut lf: LoadF = LoadF {
        n: 0,
        f: 0 as *mut FILE,
        buff: [0; 8192],
    };
    let mut status: i32 = 0;
    let mut readstatus: i32 = 0;
    let mut c: i32 = 0;
    let mut fnameindex: i32 = lua_gettop(L) + 1 as i32;
    if filename.is_null() {
        lua_pushstring(L, c"=stdin".as_ptr());
        lf.f = stdin;
    } else {
        lua_pushfstring(L, c"@%s".as_ptr(), filename);
        *__errno_location() = 0;
        lf.f = fopen(filename, c"r".as_ptr());
        if (lf.f).is_null() {
            return errfile(L, c"open".as_ptr(), fnameindex);
        }
    }
    lf.n = 0;
    if skipcomment(lf.f, &mut c) != 0 {
        let fresh148 = lf.n;
        lf.n = lf.n + 1;
        lf.buff[fresh148 as usize] = '\n' as i32 as std::ffi::c_char;
    }
    if c == (*::core::mem::transmute::<&[u8; 5], &[std::ffi::c_char; 5]>(b"\x1BLua\0"))[0 as usize]
        as i32
    {
        lf.n = 0;
        if !filename.is_null() {
            *__errno_location() = 0;
            lf.f = freopen(filename, c"rb".as_ptr(), lf.f);
            if (lf.f).is_null() {
                return errfile(L, c"reopen".as_ptr(), fnameindex);
            }
            skipcomment(lf.f, &mut c);
        }
    }
    if c != -(1 as i32) {
        let fresh149 = lf.n;
        lf.n = lf.n + 1;
        lf.buff[fresh149 as usize] = c as std::ffi::c_char;
    }
    *__errno_location() = 0;
    status = lua_load(
        L,
        Some(
            getF as unsafe extern "C-unwind" fn(
                *mut lua_State,
                *mut c_void,
                *mut size_t,
            ) -> *const std::ffi::c_char,
        ),
        &mut lf as *mut LoadF as *mut c_void,
        lua_tolstring(L, -(1 as i32), 0 as *mut size_t),
        mode,
    );
    readstatus = ferror(lf.f);
    if !filename.is_null() {
        fclose(lf.f);
    }
    if readstatus != 0 {
        lua_settop(L, fnameindex);
        return errfile(L, c"read".as_ptr(), fnameindex);
    }
    lua_rotate(L, fnameindex, -(1 as i32));
    lua_settop(L, -(1 as i32) - 1 as i32);
    return status;
}
unsafe extern "C-unwind" fn getS(
    mut L: *mut lua_State,
    mut ud: *mut c_void,
    mut size: *mut size_t,
) -> *const std::ffi::c_char {
    let mut ls: *mut LoadS = ud as *mut LoadS;
    if (*ls).size == 0 as size_t {
        return 0 as *const std::ffi::c_char;
    }
    *size = (*ls).size;
    (*ls).size = 0 as size_t;
    return (*ls).s;
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaL_loadbufferx(
    mut L: *mut lua_State,
    mut buff: *const std::ffi::c_char,
    mut size: size_t,
    mut name: *const std::ffi::c_char,
    mut mode: *const std::ffi::c_char,
) -> i32 {
    let mut ls: LoadS = LoadS {
        s: 0 as *const std::ffi::c_char,
        size: 0,
    };
    ls.s = buff;
    ls.size = size;
    return lua_load(
        L,
        Some(
            getS as unsafe extern "C-unwind" fn(
                *mut lua_State,
                *mut c_void,
                *mut size_t,
            ) -> *const std::ffi::c_char,
        ),
        &mut ls as *mut LoadS as *mut c_void,
        name,
        mode,
    );
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaL_loadstring(
    mut L: *mut lua_State,
    mut s: *const std::ffi::c_char,
) -> i32 {
    return luaL_loadbufferx(L, s, strlen(s), s, 0 as *const std::ffi::c_char);
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaL_getmetafield(
    mut L: *mut lua_State,
    mut obj: i32,
    mut event: *const std::ffi::c_char,
) -> i32 {
    if lua_getmetatable(L, obj) == 0 {
        return 0;
    } else {
        let mut tt: i32 = 0;
        lua_pushstring(L, event);
        tt = lua_rawget(L, -(2 as i32));
        if tt == 0 {
            lua_settop(L, -(2 as i32) - 1 as i32);
        } else {
            lua_rotate(L, -(2 as i32), -(1 as i32));
            lua_settop(L, -(1 as i32) - 1 as i32);
        }
        return tt;
    };
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaL_callmeta(
    mut L: *mut lua_State,
    mut obj: i32,
    mut event: *const std::ffi::c_char,
) -> i32 {
    obj = lua_absindex(L, obj);
    if luaL_getmetafield(L, obj, event) == 0 {
        return 0;
    }
    lua_pushvalue(L, obj);
    lua_callk(L, 1 as i32, 1 as i32, 0 as lua_KContext, None);
    return 1 as i32;
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaL_len(mut L: *mut lua_State, mut idx: i32) -> lua_Integer {
    let mut l: lua_Integer = 0;
    let mut isnum: i32 = 0;
    lua_len(L, idx);
    l = lua_tointegerx(L, -(1 as i32), &mut isnum);
    if ((isnum == 0) as i32 != 0) as i32 as std::ffi::c_long != 0 {
        luaL_error(L, c"object length is not an integer".as_ptr());
    }
    lua_settop(L, -(1 as i32) - 1 as i32);
    return l;
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaL_tolstring(
    mut L: *mut lua_State,
    mut idx: i32,
    mut len: *mut size_t,
) -> *const std::ffi::c_char {
    idx = lua_absindex(L, idx);
    if luaL_callmeta(L, idx, c"__tostring".as_ptr()) != 0 {
        if lua_isstring(L, -(1 as i32)) == 0 {
            luaL_error(L, c"'__tostring' must return a string".as_ptr());
        }
    } else {
        match lua_type(L, idx) {
            3 => {
                if lua_isinteger(L, idx) != 0 {
                    lua_pushfstring(L, c"%I".as_ptr(), lua_tointegerx(L, idx, 0 as *mut i32));
                } else {
                    lua_pushfstring(L, c"%f".as_ptr(), lua_tonumberx(L, idx, 0 as *mut i32));
                }
            }
            4 => {
                lua_pushvalue(L, idx);
            }
            1 => {
                lua_pushstring(
                    L,
                    if lua_toboolean(L, idx) != 0 {
                        c"true".as_ptr()
                    } else {
                        c"false".as_ptr()
                    },
                );
            }
            0 => {
                lua_pushstring(L, c"nil".as_ptr());
            }
            _ => {
                let mut tt: i32 = luaL_getmetafield(L, idx, c"__name".as_ptr());
                let mut kind: *const std::ffi::c_char = if tt == 4 as i32 {
                    lua_tolstring(L, -(1 as i32), 0 as *mut size_t)
                } else {
                    lua_typename(L, lua_type(L, idx))
                };
                lua_pushfstring(L, c"%s: %p".as_ptr(), kind, lua_topointer(L, idx));
                if tt != 0 {
                    lua_rotate(L, -(2 as i32), -(1 as i32));
                    lua_settop(L, -(1 as i32) - 1 as i32);
                }
            }
        }
    }
    return lua_tolstring(L, -(1 as i32), len);
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaL_setfuncs(
    mut L: *mut lua_State,
    mut l: *const luaL_Reg,
    mut nup: i32,
) {
    luaL_checkstack(L, nup, c"too many upvalues".as_ptr());
    while !((*l).name).is_null() {
        if ((*l).func).is_none() {
            lua_pushboolean(L, 0);
        } else {
            let mut i: i32 = 0;
            i = 0;
            while i < nup {
                lua_pushvalue(L, -nup);
                i += 1;
                i;
            }
            lua_pushcclosure(L, (*l).func, nup);
        }
        lua_setfield(L, -(nup + 2 as i32), (*l).name);
        l = l.offset(1);
        l;
    }
    lua_settop(L, -nup - 1 as i32);
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaL_getsubtable(
    mut L: *mut lua_State,
    mut idx: i32,
    mut fname: *const std::ffi::c_char,
) -> i32 {
    if lua_getfield(L, idx, fname) == 5 as i32 {
        return 1 as i32;
    } else {
        lua_settop(L, -(1 as i32) - 1 as i32);
        idx = lua_absindex(L, idx);
        lua_createtable(L, 0, 0);
        lua_pushvalue(L, -(1 as i32));
        lua_setfield(L, idx, fname);
        return 0;
    };
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaL_requiref(
    mut L: *mut lua_State,
    mut modname: *const std::ffi::c_char,
    mut openf: lua_CFunction,
    mut glb: i32,
) {
    luaL_getsubtable(L, -(1000000) - 1000, c"_LOADED".as_ptr());
    lua_getfield(L, -(1 as i32), modname);
    if lua_toboolean(L, -(1 as i32)) == 0 {
        lua_settop(L, -(1 as i32) - 1 as i32);
        lua_pushcclosure(L, openf, 0);
        lua_pushstring(L, modname);
        lua_callk(L, 1 as i32, 1 as i32, 0 as lua_KContext, None);
        lua_pushvalue(L, -(1 as i32));
        lua_setfield(L, -(3 as i32), modname);
    }
    lua_rotate(L, -(2 as i32), -(1 as i32));
    lua_settop(L, -(1 as i32) - 1 as i32);
    if glb != 0 {
        lua_pushvalue(L, -(1 as i32));
        lua_setglobal(L, modname);
    }
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaL_addgsub(
    mut b: *mut luaL_Buffer,
    mut s: *const std::ffi::c_char,
    mut p: *const std::ffi::c_char,
    mut r: *const std::ffi::c_char,
) {
    let mut wild: *const std::ffi::c_char = 0 as *const std::ffi::c_char;
    let mut l: size_t = strlen(p);
    loop {
        wild = strstr(s, p);
        if wild.is_null() {
            break;
        }
        luaL_addlstring(b, s, wild.offset_from(s) as std::ffi::c_long as size_t);
        luaL_addstring(b, r);
        s = wild.offset(l as isize);
    }
    luaL_addstring(b, s);
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaL_gsub(
    mut L: *mut lua_State,
    mut s: *const std::ffi::c_char,
    mut p: *const std::ffi::c_char,
    mut r: *const std::ffi::c_char,
) -> *const std::ffi::c_char {
    let mut b: luaL_Buffer = luaL_Buffer {
        b: 0 as *mut std::ffi::c_char,
        size: 0,
        n: 0,
        L: 0 as *mut lua_State,
        init: C2RustUnnamed_15 { n: 0. },
    };
    luaL_buffinit(L, &mut b);
    luaL_addgsub(&mut b, s, p, r);
    luaL_pushresult(&mut b);
    return lua_tolstring(L, -(1 as i32), 0 as *mut size_t);
}
unsafe extern "C-unwind" fn l_alloc(
    mut ud: *mut c_void,
    mut ptr: *mut c_void,
    mut osize: size_t,
    mut nsize: size_t,
) -> *mut c_void {
    if nsize == 0 as size_t {
        free(ptr);
        return 0 as *mut c_void;
    } else {
        return realloc(ptr, nsize);
    };
}
unsafe extern "C-unwind" fn panic(mut L: *mut lua_State) -> i32 {
    let mut msg: *const std::ffi::c_char = if lua_type(L, -(1 as i32)) == 4 as i32 {
        lua_tolstring(L, -(1 as i32), 0 as *mut size_t)
    } else {
        c"error object is not a string".as_ptr()
    };
    fprintf(
        stderr,
        c"PANIC: unprotected error in call to Lua API (%s)\n".as_ptr(),
        msg,
    );
    fflush(stderr);
    return 0;
}
unsafe extern "C-unwind" fn checkcontrol(
    mut L: *mut lua_State,
    mut message: *const std::ffi::c_char,
    mut tocont: i32,
) -> i32 {
    if tocont != 0 || {
        let fresh150 = message;
        message = message.offset(1);
        *fresh150 != b'@' as std::ffi::c_char
    } {
        return 0;
    } else {
        if strcmp(message, c"off".as_ptr()) == 0 {
            lua_setwarnf(
                L,
                Some(
                    warnfoff
                        as unsafe extern "C-unwind" fn(
                            *mut c_void,
                            *const std::ffi::c_char,
                            i32,
                        ) -> (),
                ),
                L as *mut c_void,
            );
        } else if strcmp(message, c"on".as_ptr()) == 0 {
            lua_setwarnf(
                L,
                Some(
                    warnfon
                        as unsafe extern "C-unwind" fn(
                            *mut c_void,
                            *const std::ffi::c_char,
                            i32,
                        ) -> (),
                ),
                L as *mut c_void,
            );
        }
        return 1 as i32;
    };
}
unsafe extern "C-unwind" fn warnfoff(
    mut ud: *mut c_void,
    mut message: *const std::ffi::c_char,
    mut tocont: i32,
) {
    checkcontrol(ud as *mut lua_State, message, tocont);
}
unsafe extern "C-unwind" fn warnfcont(
    mut ud: *mut c_void,
    mut message: *const std::ffi::c_char,
    mut tocont: i32,
) {
    let mut L: *mut lua_State = ud as *mut lua_State;
    fprintf(stderr, c"%s".as_ptr(), message);
    fflush(stderr);
    if tocont != 0 {
        lua_setwarnf(
            L,
            Some(
                warnfcont
                    as unsafe extern "C-unwind" fn(*mut c_void, *const std::ffi::c_char, i32) -> (),
            ),
            L as *mut c_void,
        );
    } else {
        fprintf(stderr, c"%s".as_ptr(), c"\n".as_ptr());
        fflush(stderr);
        lua_setwarnf(
            L,
            Some(
                warnfon
                    as unsafe extern "C-unwind" fn(*mut c_void, *const std::ffi::c_char, i32) -> (),
            ),
            L as *mut c_void,
        );
    };
}
unsafe extern "C-unwind" fn warnfon(
    mut ud: *mut c_void,
    mut message: *const std::ffi::c_char,
    mut tocont: i32,
) {
    if checkcontrol(ud as *mut lua_State, message, tocont) != 0 {
        return;
    }
    fprintf(stderr, c"%s".as_ptr(), c"Lua warning: ".as_ptr());
    fflush(stderr);
    warnfcont(ud, message, tocont);
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaL_newstate() -> *mut lua_State {
    struct AllocOom {
        // Heap base ptr -> heap size
        heaps: std::collections::BTreeMap<usize, std::alloc::Layout>,
    }

    impl Drop for AllocOom {
        fn drop(&mut self) {
            for (ptr, layout) in self.heaps.iter() {
                let ptr = *ptr as *mut u8;
                unsafe { std::alloc::dealloc(ptr, *layout) };
            }
        }
    }

    impl talc::OomHandler for AllocOom {
        fn handle_oom(talc: &mut talc::Talc<Self>, layout: std::alloc::Layout) -> Result<(), ()> {
            const BLOCK_SIZE: usize = 4; // Multiplied by BLOCK_ALIGN bytes
            const BLOCK_ALIGN: usize = 4096;

            let mut min_size = BLOCK_SIZE * BLOCK_ALIGN;
            while min_size <= layout.pad_to_align().size() {
                min_size *= 2;
            }
            let mut min_align = BLOCK_ALIGN.max(layout.align());

            let block_layout =
                unsafe { std::alloc::Layout::from_size_align_unchecked(min_size, min_align) };

            let ptr = unsafe { std::alloc::alloc(block_layout) };

            if ptr.is_null() {
                return Err(());
            }

            talc.oom_handler.heaps.insert(ptr as usize, block_layout);

            unsafe { talc.claim(talc::Span::from_base_size(ptr, min_size))? };

            Ok(())
        }
    }

    unsafe extern "C-unwind" fn alloc_talc(
        ud: *mut c_void,
        ptr: *mut c_void,
        osize: usize,
        nsize: usize,
    ) -> *mut c_void {
        let talc = &mut *ud.cast::<talc::Talc<AllocOom>>();
        if nsize == 0 {
            if !ptr.is_null() {
                talc.free(
                    NonNull::new_unchecked(ptr.cast()),
                    std::alloc::Layout::from_size_align_unchecked(osize, 8),
                );

                // TODO: Periodically free unused heaps
            }
            ptr::null_mut()
        } else {
            let ptr = if ptr.is_null() {
                let layout = std::alloc::Layout::from_size_align_unchecked(nsize, 8);
                talc.malloc(layout)
            } else {
                let layout = std::alloc::Layout::from_size_align_unchecked(osize, 8);
                talc.grow(NonNull::new_unchecked(ptr.cast()), layout, nsize)
            };

            let Ok(ptr) = ptr else {
                return ptr::null_mut();
            };
            ptr.as_ptr().cast()
        }
    }

    // TODO: Drop allocator on lua_close. Somehow.
    let mut talc = Box::new(talc::Talc::new(AllocOom {
        heaps: Default::default(),
    }));

    let mut L: *mut lua_State = lua_newstate(
        // Some(
        //     l_alloc
        //         as unsafe extern "C-unwind" fn(
        //             *mut c_void,
        //             *mut c_void,
        //             size_t,
        //             size_t,
        //         ) -> *mut c_void,
        // ),
        // 0 as *mut c_void,
        Some(alloc_talc),
        Box::into_raw(talc) as *mut c_void,
    );
    if (L != 0 as *mut lua_State) as i32 as std::ffi::c_long != 0 {
        lua_atpanic(
            L,
            Some(panic as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
        );
        lua_setwarnf(
            L,
            Some(
                warnfoff
                    as unsafe extern "C-unwind" fn(*mut c_void, *const std::ffi::c_char, i32) -> (),
            ),
            L as *mut c_void,
        );
    }
    return L;
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaL_checkversion_(
    mut L: *mut lua_State,
    mut ver: lua_Number,
    mut sz: size_t,
) {
    let mut v: lua_Number = lua_version(L);
    if sz
        != (size_of::<lua_Integer>() as usize)
            .wrapping_mul(16)
            .wrapping_add(size_of::<lua_Number>() as usize)
    {
        luaL_error(
            L,
            c"core and library have incompatible numeric types".as_ptr(),
        );
    } else if v != ver {
        luaL_error(
            L,
            c"version mismatch: app. needs %f, Lua core provides %f".as_ptr(),
            ver,
            v,
        );
    }
}
unsafe extern "C-unwind" fn luaB_print(mut L: *mut lua_State) -> i32 {
    let mut n: i32 = lua_gettop(L);
    let mut i: i32 = 0;
    i = 1 as i32;
    while i <= n {
        let mut l: size_t = 0;
        let mut s: *const std::ffi::c_char = luaL_tolstring(L, i, &mut l);
        if i > 1 as i32 {
            fwrite(
                c"\t".as_ptr() as *const c_void,
                size_of::<std::ffi::c_char>() as usize,
                1,
                stdout,
            );
        }
        fwrite(
            s as *const c_void,
            size_of::<std::ffi::c_char>() as usize,
            l,
            stdout,
        );
        lua_settop(L, -(1 as i32) - 1 as i32);
        i += 1;
        i;
    }
    fwrite(
        c"\n".as_ptr() as *const c_void,
        size_of::<std::ffi::c_char>() as usize,
        1,
        stdout,
    );
    fflush(stdout);
    return 0;
}
unsafe extern "C-unwind" fn luaB_warn(mut L: *mut lua_State) -> i32 {
    let mut n: i32 = lua_gettop(L);
    let mut i: i32 = 0;
    luaL_checklstring(L, 1 as i32, 0 as *mut size_t);
    i = 2 as i32;
    while i <= n {
        luaL_checklstring(L, i, 0 as *mut size_t);
        i += 1;
        i;
    }
    i = 1 as i32;
    while i < n {
        lua_warning(L, lua_tolstring(L, i, 0 as *mut size_t), 1 as i32);
        i += 1;
        i;
    }
    lua_warning(L, lua_tolstring(L, n, 0 as *mut size_t), 0);
    return 0;
}
unsafe extern "C-unwind" fn b_str2int(
    mut s: *const std::ffi::c_char,
    mut base: i32,
    mut pn: *mut lua_Integer,
) -> *const std::ffi::c_char {
    let mut n: lua_Unsigned = 0 as lua_Unsigned;
    let mut neg: i32 = 0;
    s = s.offset(strspn(s, c" \x0C\n\r\t\x0B".as_ptr()) as isize);
    if *s as i32 == '-' as i32 {
        s = s.offset(1);
        s;
        neg = 1 as i32;
    } else if *s as i32 == '+' as i32 {
        s = s.offset(1);
        s;
    }
    if *(*__ctype_b_loc()).offset(*s as u8 as i32 as isize) as i32 & _ISalnum as i32 as u16 as i32
        == 0
    {
        return 0 as *const std::ffi::c_char;
    }
    loop {
        let mut digit_0: i32 = if *(*__ctype_b_loc()).offset(*s as u8 as i32 as isize) as i32
            & _ISdigit as i32 as u16 as i32
            != 0
        {
            *s as i32 - '0' as i32
        } else {
            toupper(*s as u8 as i32) - 'A' as i32 + 10
        };
        if digit_0 >= base {
            return 0 as *const std::ffi::c_char;
        }
        n = (n * base as lua_Unsigned).wrapping_add(digit_0 as lua_Unsigned);
        s = s.offset(1);
        s;
        if !(*(*__ctype_b_loc()).offset(*s as u8 as i32 as isize) as i32
            & _ISalnum as i32 as u16 as i32
            != 0)
        {
            break;
        }
    }
    s = s.offset(strspn(s, c" \x0C\n\r\t\x0B".as_ptr()) as isize);
    *pn = (if neg != 0 {
        (0 as u32 as lua_Unsigned).wrapping_sub(n)
    } else {
        n
    }) as lua_Integer;
    return s;
}
unsafe extern "C-unwind" fn luaB_tonumber(mut L: *mut lua_State) -> i32 {
    if lua_type(L, 2 as i32) <= 0 {
        if lua_type(L, 1 as i32) == 3 as i32 {
            lua_settop(L, 1 as i32);
            return 1 as i32;
        } else {
            let mut l: size_t = 0;
            let mut s: *const std::ffi::c_char = lua_tolstring(L, 1 as i32, &mut l);
            if !s.is_null() && lua_stringtonumber(L, s) == l.wrapping_add(1 as i32 as size_t) {
                return 1 as i32;
            }
            luaL_checkany(L, 1 as i32);
        }
    } else {
        let mut l_0: size_t = 0;
        let mut s_0: *const std::ffi::c_char = 0 as *const std::ffi::c_char;
        let mut n: lua_Integer = 0 as lua_Integer;
        let mut base: lua_Integer = luaL_checkinteger(L, 2 as i32);
        luaL_checktype(L, 1 as i32, 4 as i32);
        s_0 = lua_tolstring(L, 1 as i32, &mut l_0);
        (((2 as i32 as lua_Integer <= base && base <= 36 as i32 as lua_Integer) as i32 != 0) as i32
            as std::ffi::c_long
            != 0
            || luaL_argerror(L, 2 as i32, c"base out of range".as_ptr()) != 0) as i32;
        if b_str2int(s_0, base as i32, &mut n) == s_0.offset(l_0 as isize) {
            lua_pushinteger(L, n);
            return 1 as i32;
        }
    }
    lua_pushnil(L);
    return 1 as i32;
}
unsafe extern "C-unwind" fn luaB_error(mut L: *mut lua_State) -> i32 {
    let mut level: i32 = luaL_optinteger(L, 2 as i32, 1 as i32 as lua_Integer) as i32;
    lua_settop(L, 1 as i32);
    if lua_type(L, 1 as i32) == 4 as i32 && level > 0 {
        luaL_where(L, level);
        lua_pushvalue(L, 1 as i32);
        lua_concat(L, 2 as i32);
    }
    return lua_error(L);
}
unsafe extern "C-unwind" fn luaB_getmetatable(mut L: *mut lua_State) -> i32 {
    luaL_checkany(L, 1 as i32);
    if lua_getmetatable(L, 1 as i32) == 0 {
        lua_pushnil(L);
        return 1 as i32;
    }
    luaL_getmetafield(L, 1 as i32, c"__metatable".as_ptr());
    return 1 as i32;
}
unsafe extern "C-unwind" fn luaB_setmetatable(mut L: *mut lua_State) -> i32 {
    let mut t: i32 = lua_type(L, 2 as i32);
    luaL_checktype(L, 1 as i32, 5 as i32);
    (((t == 0 || t == 5 as i32) as i32 != 0) as i32 as std::ffi::c_long != 0
        || luaL_typeerror(L, 2 as i32, c"nil or table".as_ptr()) != 0) as i32;
    if ((luaL_getmetafield(L, 1 as i32, c"__metatable".as_ptr()) != 0) as i32 != 0) as i32
        as std::ffi::c_long
        != 0
    {
        return luaL_error(L, c"cannot change a protected metatable".as_ptr());
    }
    lua_settop(L, 2 as i32);
    lua_setmetatable(L, 1 as i32);
    return 1 as i32;
}
unsafe extern "C-unwind" fn luaB_rawequal(mut L: *mut lua_State) -> i32 {
    luaL_checkany(L, 1 as i32);
    luaL_checkany(L, 2 as i32);
    lua_pushboolean(L, lua_rawequal(L, 1 as i32, 2 as i32));
    return 1 as i32;
}
unsafe extern "C-unwind" fn luaB_rawlen(mut L: *mut lua_State) -> i32 {
    let mut t: i32 = lua_type(L, 1 as i32);
    (((t == 5 as i32 || t == 4 as i32) as i32 != 0) as i32 as std::ffi::c_long != 0
        || luaL_typeerror(L, 1 as i32, c"table or string".as_ptr()) != 0) as i32;
    lua_pushinteger(L, lua_rawlen(L, 1 as i32) as lua_Integer);
    return 1 as i32;
}
unsafe extern "C-unwind" fn luaB_rawget(mut L: *mut lua_State) -> i32 {
    luaL_checktype(L, 1 as i32, 5 as i32);
    luaL_checkany(L, 2 as i32);
    lua_settop(L, 2 as i32);
    lua_rawget(L, 1 as i32);
    return 1 as i32;
}
unsafe extern "C-unwind" fn luaB_rawset(mut L: *mut lua_State) -> i32 {
    luaL_checktype(L, 1 as i32, 5 as i32);
    luaL_checkany(L, 2 as i32);
    luaL_checkany(L, 3 as i32);
    lua_settop(L, 3 as i32);
    lua_rawset(L, 1 as i32);
    return 1 as i32;
}
unsafe extern "C-unwind" fn pushmode(mut L: *mut lua_State, mut oldmode: i32) -> i32 {
    if oldmode == -(1 as i32) {
        lua_pushnil(L);
    } else {
        lua_pushstring(
            L,
            if oldmode == 11 as i32 {
                c"incremental".as_ptr()
            } else {
                c"generational".as_ptr()
            },
        );
    }
    return 1 as i32;
}
unsafe extern "C-unwind" fn luaB_collectgarbage(mut L: *mut lua_State) -> i32 {
    static mut opts: [*const std::ffi::c_char; 11] = [
        c"stop".as_ptr(),
        c"restart".as_ptr(),
        c"collect".as_ptr(),
        c"count".as_ptr(),
        c"step".as_ptr(),
        c"setpause".as_ptr(),
        c"setstepmul".as_ptr(),
        c"isrunning".as_ptr(),
        c"generational".as_ptr(),
        c"incremental".as_ptr(),
        0 as *const std::ffi::c_char,
    ];
    static mut optsnum: [i32; 10] = [
        0, 1 as i32, 2 as i32, 3 as i32, 5 as i32, 6 as i32, 7 as i32, 9 as i32, 10, 11 as i32,
    ];
    let mut o: i32 = optsnum
        [luaL_checkoption(L, 1 as i32, c"collect".as_ptr(), (&raw const opts).cast()) as usize];
    match o {
        3 => {
            let mut k: i32 = lua_gc(L, o);
            let mut b: i32 = lua_gc(L, 4 as i32);
            if !(k == -(1 as i32)) {
                lua_pushnumber(
                    L,
                    k as lua_Number + b as lua_Number / 1024 as i32 as lua_Number,
                );
                return 1 as i32;
            }
        }
        5 => {
            let mut step: i32 = luaL_optinteger(L, 2 as i32, 0 as lua_Integer) as i32;
            let mut res: i32 = lua_gc(L, o, step);
            if !(res == -(1 as i32)) {
                lua_pushboolean(L, res);
                return 1 as i32;
            }
        }
        6 | 7 => {
            let mut p: i32 = luaL_optinteger(L, 2 as i32, 0 as lua_Integer) as i32;
            let mut previous: i32 = lua_gc(L, o, p);
            if !(previous == -(1 as i32)) {
                lua_pushinteger(L, previous as lua_Integer);
                return 1 as i32;
            }
        }
        9 => {
            let mut res_0: i32 = lua_gc(L, o);
            if !(res_0 == -(1 as i32)) {
                lua_pushboolean(L, res_0);
                return 1 as i32;
            }
        }
        10 => {
            let mut minormul: i32 = luaL_optinteger(L, 2 as i32, 0 as lua_Integer) as i32;
            let mut majormul: i32 = luaL_optinteger(L, 3 as i32, 0 as lua_Integer) as i32;
            return pushmode(L, lua_gc(L, o, minormul, majormul));
        }
        11 => {
            let mut pause: i32 = luaL_optinteger(L, 2 as i32, 0 as lua_Integer) as i32;
            let mut stepmul: i32 = luaL_optinteger(L, 3 as i32, 0 as lua_Integer) as i32;
            let mut stepsize: i32 = luaL_optinteger(L, 4 as i32, 0 as lua_Integer) as i32;
            return pushmode(L, lua_gc(L, o, pause, stepmul, stepsize));
        }
        _ => {
            let mut res_1: i32 = lua_gc(L, o);
            if !(res_1 == -(1 as i32)) {
                lua_pushinteger(L, res_1 as lua_Integer);
                return 1 as i32;
            }
        }
    }
    lua_pushnil(L);
    return 1 as i32;
}
unsafe extern "C-unwind" fn luaB_type(mut L: *mut lua_State) -> i32 {
    let mut t: i32 = lua_type(L, 1 as i32);
    (((t != -(1 as i32)) as i32 != 0) as i32 as std::ffi::c_long != 0
        || luaL_argerror(L, 1 as i32, c"value expected".as_ptr()) != 0) as i32;
    lua_pushstring(L, lua_typename(L, t));
    return 1 as i32;
}
unsafe extern "C-unwind" fn luaB_next(mut L: *mut lua_State) -> i32 {
    luaL_checktype(L, 1 as i32, 5 as i32);
    lua_settop(L, 2 as i32);
    if lua_next(L, 1 as i32) != 0 {
        return 2 as i32;
    } else {
        lua_pushnil(L);
        return 1 as i32;
    };
}
unsafe extern "C-unwind" fn pairscont(
    mut L: *mut lua_State,
    mut status: i32,
    mut k: lua_KContext,
) -> i32 {
    return 3 as i32;
}
unsafe extern "C-unwind" fn luaB_pairs(mut L: *mut lua_State) -> i32 {
    luaL_checkany(L, 1 as i32);
    if luaL_getmetafield(L, 1 as i32, c"__pairs".as_ptr()) == 0 {
        lua_pushcclosure(
            L,
            Some(luaB_next as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            0,
        );
        lua_pushvalue(L, 1 as i32);
        lua_pushnil(L);
    } else {
        lua_pushvalue(L, 1 as i32);
        lua_callk(
            L,
            1 as i32,
            3 as i32,
            0 as lua_KContext,
            Some(
                pairscont as unsafe extern "C-unwind" fn(*mut lua_State, i32, lua_KContext) -> i32,
            ),
        );
    }
    return 3 as i32;
}
unsafe extern "C-unwind" fn ipairsaux(mut L: *mut lua_State) -> i32 {
    let mut i: lua_Integer = luaL_checkinteger(L, 2 as i32);
    i = (i as lua_Unsigned).wrapping_add(1 as i32 as lua_Unsigned) as lua_Integer;
    lua_pushinteger(L, i);
    return if lua_geti(L, 1 as i32, i) == 0 {
        1 as i32
    } else {
        2 as i32
    };
}
unsafe extern "C-unwind" fn luaB_ipairs(mut L: *mut lua_State) -> i32 {
    luaL_checkany(L, 1 as i32);
    lua_pushcclosure(
        L,
        Some(ipairsaux as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
        0,
    );
    lua_pushvalue(L, 1 as i32);
    lua_pushinteger(L, 0 as lua_Integer);
    return 3 as i32;
}
unsafe extern "C-unwind" fn load_aux(
    mut L: *mut lua_State,
    mut status: i32,
    mut envidx: i32,
) -> i32 {
    if ((status == 0) as i32 != 0) as i32 as std::ffi::c_long != 0 {
        if envidx != 0 {
            lua_pushvalue(L, envidx);
            if (lua_setupvalue(L, -(2 as i32), 1 as i32)).is_null() {
                lua_settop(L, -(1 as i32) - 1 as i32);
            }
        }
        return 1 as i32;
    } else {
        lua_pushnil(L);
        lua_rotate(L, -(2 as i32), 1 as i32);
        return 2 as i32;
    };
}
unsafe extern "C-unwind" fn luaB_loadfile(mut L: *mut lua_State) -> i32 {
    let mut fname: *const std::ffi::c_char =
        luaL_optlstring(L, 1 as i32, 0 as *const std::ffi::c_char, 0 as *mut size_t);
    let mut mode: *const std::ffi::c_char =
        luaL_optlstring(L, 2 as i32, 0 as *const std::ffi::c_char, 0 as *mut size_t);
    let mut env: i32 = if !(lua_type(L, 3 as i32) == -(1 as i32)) {
        3 as i32
    } else {
        0
    };
    let mut status: i32 = luaL_loadfilex(L, fname, mode);
    return load_aux(L, status, env);
}
unsafe extern "C-unwind" fn generic_reader(
    mut L: *mut lua_State,
    mut ud: *mut c_void,
    mut size: *mut size_t,
) -> *const std::ffi::c_char {
    luaL_checkstack(L, 2 as i32, c"too many nested functions".as_ptr());
    lua_pushvalue(L, 1 as i32);
    lua_callk(L, 0, 1 as i32, 0 as lua_KContext, None);
    if lua_type(L, -(1 as i32)) == 0 {
        lua_settop(L, -(1 as i32) - 1 as i32);
        *size = 0 as size_t;
        return 0 as *const std::ffi::c_char;
    } else if ((lua_isstring(L, -(1 as i32)) == 0) as i32 != 0) as i32 as std::ffi::c_long != 0 {
        luaL_error(L, c"reader function must return a string".as_ptr());
    }
    lua_copy(L, -(1 as i32), 5 as i32);
    lua_settop(L, -(1 as i32) - 1 as i32);
    return lua_tolstring(L, 5 as i32, size);
}
unsafe extern "C-unwind" fn luaB_load(mut L: *mut lua_State) -> i32 {
    let mut status: i32 = 0;
    let mut l: size_t = 0;
    let mut s: *const std::ffi::c_char = lua_tolstring(L, 1 as i32, &mut l);
    let mut mode: *const std::ffi::c_char =
        luaL_optlstring(L, 3 as i32, c"bt".as_ptr(), 0 as *mut size_t);
    let mut env: i32 = if !(lua_type(L, 4 as i32) == -(1 as i32)) {
        4 as i32
    } else {
        0
    };
    if !s.is_null() {
        let mut chunkname: *const std::ffi::c_char =
            luaL_optlstring(L, 2 as i32, s, 0 as *mut size_t);
        status = luaL_loadbufferx(L, s, l, chunkname, mode);
    } else {
        let mut chunkname_0: *const std::ffi::c_char =
            luaL_optlstring(L, 2 as i32, c"=(load)".as_ptr(), 0 as *mut size_t);
        luaL_checktype(L, 1 as i32, 6 as i32);
        lua_settop(L, 5 as i32);
        status = lua_load(
            L,
            Some(
                generic_reader
                    as unsafe extern "C-unwind" fn(
                        *mut lua_State,
                        *mut c_void,
                        *mut size_t,
                    ) -> *const std::ffi::c_char,
            ),
            0 as *mut c_void,
            chunkname_0,
            mode,
        );
    }
    return load_aux(L, status, env);
}
unsafe extern "C-unwind" fn dofilecont(
    mut L: *mut lua_State,
    mut d1: i32,
    mut d2: lua_KContext,
) -> i32 {
    return lua_gettop(L) - 1 as i32;
}
unsafe extern "C-unwind" fn luaB_dofile(mut L: *mut lua_State) -> i32 {
    let mut fname: *const std::ffi::c_char =
        luaL_optlstring(L, 1 as i32, 0 as *const std::ffi::c_char, 0 as *mut size_t);
    lua_settop(L, 1 as i32);
    if ((luaL_loadfilex(L, fname, 0 as *const std::ffi::c_char) != 0) as i32 != 0) as i32
        as std::ffi::c_long
        != 0
    {
        return lua_error(L);
    }
    lua_callk(
        L,
        0,
        -(1 as i32),
        0 as lua_KContext,
        Some(dofilecont as unsafe extern "C-unwind" fn(*mut lua_State, i32, lua_KContext) -> i32),
    );
    return dofilecont(L, 0, 0 as lua_KContext);
}
unsafe extern "C-unwind" fn luaB_assert(mut L: *mut lua_State) -> i32 {
    if (lua_toboolean(L, 1 as i32) != 0) as i32 as std::ffi::c_long != 0 {
        return lua_gettop(L);
    } else {
        luaL_checkany(L, 1 as i32);
        lua_rotate(L, 1 as i32, -(1 as i32));
        lua_settop(L, -(1 as i32) - 1 as i32);
        lua_pushstring(L, c"assertion failed!".as_ptr());
        lua_settop(L, 1 as i32);
        return luaB_error(L);
    };
}
unsafe extern "C-unwind" fn luaB_select(mut L: *mut lua_State) -> i32 {
    let mut n: i32 = lua_gettop(L);
    if lua_type(L, 1 as i32) == 4 as i32
        && *lua_tolstring(L, 1 as i32, 0 as *mut size_t) as i32 == '#' as i32
    {
        lua_pushinteger(L, (n - 1 as i32) as lua_Integer);
        return 1 as i32;
    } else {
        let mut i: lua_Integer = luaL_checkinteger(L, 1 as i32);
        if i < 0 as lua_Integer {
            i = n as lua_Integer + i;
        } else if i > n as lua_Integer {
            i = n as lua_Integer;
        }
        (((1 as i32 as lua_Integer <= i) as i32 != 0) as i32 as std::ffi::c_long != 0
            || luaL_argerror(L, 1 as i32, c"index out of range".as_ptr()) != 0) as i32;
        return n - i as i32;
    };
}
unsafe extern "C-unwind" fn finishpcall(
    mut L: *mut lua_State,
    mut status: i32,
    mut extra: lua_KContext,
) -> i32 {
    if ((status != 0 && status != 1 as i32) as i32 != 0) as i32 as std::ffi::c_long != 0 {
        lua_pushboolean(L, 0);
        lua_pushvalue(L, -(2 as i32));
        return 2 as i32;
    } else {
        return lua_gettop(L) - extra as i32;
    };
}
unsafe extern "C-unwind" fn luaB_pcall(mut L: *mut lua_State) -> i32 {
    let mut status: i32 = 0;
    luaL_checkany(L, 1 as i32);
    lua_pushboolean(L, 1 as i32);
    lua_rotate(L, 1 as i32, 1 as i32);
    status = lua_pcallk(
        L,
        lua_gettop(L) - 2 as i32,
        -(1 as i32),
        0,
        0 as lua_KContext,
        Some(finishpcall as unsafe extern "C-unwind" fn(*mut lua_State, i32, lua_KContext) -> i32),
    );
    return finishpcall(L, status, 0 as lua_KContext);
}
unsafe extern "C-unwind" fn luaB_xpcall(mut L: *mut lua_State) -> i32 {
    let mut status: i32 = 0;
    let mut n: i32 = lua_gettop(L);
    luaL_checktype(L, 2 as i32, 6 as i32);
    lua_pushboolean(L, 1 as i32);
    lua_pushvalue(L, 1 as i32);
    lua_rotate(L, 3 as i32, 2 as i32);
    status = lua_pcallk(
        L,
        n - 2 as i32,
        -(1 as i32),
        2 as i32,
        2 as i32 as lua_KContext,
        Some(finishpcall as unsafe extern "C-unwind" fn(*mut lua_State, i32, lua_KContext) -> i32),
    );
    return finishpcall(L, status, 2 as i32 as lua_KContext);
}
unsafe extern "C-unwind" fn luaB_tostring(mut L: *mut lua_State) -> i32 {
    luaL_checkany(L, 1 as i32);
    luaL_tolstring(L, 1 as i32, 0 as *mut size_t);
    return 1 as i32;
}
static mut base_funcs: [luaL_Reg; 26] = unsafe {
    [
        {
            let mut init = luaL_Reg {
                name: c"assert".as_ptr(),
                func: Some(luaB_assert as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"collectgarbage".as_ptr(),
                func: Some(
                    luaB_collectgarbage as unsafe extern "C-unwind" fn(*mut lua_State) -> i32,
                ),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"dofile".as_ptr(),
                func: Some(luaB_dofile as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"error".as_ptr(),
                func: Some(luaB_error as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"getmetatable".as_ptr(),
                func: Some(luaB_getmetatable as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"ipairs".as_ptr(),
                func: Some(luaB_ipairs as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"loadfile".as_ptr(),
                func: Some(luaB_loadfile as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"load".as_ptr(),
                func: Some(luaB_load as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"next".as_ptr(),
                func: Some(luaB_next as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"pairs".as_ptr(),
                func: Some(luaB_pairs as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"pcall".as_ptr(),
                func: Some(luaB_pcall as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"print".as_ptr(),
                func: Some(luaB_print as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"warn".as_ptr(),
                func: Some(luaB_warn as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"rawequal".as_ptr(),
                func: Some(luaB_rawequal as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"rawlen".as_ptr(),
                func: Some(luaB_rawlen as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"rawget".as_ptr(),
                func: Some(luaB_rawget as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"rawset".as_ptr(),
                func: Some(luaB_rawset as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"select".as_ptr(),
                func: Some(luaB_select as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"setmetatable".as_ptr(),
                func: Some(luaB_setmetatable as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"tonumber".as_ptr(),
                func: Some(luaB_tonumber as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"tostring".as_ptr(),
                func: Some(luaB_tostring as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"type".as_ptr(),
                func: Some(luaB_type as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"xpcall".as_ptr(),
                func: Some(luaB_xpcall as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"_G".as_ptr(),
                func: None,
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"_VERSION".as_ptr(),
                func: None,
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: 0 as *const std::ffi::c_char,
                func: None,
            };
            init
        },
    ]
};
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaopen_base(mut L: *mut lua_State) -> i32 {
    lua_rawgeti(L, -(1000000) - 1000, 2 as i32 as lua_Integer);
    luaL_setfuncs(L, (&raw const base_funcs).cast(), 0);
    lua_pushvalue(L, -(1 as i32));
    lua_setfield(L, -(2 as i32), c"_G".as_ptr());
    lua_pushstring(L, c"Lua 5.4".as_ptr());
    lua_setfield(L, -(2 as i32), c"_VERSION".as_ptr());
    return 1 as i32;
}
unsafe extern "C-unwind" fn getco(mut L: *mut lua_State) -> *mut lua_State {
    let mut co: *mut lua_State = lua_tothread(L, 1 as i32);
    ((co != 0 as *mut lua_State) as i32 as std::ffi::c_long != 0
        || luaL_typeerror(L, 1 as i32, c"thread".as_ptr()) != 0) as i32;
    return co;
}
unsafe extern "C-unwind" fn auxresume(
    mut L: *mut lua_State,
    mut co: *mut lua_State,
    mut narg: i32,
) -> i32 {
    let mut status: i32 = 0;
    let mut nres: i32 = 0;
    if ((lua_checkstack(co, narg) == 0) as i32 != 0) as i32 as std::ffi::c_long != 0 {
        lua_pushstring(L, c"too many arguments to resume".as_ptr());
        return -(1 as i32);
    }
    lua_xmove(L, co, narg);
    status = lua_resume(co, L, narg, &mut nres);
    if ((status == 0 || status == 1 as i32) as i32 != 0) as i32 as std::ffi::c_long != 0 {
        if ((lua_checkstack(L, nres + 1 as i32) == 0) as i32 != 0) as i32 as std::ffi::c_long != 0 {
            lua_settop(co, -nres - 1 as i32);
            lua_pushstring(L, c"too many results to resume".as_ptr());
            return -(1 as i32);
        }
        lua_xmove(co, L, nres);
        return nres;
    } else {
        lua_xmove(co, L, 1 as i32);
        return -(1 as i32);
    };
}
unsafe extern "C-unwind" fn luaB_coresume(mut L: *mut lua_State) -> i32 {
    let mut co: *mut lua_State = getco(L);
    let mut r: i32 = 0;
    r = auxresume(L, co, lua_gettop(L) - 1 as i32);
    if ((r < 0) as i32 != 0) as i32 as std::ffi::c_long != 0 {
        lua_pushboolean(L, 0);
        lua_rotate(L, -(2 as i32), 1 as i32);
        return 2 as i32;
    } else {
        lua_pushboolean(L, 1 as i32);
        lua_rotate(L, -(r + 1 as i32), 1 as i32);
        return r + 1 as i32;
    };
}
unsafe extern "C-unwind" fn luaB_auxwrap(mut L: *mut lua_State) -> i32 {
    let mut co: *mut lua_State = lua_tothread(L, -(1000000) - 1000 - 1 as i32);
    let mut r: i32 = auxresume(L, co, lua_gettop(L));
    if ((r < 0) as i32 != 0) as i32 as std::ffi::c_long != 0 {
        let mut stat: i32 = lua_status(co);
        if stat != 0 && stat != 1 as i32 {
            stat = lua_closethread(co, L);
            lua_xmove(co, L, 1 as i32);
        }
        if stat != 4 as i32 && lua_type(L, -(1 as i32)) == 4 as i32 {
            luaL_where(L, 1 as i32);
            lua_rotate(L, -(2 as i32), 1 as i32);
            lua_concat(L, 2 as i32);
        }
        return lua_error(L);
    }
    return r;
}
unsafe extern "C-unwind" fn luaB_cocreate(mut L: *mut lua_State) -> i32 {
    let mut NL: *mut lua_State = 0 as *mut lua_State;
    luaL_checktype(L, 1 as i32, 6 as i32);
    NL = lua_newthread(L);
    lua_pushvalue(L, 1 as i32);
    lua_xmove(L, NL, 1 as i32);
    return 1 as i32;
}
unsafe extern "C-unwind" fn luaB_cowrap(mut L: *mut lua_State) -> i32 {
    luaB_cocreate(L);
    lua_pushcclosure(
        L,
        Some(luaB_auxwrap as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
        1 as i32,
    );
    return 1 as i32;
}
unsafe extern "C-unwind" fn luaB_yield(mut L: *mut lua_State) -> i32 {
    return lua_yieldk(L, lua_gettop(L), 0 as lua_KContext, None);
}
static mut statname: [*const std::ffi::c_char; 4] = [
    c"running".as_ptr(),
    c"dead".as_ptr(),
    c"suspended".as_ptr(),
    c"normal".as_ptr(),
];
unsafe extern "C-unwind" fn auxstatus(mut L: *mut lua_State, mut co: *mut lua_State) -> i32 {
    if L == co {
        return 0;
    } else {
        match lua_status(co) {
            1 => return 2 as i32,
            0 => {
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
                if lua_getstack(co, 0, &mut ar) != 0 {
                    return 3 as i32;
                } else if lua_gettop(co) == 0 {
                    return 1 as i32;
                } else {
                    return 2 as i32;
                }
            }
            _ => return 1 as i32,
        }
    };
}
unsafe extern "C-unwind" fn luaB_costatus(mut L: *mut lua_State) -> i32 {
    let mut co: *mut lua_State = getco(L);
    lua_pushstring(L, statname[auxstatus(L, co) as usize]);
    return 1 as i32;
}
unsafe extern "C-unwind" fn luaB_yieldable(mut L: *mut lua_State) -> i32 {
    let mut co: *mut lua_State = if lua_type(L, 1 as i32) == -(1 as i32) {
        L
    } else {
        getco(L)
    };
    lua_pushboolean(L, lua_isyieldable(co));
    return 1 as i32;
}
unsafe extern "C-unwind" fn luaB_corunning(mut L: *mut lua_State) -> i32 {
    let mut ismain: i32 = lua_pushthread(L);
    lua_pushboolean(L, ismain);
    return 2 as i32;
}
unsafe extern "C-unwind" fn luaB_close(mut L: *mut lua_State) -> i32 {
    let mut co: *mut lua_State = getco(L);
    let mut status: i32 = auxstatus(L, co);
    match status {
        1 | 2 => {
            status = lua_closethread(co, L);
            if status == 0 {
                lua_pushboolean(L, 1 as i32);
                return 1 as i32;
            } else {
                lua_pushboolean(L, 0);
                lua_xmove(co, L, 1 as i32);
                return 2 as i32;
            }
        }
        _ => {
            return luaL_error(
                L,
                c"cannot close a %s coroutine".as_ptr(),
                statname[status as usize],
            );
        }
    };
}
static mut co_funcs: [luaL_Reg; 9] = unsafe {
    [
        {
            let mut init = luaL_Reg {
                name: c"create".as_ptr(),
                func: Some(luaB_cocreate as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"resume".as_ptr(),
                func: Some(luaB_coresume as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"running".as_ptr(),
                func: Some(luaB_corunning as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"status".as_ptr(),
                func: Some(luaB_costatus as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"wrap".as_ptr(),
                func: Some(luaB_cowrap as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"yield".as_ptr(),
                func: Some(luaB_yield as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"isyieldable".as_ptr(),
                func: Some(luaB_yieldable as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"close".as_ptr(),
                func: Some(luaB_close as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: 0 as *const std::ffi::c_char,
                func: None,
            };
            init
        },
    ]
};
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaopen_coroutine(mut L: *mut lua_State) -> i32 {
    luaL_checkversion_(
        L,
        504 as i32 as lua_Number,
        (size_of::<lua_Integer>() as usize)
            .wrapping_mul(16)
            .wrapping_add(size_of::<lua_Number>() as usize),
    );
    lua_createtable(
        L,
        0,
        (size_of::<[luaL_Reg; 9]>() as usize)
            .wrapping_div(size_of::<luaL_Reg>() as usize)
            .wrapping_sub(1) as i32,
    );
    luaL_setfuncs(L, (&raw const co_funcs).cast(), 0);
    return 1 as i32;
}
static mut HOOKKEY: *const std::ffi::c_char = c"_HOOKKEY".as_ptr();
unsafe extern "C-unwind" fn checkstack(mut L: *mut lua_State, mut L1: *mut lua_State, mut n: i32) {
    if ((L != L1 && lua_checkstack(L1, n) == 0) as i32 != 0) as i32 as std::ffi::c_long != 0 {
        luaL_error(L, c"stack overflow".as_ptr());
    }
}
unsafe extern "C-unwind" fn db_getregistry(mut L: *mut lua_State) -> i32 {
    lua_pushvalue(L, -(1000000) - 1000);
    return 1 as i32;
}
unsafe extern "C-unwind" fn db_getmetatable(mut L: *mut lua_State) -> i32 {
    luaL_checkany(L, 1 as i32);
    if lua_getmetatable(L, 1 as i32) == 0 {
        lua_pushnil(L);
    }
    return 1 as i32;
}
unsafe extern "C-unwind" fn db_setmetatable(mut L: *mut lua_State) -> i32 {
    let mut t: i32 = lua_type(L, 2 as i32);
    (((t == 0 || t == 5 as i32) as i32 != 0) as i32 as std::ffi::c_long != 0
        || luaL_typeerror(L, 2 as i32, c"nil or table".as_ptr()) != 0) as i32;
    lua_settop(L, 2 as i32);
    lua_setmetatable(L, 1 as i32);
    return 1 as i32;
}
unsafe extern "C-unwind" fn db_getuservalue(mut L: *mut lua_State) -> i32 {
    let mut n: i32 = luaL_optinteger(L, 2 as i32, 1 as i32 as lua_Integer) as i32;
    if lua_type(L, 1 as i32) != 7 as i32 {
        lua_pushnil(L);
    } else if lua_getiuservalue(L, 1 as i32, n) != -(1 as i32) {
        lua_pushboolean(L, 1 as i32);
        return 2 as i32;
    }
    return 1 as i32;
}
unsafe extern "C-unwind" fn db_setuservalue(mut L: *mut lua_State) -> i32 {
    let mut n: i32 = luaL_optinteger(L, 3 as i32, 1 as i32 as lua_Integer) as i32;
    luaL_checktype(L, 1 as i32, 7 as i32);
    luaL_checkany(L, 2 as i32);
    lua_settop(L, 2 as i32);
    if lua_setiuservalue(L, 1 as i32, n) == 0 {
        lua_pushnil(L);
    }
    return 1 as i32;
}
unsafe extern "C-unwind" fn getthread(mut L: *mut lua_State, mut arg: *mut i32) -> *mut lua_State {
    if lua_type(L, 1 as i32) == 8 as i32 {
        *arg = 1 as i32;
        return lua_tothread(L, 1 as i32);
    } else {
        *arg = 0;
        return L;
    };
}
unsafe extern "C-unwind" fn settabss(
    mut L: *mut lua_State,
    mut k: *const std::ffi::c_char,
    mut v: *const std::ffi::c_char,
) {
    lua_pushstring(L, v);
    lua_setfield(L, -(2 as i32), k);
}
unsafe extern "C-unwind" fn settabsi(
    mut L: *mut lua_State,
    mut k: *const std::ffi::c_char,
    mut v: i32,
) {
    lua_pushinteger(L, v as lua_Integer);
    lua_setfield(L, -(2 as i32), k);
}
unsafe extern "C-unwind" fn settabsb(
    mut L: *mut lua_State,
    mut k: *const std::ffi::c_char,
    mut v: i32,
) {
    lua_pushboolean(L, v);
    lua_setfield(L, -(2 as i32), k);
}
unsafe extern "C-unwind" fn treatstackoption(
    mut L: *mut lua_State,
    mut L1: *mut lua_State,
    mut fname: *const std::ffi::c_char,
) {
    if L == L1 {
        lua_rotate(L, -(2 as i32), 1 as i32);
    } else {
        lua_xmove(L1, L, 1 as i32);
    }
    lua_setfield(L, -(2 as i32), fname);
}
unsafe extern "C-unwind" fn db_getinfo(mut L: *mut lua_State) -> i32 {
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
    let mut arg: i32 = 0;
    let mut L1: *mut lua_State = getthread(L, &mut arg);
    let mut options: *const std::ffi::c_char =
        luaL_optlstring(L, arg + 2 as i32, c"flnSrtu".as_ptr(), 0 as *mut size_t);
    checkstack(L, L1, 3 as i32);
    (((*options.offset(0 as isize) as i32 != '>' as i32) as i32 != 0) as i32 as std::ffi::c_long
        != 0
        || luaL_argerror(L, arg + 2 as i32, c"invalid option '>'".as_ptr()) != 0) as i32;
    if lua_type(L, arg + 1 as i32) == 6 as i32 {
        options = lua_pushfstring(L, c">%s".as_ptr(), options);
        lua_pushvalue(L, arg + 1 as i32);
        lua_xmove(L, L1, 1 as i32);
    } else if lua_getstack(L1, luaL_checkinteger(L, arg + 1 as i32) as i32, &mut ar) == 0 {
        lua_pushnil(L);
        return 1 as i32;
    }
    if lua_getinfo(L1, options, &mut ar) == 0 {
        return luaL_argerror(L, arg + 2 as i32, c"invalid option".as_ptr());
    }
    lua_createtable(L, 0, 0);
    if !(strchr(options, 'S' as i32)).is_null() {
        lua_pushlstring(L, ar.source, ar.srclen);
        lua_setfield(L, -(2 as i32), c"source".as_ptr());
        settabss(L, c"short_src".as_ptr(), (ar.short_src).as_mut_ptr());
        settabsi(L, c"linedefined".as_ptr(), ar.linedefined);
        settabsi(L, c"lastlinedefined".as_ptr(), ar.lastlinedefined);
        settabss(L, c"what".as_ptr(), ar.what);
    }
    if !(strchr(options, 'l' as i32)).is_null() {
        settabsi(L, c"currentline".as_ptr(), ar.currentline);
    }
    if !(strchr(options, 'u' as i32)).is_null() {
        settabsi(L, c"nups".as_ptr(), ar.nups as i32);
        settabsi(L, c"nparams".as_ptr(), ar.nparams as i32);
        settabsb(L, c"isvararg".as_ptr(), ar.isvararg as i32);
    }
    if !(strchr(options, 'n' as i32)).is_null() {
        settabss(L, c"name".as_ptr(), ar.name);
        settabss(L, c"namewhat".as_ptr(), ar.namewhat);
    }
    if !(strchr(options, 'r' as i32)).is_null() {
        settabsi(L, c"ftransfer".as_ptr(), ar.ftransfer as i32);
        settabsi(L, c"ntransfer".as_ptr(), ar.ntransfer as i32);
    }
    if !(strchr(options, 't' as i32)).is_null() {
        settabsb(L, c"istailcall".as_ptr(), ar.istailcall as i32);
    }
    if !(strchr(options, 'L' as i32)).is_null() {
        treatstackoption(L, L1, c"activelines".as_ptr());
    }
    if !(strchr(options, 'f' as i32)).is_null() {
        treatstackoption(L, L1, c"func".as_ptr());
    }
    return 1 as i32;
}
unsafe extern "C-unwind" fn db_getlocal(mut L: *mut lua_State) -> i32 {
    let mut arg: i32 = 0;
    let mut L1: *mut lua_State = getthread(L, &mut arg);
    let mut nvar: i32 = luaL_checkinteger(L, arg + 2 as i32) as i32;
    if lua_type(L, arg + 1 as i32) == 6 as i32 {
        lua_pushvalue(L, arg + 1 as i32);
        lua_pushstring(L, lua_getlocal(L, 0 as *const lua_Debug, nvar));
        return 1 as i32;
    } else {
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
        let mut name: *const std::ffi::c_char = 0 as *const std::ffi::c_char;
        let mut level: i32 = luaL_checkinteger(L, arg + 1 as i32) as i32;
        if ((lua_getstack(L1, level, &mut ar) == 0) as i32 != 0) as i32 as std::ffi::c_long != 0 {
            return luaL_argerror(L, arg + 1 as i32, c"level out of range".as_ptr());
        }
        checkstack(L, L1, 1 as i32);
        name = lua_getlocal(L1, &mut ar, nvar);
        if !name.is_null() {
            lua_xmove(L1, L, 1 as i32);
            lua_pushstring(L, name);
            lua_rotate(L, -(2 as i32), 1 as i32);
            return 2 as i32;
        } else {
            lua_pushnil(L);
            return 1 as i32;
        }
    };
}
unsafe extern "C-unwind" fn db_setlocal(mut L: *mut lua_State) -> i32 {
    let mut arg: i32 = 0;
    let mut name: *const std::ffi::c_char = 0 as *const std::ffi::c_char;
    let mut L1: *mut lua_State = getthread(L, &mut arg);
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
    let mut level: i32 = luaL_checkinteger(L, arg + 1 as i32) as i32;
    let mut nvar: i32 = luaL_checkinteger(L, arg + 2 as i32) as i32;
    if ((lua_getstack(L1, level, &mut ar) == 0) as i32 != 0) as i32 as std::ffi::c_long != 0 {
        return luaL_argerror(L, arg + 1 as i32, c"level out of range".as_ptr());
    }
    luaL_checkany(L, arg + 3 as i32);
    lua_settop(L, arg + 3 as i32);
    checkstack(L, L1, 1 as i32);
    lua_xmove(L, L1, 1 as i32);
    name = lua_setlocal(L1, &mut ar, nvar);
    if name.is_null() {
        lua_settop(L1, -(1 as i32) - 1 as i32);
    }
    lua_pushstring(L, name);
    return 1 as i32;
}
unsafe extern "C-unwind" fn auxupvalue(mut L: *mut lua_State, mut get: i32) -> i32 {
    let mut name: *const std::ffi::c_char = 0 as *const std::ffi::c_char;
    let mut n: i32 = luaL_checkinteger(L, 2 as i32) as i32;
    luaL_checktype(L, 1 as i32, 6 as i32);
    name = if get != 0 {
        lua_getupvalue(L, 1 as i32, n)
    } else {
        lua_setupvalue(L, 1 as i32, n)
    };
    if name.is_null() {
        return 0;
    }
    lua_pushstring(L, name);
    lua_rotate(L, -(get + 1 as i32), 1 as i32);
    return get + 1 as i32;
}
unsafe extern "C-unwind" fn db_getupvalue(mut L: *mut lua_State) -> i32 {
    return auxupvalue(L, 1 as i32);
}
unsafe extern "C-unwind" fn db_setupvalue(mut L: *mut lua_State) -> i32 {
    luaL_checkany(L, 3 as i32);
    return auxupvalue(L, 0);
}
unsafe extern "C-unwind" fn checkupval(
    mut L: *mut lua_State,
    mut argf: i32,
    mut argnup: i32,
    mut pnup: *mut i32,
) -> *mut c_void {
    let mut id: *mut c_void = 0 as *mut c_void;
    let mut nup: i32 = luaL_checkinteger(L, argnup) as i32;
    luaL_checktype(L, argf, 6 as i32);
    id = lua_upvalueid(L, argf, nup);
    if !pnup.is_null() {
        (((id != 0 as *mut c_void) as i32 != 0) as i32 as std::ffi::c_long != 0
            || luaL_argerror(L, argnup, c"invalid upvalue index".as_ptr()) != 0) as i32;
        *pnup = nup;
    }
    return id;
}
unsafe extern "C-unwind" fn db_upvalueid(mut L: *mut lua_State) -> i32 {
    let mut id: *mut c_void = checkupval(L, 1 as i32, 2 as i32, 0 as *mut i32);
    if !id.is_null() {
        lua_pushlightuserdata(L, id);
    } else {
        lua_pushnil(L);
    }
    return 1 as i32;
}
unsafe extern "C-unwind" fn db_upvaluejoin(mut L: *mut lua_State) -> i32 {
    let mut n1: i32 = 0;
    let mut n2: i32 = 0;
    checkupval(L, 1 as i32, 2 as i32, &mut n1);
    checkupval(L, 3 as i32, 4 as i32, &mut n2);
    (((lua_iscfunction(L, 1 as i32) == 0) as i32 != 0) as i32 as std::ffi::c_long != 0
        || luaL_argerror(L, 1 as i32, c"Lua function expected".as_ptr()) != 0) as i32;
    (((lua_iscfunction(L, 3 as i32) == 0) as i32 != 0) as i32 as std::ffi::c_long != 0
        || luaL_argerror(L, 3 as i32, c"Lua function expected".as_ptr()) != 0) as i32;
    lua_upvaluejoin(L, 1 as i32, n1, 3 as i32, n2);
    return 0;
}
unsafe extern "C-unwind" fn hookf(mut L: *mut lua_State, mut ar: *mut lua_Debug) {
    static mut hooknames: [*const std::ffi::c_char; 5] = [
        c"call".as_ptr(),
        c"return".as_ptr(),
        c"line".as_ptr(),
        c"count".as_ptr(),
        c"tail call".as_ptr(),
    ];
    lua_getfield(L, -(1000000) - 1000, HOOKKEY);
    lua_pushthread(L);
    if lua_rawget(L, -(2 as i32)) == 6 as i32 {
        lua_pushstring(L, hooknames[(*ar).event as usize]);
        if (*ar).currentline >= 0 {
            lua_pushinteger(L, (*ar).currentline as lua_Integer);
        } else {
            lua_pushnil(L);
        }
        lua_callk(L, 2 as i32, 0, 0 as lua_KContext, None);
    }
}
unsafe extern "C-unwind" fn makemask(mut smask: *const std::ffi::c_char, mut count: i32) -> i32 {
    let mut mask: i32 = 0;
    if !(strchr(smask, 'c' as i32)).is_null() {
        mask |= (1 as i32) << 0;
    }
    if !(strchr(smask, 'r' as i32)).is_null() {
        mask |= (1 as i32) << 1 as i32;
    }
    if !(strchr(smask, 'l' as i32)).is_null() {
        mask |= (1 as i32) << 2 as i32;
    }
    if count > 0 {
        mask |= (1 as i32) << 3 as i32;
    }
    return mask;
}
unsafe extern "C-unwind" fn unmakemask(
    mut mask: i32,
    mut smask: *mut std::ffi::c_char,
) -> *mut std::ffi::c_char {
    let mut i: i32 = 0;
    if mask & (1 as i32) << 0 != 0 {
        let fresh151 = i;
        i = i + 1;
        *smask.offset(fresh151 as isize) = 'c' as i32 as std::ffi::c_char;
    }
    if mask & (1 as i32) << 1 as i32 != 0 {
        let fresh152 = i;
        i = i + 1;
        *smask.offset(fresh152 as isize) = 'r' as i32 as std::ffi::c_char;
    }
    if mask & (1 as i32) << 2 as i32 != 0 {
        let fresh153 = i;
        i = i + 1;
        *smask.offset(fresh153 as isize) = 'l' as i32 as std::ffi::c_char;
    }
    *smask.offset(i as isize) = '\0' as i32 as std::ffi::c_char;
    return smask;
}
unsafe extern "C-unwind" fn db_sethook(mut L: *mut lua_State) -> i32 {
    let mut arg: i32 = 0;
    let mut mask: i32 = 0;
    let mut count: i32 = 0;
    let mut func: lua_Hook = None;
    let mut L1: *mut lua_State = getthread(L, &mut arg);
    if lua_type(L, arg + 1 as i32) <= 0 {
        lua_settop(L, arg + 1 as i32);
        func = None;
        mask = 0;
        count = 0;
    } else {
        let mut smask: *const std::ffi::c_char =
            luaL_checklstring(L, arg + 2 as i32, 0 as *mut size_t);
        luaL_checktype(L, arg + 1 as i32, 6 as i32);
        count = luaL_optinteger(L, arg + 3 as i32, 0 as lua_Integer) as i32;
        func = Some(hookf as unsafe extern "C-unwind" fn(*mut lua_State, *mut lua_Debug) -> ());
        mask = makemask(smask, count);
    }
    if luaL_getsubtable(L, -(1000000) - 1000, HOOKKEY) == 0 {
        lua_pushstring(L, c"k".as_ptr());
        lua_setfield(L, -(2 as i32), c"__mode".as_ptr());
        lua_pushvalue(L, -(1 as i32));
        lua_setmetatable(L, -(2 as i32));
    }
    checkstack(L, L1, 1 as i32);
    lua_pushthread(L1);
    lua_xmove(L1, L, 1 as i32);
    lua_pushvalue(L, arg + 1 as i32);
    lua_rawset(L, -(3 as i32));
    lua_sethook(L1, func, mask, count);
    return 0;
}
unsafe extern "C-unwind" fn db_gethook(mut L: *mut lua_State) -> i32 {
    let mut arg: i32 = 0;
    let mut L1: *mut lua_State = getthread(L, &mut arg);
    let mut buff: [std::ffi::c_char; 5] = [0; 5];
    let mut mask: i32 = lua_gethookmask(L1);
    let mut hook: lua_Hook = lua_gethook(L1);
    if hook.is_none() {
        lua_pushnil(L);
        return 1 as i32;
    } else if hook
        != Some(hookf as unsafe extern "C-unwind" fn(*mut lua_State, *mut lua_Debug) -> ())
    {
        lua_pushstring(L, c"external hook".as_ptr());
    } else {
        lua_getfield(L, -(1000000) - 1000, HOOKKEY);
        checkstack(L, L1, 1 as i32);
        lua_pushthread(L1);
        lua_xmove(L1, L, 1 as i32);
        lua_rawget(L, -(2 as i32));
        lua_rotate(L, -(2 as i32), -(1 as i32));
        lua_settop(L, -(1 as i32) - 1 as i32);
    }
    lua_pushstring(L, unmakemask(mask, buff.as_mut_ptr()));
    lua_pushinteger(L, lua_gethookcount(L1) as lua_Integer);
    return 3 as i32;
}
unsafe extern "C-unwind" fn db_debug(mut L: *mut lua_State) -> i32 {
    loop {
        let mut buffer: [std::ffi::c_char; 250] = [0; 250];
        fprintf(stderr, c"%s".as_ptr(), c"lua_debug> ".as_ptr());
        fflush(stderr);
        if (fgets(
            buffer.as_mut_ptr(),
            size_of::<[std::ffi::c_char; 250]>() as usize as i32,
            stdin,
        ))
        .is_null()
            || strcmp(buffer.as_mut_ptr(), c"cont\n".as_ptr()) == 0
        {
            return 0;
        }
        if luaL_loadbufferx(
            L,
            buffer.as_mut_ptr(),
            strlen(buffer.as_mut_ptr()),
            c"=(debug command)".as_ptr(),
            0 as *const std::ffi::c_char,
        ) != 0
            || lua_pcallk(L, 0, 0, 0, 0 as lua_KContext, None) != 0
        {
            fprintf(
                stderr,
                c"%s\n".as_ptr(),
                luaL_tolstring(L, -(1 as i32), 0 as *mut size_t),
            );
            fflush(stderr);
        }
        lua_settop(L, 0);
    }
}
unsafe extern "C-unwind" fn db_traceback(mut L: *mut lua_State) -> i32 {
    let mut arg: i32 = 0;
    let mut L1: *mut lua_State = getthread(L, &mut arg);
    let mut msg: *const std::ffi::c_char = lua_tolstring(L, arg + 1 as i32, 0 as *mut size_t);
    if msg.is_null() && !(lua_type(L, arg + 1 as i32) <= 0) {
        lua_pushvalue(L, arg + 1 as i32);
    } else {
        let mut level: i32 = luaL_optinteger(
            L,
            arg + 2 as i32,
            (if L == L1 { 1 as i32 } else { 0 }) as lua_Integer,
        ) as i32;
        luaL_traceback(L, L1, msg, level);
    }
    return 1 as i32;
}
unsafe extern "C-unwind" fn db_setcstacklimit(mut L: *mut lua_State) -> i32 {
    let mut limit: i32 = luaL_checkinteger(L, 1 as i32) as i32;
    let mut res: i32 = lua_setcstacklimit(L, limit as u32);
    lua_pushinteger(L, res as lua_Integer);
    return 1 as i32;
}
static mut dblib: [luaL_Reg; 18] = unsafe {
    [
        {
            let mut init = luaL_Reg {
                name: c"debug".as_ptr(),
                func: Some(db_debug as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"getuservalue".as_ptr(),
                func: Some(db_getuservalue as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"gethook".as_ptr(),
                func: Some(db_gethook as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"getinfo".as_ptr(),
                func: Some(db_getinfo as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"getlocal".as_ptr(),
                func: Some(db_getlocal as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"getregistry".as_ptr(),
                func: Some(db_getregistry as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"getmetatable".as_ptr(),
                func: Some(db_getmetatable as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"getupvalue".as_ptr(),
                func: Some(db_getupvalue as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"upvaluejoin".as_ptr(),
                func: Some(db_upvaluejoin as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"upvalueid".as_ptr(),
                func: Some(db_upvalueid as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"setuservalue".as_ptr(),
                func: Some(db_setuservalue as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"sethook".as_ptr(),
                func: Some(db_sethook as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"setlocal".as_ptr(),
                func: Some(db_setlocal as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"setmetatable".as_ptr(),
                func: Some(db_setmetatable as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"setupvalue".as_ptr(),
                func: Some(db_setupvalue as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"traceback".as_ptr(),
                func: Some(db_traceback as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"setcstacklimit".as_ptr(),
                func: Some(db_setcstacklimit as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: 0 as *const std::ffi::c_char,
                func: None,
            };
            init
        },
    ]
};
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaopen_debug(mut L: *mut lua_State) -> i32 {
    luaL_checkversion_(
        L,
        504 as i32 as lua_Number,
        (size_of::<lua_Integer>() as usize)
            .wrapping_mul(16)
            .wrapping_add(size_of::<lua_Number>() as usize),
    );
    lua_createtable(
        L,
        0,
        (size_of::<[luaL_Reg; 18]>() as usize)
            .wrapping_div(size_of::<luaL_Reg>() as usize)
            .wrapping_sub(1) as i32,
    );
    luaL_setfuncs(L, (&raw const dblib).cast(), 0);
    return 1 as i32;
}
unsafe extern "C-unwind" fn l_checkmode(mut mode: *const std::ffi::c_char) -> i32 {
    return (*mode as i32 != '\0' as i32
        && {
            let fresh154 = mode;
            mode = mode.offset(1);
            !(strchr(c"rwa".as_ptr(), *fresh154 as i32)).is_null()
        }
        && (*mode as i32 != '+' as i32 || {
            mode = mode.offset(1);
            mode;
            1 as i32 != 0
        })
        && strspn(mode, c"b".as_ptr()) == strlen(mode)) as i32;
}
unsafe extern "C-unwind" fn io_type(mut L: *mut lua_State) -> i32 {
    let mut p: *mut LStream = 0 as *mut LStream;
    luaL_checkany(L, 1 as i32);
    p = luaL_testudata(L, 1 as i32, c"FILE*".as_ptr()) as *mut LStream;
    if p.is_null() {
        lua_pushnil(L);
    } else if ((*p).closef).is_none() {
        lua_pushstring(L, c"closed file".as_ptr());
    } else {
        lua_pushstring(L, c"file".as_ptr());
    }
    return 1 as i32;
}
unsafe extern "C-unwind" fn f_tostring(mut L: *mut lua_State) -> i32 {
    let mut p: *mut LStream = luaL_checkudata(L, 1 as i32, c"FILE*".as_ptr()) as *mut LStream;
    if ((*p).closef).is_none() {
        lua_pushstring(L, c"file (closed)".as_ptr());
    } else {
        lua_pushfstring(L, c"file (%p)".as_ptr(), (*p).f);
    }
    return 1 as i32;
}
unsafe extern "C-unwind" fn tofile(mut L: *mut lua_State) -> *mut FILE {
    let mut p: *mut LStream = luaL_checkudata(L, 1 as i32, c"FILE*".as_ptr()) as *mut LStream;
    if (((*p).closef).is_none() as i32 != 0) as i32 as std::ffi::c_long != 0 {
        luaL_error(L, c"attempt to use a closed file".as_ptr());
    }
    return (*p).f;
}
unsafe extern "C-unwind" fn newprefile(mut L: *mut lua_State) -> *mut LStream {
    let mut p: *mut LStream =
        lua_newuserdatauv(L, size_of::<LStream>() as usize, 0) as *mut LStream;
    (*p).closef = None;
    luaL_setmetatable(L, c"FILE*".as_ptr());
    return p;
}
unsafe extern "C-unwind" fn aux_close(mut L: *mut lua_State) -> i32 {
    let mut p: *mut LStream = luaL_checkudata(L, 1 as i32, c"FILE*".as_ptr()) as *mut LStream;
    let mut cf: lua_CFunction = (*p).closef;
    (*p).closef = None;
    return (Some(cf.expect("non-null function pointer"))).expect("non-null function pointer")(L);
}
unsafe extern "C-unwind" fn f_close(mut L: *mut lua_State) -> i32 {
    tofile(L);
    return aux_close(L);
}
unsafe extern "C-unwind" fn io_close(mut L: *mut lua_State) -> i32 {
    if lua_type(L, 1 as i32) == -(1 as i32) {
        lua_getfield(L, -(1000000) - 1000, c"_IO_output".as_ptr());
    }
    return f_close(L);
}
unsafe extern "C-unwind" fn f_gc(mut L: *mut lua_State) -> i32 {
    let mut p: *mut LStream = luaL_checkudata(L, 1 as i32, c"FILE*".as_ptr()) as *mut LStream;
    if ((*p).closef).is_some() && !((*p).f).is_null() {
        aux_close(L);
    }
    return 0;
}
unsafe extern "C-unwind" fn io_fclose(mut L: *mut lua_State) -> i32 {
    let mut p: *mut LStream = luaL_checkudata(L, 1 as i32, c"FILE*".as_ptr()) as *mut LStream;
    *__errno_location() = 0;
    return luaL_fileresult(
        L,
        (fclose((*p).f) == 0) as i32,
        0 as *const std::ffi::c_char,
    );
}
unsafe extern "C-unwind" fn newfile(mut L: *mut lua_State) -> *mut LStream {
    let mut p: *mut LStream = newprefile(L);
    (*p).f = 0 as *mut FILE;
    (*p).closef = Some(io_fclose as unsafe extern "C-unwind" fn(*mut lua_State) -> i32);
    return p;
}
unsafe extern "C-unwind" fn opencheck(
    mut L: *mut lua_State,
    mut fname: *const std::ffi::c_char,
    mut mode: *const std::ffi::c_char,
) {
    let mut p: *mut LStream = newfile(L);
    (*p).f = fopen(fname, mode);
    if (((*p).f == 0 as *mut c_void as *mut FILE) as i32 != 0) as i32 as std::ffi::c_long != 0 {
        luaL_error(
            L,
            c"cannot open file '%s' (%s)".as_ptr(),
            fname,
            strerror(*__errno_location()),
        );
    }
}
unsafe extern "C-unwind" fn io_open(mut L: *mut lua_State) -> i32 {
    let mut filename: *const std::ffi::c_char = luaL_checklstring(L, 1 as i32, 0 as *mut size_t);
    let mut mode: *const std::ffi::c_char =
        luaL_optlstring(L, 2 as i32, c"r".as_ptr(), 0 as *mut size_t);
    let mut p: *mut LStream = newfile(L);
    let mut md: *const std::ffi::c_char = mode;
    ((l_checkmode(md) != 0) as i32 as std::ffi::c_long != 0
        || luaL_argerror(L, 2 as i32, c"invalid mode".as_ptr()) != 0) as i32;
    *__errno_location() = 0;
    (*p).f = fopen(filename, mode);
    return if ((*p).f).is_null() {
        luaL_fileresult(L, 0, filename)
    } else {
        1 as i32
    };
}
unsafe extern "C-unwind" fn io_pclose(mut L: *mut lua_State) -> i32 {
    let mut p: *mut LStream = luaL_checkudata(L, 1 as i32, c"FILE*".as_ptr()) as *mut LStream;
    *__errno_location() = 0;
    return luaL_execresult(L, pclose((*p).f));
}
unsafe extern "C-unwind" fn io_popen(mut L: *mut lua_State) -> i32 {
    let mut filename: *const std::ffi::c_char = luaL_checklstring(L, 1 as i32, 0 as *mut size_t);
    let mut mode: *const std::ffi::c_char =
        luaL_optlstring(L, 2 as i32, c"r".as_ptr(), 0 as *mut size_t);
    let mut p: *mut LStream = newprefile(L);
    ((((*mode.offset(0 as isize) as i32 == 'r' as i32
        || *mode.offset(0 as isize) as i32 == 'w' as i32)
        && *mode.offset(1) as i32 == '\0' as i32) as i32
        != 0) as i32 as std::ffi::c_long
        != 0
        || luaL_argerror(L, 2 as i32, c"invalid mode".as_ptr()) != 0) as i32;
    *__errno_location() = 0;
    fflush(0 as *mut FILE);
    (*p).f = popen(filename, mode);
    (*p).closef = Some(io_pclose as unsafe extern "C-unwind" fn(*mut lua_State) -> i32);
    return if ((*p).f).is_null() {
        luaL_fileresult(L, 0, filename)
    } else {
        1 as i32
    };
}
unsafe extern "C-unwind" fn io_tmpfile(mut L: *mut lua_State) -> i32 {
    let mut p: *mut LStream = newfile(L);
    *__errno_location() = 0;
    (*p).f = tmpfile();
    return if ((*p).f).is_null() {
        luaL_fileresult(L, 0, 0 as *const std::ffi::c_char)
    } else {
        1 as i32
    };
}
unsafe extern "C-unwind" fn getiofile(
    mut L: *mut lua_State,
    mut findex: *const std::ffi::c_char,
) -> *mut FILE {
    let mut p: *mut LStream = 0 as *mut LStream;
    lua_getfield(L, -(1000000) - 1000, findex);
    p = lua_touserdata(L, -(1 as i32)) as *mut LStream;
    if (((*p).closef).is_none() as i32 != 0) as i32 as std::ffi::c_long != 0 {
        luaL_error(
            L,
            c"default %s file is closed".as_ptr(),
            findex.offset(
                (size_of::<[std::ffi::c_char; 5]>() as usize)
                    .wrapping_div(size_of::<std::ffi::c_char>() as usize)
                    .wrapping_sub(1) as isize,
            ),
        );
    }
    return (*p).f;
}
unsafe extern "C-unwind" fn g_iofile(
    mut L: *mut lua_State,
    mut f: *const std::ffi::c_char,
    mut mode: *const std::ffi::c_char,
) -> i32 {
    if !(lua_type(L, 1 as i32) <= 0) {
        let mut filename: *const std::ffi::c_char = lua_tolstring(L, 1 as i32, 0 as *mut size_t);
        if !filename.is_null() {
            opencheck(L, filename, mode);
        } else {
            tofile(L);
            lua_pushvalue(L, 1 as i32);
        }
        lua_setfield(L, -(1000000) - 1000, f);
    }
    lua_getfield(L, -(1000000) - 1000, f);
    return 1 as i32;
}
unsafe extern "C-unwind" fn io_input(mut L: *mut lua_State) -> i32 {
    return g_iofile(L, c"_IO_input".as_ptr(), c"r".as_ptr());
}
unsafe extern "C-unwind" fn io_output(mut L: *mut lua_State) -> i32 {
    return g_iofile(L, c"_IO_output".as_ptr(), c"w".as_ptr());
}
unsafe extern "C-unwind" fn aux_lines(mut L: *mut lua_State, mut toclose: i32) {
    let mut n: i32 = lua_gettop(L) - 1 as i32;
    (((n <= 250) as i32 != 0) as i32 as std::ffi::c_long != 0
        || luaL_argerror(L, 250 + 2 as i32, c"too many arguments".as_ptr()) != 0) as i32;
    lua_pushvalue(L, 1 as i32);
    lua_pushinteger(L, n as lua_Integer);
    lua_pushboolean(L, toclose);
    lua_rotate(L, 2 as i32, 3 as i32);
    lua_pushcclosure(
        L,
        Some(io_readline as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
        3 as i32 + n,
    );
}
unsafe extern "C-unwind" fn f_lines(mut L: *mut lua_State) -> i32 {
    tofile(L);
    aux_lines(L, 0);
    return 1 as i32;
}
unsafe extern "C-unwind" fn io_lines(mut L: *mut lua_State) -> i32 {
    let mut toclose: i32 = 0;
    if lua_type(L, 1 as i32) == -(1 as i32) {
        lua_pushnil(L);
    }
    if lua_type(L, 1 as i32) == 0 {
        lua_getfield(L, -(1000000) - 1000, c"_IO_input".as_ptr());
        lua_copy(L, -(1 as i32), 1 as i32);
        lua_settop(L, -(1 as i32) - 1 as i32);
        tofile(L);
        toclose = 0;
    } else {
        let mut filename: *const std::ffi::c_char =
            luaL_checklstring(L, 1 as i32, 0 as *mut size_t);
        opencheck(L, filename, c"r".as_ptr());
        lua_copy(L, -(1 as i32), 1 as i32);
        lua_settop(L, -(1 as i32) - 1 as i32);
        toclose = 1 as i32;
    }
    aux_lines(L, toclose);
    if toclose != 0 {
        lua_pushnil(L);
        lua_pushnil(L);
        lua_pushvalue(L, 1 as i32);
        return 4 as i32;
    } else {
        return 1 as i32;
    };
}
unsafe extern "C-unwind" fn nextc(mut rn: *mut RN) -> i32 {
    if (((*rn).n >= 200) as i32 != 0) as i32 as std::ffi::c_long != 0 {
        (*rn).buff[0 as usize] = '\0' as i32 as std::ffi::c_char;
        return 0;
    } else {
        let fresh155 = (*rn).n;
        (*rn).n = (*rn).n + 1;
        (*rn).buff[fresh155 as usize] = (*rn).c as std::ffi::c_char;
        (*rn).c = getc_unlocked((*rn).f);
        return 1 as i32;
    };
}
unsafe extern "C-unwind" fn test2(mut rn: *mut RN, mut set: *const std::ffi::c_char) -> i32 {
    if (*rn).c == *set.offset(0 as isize) as i32 || (*rn).c == *set.offset(1) as i32 {
        return nextc(rn);
    } else {
        return 0;
    };
}
unsafe extern "C-unwind" fn readdigits(mut rn: *mut RN, mut hex: i32) -> i32 {
    let mut count: i32 = 0;
    while (if hex != 0 {
        *(*__ctype_b_loc()).offset((*rn).c as isize) as i32 & _ISxdigit as i32 as u16 as i32
    } else {
        *(*__ctype_b_loc()).offset((*rn).c as isize) as i32 & _ISdigit as i32 as u16 as i32
    }) != 0
        && nextc(rn) != 0
    {
        count += 1;
        count;
    }
    return count;
}
unsafe extern "C-unwind" fn read_number(mut L: *mut lua_State, mut f: *mut FILE) -> i32 {
    let mut rn: RN = RN {
        f: 0 as *mut FILE,
        c: 0,
        n: 0,
        buff: [0; 201],
    };
    let mut count: i32 = 0;
    let mut hex: i32 = 0;
    let mut decp: [std::ffi::c_char; 2] = [0; 2];
    rn.f = f;
    rn.n = 0;
    decp[0 as usize] = *((*localeconv()).decimal_point).offset(0 as isize);
    decp[1] = '.' as i32 as std::ffi::c_char;
    flockfile(rn.f);
    loop {
        rn.c = getc_unlocked(rn.f);
        if !(*(*__ctype_b_loc()).offset(rn.c as isize) as i32 & _ISspace as i32 as u16 as i32 != 0)
        {
            break;
        }
    }
    test2(&mut rn, c"-+".as_ptr());
    if test2(&mut rn, c"00".as_ptr()) != 0 {
        if test2(&mut rn, c"xX".as_ptr()) != 0 {
            hex = 1 as i32;
        } else {
            count = 1 as i32;
        }
    }
    count += readdigits(&mut rn, hex);
    if test2(&mut rn, decp.as_mut_ptr()) != 0 {
        count += readdigits(&mut rn, hex);
    }
    if count > 0
        && test2(
            &mut rn,
            (if hex != 0 {
                c"pP".as_ptr()
            } else {
                c"eE".as_ptr()
            }),
        ) != 0
    {
        test2(&mut rn, c"-+".as_ptr());
        readdigits(&mut rn, 0);
    }
    ungetc(rn.c, rn.f);
    funlockfile(rn.f);
    rn.buff[rn.n as usize] = '\0' as i32 as std::ffi::c_char;
    if (lua_stringtonumber(L, (rn.buff).as_mut_ptr()) != 0 as size_t) as i32 as std::ffi::c_long
        != 0
    {
        return 1 as i32;
    } else {
        lua_pushnil(L);
        return 0;
    };
}
unsafe extern "C-unwind" fn test_eof(mut L: *mut lua_State, mut f: *mut FILE) -> i32 {
    let mut c: i32 = getc(f);
    ungetc(c, f);
    lua_pushstring(L, c"".as_ptr());
    return (c != -(1 as i32)) as i32;
}
unsafe extern "C-unwind" fn read_line(
    mut L: *mut lua_State,
    mut f: *mut FILE,
    mut chop: i32,
) -> i32 {
    let mut b: luaL_Buffer = luaL_Buffer {
        b: 0 as *mut std::ffi::c_char,
        size: 0,
        n: 0,
        L: 0 as *mut lua_State,
        init: C2RustUnnamed_15 { n: 0. },
    };
    let mut c: i32 = 0;
    luaL_buffinit(L, &mut b);
    loop {
        let mut buff: *mut std::ffi::c_char = luaL_prepbuffsize(
            &mut b,
            (16usize)
                .wrapping_mul(size_of::<*mut c_void>() as usize)
                .wrapping_mul(size_of::<lua_Number>() as usize) as i32 as size_t,
        );
        let mut i: i32 = 0;
        flockfile(f);
        while i
            < (16usize)
                .wrapping_mul(size_of::<*mut c_void>() as usize)
                .wrapping_mul(size_of::<lua_Number>() as usize) as i32
            && {
                c = getc_unlocked(f);
                c != -(1 as i32)
            }
            && c != '\n' as i32
        {
            let fresh156 = i;
            i = i + 1;
            *buff.offset(fresh156 as isize) = c as std::ffi::c_char;
        }
        funlockfile(f);
        b.n = (b.n).wrapping_add(i as size_t);
        if !(c != -(1 as i32) && c != '\n' as i32) {
            break;
        }
    }
    if chop == 0 && c == '\n' as i32 {
        (b.n < b.size || !(luaL_prepbuffsize(&mut b, 1 as i32 as size_t)).is_null()) as i32;
        let fresh157 = b.n;
        b.n = (b.n).wrapping_add(1);
        *(b.b).offset(fresh157 as isize) = c as std::ffi::c_char;
    }
    luaL_pushresult(&mut b);
    return (c == '\n' as i32 || lua_rawlen(L, -(1 as i32)) > 0 as lua_Unsigned) as i32;
}
unsafe extern "C-unwind" fn read_all(mut L: *mut lua_State, mut f: *mut FILE) {
    let mut nr: size_t = 0;
    let mut b: luaL_Buffer = luaL_Buffer {
        b: 0 as *mut std::ffi::c_char,
        size: 0,
        n: 0,
        L: 0 as *mut lua_State,
        init: C2RustUnnamed_15 { n: 0. },
    };
    luaL_buffinit(L, &mut b);
    loop {
        let mut p: *mut std::ffi::c_char = luaL_prepbuffsize(
            &mut b,
            (16usize)
                .wrapping_mul(size_of::<*mut c_void>() as usize)
                .wrapping_mul(size_of::<lua_Number>() as usize) as i32 as size_t,
        );
        nr = fread(
            p as *mut c_void,
            size_of::<std::ffi::c_char>() as usize,
            (16usize)
                .wrapping_mul(size_of::<*mut c_void>() as usize)
                .wrapping_mul(size_of::<lua_Number>() as usize) as i32 as usize,
            f,
        );
        b.n = (b.n).wrapping_add(nr);
        if !(nr
            == (16usize)
                .wrapping_mul(size_of::<*mut c_void>() as usize)
                .wrapping_mul(size_of::<lua_Number>() as usize) as i32 as size_t)
        {
            break;
        }
    }
    luaL_pushresult(&mut b);
}
unsafe extern "C-unwind" fn read_chars(
    mut L: *mut lua_State,
    mut f: *mut FILE,
    mut n: size_t,
) -> i32 {
    let mut nr: size_t = 0;
    let mut p: *mut std::ffi::c_char = 0 as *mut std::ffi::c_char;
    let mut b: luaL_Buffer = luaL_Buffer {
        b: 0 as *mut std::ffi::c_char,
        size: 0,
        n: 0,
        L: 0 as *mut lua_State,
        init: C2RustUnnamed_15 { n: 0. },
    };
    luaL_buffinit(L, &mut b);
    p = luaL_prepbuffsize(&mut b, n);
    nr = fread(
        p as *mut c_void,
        size_of::<std::ffi::c_char>() as usize,
        n,
        f,
    );
    b.n = (b.n).wrapping_add(nr);
    luaL_pushresult(&mut b);
    return (nr > 0 as size_t) as i32;
}
unsafe extern "C-unwind" fn g_read(mut L: *mut lua_State, mut f: *mut FILE, mut first: i32) -> i32 {
    let mut nargs: i32 = lua_gettop(L) - 1 as i32;
    let mut n: i32 = 0;
    let mut success: i32 = 0;
    clearerr(f);
    *__errno_location() = 0;
    if nargs == 0 {
        success = read_line(L, f, 1 as i32);
        n = first + 1 as i32;
    } else {
        luaL_checkstack(L, nargs + 20, c"too many arguments".as_ptr());
        success = 1 as i32;
        n = first;
        loop {
            let fresh158 = nargs;
            nargs = nargs - 1;
            if !(fresh158 != 0 && success != 0) {
                break;
            }
            if lua_type(L, n) == 3 as i32 {
                let mut l: size_t = luaL_checkinteger(L, n) as size_t;
                success = if l == 0 as size_t {
                    test_eof(L, f)
                } else {
                    read_chars(L, f, l)
                };
            } else {
                let mut p: *const std::ffi::c_char = luaL_checklstring(L, n, 0 as *mut size_t);
                if *p as i32 == '*' as i32 {
                    p = p.offset(1);
                    p;
                }
                match *p as i32 {
                    110 => {
                        success = read_number(L, f);
                    }
                    108 => {
                        success = read_line(L, f, 1 as i32);
                    }
                    76 => {
                        success = read_line(L, f, 0);
                    }
                    97 => {
                        read_all(L, f);
                        success = 1 as i32;
                    }
                    _ => {
                        return luaL_argerror(L, n, c"invalid format".as_ptr());
                    }
                }
            }
            n += 1;
            n;
        }
    }
    if ferror(f) != 0 {
        return luaL_fileresult(L, 0, 0 as *const std::ffi::c_char);
    }
    if success == 0 {
        lua_settop(L, -(1 as i32) - 1 as i32);
        lua_pushnil(L);
    }
    return n - first;
}
unsafe extern "C-unwind" fn io_read(mut L: *mut lua_State) -> i32 {
    return g_read(L, getiofile(L, c"_IO_input".as_ptr()), 1 as i32);
}
unsafe extern "C-unwind" fn f_read(mut L: *mut lua_State) -> i32 {
    return g_read(L, tofile(L), 2 as i32);
}
unsafe extern "C-unwind" fn io_readline(mut L: *mut lua_State) -> i32 {
    let mut p: *mut LStream = lua_touserdata(L, -(1000000) - 1000 - 1 as i32) as *mut LStream;
    let mut i: i32 = 0;
    let mut n: i32 = lua_tointegerx(L, -(1000000) - 1000 - 2 as i32, 0 as *mut i32) as i32;
    if ((*p).closef).is_none() {
        return luaL_error(L, c"file is already closed".as_ptr());
    }
    lua_settop(L, 1 as i32);
    luaL_checkstack(L, n, c"too many arguments".as_ptr());
    i = 1 as i32;
    while i <= n {
        lua_pushvalue(L, -(1000000) - 1000 - (3 as i32 + i));
        i += 1;
        i;
    }
    n = g_read(L, (*p).f, 2 as i32);
    if lua_toboolean(L, -n) != 0 {
        return n;
    } else {
        if n > 1 as i32 {
            return luaL_error(
                L,
                c"%s".as_ptr(),
                lua_tolstring(L, -n + 1 as i32, 0 as *mut size_t),
            );
        }
        if lua_toboolean(L, -(1000000) - 1000 - 3 as i32) != 0 {
            lua_settop(L, 0);
            lua_pushvalue(L, -(1000000) - 1000 - 1 as i32);
            aux_close(L);
        }
        return 0;
    };
}
unsafe extern "C-unwind" fn g_write(mut L: *mut lua_State, mut f: *mut FILE, mut arg: i32) -> i32 {
    let mut nargs: i32 = lua_gettop(L) - arg;
    let mut status: i32 = 1 as i32;
    *__errno_location() = 0;
    loop {
        let fresh159 = nargs;
        nargs = nargs - 1;
        if !(fresh159 != 0) {
            break;
        }
        if lua_type(L, arg) == 3 as i32 {
            let mut len: i32 = if lua_isinteger(L, arg) != 0 {
                fprintf(f, c"%lld".as_ptr(), lua_tointegerx(L, arg, 0 as *mut i32))
            } else {
                fprintf(f, c"%.14g".as_ptr(), lua_tonumberx(L, arg, 0 as *mut i32))
            };
            status = (status != 0 && len > 0) as i32;
        } else {
            let mut l: size_t = 0;
            let mut s: *const std::ffi::c_char = luaL_checklstring(L, arg, &mut l);
            status = (status != 0
                && fwrite(
                    s as *const c_void,
                    size_of::<std::ffi::c_char>() as usize,
                    l,
                    f,
                ) == l) as i32;
        }
        arg += 1;
        arg;
    }
    if (status != 0) as i32 as std::ffi::c_long != 0 {
        return 1 as i32;
    } else {
        return luaL_fileresult(L, status, 0 as *const std::ffi::c_char);
    };
}
unsafe extern "C-unwind" fn io_write(mut L: *mut lua_State) -> i32 {
    return g_write(L, getiofile(L, c"_IO_output".as_ptr()), 1 as i32);
}
unsafe extern "C-unwind" fn f_write(mut L: *mut lua_State) -> i32 {
    let mut f: *mut FILE = tofile(L);
    lua_pushvalue(L, 1 as i32);
    return g_write(L, f, 2 as i32);
}
unsafe extern "C-unwind" fn f_seek(mut L: *mut lua_State) -> i32 {
    static mut mode: [i32; 3] = [0, 1 as i32, 2 as i32];
    static mut modenames: [*const std::ffi::c_char; 4] = [
        c"set".as_ptr(),
        c"cur".as_ptr(),
        c"end".as_ptr(),
        0 as *const std::ffi::c_char,
    ];
    let mut f: *mut FILE = tofile(L);
    let mut op: i32 = luaL_checkoption(L, 2 as i32, c"cur".as_ptr(), (&raw const modenames).cast());
    let mut p3: lua_Integer = luaL_optinteger(L, 3 as i32, 0 as lua_Integer);
    let mut offset: off_t = p3 as off_t;
    (((offset as lua_Integer == p3) as i32 != 0) as i32 as std::ffi::c_long != 0
        || luaL_argerror(L, 3 as i32, c"not an integer in proper range".as_ptr()) != 0) as i32;
    *__errno_location() = 0;
    op = fseeko(f, offset, mode[op as usize]);
    if (op != 0) as i32 as std::ffi::c_long != 0 {
        return luaL_fileresult(L, 0, 0 as *const std::ffi::c_char);
    } else {
        lua_pushinteger(L, ftello(f) as lua_Integer);
        return 1 as i32;
    };
}
unsafe extern "C-unwind" fn f_setvbuf(mut L: *mut lua_State) -> i32 {
    static mut mode: [i32; 3] = [2 as i32, 0, 1 as i32];
    static mut modenames: [*const std::ffi::c_char; 4] = [
        c"no".as_ptr(),
        c"full".as_ptr(),
        c"line".as_ptr(),
        0 as *const std::ffi::c_char,
    ];
    let mut f: *mut FILE = tofile(L);
    let mut op: i32 = luaL_checkoption(
        L,
        2 as i32,
        0 as *const std::ffi::c_char,
        (&raw const modenames).cast(),
    );
    let mut sz: lua_Integer = luaL_optinteger(
        L,
        3 as i32,
        (16usize)
            .wrapping_mul(size_of::<*mut c_void>() as usize)
            .wrapping_mul(size_of::<lua_Number>() as usize) as i32 as lua_Integer,
    );
    let mut res: i32 = 0;
    *__errno_location() = 0;
    res = setvbuf(
        f,
        0 as *mut std::ffi::c_char,
        mode[op as usize],
        sz as size_t,
    );
    return luaL_fileresult(L, (res == 0) as i32, 0 as *const std::ffi::c_char);
}
unsafe extern "C-unwind" fn io_flush(mut L: *mut lua_State) -> i32 {
    let mut f: *mut FILE = getiofile(L, c"_IO_output".as_ptr());
    *__errno_location() = 0;
    return luaL_fileresult(L, (fflush(f) == 0) as i32, 0 as *const std::ffi::c_char);
}
unsafe extern "C-unwind" fn f_flush(mut L: *mut lua_State) -> i32 {
    let mut f: *mut FILE = tofile(L);
    *__errno_location() = 0;
    return luaL_fileresult(L, (fflush(f) == 0) as i32, 0 as *const std::ffi::c_char);
}
static mut iolib: [luaL_Reg; 12] = unsafe {
    [
        {
            let mut init = luaL_Reg {
                name: c"close".as_ptr(),
                func: Some(io_close as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"flush".as_ptr(),
                func: Some(io_flush as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"input".as_ptr(),
                func: Some(io_input as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"lines".as_ptr(),
                func: Some(io_lines as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"open".as_ptr(),
                func: Some(io_open as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"output".as_ptr(),
                func: Some(io_output as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"popen".as_ptr(),
                func: Some(io_popen as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"read".as_ptr(),
                func: Some(io_read as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"tmpfile".as_ptr(),
                func: Some(io_tmpfile as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"type".as_ptr(),
                func: Some(io_type as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"write".as_ptr(),
                func: Some(io_write as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: 0 as *const std::ffi::c_char,
                func: None,
            };
            init
        },
    ]
};
static mut meth: [luaL_Reg; 8] = unsafe {
    [
        {
            let mut init = luaL_Reg {
                name: c"read".as_ptr(),
                func: Some(f_read as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"write".as_ptr(),
                func: Some(f_write as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"lines".as_ptr(),
                func: Some(f_lines as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"flush".as_ptr(),
                func: Some(f_flush as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"seek".as_ptr(),
                func: Some(f_seek as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"close".as_ptr(),
                func: Some(f_close as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"setvbuf".as_ptr(),
                func: Some(f_setvbuf as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: 0 as *const std::ffi::c_char,
                func: None,
            };
            init
        },
    ]
};
static mut metameth: [luaL_Reg; 5] = unsafe {
    [
        {
            let mut init = luaL_Reg {
                name: c"__index".as_ptr(),
                func: None,
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"__gc".as_ptr(),
                func: Some(f_gc as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"__close".as_ptr(),
                func: Some(f_gc as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"__tostring".as_ptr(),
                func: Some(f_tostring as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: 0 as *const std::ffi::c_char,
                func: None,
            };
            init
        },
    ]
};
unsafe extern "C-unwind" fn createmeta(mut L: *mut lua_State) {
    luaL_newmetatable(L, c"FILE*".as_ptr());
    luaL_setfuncs(L, (&raw const metameth).cast(), 0);
    lua_createtable(
        L,
        0,
        (size_of::<[luaL_Reg; 8]>() as usize)
            .wrapping_div(size_of::<luaL_Reg>() as usize)
            .wrapping_sub(1) as i32,
    );
    luaL_setfuncs(L, (&raw const meth).cast(), 0);
    lua_setfield(L, -(2 as i32), c"__index".as_ptr());
    lua_settop(L, -(1 as i32) - 1 as i32);
}
unsafe extern "C-unwind" fn io_noclose(mut L: *mut lua_State) -> i32 {
    let mut p: *mut LStream = luaL_checkudata(L, 1 as i32, c"FILE*".as_ptr()) as *mut LStream;
    (*p).closef = Some(io_noclose as unsafe extern "C-unwind" fn(*mut lua_State) -> i32);
    lua_pushnil(L);
    lua_pushstring(L, c"cannot close standard file".as_ptr());
    return 2 as i32;
}
unsafe extern "C-unwind" fn createstdfile(
    mut L: *mut lua_State,
    mut f: *mut FILE,
    mut k: *const std::ffi::c_char,
    mut fname: *const std::ffi::c_char,
) {
    let mut p: *mut LStream = newprefile(L);
    (*p).f = f;
    (*p).closef = Some(io_noclose as unsafe extern "C-unwind" fn(*mut lua_State) -> i32);
    if !k.is_null() {
        lua_pushvalue(L, -(1 as i32));
        lua_setfield(L, -(1000000) - 1000, k);
    }
    lua_setfield(L, -(2 as i32), fname);
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaopen_io(mut L: *mut lua_State) -> i32 {
    luaL_checkversion_(
        L,
        504 as i32 as lua_Number,
        (size_of::<lua_Integer>() as usize)
            .wrapping_mul(16)
            .wrapping_add(size_of::<lua_Number>() as usize),
    );
    lua_createtable(
        L,
        0,
        (size_of::<[luaL_Reg; 12]>() as usize)
            .wrapping_div(size_of::<luaL_Reg>() as usize)
            .wrapping_sub(1) as i32,
    );
    luaL_setfuncs(L, (&raw const iolib).cast(), 0);
    createmeta(L);
    createstdfile(L, stdin, c"_IO_input".as_ptr(), c"stdin".as_ptr());
    createstdfile(L, stdout, c"_IO_output".as_ptr(), c"stdout".as_ptr());
    createstdfile(L, stderr, 0 as *const std::ffi::c_char, c"stderr".as_ptr());
    return 1 as i32;
}
unsafe extern "C-unwind" fn math_abs(mut L: *mut lua_State) -> i32 {
    if lua_isinteger(L, 1 as i32) != 0 {
        let mut n: lua_Integer = lua_tointegerx(L, 1 as i32, 0 as *mut i32);
        if n < 0 as lua_Integer {
            n = (0 as u32 as lua_Unsigned).wrapping_sub(n as lua_Unsigned) as lua_Integer;
        }
        lua_pushinteger(L, n);
    } else {
        lua_pushnumber(L, luaL_checknumber(L, 1 as i32).abs());
    }
    return 1 as i32;
}
unsafe extern "C-unwind" fn math_sin(mut L: *mut lua_State) -> i32 {
    lua_pushnumber(L, luaL_checknumber(L, 1 as i32).sin());
    return 1 as i32;
}
unsafe extern "C-unwind" fn math_cos(mut L: *mut lua_State) -> i32 {
    lua_pushnumber(L, luaL_checknumber(L, 1 as i32).cos());
    return 1 as i32;
}
unsafe extern "C-unwind" fn math_tan(mut L: *mut lua_State) -> i32 {
    lua_pushnumber(L, luaL_checknumber(L, 1 as i32).tan());
    return 1 as i32;
}
unsafe extern "C-unwind" fn math_asin(mut L: *mut lua_State) -> i32 {
    lua_pushnumber(L, luaL_checknumber(L, 1 as i32).asin());
    return 1 as i32;
}
unsafe extern "C-unwind" fn math_acos(mut L: *mut lua_State) -> i32 {
    lua_pushnumber(L, luaL_checknumber(L, 1 as i32).acos());
    return 1 as i32;
}
unsafe extern "C-unwind" fn math_atan(mut L: *mut lua_State) -> i32 {
    let mut y: lua_Number = luaL_checknumber(L, 1 as i32);
    let mut x: lua_Number = luaL_optnumber(L, 2 as i32, 1 as i32 as lua_Number);
    lua_pushnumber(L, y.atan2(x));
    return 1 as i32;
}
unsafe extern "C-unwind" fn math_toint(mut L: *mut lua_State) -> i32 {
    let mut valid: i32 = 0;
    let mut n: lua_Integer = lua_tointegerx(L, 1 as i32, &mut valid);
    if (valid != 0) as i32 as std::ffi::c_long != 0 {
        lua_pushinteger(L, n);
    } else {
        luaL_checkany(L, 1 as i32);
        lua_pushnil(L);
    }
    return 1 as i32;
}
unsafe extern "C-unwind" fn pushnumint(mut L: *mut lua_State, mut d: lua_Number) {
    let mut n: lua_Integer = 0;
    if d >= (-(9223372036854775807 as std::ffi::c_longlong) - 1 as std::ffi::c_longlong)
        as std::ffi::c_double
        && d < -((-(9223372036854775807 as std::ffi::c_longlong) - 1 as std::ffi::c_longlong)
            as std::ffi::c_double)
        && {
            n = d as std::ffi::c_longlong;
            1 as i32 != 0
        }
    {
        lua_pushinteger(L, n);
    } else {
        lua_pushnumber(L, d);
    };
}
unsafe extern "C-unwind" fn math_floor(mut L: *mut lua_State) -> i32 {
    if lua_isinteger(L, 1 as i32) != 0 {
        lua_settop(L, 1 as i32);
    } else {
        let mut d: lua_Number = luaL_checknumber(L, 1 as i32).floor();
        pushnumint(L, d);
    }
    return 1 as i32;
}
unsafe extern "C-unwind" fn math_ceil(mut L: *mut lua_State) -> i32 {
    if lua_isinteger(L, 1 as i32) != 0 {
        lua_settop(L, 1 as i32);
    } else {
        let mut d: lua_Number = luaL_checknumber(L, 1 as i32).ceil();
        pushnumint(L, d);
    }
    return 1 as i32;
}
unsafe extern "C-unwind" fn math_fmod(mut L: *mut lua_State) -> i32 {
    if lua_isinteger(L, 1 as i32) != 0 && lua_isinteger(L, 2 as i32) != 0 {
        let mut d: lua_Integer = lua_tointegerx(L, 2 as i32, 0 as *mut i32);
        if (d as lua_Unsigned).wrapping_add(1 as u32 as lua_Unsigned) <= 1 as u32 as lua_Unsigned {
            (((d != 0 as lua_Integer) as i32 != 0) as i32 as std::ffi::c_long != 0
                || luaL_argerror(L, 2 as i32, c"zero".as_ptr()) != 0) as i32;
            lua_pushinteger(L, 0 as lua_Integer);
        } else {
            lua_pushinteger(L, lua_tointegerx(L, 1 as i32, 0 as *mut i32) % d);
        }
    } else {
        lua_pushnumber(
            L,
            fmod(luaL_checknumber(L, 1 as i32), luaL_checknumber(L, 2 as i32)),
        );
    }
    return 1 as i32;
}
unsafe extern "C-unwind" fn math_modf(mut L: *mut lua_State) -> i32 {
    if lua_isinteger(L, 1 as i32) != 0 {
        lua_settop(L, 1 as i32);
        lua_pushnumber(L, 0 as lua_Number);
    } else {
        let mut n: lua_Number = luaL_checknumber(L, 1 as i32);
        let mut ip: lua_Number = if n < 0 as lua_Number {
            n.ceil()
        } else {
            n.floor()
        };
        pushnumint(L, ip);
        lua_pushnumber(L, if n == ip { 0.0f64 } else { n - ip });
    }
    return 2 as i32;
}
unsafe extern "C-unwind" fn math_sqrt(mut L: *mut lua_State) -> i32 {
    lua_pushnumber(L, luaL_checknumber(L, 1 as i32).sqrt());
    return 1 as i32;
}
unsafe extern "C-unwind" fn math_ult(mut L: *mut lua_State) -> i32 {
    let mut a: lua_Integer = luaL_checkinteger(L, 1 as i32);
    let mut b: lua_Integer = luaL_checkinteger(L, 2 as i32);
    lua_pushboolean(L, ((a as lua_Unsigned) < b as lua_Unsigned) as i32);
    return 1 as i32;
}
unsafe extern "C-unwind" fn math_log(mut L: *mut lua_State) -> i32 {
    let mut x: lua_Number = luaL_checknumber(L, 1 as i32);
    let mut res: lua_Number = 0.;
    if lua_type(L, 2 as i32) <= 0 {
        res = x.ln();
    } else {
        let mut base: lua_Number = luaL_checknumber(L, 2 as i32);
        if base == 2.0f64 {
            res = x.log2();
        } else if base == 10.0f64 {
            res = x.log10();
        } else {
            res = x.log(base);
        }
    }
    lua_pushnumber(L, res);
    return 1 as i32;
}
unsafe extern "C-unwind" fn math_exp(mut L: *mut lua_State) -> i32 {
    lua_pushnumber(L, luaL_checknumber(L, 1 as i32).exp());
    return 1 as i32;
}
unsafe extern "C-unwind" fn math_deg(mut L: *mut lua_State) -> i32 {
    lua_pushnumber(
        L,
        luaL_checknumber(L, 1 as i32) * (180.0f64 / 3.141592653589793238462643383279502884f64),
    );
    return 1 as i32;
}
unsafe extern "C-unwind" fn math_rad(mut L: *mut lua_State) -> i32 {
    lua_pushnumber(
        L,
        luaL_checknumber(L, 1 as i32) * (3.141592653589793238462643383279502884f64 / 180.0f64),
    );
    return 1 as i32;
}
unsafe extern "C-unwind" fn math_min(mut L: *mut lua_State) -> i32 {
    let mut n: i32 = lua_gettop(L);
    let mut imin: i32 = 1 as i32;
    let mut i: i32 = 0;
    (((n >= 1 as i32) as i32 != 0) as i32 as std::ffi::c_long != 0
        || luaL_argerror(L, 1 as i32, c"value expected".as_ptr()) != 0) as i32;
    i = 2 as i32;
    while i <= n {
        if lua_compare(L, i, imin, 1 as i32) != 0 {
            imin = i;
        }
        i += 1;
        i;
    }
    lua_pushvalue(L, imin);
    return 1 as i32;
}
unsafe extern "C-unwind" fn math_max(mut L: *mut lua_State) -> i32 {
    let mut n: i32 = lua_gettop(L);
    let mut imax: i32 = 1 as i32;
    let mut i: i32 = 0;
    (((n >= 1 as i32) as i32 != 0) as i32 as std::ffi::c_long != 0
        || luaL_argerror(L, 1 as i32, c"value expected".as_ptr()) != 0) as i32;
    i = 2 as i32;
    while i <= n {
        if lua_compare(L, imax, i, 1 as i32) != 0 {
            imax = i;
        }
        i += 1;
        i;
    }
    lua_pushvalue(L, imax);
    return 1 as i32;
}
unsafe extern "C-unwind" fn math_type(mut L: *mut lua_State) -> i32 {
    if lua_type(L, 1 as i32) == 3 as i32 {
        lua_pushstring(
            L,
            if lua_isinteger(L, 1 as i32) != 0 {
                c"integer".as_ptr()
            } else {
                c"float".as_ptr()
            },
        );
    } else {
        luaL_checkany(L, 1 as i32);
        lua_pushnil(L);
    }
    return 1 as i32;
}
unsafe extern "C-unwind" fn rotl(mut x: usize, mut n: i32) -> usize {
    return x << n | (x & 0xffffffffffffffff as usize) >> 64 as i32 - n;
}
unsafe extern "C-unwind" fn nextrand(mut state: *mut usize) -> usize {
    let mut state0: usize = *state.offset(0 as isize);
    let mut state1: usize = *state.offset(1);
    let mut state2: usize = *state.offset(2) ^ state0;
    let mut state3: usize = *state.offset(3) ^ state1;
    let mut res: usize = (rotl(state1.wrapping_mul(5), 7 as i32)).wrapping_mul(9);
    *state.offset(0 as isize) = state0 ^ state3;
    *state.offset(1) = state1 ^ state2;
    *state.offset(2) = state2 ^ state1 << 17 as i32;
    *state.offset(3) = rotl(state3, 45 as i32);
    return res;
}
unsafe extern "C-unwind" fn I2d(mut x: usize) -> lua_Number {
    let mut sx: std::ffi::c_long =
        ((x & 0xffffffffffffffff as usize) >> 64 as i32 - 53 as i32) as std::ffi::c_long;
    let mut res: lua_Number =
        sx as lua_Number * (0.5f64 / ((1usize) << 53 as i32 - 1 as i32) as std::ffi::c_double);
    if sx < 0 as std::ffi::c_long {
        res += 1.0f64;
    }
    return res;
}
unsafe extern "C-unwind" fn project(
    mut ran: lua_Unsigned,
    mut n: lua_Unsigned,
    mut state: *mut RanState,
) -> lua_Unsigned {
    if n & n.wrapping_add(1 as i32 as lua_Unsigned) == 0 as lua_Unsigned {
        return ran & n;
    } else {
        let mut lim: lua_Unsigned = n;
        lim |= lim >> 1 as i32;
        lim |= lim >> 2 as i32;
        lim |= lim >> 4 as i32;
        lim |= lim >> 8 as i32;
        lim |= lim >> 16 as i32;
        lim |= lim >> 32 as i32;
        loop {
            ran &= lim;
            if !(ran > n) {
                break;
            }
            ran =
                (nextrand(((*state).s).as_mut_ptr()) & 0xffffffffffffffff as usize) as lua_Unsigned;
        }
        return ran;
    };
}
unsafe extern "C-unwind" fn math_random(mut L: *mut lua_State) -> i32 {
    let mut low: lua_Integer = 0;
    let mut up: lua_Integer = 0;
    let mut p: lua_Unsigned = 0;
    let mut state: *mut RanState = lua_touserdata(L, -(1000000) - 1000 - 1 as i32) as *mut RanState;
    let mut rv: usize = nextrand(((*state).s).as_mut_ptr());
    match lua_gettop(L) {
        0 => {
            lua_pushnumber(L, I2d(rv));
            return 1 as i32;
        }
        1 => {
            low = 1 as i32 as lua_Integer;
            up = luaL_checkinteger(L, 1 as i32);
            if up == 0 as lua_Integer {
                lua_pushinteger(
                    L,
                    (rv & 0xffffffffffffffff as usize) as lua_Unsigned as lua_Integer,
                );
                return 1 as i32;
            }
        }
        2 => {
            low = luaL_checkinteger(L, 1 as i32);
            up = luaL_checkinteger(L, 2 as i32);
        }
        _ => {
            return luaL_error(L, c"wrong number of arguments".as_ptr());
        }
    }
    (((low <= up) as i32 != 0) as i32 as std::ffi::c_long != 0
        || luaL_argerror(L, 1 as i32, c"interval is empty".as_ptr()) != 0) as i32;
    p = project(
        (rv & 0xffffffffffffffff as usize) as lua_Unsigned,
        (up as lua_Unsigned).wrapping_sub(low as lua_Unsigned),
        state,
    );
    lua_pushinteger(L, p.wrapping_add(low as lua_Unsigned) as lua_Integer);
    return 1 as i32;
}
unsafe extern "C-unwind" fn setseed(
    mut L: *mut lua_State,
    mut state: *mut usize,
    mut n1: lua_Unsigned,
    mut n2: lua_Unsigned,
) {
    let mut i: i32 = 0;
    *state.offset(0 as isize) = n1 as usize;
    *state.offset(1) = 0xff as i32 as usize;
    *state.offset(2) = n2 as usize;
    *state.offset(3) = 0 as usize;
    i = 0;
    while i < 16 as i32 {
        nextrand(state);
        i += 1;
        i;
    }
    lua_pushinteger(L, n1 as lua_Integer);
    lua_pushinteger(L, n2 as lua_Integer);
}
unsafe extern "C-unwind" fn randseed(mut L: *mut lua_State, mut state: *mut RanState) {
    let mut seed1: lua_Unsigned = time(0 as *mut time_t) as lua_Unsigned;
    let mut seed2: lua_Unsigned = L as size_t as lua_Unsigned;
    setseed(L, ((*state).s).as_mut_ptr(), seed1, seed2);
}
unsafe extern "C-unwind" fn math_randomseed(mut L: *mut lua_State) -> i32 {
    let mut state: *mut RanState = lua_touserdata(L, -(1000000) - 1000 - 1 as i32) as *mut RanState;
    if lua_type(L, 1 as i32) == -(1 as i32) {
        randseed(L, state);
    } else {
        let mut n1: lua_Integer = luaL_checkinteger(L, 1 as i32);
        let mut n2: lua_Integer = luaL_optinteger(L, 2 as i32, 0 as lua_Integer);
        setseed(
            L,
            ((*state).s).as_mut_ptr(),
            n1 as lua_Unsigned,
            n2 as lua_Unsigned,
        );
    }
    return 2 as i32;
}
static mut randfuncs: [luaL_Reg; 3] = unsafe {
    [
        {
            let mut init = luaL_Reg {
                name: c"random".as_ptr(),
                func: Some(math_random as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"randomseed".as_ptr(),
                func: Some(math_randomseed as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: 0 as *const std::ffi::c_char,
                func: None,
            };
            init
        },
    ]
};
unsafe extern "C-unwind" fn setrandfunc(mut L: *mut lua_State) {
    let mut state: *mut RanState =
        lua_newuserdatauv(L, size_of::<RanState>() as usize, 0) as *mut RanState;
    randseed(L, state);
    lua_settop(L, -(2 as i32) - 1 as i32);
    luaL_setfuncs(L, (&raw const randfuncs).cast(), 1 as i32);
}
static mut mathlib: [luaL_Reg; 28] = unsafe {
    [
        {
            let mut init = luaL_Reg {
                name: c"abs".as_ptr(),
                func: Some(math_abs as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"acos".as_ptr(),
                func: Some(math_acos as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"asin".as_ptr(),
                func: Some(math_asin as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"atan".as_ptr(),
                func: Some(math_atan as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"ceil".as_ptr(),
                func: Some(math_ceil as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"cos".as_ptr(),
                func: Some(math_cos as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"deg".as_ptr(),
                func: Some(math_deg as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"exp".as_ptr(),
                func: Some(math_exp as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"tointeger".as_ptr(),
                func: Some(math_toint as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"floor".as_ptr(),
                func: Some(math_floor as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"fmod".as_ptr(),
                func: Some(math_fmod as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"ult".as_ptr(),
                func: Some(math_ult as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"log".as_ptr(),
                func: Some(math_log as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"max".as_ptr(),
                func: Some(math_max as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"min".as_ptr(),
                func: Some(math_min as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"modf".as_ptr(),
                func: Some(math_modf as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"rad".as_ptr(),
                func: Some(math_rad as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"sin".as_ptr(),
                func: Some(math_sin as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"sqrt".as_ptr(),
                func: Some(math_sqrt as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"tan".as_ptr(),
                func: Some(math_tan as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"type".as_ptr(),
                func: Some(math_type as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"random".as_ptr(),
                func: None,
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"randomseed".as_ptr(),
                func: None,
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"pi".as_ptr(),
                func: None,
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"huge".as_ptr(),
                func: None,
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"maxinteger".as_ptr(),
                func: None,
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"mininteger".as_ptr(),
                func: None,
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: 0 as *const std::ffi::c_char,
                func: None,
            };
            init
        },
    ]
};
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaopen_math(mut L: *mut lua_State) -> i32 {
    luaL_checkversion_(
        L,
        504 as i32 as lua_Number,
        (size_of::<lua_Integer>() as usize)
            .wrapping_mul(16)
            .wrapping_add(size_of::<lua_Number>() as usize),
    );
    lua_createtable(
        L,
        0,
        (size_of::<[luaL_Reg; 28]>() as usize)
            .wrapping_div(size_of::<luaL_Reg>() as usize)
            .wrapping_sub(1) as i32,
    );
    luaL_setfuncs(L, (&raw const mathlib).cast(), 0);
    lua_pushnumber(L, 3.141592653589793238462643383279502884f64);
    lua_setfield(L, -(2 as i32), c"pi".as_ptr());
    lua_pushnumber(L, ::core::f64::INFINITY);
    lua_setfield(L, -(2 as i32), c"huge".as_ptr());
    lua_pushinteger(L, 9223372036854775807 as std::ffi::c_longlong);
    lua_setfield(L, -(2 as i32), c"maxinteger".as_ptr());
    lua_pushinteger(
        L,
        -(9223372036854775807 as std::ffi::c_longlong) - 1 as std::ffi::c_longlong,
    );
    lua_setfield(L, -(2 as i32), c"mininteger".as_ptr());
    setrandfunc(L);
    return 1 as i32;
}
static mut CLIBS: *const std::ffi::c_char = c"_CLIBS".as_ptr();
unsafe extern "C-unwind" fn lsys_unloadlib(mut lib: *mut c_void) {
    dlclose(lib);
}
unsafe extern "C-unwind" fn lsys_load(
    mut L: *mut lua_State,
    mut path: *const std::ffi::c_char,
    mut seeglb: i32,
) -> *mut c_void {
    let mut lib: *mut c_void = dlopen(path, 0x2 as i32 | (if seeglb != 0 { 0x100 } else { 0 }));
    if ((lib == 0 as *mut c_void) as i32 != 0) as i32 as std::ffi::c_long != 0 {
        lua_pushstring(L, dlerror());
    }
    return lib;
}
unsafe extern "C-unwind" fn lsys_sym(
    mut L: *mut lua_State,
    mut lib: *mut c_void,
    mut sym: *const std::ffi::c_char,
) -> lua_CFunction {
    let mut f: lua_CFunction =
        ::core::mem::transmute::<*mut c_void, lua_CFunction>(dlsym(lib, sym));
    if (f.is_none() as i32 != 0) as i32 as std::ffi::c_long != 0 {
        lua_pushstring(L, dlerror());
    }
    return f;
}
unsafe extern "C-unwind" fn noenv(mut L: *mut lua_State) -> i32 {
    let mut b: i32 = 0;
    lua_getfield(L, -(1000000) - 1000, c"LUA_NOENV".as_ptr());
    b = lua_toboolean(L, -(1 as i32));
    lua_settop(L, -(1 as i32) - 1 as i32);
    return b;
}
unsafe extern "C-unwind" fn setpath(
    mut L: *mut lua_State,
    mut fieldname: *const std::ffi::c_char,
    mut envname: *const std::ffi::c_char,
    mut dft: *const std::ffi::c_char,
) {
    let mut dftmark: *const std::ffi::c_char = 0 as *const std::ffi::c_char;
    let mut nver: *const std::ffi::c_char =
        lua_pushfstring(L, c"%s%s".as_ptr(), envname, c"_5_4".as_ptr());
    let mut path: *const std::ffi::c_char = getenv(nver);
    if path.is_null() {
        path = getenv(envname);
    }
    if path.is_null() || noenv(L) != 0 {
        lua_pushstring(L, dft);
    } else {
        dftmark = strstr(path, c";;".as_ptr());
        if dftmark.is_null() {
            lua_pushstring(L, path);
        } else {
            let mut len: size_t = strlen(path);
            let mut b: luaL_Buffer = luaL_Buffer {
                b: 0 as *mut std::ffi::c_char,
                size: 0,
                n: 0,
                L: 0 as *mut lua_State,
                init: C2RustUnnamed_15 { n: 0. },
            };
            luaL_buffinit(L, &mut b);
            if path < dftmark {
                luaL_addlstring(
                    &mut b,
                    path,
                    dftmark.offset_from(path) as std::ffi::c_long as size_t,
                );
                (b.n < b.size || !(luaL_prepbuffsize(&mut b, 1 as i32 as size_t)).is_null()) as i32;
                let fresh160 = b.n;
                b.n = (b.n).wrapping_add(1);
                *(b.b).offset(fresh160 as isize) = *(c";".as_ptr());
            }
            luaL_addstring(&mut b, dft);
            if dftmark < path.offset(len as isize).offset(-(2)) {
                (b.n < b.size || !(luaL_prepbuffsize(&mut b, 1 as i32 as size_t)).is_null()) as i32;
                let fresh161 = b.n;
                b.n = (b.n).wrapping_add(1);
                *(b.b).offset(fresh161 as isize) = *(c";".as_ptr());
                luaL_addlstring(
                    &mut b,
                    dftmark.offset(2),
                    path.offset(len as isize).offset(-(2)).offset_from(dftmark) as std::ffi::c_long
                        as size_t,
                );
            }
            luaL_pushresult(&mut b);
        }
    }
    lua_setfield(L, -(3 as i32), fieldname);
    lua_settop(L, -(1 as i32) - 1 as i32);
}
unsafe extern "C-unwind" fn checkclib(
    mut L: *mut lua_State,
    mut path: *const std::ffi::c_char,
) -> *mut c_void {
    let mut plib: *mut c_void = 0 as *mut c_void;
    lua_getfield(L, -(1000000) - 1000, CLIBS);
    lua_getfield(L, -(1 as i32), path);
    plib = lua_touserdata(L, -(1 as i32));
    lua_settop(L, -(2 as i32) - 1 as i32);
    return plib;
}
unsafe extern "C-unwind" fn addtoclib(
    mut L: *mut lua_State,
    mut path: *const std::ffi::c_char,
    mut plib: *mut c_void,
) {
    lua_getfield(L, -(1000000) - 1000, CLIBS);
    lua_pushlightuserdata(L, plib);
    lua_pushvalue(L, -(1 as i32));
    lua_setfield(L, -(3 as i32), path);
    lua_rawseti(
        L,
        -(2 as i32),
        luaL_len(L, -(2 as i32)) + 1 as i32 as lua_Integer,
    );
    lua_settop(L, -(1 as i32) - 1 as i32);
}
unsafe extern "C-unwind" fn gctm(mut L: *mut lua_State) -> i32 {
    let mut n: lua_Integer = luaL_len(L, 1 as i32);
    while n >= 1 as i32 as lua_Integer {
        lua_rawgeti(L, 1 as i32, n);
        lsys_unloadlib(lua_touserdata(L, -(1 as i32)));
        lua_settop(L, -(1 as i32) - 1 as i32);
        n -= 1;
        n;
    }
    return 0;
}
unsafe extern "C-unwind" fn lookforfunc(
    mut L: *mut lua_State,
    mut path: *const std::ffi::c_char,
    mut sym: *const std::ffi::c_char,
) -> i32 {
    let mut reg: *mut c_void = checkclib(L, path);
    if reg.is_null() {
        reg = lsys_load(L, path, (*sym as i32 == '*' as i32) as i32);
        if reg.is_null() {
            return 1 as i32;
        }
        addtoclib(L, path, reg);
    }
    if *sym as i32 == '*' as i32 {
        lua_pushboolean(L, 1 as i32);
        return 0;
    } else {
        let mut f: lua_CFunction = lsys_sym(L, reg, sym);
        if f.is_none() {
            return 2 as i32;
        }
        lua_pushcclosure(L, f, 0);
        return 0;
    };
}
unsafe extern "C-unwind" fn ll_loadlib(mut L: *mut lua_State) -> i32 {
    let mut path: *const std::ffi::c_char = luaL_checklstring(L, 1 as i32, 0 as *mut size_t);
    let mut init: *const std::ffi::c_char = luaL_checklstring(L, 2 as i32, 0 as *mut size_t);
    let mut stat: i32 = lookforfunc(L, path, init);
    if ((stat == 0) as i32 != 0) as i32 as std::ffi::c_long != 0 {
        return 1 as i32;
    } else {
        lua_pushnil(L);
        lua_rotate(L, -(2 as i32), 1 as i32);
        lua_pushstring(
            L,
            if stat == 1 as i32 {
                c"open".as_ptr()
            } else {
                c"init".as_ptr()
            },
        );
        return 3 as i32;
    };
}
unsafe extern "C-unwind" fn readable(mut filename: *const std::ffi::c_char) -> i32 {
    let mut f: *mut FILE = fopen(filename, c"r".as_ptr());
    if f.is_null() {
        return 0;
    }
    fclose(f);
    return 1 as i32;
}
unsafe extern "C-unwind" fn getnextfilename(
    mut path: *mut *mut std::ffi::c_char,
    mut end: *mut std::ffi::c_char,
) -> *const std::ffi::c_char {
    let mut sep: *mut std::ffi::c_char = 0 as *mut std::ffi::c_char;
    let mut name: *mut std::ffi::c_char = *path;
    if name == end {
        return 0 as *const std::ffi::c_char;
    } else if *name as i32 == '\0' as i32 {
        *name = *(c";".as_ptr());
        name = name.offset(1);
        name;
    }
    sep = strchr(name, *(c";".as_ptr()) as i32);
    if sep.is_null() {
        sep = end;
    }
    *sep = '\0' as i32 as std::ffi::c_char;
    *path = sep;
    return name;
}
unsafe extern "C-unwind" fn pusherrornotfound(
    mut L: *mut lua_State,
    mut path: *const std::ffi::c_char,
) {
    let mut b: luaL_Buffer = luaL_Buffer {
        b: 0 as *mut std::ffi::c_char,
        size: 0,
        n: 0,
        L: 0 as *mut lua_State,
        init: C2RustUnnamed_15 { n: 0. },
    };
    luaL_buffinit(L, &mut b);
    luaL_addstring(&mut b, c"no file '".as_ptr());
    luaL_addgsub(&mut b, path, c";".as_ptr(), c"'\n\tno file '".as_ptr());
    luaL_addstring(&mut b, c"'".as_ptr());
    luaL_pushresult(&mut b);
}
unsafe extern "C-unwind" fn searchpath(
    mut L: *mut lua_State,
    mut name: *const std::ffi::c_char,
    mut path: *const std::ffi::c_char,
    mut sep: *const std::ffi::c_char,
    mut dirsep: *const std::ffi::c_char,
) -> *const std::ffi::c_char {
    let mut buff: luaL_Buffer = luaL_Buffer {
        b: 0 as *mut std::ffi::c_char,
        size: 0,
        n: 0,
        L: 0 as *mut lua_State,
        init: C2RustUnnamed_15 { n: 0. },
    };
    let mut pathname: *mut std::ffi::c_char = 0 as *mut std::ffi::c_char;
    let mut endpathname: *mut std::ffi::c_char = 0 as *mut std::ffi::c_char;
    let mut filename: *const std::ffi::c_char = 0 as *const std::ffi::c_char;
    if *sep as i32 != '\0' as i32 && !(strchr(name, *sep as i32)).is_null() {
        name = luaL_gsub(L, name, sep, dirsep);
    }
    luaL_buffinit(L, &mut buff);
    luaL_addgsub(&mut buff, path, c"?".as_ptr(), name);
    (buff.n < buff.size || !(luaL_prepbuffsize(&mut buff, 1 as i32 as size_t)).is_null()) as i32;
    let fresh162 = buff.n;
    buff.n = (buff.n).wrapping_add(1);
    *(buff.b).offset(fresh162 as isize) = '\0' as i32 as std::ffi::c_char;
    pathname = buff.b;
    endpathname = pathname.offset(buff.n as isize).offset(-(1));
    loop {
        filename = getnextfilename(&mut pathname, endpathname);
        if filename.is_null() {
            break;
        }
        if readable(filename) != 0 {
            return lua_pushstring(L, filename);
        }
    }
    luaL_pushresult(&mut buff);
    pusherrornotfound(L, lua_tolstring(L, -(1 as i32), 0 as *mut size_t));
    return 0 as *const std::ffi::c_char;
}
unsafe extern "C-unwind" fn ll_searchpath(mut L: *mut lua_State) -> i32 {
    let mut f: *const std::ffi::c_char = searchpath(
        L,
        luaL_checklstring(L, 1 as i32, 0 as *mut size_t),
        luaL_checklstring(L, 2 as i32, 0 as *mut size_t),
        luaL_optlstring(L, 3 as i32, c".".as_ptr(), 0 as *mut size_t),
        luaL_optlstring(L, 4 as i32, c"/".as_ptr(), 0 as *mut size_t),
    );
    if !f.is_null() {
        return 1 as i32;
    } else {
        lua_pushnil(L);
        lua_rotate(L, -(2 as i32), 1 as i32);
        return 2 as i32;
    };
}
unsafe extern "C-unwind" fn findfile(
    mut L: *mut lua_State,
    mut name: *const std::ffi::c_char,
    mut pname: *const std::ffi::c_char,
    mut dirsep: *const std::ffi::c_char,
) -> *const std::ffi::c_char {
    let mut path: *const std::ffi::c_char = 0 as *const std::ffi::c_char;
    lua_getfield(L, -(1000000) - 1000 - 1 as i32, pname);
    path = lua_tolstring(L, -(1 as i32), 0 as *mut size_t);
    if ((path == 0 as *mut c_void as *const std::ffi::c_char) as i32 != 0) as i32
        as std::ffi::c_long
        != 0
    {
        luaL_error(L, c"'package.%s' must be a string".as_ptr(), pname);
    }
    return searchpath(L, name, path, c".".as_ptr(), dirsep);
}
unsafe extern "C-unwind" fn checkload(
    mut L: *mut lua_State,
    mut stat: i32,
    mut filename: *const std::ffi::c_char,
) -> i32 {
    if (stat != 0) as i32 as std::ffi::c_long != 0 {
        lua_pushstring(L, filename);
        return 2 as i32;
    } else {
        return luaL_error(
            L,
            c"error loading module '%s' from file '%s':\n\t%s".as_ptr(),
            lua_tolstring(L, 1 as i32, 0 as *mut size_t),
            filename,
            lua_tolstring(L, -(1 as i32), 0 as *mut size_t),
        );
    };
}
unsafe extern "C-unwind" fn searcher_Lua(mut L: *mut lua_State) -> i32 {
    let mut filename: *const std::ffi::c_char = 0 as *const std::ffi::c_char;
    let mut name: *const std::ffi::c_char = luaL_checklstring(L, 1 as i32, 0 as *mut size_t);
    filename = findfile(L, name, c"path".as_ptr(), c"/".as_ptr());
    if filename.is_null() {
        return 1 as i32;
    }
    return checkload(
        L,
        (luaL_loadfilex(L, filename, 0 as *const std::ffi::c_char) == 0) as i32,
        filename,
    );
}
unsafe extern "C-unwind" fn loadfunc(
    mut L: *mut lua_State,
    mut filename: *const std::ffi::c_char,
    mut modname: *const std::ffi::c_char,
) -> i32 {
    let mut openfunc: *const std::ffi::c_char = 0 as *const std::ffi::c_char;
    let mut mark: *const std::ffi::c_char = 0 as *const std::ffi::c_char;
    modname = luaL_gsub(L, modname, c".".as_ptr(), c"_".as_ptr());
    mark = strchr(modname, *(c"-".as_ptr()) as i32);
    if !mark.is_null() {
        let mut stat: i32 = 0;
        openfunc = lua_pushlstring(
            L,
            modname,
            mark.offset_from(modname) as std::ffi::c_long as size_t,
        );
        openfunc = lua_pushfstring(L, c"luaopen_%s".as_ptr(), openfunc);
        stat = lookforfunc(L, filename, openfunc);
        if stat != 2 as i32 {
            return stat;
        }
        modname = mark.offset(1);
    }
    openfunc = lua_pushfstring(L, c"luaopen_%s".as_ptr(), modname);
    return lookforfunc(L, filename, openfunc);
}
unsafe extern "C-unwind" fn searcher_C(mut L: *mut lua_State) -> i32 {
    let mut name: *const std::ffi::c_char = luaL_checklstring(L, 1 as i32, 0 as *mut size_t);
    let mut filename: *const std::ffi::c_char = findfile(L, name, c"cpath".as_ptr(), c"/".as_ptr());
    if filename.is_null() {
        return 1 as i32;
    }
    return checkload(L, (loadfunc(L, filename, name) == 0) as i32, filename);
}
unsafe extern "C-unwind" fn searcher_Croot(mut L: *mut lua_State) -> i32 {
    let mut filename: *const std::ffi::c_char = 0 as *const std::ffi::c_char;
    let mut name: *const std::ffi::c_char = luaL_checklstring(L, 1 as i32, 0 as *mut size_t);
    let mut p: *const std::ffi::c_char = strchr(name, '.' as i32);
    let mut stat: i32 = 0;
    if p.is_null() {
        return 0;
    }
    lua_pushlstring(L, name, p.offset_from(name) as std::ffi::c_long as size_t);
    filename = findfile(
        L,
        lua_tolstring(L, -(1 as i32), 0 as *mut size_t),
        c"cpath".as_ptr(),
        c"/".as_ptr(),
    );
    if filename.is_null() {
        return 1 as i32;
    }
    stat = loadfunc(L, filename, name);
    if stat != 0 {
        if stat != 2 as i32 {
            return checkload(L, 0, filename);
        } else {
            lua_pushfstring(L, c"no module '%s' in file '%s'".as_ptr(), name, filename);
            return 1 as i32;
        }
    }
    lua_pushstring(L, filename);
    return 2 as i32;
}
unsafe extern "C-unwind" fn searcher_preload(mut L: *mut lua_State) -> i32 {
    let mut name: *const std::ffi::c_char = luaL_checklstring(L, 1 as i32, 0 as *mut size_t);
    lua_getfield(L, -(1000000) - 1000, c"_PRELOAD".as_ptr());
    if lua_getfield(L, -(1 as i32), name) == 0 {
        lua_pushfstring(L, c"no field package.preload['%s']".as_ptr(), name);
        return 1 as i32;
    } else {
        lua_pushstring(L, c":preload:".as_ptr());
        return 2 as i32;
    };
}
unsafe extern "C-unwind" fn findloader(mut L: *mut lua_State, mut name: *const std::ffi::c_char) {
    let mut i: i32 = 0;
    let mut msg: luaL_Buffer = luaL_Buffer {
        b: 0 as *mut std::ffi::c_char,
        size: 0,
        n: 0,
        L: 0 as *mut lua_State,
        init: C2RustUnnamed_15 { n: 0. },
    };
    if ((lua_getfield(L, -(1000000) - 1000 - 1 as i32, c"searchers".as_ptr()) != 5 as i32) as i32
        != 0) as i32 as std::ffi::c_long
        != 0
    {
        luaL_error(L, c"'package.searchers' must be a table".as_ptr());
    }
    luaL_buffinit(L, &mut msg);
    i = 1 as i32;
    loop {
        luaL_addstring(&mut msg, c"\n\t".as_ptr());
        if ((lua_rawgeti(L, 3 as i32, i as lua_Integer) == 0) as i32 != 0) as i32
            as std::ffi::c_long
            != 0
        {
            lua_settop(L, -(1 as i32) - 1 as i32);
            msg.n = (msg.n).wrapping_sub(2 as i32 as size_t);
            luaL_pushresult(&mut msg);
            luaL_error(
                L,
                c"module '%s' not found:%s".as_ptr(),
                name,
                lua_tolstring(L, -(1 as i32), 0 as *mut size_t),
            );
        }
        lua_pushstring(L, name);
        lua_callk(L, 1 as i32, 2 as i32, 0 as lua_KContext, None);
        if lua_type(L, -(2 as i32)) == 6 as i32 {
            return;
        } else if lua_isstring(L, -(2 as i32)) != 0 {
            lua_settop(L, -(1 as i32) - 1 as i32);
            luaL_addvalue(&mut msg);
        } else {
            lua_settop(L, -(2 as i32) - 1 as i32);
            msg.n = (msg.n).wrapping_sub(2 as i32 as size_t);
        }
        i += 1;
        i;
    }
}
unsafe extern "C-unwind" fn ll_require(mut L: *mut lua_State) -> i32 {
    let mut name: *const std::ffi::c_char = luaL_checklstring(L, 1 as i32, 0 as *mut size_t);
    lua_settop(L, 1 as i32);
    lua_getfield(L, -(1000000) - 1000, c"_LOADED".as_ptr());
    lua_getfield(L, 2 as i32, name);
    if lua_toboolean(L, -(1 as i32)) != 0 {
        return 1 as i32;
    }
    lua_settop(L, -(1 as i32) - 1 as i32);
    findloader(L, name);
    lua_rotate(L, -(2 as i32), 1 as i32);
    lua_pushvalue(L, 1 as i32);
    lua_pushvalue(L, -(3 as i32));
    lua_callk(L, 2 as i32, 1 as i32, 0 as lua_KContext, None);
    if !(lua_type(L, -(1 as i32)) == 0) {
        lua_setfield(L, 2 as i32, name);
    } else {
        lua_settop(L, -(1 as i32) - 1 as i32);
    }
    if lua_getfield(L, 2 as i32, name) == 0 {
        lua_pushboolean(L, 1 as i32);
        lua_copy(L, -(1 as i32), -(2 as i32));
        lua_setfield(L, 2 as i32, name);
    }
    lua_rotate(L, -(2 as i32), 1 as i32);
    return 2 as i32;
}
static mut pk_funcs: [luaL_Reg; 8] = unsafe {
    [
        {
            let mut init = luaL_Reg {
                name: c"loadlib".as_ptr(),
                func: Some(ll_loadlib as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"searchpath".as_ptr(),
                func: Some(ll_searchpath as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"preload".as_ptr(),
                func: None,
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"cpath".as_ptr(),
                func: None,
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"path".as_ptr(),
                func: None,
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"searchers".as_ptr(),
                func: None,
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"loaded".as_ptr(),
                func: None,
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: 0 as *const std::ffi::c_char,
                func: None,
            };
            init
        },
    ]
};
static mut ll_funcs: [luaL_Reg; 2] = unsafe {
    [
        {
            let mut init = luaL_Reg {
                name: c"require".as_ptr(),
                func: Some(ll_require as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: 0 as *const std::ffi::c_char,
                func: None,
            };
            init
        },
    ]
};
unsafe extern "C-unwind" fn createsearcherstable(mut L: *mut lua_State) {
    static mut searchers: [lua_CFunction; 5] = unsafe {
        [
            Some(searcher_preload as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            Some(searcher_Lua as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            Some(searcher_C as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            Some(searcher_Croot as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            None,
        ]
    };
    let mut i: i32 = 0;
    lua_createtable(
        L,
        (size_of::<[lua_CFunction; 5]>() as usize)
            .wrapping_div(size_of::<lua_CFunction>() as usize)
            .wrapping_sub(1) as i32,
        0,
    );
    i = 0;
    while (searchers[i as usize]).is_some() {
        lua_pushvalue(L, -(2 as i32));
        lua_pushcclosure(L, searchers[i as usize], 1 as i32);
        lua_rawseti(L, -(2 as i32), (i + 1 as i32) as lua_Integer);
        i += 1;
        i;
    }
    lua_setfield(L, -(2 as i32), c"searchers".as_ptr());
}
unsafe extern "C-unwind" fn createclibstable(mut L: *mut lua_State) {
    luaL_getsubtable(L, -(1000000) - 1000, CLIBS);
    lua_createtable(L, 0, 1 as i32);
    lua_pushcclosure(
        L,
        Some(gctm as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
        0,
    );
    lua_setfield(L, -(2 as i32), c"__gc".as_ptr());
    lua_setmetatable(L, -(2 as i32));
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaopen_package(mut L: *mut lua_State) -> i32 {
    createclibstable(L);
    luaL_checkversion_(
        L,
        504 as i32 as lua_Number,
        (size_of::<lua_Integer>() as usize)
            .wrapping_mul(16)
            .wrapping_add(size_of::<lua_Number>() as usize),
    );
    lua_createtable(
        L,
        0,
        (size_of::<[luaL_Reg; 8]>() as usize)
            .wrapping_div(size_of::<luaL_Reg>() as usize)
            .wrapping_sub(1) as i32,
    );
    luaL_setfuncs(L, (&raw const pk_funcs).cast(), 0);
    createsearcherstable(L);
    setpath(
        L,
        c"path".as_ptr(),
        c"LUA_PATH".as_ptr(),
        c"/usr/local/share/lua/5.4/?.lua;/usr/local/share/lua/5.4/?/init.lua;/usr/local/lib/lua/5.4/?.lua;/usr/local/lib/lua/5.4/?/init.lua;./?.lua;./?/init.lua"
            .as_ptr(),
    );
    setpath(
        L,
        c"cpath".as_ptr(),
        c"LUA_CPATH".as_ptr(),
        c"/usr/local/lib/lua/5.4/?.so;/usr/local/lib/lua/5.4/loadall.so;./?.so".as_ptr(),
    );
    lua_pushstring(L, c"/\n;\n?\n!\n-\n".as_ptr());
    lua_setfield(L, -(2 as i32), c"config".as_ptr());
    luaL_getsubtable(L, -(1000000) - 1000, c"_LOADED".as_ptr());
    lua_setfield(L, -(2 as i32), c"loaded".as_ptr());
    luaL_getsubtable(L, -(1000000) - 1000, c"_PRELOAD".as_ptr());
    lua_setfield(L, -(2 as i32), c"preload".as_ptr());
    lua_rawgeti(L, -(1000000) - 1000, 2 as i32 as lua_Integer);
    lua_pushvalue(L, -(2 as i32));
    luaL_setfuncs(L, (&raw const ll_funcs).cast(), 1 as i32);
    lua_settop(L, -(1 as i32) - 1 as i32);
    return 1 as i32;
}
unsafe extern "C-unwind" fn os_execute(mut L: *mut lua_State) -> i32 {
    let mut cmd: *const std::ffi::c_char =
        luaL_optlstring(L, 1 as i32, 0 as *const std::ffi::c_char, 0 as *mut size_t);
    let mut stat: i32 = 0;
    *__errno_location() = 0;
    stat = system(cmd);
    if !cmd.is_null() {
        return luaL_execresult(L, stat);
    } else {
        lua_pushboolean(L, stat);
        return 1 as i32;
    };
}
unsafe extern "C-unwind" fn os_remove(mut L: *mut lua_State) -> i32 {
    let mut filename: *const std::ffi::c_char = luaL_checklstring(L, 1 as i32, 0 as *mut size_t);
    *__errno_location() = 0;
    return luaL_fileresult(L, (remove(filename) == 0) as i32, filename);
}
unsafe extern "C-unwind" fn os_rename(mut L: *mut lua_State) -> i32 {
    let mut fromname: *const std::ffi::c_char = luaL_checklstring(L, 1 as i32, 0 as *mut size_t);
    let mut toname: *const std::ffi::c_char = luaL_checklstring(L, 2 as i32, 0 as *mut size_t);
    *__errno_location() = 0;
    return luaL_fileresult(
        L,
        (rename(fromname, toname) == 0) as i32,
        0 as *const std::ffi::c_char,
    );
}
unsafe extern "C-unwind" fn os_tmpname(mut L: *mut lua_State) -> i32 {
    let mut buff: [std::ffi::c_char; 32] = [0; 32];
    let mut err: i32 = 0;
    strcpy(buff.as_mut_ptr(), c"/tmp/lua_XXXXXX".as_ptr());
    err = mkstemp(buff.as_mut_ptr());
    if err != -(1 as i32) {
        close(err);
    }
    err = (err == -(1 as i32)) as i32;
    if (err != 0) as i32 as std::ffi::c_long != 0 {
        return luaL_error(L, c"unable to generate a unique filename".as_ptr());
    }
    lua_pushstring(L, buff.as_mut_ptr());
    return 1 as i32;
}
unsafe extern "C-unwind" fn os_getenv(mut L: *mut lua_State) -> i32 {
    lua_pushstring(L, getenv(luaL_checklstring(L, 1 as i32, 0 as *mut size_t)));
    return 1 as i32;
}
unsafe extern "C-unwind" fn os_clock(mut L: *mut lua_State) -> i32 {
    lua_pushnumber(
        L,
        clock() as lua_Number / 1000000 as __clock_t as lua_Number,
    );
    return 1 as i32;
}
unsafe extern "C-unwind" fn setfield(
    mut L: *mut lua_State,
    mut key: *const std::ffi::c_char,
    mut value: i32,
    mut delta: i32,
) {
    lua_pushinteger(L, value as lua_Integer + delta as lua_Integer);
    lua_setfield(L, -(2 as i32), key);
}
unsafe extern "C-unwind" fn setboolfield(
    mut L: *mut lua_State,
    mut key: *const std::ffi::c_char,
    mut value: i32,
) {
    if value < 0 {
        return;
    }
    lua_pushboolean(L, value);
    lua_setfield(L, -(2 as i32), key);
}
unsafe extern "C-unwind" fn setallfields(mut L: *mut lua_State, mut stm: *mut tm) {
    setfield(L, c"year".as_ptr(), (*stm).tm_year, 1900);
    setfield(L, c"month".as_ptr(), (*stm).tm_mon, 1 as i32);
    setfield(L, c"day".as_ptr(), (*stm).tm_mday, 0);
    setfield(L, c"hour".as_ptr(), (*stm).tm_hour, 0);
    setfield(L, c"min".as_ptr(), (*stm).tm_min, 0);
    setfield(L, c"sec".as_ptr(), (*stm).tm_sec, 0);
    setfield(L, c"yday".as_ptr(), (*stm).tm_yday, 1 as i32);
    setfield(L, c"wday".as_ptr(), (*stm).tm_wday, 1 as i32);
    setboolfield(L, c"isdst".as_ptr(), (*stm).tm_isdst);
}
unsafe extern "C-unwind" fn getboolfield(
    mut L: *mut lua_State,
    mut key: *const std::ffi::c_char,
) -> i32 {
    let mut res: i32 = 0;
    res = if lua_getfield(L, -(1 as i32), key) == 0 {
        -(1 as i32)
    } else {
        lua_toboolean(L, -(1 as i32))
    };
    lua_settop(L, -(1 as i32) - 1 as i32);
    return res;
}
unsafe extern "C-unwind" fn getfield(
    mut L: *mut lua_State,
    mut key: *const std::ffi::c_char,
    mut d: i32,
    mut delta: i32,
) -> i32 {
    let mut isnum: i32 = 0;
    let mut t: i32 = lua_getfield(L, -(1 as i32), key);
    let mut res: lua_Integer = lua_tointegerx(L, -(1 as i32), &mut isnum);
    if isnum == 0 {
        if ((t != 0) as i32 != 0) as i32 as std::ffi::c_long != 0 {
            return luaL_error(L, c"field '%s' is not an integer".as_ptr(), key);
        } else if ((d < 0) as i32 != 0) as i32 as std::ffi::c_long != 0 {
            return luaL_error(L, c"field '%s' missing in date table".as_ptr(), key);
        }
        res = d as lua_Integer;
    } else {
        if if res >= 0 as lua_Integer {
            (res - delta as lua_Integer <= 2147483647 as i32 as lua_Integer) as i32
        } else {
            ((-(2147483647 as i32) - 1 as i32 + delta) as lua_Integer <= res) as i32
        } == 0
        {
            return luaL_error(L, c"field '%s' is out-of-bound".as_ptr(), key);
        }
        res -= delta as lua_Integer;
    }
    lua_settop(L, -(1 as i32) - 1 as i32);
    return res as i32;
}
unsafe extern "C-unwind" fn checkoption(
    mut L: *mut lua_State,
    mut conv: *const std::ffi::c_char,
    mut convlen: ptrdiff_t,
    mut buff: *mut std::ffi::c_char,
) -> *const std::ffi::c_char {
    let mut option: *const std::ffi::c_char =
        c"aAbBcCdDeFgGhHIjmMnprRStTuUVwWxXyYzZ%||EcECExEXEyEYOdOeOHOIOmOMOSOuOUOVOwOWOy".as_ptr();
    let mut oplen: i32 = 1 as i32;
    while *option as i32 != '\0' as i32 && oplen as ptrdiff_t <= convlen {
        if *option as i32 == '|' as i32 {
            oplen += 1;
            oplen;
        } else if memcmp(
            conv as *const c_void,
            option as *const c_void,
            oplen as usize,
        ) == 0
        {
            memcpy(buff as *mut c_void, conv as *const c_void, oplen as usize);
            *buff.offset(oplen as isize) = b'\0' as std::ffi::c_char;
            return conv.offset(oplen as isize);
        }
        option = option.offset(oplen as isize);
    }
    luaL_argerror(
        L,
        1 as i32,
        lua_pushfstring(L, c"invalid conversion specifier '%%%s'".as_ptr(), conv),
    );
    return conv;
}
unsafe extern "C-unwind" fn l_checktime(mut L: *mut lua_State, mut arg: i32) -> time_t {
    let mut t: lua_Integer = luaL_checkinteger(L, arg);
    (((t as time_t as lua_Integer == t) as i32 != 0) as i32 as std::ffi::c_long != 0
        || luaL_argerror(L, arg, c"time out-of-bounds".as_ptr()) != 0) as i32;
    return t as time_t;
}
unsafe extern "C-unwind" fn os_date(mut L: *mut lua_State) -> i32 {
    let mut slen: size_t = 0;
    let mut s: *const std::ffi::c_char = luaL_optlstring(L, 1 as i32, c"%c".as_ptr(), &mut slen);
    let mut t: time_t = if lua_type(L, 2 as i32) <= 0 {
        time(0 as *mut time_t)
    } else {
        l_checktime(L, 2 as i32)
    };
    let mut se: *const std::ffi::c_char = s.offset(slen as isize);
    let mut tmr: tm = tm {
        tm_sec: 0,
        tm_min: 0,
        tm_hour: 0,
        tm_mday: 0,
        tm_mon: 0,
        tm_year: 0,
        tm_wday: 0,
        tm_yday: 0,
        tm_isdst: 0,
        tm_gmtoff: 0,
        tm_zone: 0 as *const std::ffi::c_char,
    };
    let mut stm: *mut tm = 0 as *mut tm;
    if *s as i32 == '!' as i32 {
        stm = gmtime_r(&mut t, &mut tmr);
        s = s.offset(1);
        s;
    } else {
        stm = localtime_r(&mut t, &mut tmr);
    }
    if stm.is_null() {
        return luaL_error(
            L,
            c"date result cannot be represented in this installation".as_ptr(),
        );
    }
    if strcmp(s, c"*t".as_ptr()) == 0 {
        lua_createtable(L, 0, 9 as i32);
        setallfields(L, stm);
    } else {
        let mut cc: [std::ffi::c_char; 4] = [0; 4];
        let mut b: luaL_Buffer = luaL_Buffer {
            b: 0 as *mut std::ffi::c_char,
            size: 0,
            n: 0,
            L: 0 as *mut lua_State,
            init: C2RustUnnamed_15 { n: 0. },
        };
        cc[0 as usize] = '%' as i32 as std::ffi::c_char;
        luaL_buffinit(L, &mut b);
        while s < se {
            if *s as i32 != '%' as i32 {
                (b.n < b.size || !(luaL_prepbuffsize(&mut b, 1 as i32 as size_t)).is_null()) as i32;
                let fresh163 = s;
                s = s.offset(1);
                let fresh164 = b.n;
                b.n = (b.n).wrapping_add(1);
                *(b.b).offset(fresh164 as isize) = *fresh163;
            } else {
                let mut reslen: size_t = 0;
                let mut buff: *mut std::ffi::c_char = luaL_prepbuffsize(&mut b, 250 as size_t);
                s = s.offset(1);
                s;
                s = checkoption(L, s, se.offset_from(s), cc.as_mut_ptr().offset(1));
                reslen = strftime(buff, 250 as size_t, cc.as_mut_ptr(), stm);
                b.n = (b.n).wrapping_add(reslen);
            }
        }
        luaL_pushresult(&mut b);
    }
    return 1 as i32;
}
unsafe extern "C-unwind" fn os_time(mut L: *mut lua_State) -> i32 {
    let mut t: time_t = 0;
    if lua_type(L, 1 as i32) <= 0 {
        t = time(0 as *mut time_t);
    } else {
        let mut ts: tm = tm {
            tm_sec: 0,
            tm_min: 0,
            tm_hour: 0,
            tm_mday: 0,
            tm_mon: 0,
            tm_year: 0,
            tm_wday: 0,
            tm_yday: 0,
            tm_isdst: 0,
            tm_gmtoff: 0,
            tm_zone: 0 as *const std::ffi::c_char,
        };
        luaL_checktype(L, 1 as i32, 5 as i32);
        lua_settop(L, 1 as i32);
        ts.tm_year = getfield(L, c"year".as_ptr(), -(1 as i32), 1900);
        ts.tm_mon = getfield(L, c"month".as_ptr(), -(1 as i32), 1 as i32);
        ts.tm_mday = getfield(L, c"day".as_ptr(), -(1 as i32), 0);
        ts.tm_hour = getfield(L, c"hour".as_ptr(), 12 as i32, 0);
        ts.tm_min = getfield(L, c"min".as_ptr(), 0, 0);
        ts.tm_sec = getfield(L, c"sec".as_ptr(), 0, 0);
        ts.tm_isdst = getboolfield(L, c"isdst".as_ptr());
        t = mktime(&mut ts);
        setallfields(L, &mut ts);
    }
    if t != t as lua_Integer as time_t || t == -(1 as i32) as time_t {
        return luaL_error(
            L,
            c"time result cannot be represented in this installation".as_ptr(),
        );
    }
    lua_pushinteger(L, t as lua_Integer);
    return 1 as i32;
}
unsafe extern "C-unwind" fn os_difftime(mut L: *mut lua_State) -> i32 {
    let mut t1: time_t = l_checktime(L, 1 as i32);
    let mut t2: time_t = l_checktime(L, 2 as i32);
    lua_pushnumber(L, difftime(t1, t2));
    return 1 as i32;
}
unsafe extern "C-unwind" fn os_setlocale(mut L: *mut lua_State) -> i32 {
    static mut cat: [i32; 6] = [6 as i32, 3 as i32, 0, 4 as i32, 1 as i32, 2 as i32];
    static mut catnames: [*const std::ffi::c_char; 7] = [
        c"all".as_ptr(),
        c"collate".as_ptr(),
        c"ctype".as_ptr(),
        c"monetary".as_ptr(),
        c"numeric".as_ptr(),
        c"time".as_ptr(),
        0 as *const std::ffi::c_char,
    ];
    let mut l: *const std::ffi::c_char =
        luaL_optlstring(L, 1 as i32, 0 as *const std::ffi::c_char, 0 as *mut size_t);
    let mut op: i32 = luaL_checkoption(L, 2 as i32, c"all".as_ptr(), (&raw const catnames).cast());
    lua_pushstring(L, setlocale(cat[op as usize], l));
    return 1 as i32;
}
unsafe extern "C-unwind" fn os_exit(mut L: *mut lua_State) -> i32 {
    let mut status: i32 = 0;
    if lua_type(L, 1 as i32) == 1 as i32 {
        status = if lua_toboolean(L, 1 as i32) != 0 {
            0
        } else {
            1 as i32
        };
    } else {
        status = luaL_optinteger(L, 1 as i32, 0 as lua_Integer) as i32;
    }
    if lua_toboolean(L, 2 as i32) != 0 {
        lua_close(L);
    }
    if !L.is_null() {
        exit(status);
    }
    return 0;
}
static mut syslib: [luaL_Reg; 12] = unsafe {
    [
        {
            let mut init = luaL_Reg {
                name: c"clock".as_ptr(),
                func: Some(os_clock as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"date".as_ptr(),
                func: Some(os_date as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"difftime".as_ptr(),
                func: Some(os_difftime as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"execute".as_ptr(),
                func: Some(os_execute as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"exit".as_ptr(),
                func: Some(os_exit as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"getenv".as_ptr(),
                func: Some(os_getenv as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"remove".as_ptr(),
                func: Some(os_remove as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"rename".as_ptr(),
                func: Some(os_rename as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"setlocale".as_ptr(),
                func: Some(os_setlocale as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"time".as_ptr(),
                func: Some(os_time as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"tmpname".as_ptr(),
                func: Some(os_tmpname as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: 0 as *const std::ffi::c_char,
                func: None,
            };
            init
        },
    ]
};
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaopen_os(mut L: *mut lua_State) -> i32 {
    luaL_checkversion_(
        L,
        504 as i32 as lua_Number,
        (size_of::<lua_Integer>() as usize)
            .wrapping_mul(16)
            .wrapping_add(size_of::<lua_Number>() as usize),
    );
    lua_createtable(
        L,
        0,
        (size_of::<[luaL_Reg; 12]>() as usize)
            .wrapping_div(size_of::<luaL_Reg>() as usize)
            .wrapping_sub(1) as i32,
    );
    luaL_setfuncs(L, (&raw const syslib).cast(), 0);
    return 1 as i32;
}
unsafe extern "C-unwind" fn str_len(mut L: *mut lua_State) -> i32 {
    let mut l: size_t = 0;
    luaL_checklstring(L, 1 as i32, &mut l);
    lua_pushinteger(L, l as lua_Integer);
    return 1 as i32;
}
unsafe extern "C-unwind" fn posrelatI(mut pos: lua_Integer, mut len: size_t) -> size_t {
    if pos > 0 as lua_Integer {
        return pos as size_t;
    } else if pos == 0 as lua_Integer {
        return 1 as i32 as size_t;
    } else if pos < -(len as lua_Integer) {
        return 1 as i32 as size_t;
    } else {
        return len
            .wrapping_add(pos as size_t)
            .wrapping_add(1 as i32 as size_t);
    };
}
unsafe extern "C-unwind" fn getendpos(
    mut L: *mut lua_State,
    mut arg: i32,
    mut def: lua_Integer,
    mut len: size_t,
) -> size_t {
    let mut pos: lua_Integer = luaL_optinteger(L, arg, def);
    if pos > len as lua_Integer {
        return len;
    } else if pos >= 0 as lua_Integer {
        return pos as size_t;
    } else if pos < -(len as lua_Integer) {
        return 0 as size_t;
    } else {
        return len
            .wrapping_add(pos as size_t)
            .wrapping_add(1 as i32 as size_t);
    };
}
unsafe extern "C-unwind" fn str_sub(mut L: *mut lua_State) -> i32 {
    let mut l: size_t = 0;
    let mut s: *const std::ffi::c_char = luaL_checklstring(L, 1 as i32, &mut l);
    let mut start: size_t = posrelatI(luaL_checkinteger(L, 2 as i32), l);
    let mut end: size_t = getendpos(L, 3 as i32, -(1 as i32) as lua_Integer, l);
    if start <= end {
        lua_pushlstring(
            L,
            s.offset(start as isize).offset(-(1)),
            end.wrapping_sub(start).wrapping_add(1 as i32 as size_t),
        );
    } else {
        lua_pushstring(L, c"".as_ptr());
    }
    return 1 as i32;
}
unsafe extern "C-unwind" fn str_reverse(mut L: *mut lua_State) -> i32 {
    let mut l: size_t = 0;
    let mut i: size_t = 0;
    let mut b: luaL_Buffer = luaL_Buffer {
        b: 0 as *mut std::ffi::c_char,
        size: 0,
        n: 0,
        L: 0 as *mut lua_State,
        init: C2RustUnnamed_15 { n: 0. },
    };
    let mut s: *const std::ffi::c_char = luaL_checklstring(L, 1 as i32, &mut l);
    let mut p: *mut std::ffi::c_char = luaL_buffinitsize(L, &mut b, l);
    i = 0 as size_t;
    while i < l {
        *p.offset(i as isize) =
            *s.offset(l.wrapping_sub(i).wrapping_sub(1 as i32 as size_t) as isize);
        i = i.wrapping_add(1);
        i;
    }
    luaL_pushresultsize(&mut b, l);
    return 1 as i32;
}
unsafe extern "C-unwind" fn str_lower(mut L: *mut lua_State) -> i32 {
    let mut l: size_t = 0;
    let mut i: size_t = 0;
    let mut b: luaL_Buffer = luaL_Buffer {
        b: 0 as *mut std::ffi::c_char,
        size: 0,
        n: 0,
        L: 0 as *mut lua_State,
        init: C2RustUnnamed_15 { n: 0. },
    };
    let mut s: *const std::ffi::c_char = luaL_checklstring(L, 1 as i32, &mut l);
    let mut p: *mut std::ffi::c_char = luaL_buffinitsize(L, &mut b, l);
    i = 0 as size_t;
    while i < l {
        *p.offset(i as isize) = tolower(*s.offset(i as isize) as u8 as i32) as std::ffi::c_char;
        i = i.wrapping_add(1);
        i;
    }
    luaL_pushresultsize(&mut b, l);
    return 1 as i32;
}
unsafe extern "C-unwind" fn str_upper(mut L: *mut lua_State) -> i32 {
    let mut l: size_t = 0;
    let mut i: size_t = 0;
    let mut b: luaL_Buffer = luaL_Buffer {
        b: 0 as *mut std::ffi::c_char,
        size: 0,
        n: 0,
        L: 0 as *mut lua_State,
        init: C2RustUnnamed_15 { n: 0. },
    };
    let mut s: *const std::ffi::c_char = luaL_checklstring(L, 1 as i32, &mut l);
    let mut p: *mut std::ffi::c_char = luaL_buffinitsize(L, &mut b, l);
    i = 0 as size_t;
    while i < l {
        *p.offset(i as isize) = toupper(*s.offset(i as isize) as u8 as i32) as std::ffi::c_char;
        i = i.wrapping_add(1);
        i;
    }
    luaL_pushresultsize(&mut b, l);
    return 1 as i32;
}
unsafe extern "C-unwind" fn str_rep(mut L: *mut lua_State) -> i32 {
    let mut l: size_t = 0;
    let mut lsep: size_t = 0;
    let mut s: *const std::ffi::c_char = luaL_checklstring(L, 1 as i32, &mut l);
    let mut n: lua_Integer = luaL_checkinteger(L, 2 as i32);
    let mut sep: *const std::ffi::c_char = luaL_optlstring(L, 3 as i32, c"".as_ptr(), &mut lsep);
    if n <= 0 as lua_Integer {
        lua_pushstring(L, c"".as_ptr());
    } else if ((l.wrapping_add(lsep) < l
        || l.wrapping_add(lsep) as u64
            > ((if (size_of::<size_t>() as usize) < size_of::<i32>() as usize {
                !(0 as size_t)
            } else {
                2147483647 as i32 as size_t
            }) as u64)
                .wrapping_div(n as u64)) as i32
        != 0) as i32 as std::ffi::c_long
        != 0
    {
        return luaL_error(L, c"resulting string too large".as_ptr());
    } else {
        let mut totallen: size_t =
            (n as size_t * l).wrapping_add((n - 1 as i32 as lua_Integer) as size_t * lsep);
        let mut b: luaL_Buffer = luaL_Buffer {
            b: 0 as *mut std::ffi::c_char,
            size: 0,
            n: 0,
            L: 0 as *mut lua_State,
            init: C2RustUnnamed_15 { n: 0. },
        };
        let mut p: *mut std::ffi::c_char = luaL_buffinitsize(L, &mut b, totallen);
        loop {
            let fresh165 = n;
            n = n - 1;
            if !(fresh165 > 1 as i32 as lua_Integer) {
                break;
            }
            memcpy(
                p as *mut c_void,
                s as *const c_void,
                l.wrapping_mul(size_of::<std::ffi::c_char>() as usize),
            );
            p = p.offset(l as isize);
            if lsep > 0 as size_t {
                memcpy(
                    p as *mut c_void,
                    sep as *const c_void,
                    lsep.wrapping_mul(size_of::<std::ffi::c_char>() as usize),
                );
                p = p.offset(lsep as isize);
            }
        }
        memcpy(
            p as *mut c_void,
            s as *const c_void,
            l.wrapping_mul(size_of::<std::ffi::c_char>() as usize),
        );
        luaL_pushresultsize(&mut b, totallen);
    }
    return 1 as i32;
}
unsafe extern "C-unwind" fn str_byte(mut L: *mut lua_State) -> i32 {
    let mut l: size_t = 0;
    let mut s: *const std::ffi::c_char = luaL_checklstring(L, 1 as i32, &mut l);
    let mut pi: lua_Integer = luaL_optinteger(L, 2 as i32, 1 as i32 as lua_Integer);
    let mut posi: size_t = posrelatI(pi, l);
    let mut pose: size_t = getendpos(L, 3 as i32, pi, l);
    let mut n: i32 = 0;
    let mut i: i32 = 0;
    if posi > pose {
        return 0;
    }
    if ((pose.wrapping_sub(posi) >= 2147483647 as i32 as size_t) as i32 != 0) as i32
        as std::ffi::c_long
        != 0
    {
        return luaL_error(L, c"string slice too long".as_ptr());
    }
    n = pose.wrapping_sub(posi) as i32 + 1 as i32;
    luaL_checkstack(L, n, c"string slice too long".as_ptr());
    i = 0;
    while i < n {
        lua_pushinteger(
            L,
            *s.offset(
                posi.wrapping_add(i as size_t)
                    .wrapping_sub(1 as i32 as size_t) as isize,
            ) as u8 as lua_Integer,
        );
        i += 1;
        i;
    }
    return n;
}
unsafe extern "C-unwind" fn str_char(mut L: *mut lua_State) -> i32 {
    let mut n: i32 = lua_gettop(L);
    let mut i: i32 = 0;
    let mut b: luaL_Buffer = luaL_Buffer {
        b: 0 as *mut std::ffi::c_char,
        size: 0,
        n: 0,
        L: 0 as *mut lua_State,
        init: C2RustUnnamed_15 { n: 0. },
    };
    let mut p: *mut std::ffi::c_char = luaL_buffinitsize(L, &mut b, n as size_t);
    i = 1 as i32;
    while i <= n {
        let mut c: lua_Unsigned = luaL_checkinteger(L, i) as lua_Unsigned;
        (((c <= (127 as i32 * 2 as i32 + 1 as i32) as lua_Unsigned) as i32 != 0) as i32
            as std::ffi::c_long
            != 0
            || luaL_argerror(L, i, c"value out of range".as_ptr()) != 0) as i32;
        *p.offset((i - 1 as i32) as isize) = c as u8 as std::ffi::c_char;
        i += 1;
        i;
    }
    luaL_pushresultsize(&mut b, n as size_t);
    return 1 as i32;
}
unsafe extern "C-unwind" fn writer(
    mut L: *mut lua_State,
    mut b: *const c_void,
    mut size: size_t,
    mut ud: *mut c_void,
) -> i32 {
    let mut state: *mut str_Writer = ud as *mut str_Writer;
    if (*state).init == 0 {
        (*state).init = 1 as i32;
        luaL_buffinit(L, &mut (*state).B);
    }
    luaL_addlstring(&mut (*state).B, b as *const std::ffi::c_char, size);
    return 0;
}
unsafe extern "C-unwind" fn str_dump(mut L: *mut lua_State) -> i32 {
    let mut state: str_Writer = str_Writer {
        init: 0,
        B: luaL_Buffer {
            b: 0 as *mut std::ffi::c_char,
            size: 0,
            n: 0,
            L: 0 as *mut lua_State,
            init: C2RustUnnamed_15 { n: 0. },
        },
    };
    let mut strip: i32 = lua_toboolean(L, 2 as i32);
    luaL_checktype(L, 1 as i32, 6 as i32);
    lua_settop(L, 1 as i32);
    state.init = 0;
    if ((lua_dump(
        L,
        Some(
            writer
                as unsafe extern "C-unwind" fn(
                    *mut lua_State,
                    *const c_void,
                    size_t,
                    *mut c_void,
                ) -> i32,
        ),
        &mut state as *mut str_Writer as *mut c_void,
        strip,
    ) != 0) as i32
        != 0) as i32 as std::ffi::c_long
        != 0
    {
        return luaL_error(L, c"unable to dump given function".as_ptr());
    }
    luaL_pushresult(&mut state.B);
    return 1 as i32;
}
unsafe extern "C-unwind" fn tonum(mut L: *mut lua_State, mut arg: i32) -> i32 {
    if lua_type(L, arg) == 3 as i32 {
        lua_pushvalue(L, arg);
        return 1 as i32;
    } else {
        let mut len: size_t = 0;
        let mut s: *const std::ffi::c_char = lua_tolstring(L, arg, &mut len);
        return (!s.is_null() && lua_stringtonumber(L, s) == len.wrapping_add(1 as i32 as size_t))
            as i32;
    };
}
unsafe extern "C-unwind" fn trymt(mut L: *mut lua_State, mut mtname: *const std::ffi::c_char) {
    lua_settop(L, 2 as i32);
    if ((lua_type(L, 2 as i32) == 4 as i32 || luaL_getmetafield(L, 2 as i32, mtname) == 0) as i32
        != 0) as i32 as std::ffi::c_long
        != 0
    {
        luaL_error(
            L,
            c"attempt to %s a '%s' with a '%s'".as_ptr(),
            mtname.offset(2),
            lua_typename(L, lua_type(L, -(2 as i32))),
            lua_typename(L, lua_type(L, -(1 as i32))),
        );
    }
    lua_rotate(L, -(3 as i32), 1 as i32);
    lua_callk(L, 2 as i32, 1 as i32, 0 as lua_KContext, None);
}
unsafe extern "C-unwind" fn arith(
    mut L: *mut lua_State,
    mut op: i32,
    mut mtname: *const std::ffi::c_char,
) -> i32 {
    if tonum(L, 1 as i32) != 0 && tonum(L, 2 as i32) != 0 {
        lua_arith(L, op);
    } else {
        trymt(L, mtname);
    }
    return 1 as i32;
}
unsafe extern "C-unwind" fn arith_add(mut L: *mut lua_State) -> i32 {
    return arith(L, 0, c"__add".as_ptr());
}
unsafe extern "C-unwind" fn arith_sub(mut L: *mut lua_State) -> i32 {
    return arith(L, 1 as i32, c"__sub".as_ptr());
}
unsafe extern "C-unwind" fn arith_mul(mut L: *mut lua_State) -> i32 {
    return arith(L, 2 as i32, c"__mul".as_ptr());
}
unsafe extern "C-unwind" fn arith_mod(mut L: *mut lua_State) -> i32 {
    return arith(L, 3 as i32, c"__mod".as_ptr());
}
unsafe extern "C-unwind" fn arith_pow(mut L: *mut lua_State) -> i32 {
    return arith(L, 4 as i32, c"__pow".as_ptr());
}
unsafe extern "C-unwind" fn arith_div(mut L: *mut lua_State) -> i32 {
    return arith(L, 5 as i32, c"__div".as_ptr());
}
unsafe extern "C-unwind" fn arith_idiv(mut L: *mut lua_State) -> i32 {
    return arith(L, 6 as i32, c"__idiv".as_ptr());
}
unsafe extern "C-unwind" fn arith_unm(mut L: *mut lua_State) -> i32 {
    return arith(L, 12 as i32, c"__unm".as_ptr());
}
static mut stringmetamethods: [luaL_Reg; 10] = unsafe {
    [
        {
            let mut init = luaL_Reg {
                name: c"__add".as_ptr(),
                func: Some(arith_add as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"__sub".as_ptr(),
                func: Some(arith_sub as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"__mul".as_ptr(),
                func: Some(arith_mul as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"__mod".as_ptr(),
                func: Some(arith_mod as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"__pow".as_ptr(),
                func: Some(arith_pow as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"__div".as_ptr(),
                func: Some(arith_div as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"__idiv".as_ptr(),
                func: Some(arith_idiv as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"__unm".as_ptr(),
                func: Some(arith_unm as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"__index".as_ptr(),
                func: None,
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: 0 as *const std::ffi::c_char,
                func: None,
            };
            init
        },
    ]
};
unsafe extern "C-unwind" fn check_capture(mut ms: *mut MatchState, mut l: i32) -> i32 {
    l -= '1' as i32;
    if ((l < 0
        || l >= (*ms).level as i32
        || (*ms).capture[l as usize].len == -(1 as i32) as ptrdiff_t) as i32
        != 0) as i32 as std::ffi::c_long
        != 0
    {
        return luaL_error(
            (*ms).L,
            c"invalid capture index %%%d".as_ptr(),
            l + 1 as i32,
        );
    }
    return l;
}
unsafe extern "C-unwind" fn capture_to_close(mut ms: *mut MatchState) -> i32 {
    let mut level: i32 = (*ms).level as i32;
    level -= 1;
    level;
    while level >= 0 {
        if (*ms).capture[level as usize].len == -(1 as i32) as ptrdiff_t {
            return level;
        }
        level -= 1;
        level;
    }
    return luaL_error((*ms).L, c"invalid pattern capture".as_ptr());
}
unsafe extern "C-unwind" fn classend(
    mut ms: *mut MatchState,
    mut p: *const std::ffi::c_char,
) -> *const std::ffi::c_char {
    let fresh166 = p;
    p = p.offset(1);
    match *fresh166 as i32 {
        37 => {
            if ((p == (*ms).p_end) as i32 != 0) as i32 as std::ffi::c_long != 0 {
                luaL_error((*ms).L, c"malformed pattern (ends with '%%')".as_ptr());
            }
            return p.offset(1);
        }
        91 => {
            if *p as i32 == '^' as i32 {
                p = p.offset(1);
                p;
            }
            loop {
                if ((p == (*ms).p_end) as i32 != 0) as i32 as std::ffi::c_long != 0 {
                    luaL_error((*ms).L, c"malformed pattern (missing ']')".as_ptr());
                }
                let fresh167 = p;
                p = p.offset(1);
                if *fresh167 as i32 == '%' as i32 && p < (*ms).p_end {
                    p = p.offset(1);
                    p;
                }
                if !(*p as i32 != ']' as i32) {
                    break;
                }
            }
            return p.offset(1);
        }
        _ => return p,
    };
}
unsafe extern "C-unwind" fn match_class(mut c: i32, mut cl: i32) -> i32 {
    let mut res: i32 = 0;
    match tolower(cl) {
        97 => {
            res = *(*__ctype_b_loc()).offset(c as isize) as i32 & _ISalpha as i32 as u16 as i32;
        }
        99 => {
            res = *(*__ctype_b_loc()).offset(c as isize) as i32 & _IScntrl as i32 as u16 as i32;
        }
        100 => {
            res = *(*__ctype_b_loc()).offset(c as isize) as i32 & _ISdigit as i32 as u16 as i32;
        }
        103 => {
            res = *(*__ctype_b_loc()).offset(c as isize) as i32 & _ISgraph as i32 as u16 as i32;
        }
        108 => {
            res = *(*__ctype_b_loc()).offset(c as isize) as i32 & _ISlower as i32 as u16 as i32;
        }
        112 => {
            res = *(*__ctype_b_loc()).offset(c as isize) as i32 & _ISpunct as i32 as u16 as i32;
        }
        115 => {
            res = *(*__ctype_b_loc()).offset(c as isize) as i32 & _ISspace as i32 as u16 as i32;
        }
        117 => {
            res = *(*__ctype_b_loc()).offset(c as isize) as i32 & _ISupper as i32 as u16 as i32;
        }
        119 => {
            res = *(*__ctype_b_loc()).offset(c as isize) as i32 & _ISalnum as i32 as u16 as i32;
        }
        120 => {
            res = *(*__ctype_b_loc()).offset(c as isize) as i32 & _ISxdigit as i32 as u16 as i32;
        }
        122 => {
            res = (c == 0) as i32;
        }
        _ => return (cl == c) as i32,
    }
    return if *(*__ctype_b_loc()).offset(cl as isize) as i32 & _ISlower as i32 as u16 as i32 != 0 {
        res
    } else {
        (res == 0) as i32
    };
}
unsafe extern "C-unwind" fn matchbracketclass(
    mut c: i32,
    mut p: *const std::ffi::c_char,
    mut ec: *const std::ffi::c_char,
) -> i32 {
    let mut sig: i32 = 1 as i32;
    if *p.offset(1) as i32 == '^' as i32 {
        sig = 0;
        p = p.offset(1);
        p;
    }
    loop {
        p = p.offset(1);
        if !(p < ec) {
            break;
        }
        if *p as i32 == '%' as i32 {
            p = p.offset(1);
            p;
            if match_class(c, *p as u8 as i32) != 0 {
                return sig;
            }
        } else if *p.offset(1) as i32 == '-' as i32 && p.offset(2) < ec {
            p = p.offset(2);
            if *p.offset(-(2)) as u8 as i32 <= c && c <= *p as u8 as i32 {
                return sig;
            }
        } else if *p as u8 as i32 == c {
            return sig;
        }
    }
    return (sig == 0) as i32;
}
unsafe extern "C-unwind" fn singlematch(
    mut ms: *mut MatchState,
    mut s: *const std::ffi::c_char,
    mut p: *const std::ffi::c_char,
    mut ep: *const std::ffi::c_char,
) -> i32 {
    if s >= (*ms).src_end {
        return 0;
    } else {
        let mut c: i32 = *s as u8 as i32;
        match *p as i32 {
            46 => return 1 as i32,
            37 => {
                return match_class(c, *p.offset(1) as u8 as i32);
            }
            91 => {
                return matchbracketclass(c, p, ep.offset(-(1)));
            }
            _ => {
                return (*p as u8 as i32 == c) as i32;
            }
        }
    };
}
unsafe extern "C-unwind" fn matchbalance(
    mut ms: *mut MatchState,
    mut s: *const std::ffi::c_char,
    mut p: *const std::ffi::c_char,
) -> *const std::ffi::c_char {
    if ((p >= ((*ms).p_end).offset(-(1))) as i32 != 0) as i32 as std::ffi::c_long != 0 {
        luaL_error(
            (*ms).L,
            c"malformed pattern (missing arguments to '%%b')".as_ptr(),
        );
    }
    if *s as i32 != *p as i32 {
        return 0 as *const std::ffi::c_char;
    } else {
        let mut b: i32 = *p as i32;
        let mut e: i32 = *p.offset(1) as i32;
        let mut cont: i32 = 1 as i32;
        loop {
            s = s.offset(1);
            if !(s < (*ms).src_end) {
                break;
            }
            if *s as i32 == e {
                cont -= 1;
                if cont == 0 {
                    return s.offset(1);
                }
            } else if *s as i32 == b {
                cont += 1;
                cont;
            }
        }
    }
    return 0 as *const std::ffi::c_char;
}
unsafe extern "C-unwind" fn max_expand(
    mut ms: *mut MatchState,
    mut s: *const std::ffi::c_char,
    mut p: *const std::ffi::c_char,
    mut ep: *const std::ffi::c_char,
) -> *const std::ffi::c_char {
    let mut i: ptrdiff_t = 0 as ptrdiff_t;
    while singlematch(ms, s.offset(i as isize), p, ep) != 0 {
        i += 1;
        i;
    }
    while i >= 0 as ptrdiff_t {
        let mut res: *const std::ffi::c_char = match_0(ms, s.offset(i as isize), ep.offset(1));
        if !res.is_null() {
            return res;
        }
        i -= 1;
        i;
    }
    return 0 as *const std::ffi::c_char;
}
unsafe extern "C-unwind" fn min_expand(
    mut ms: *mut MatchState,
    mut s: *const std::ffi::c_char,
    mut p: *const std::ffi::c_char,
    mut ep: *const std::ffi::c_char,
) -> *const std::ffi::c_char {
    loop {
        let mut res: *const std::ffi::c_char = match_0(ms, s, ep.offset(1));
        if !res.is_null() {
            return res;
        } else if singlematch(ms, s, p, ep) != 0 {
            s = s.offset(1);
            s;
        } else {
            return 0 as *const std::ffi::c_char;
        }
    }
}
unsafe extern "C-unwind" fn start_capture(
    mut ms: *mut MatchState,
    mut s: *const std::ffi::c_char,
    mut p: *const std::ffi::c_char,
    mut what: i32,
) -> *const std::ffi::c_char {
    let mut res: *const std::ffi::c_char = 0 as *const std::ffi::c_char;
    let mut level: i32 = (*ms).level as i32;
    if level >= 32 as i32 {
        luaL_error((*ms).L, c"too many captures".as_ptr());
    }
    (*ms).capture[level as usize].init = s;
    (*ms).capture[level as usize].len = what as ptrdiff_t;
    (*ms).level = (level + 1 as i32) as u8;
    res = match_0(ms, s, p);
    if res.is_null() {
        (*ms).level = ((*ms).level).wrapping_sub(1);
        (*ms).level;
    }
    return res;
}
unsafe extern "C-unwind" fn end_capture(
    mut ms: *mut MatchState,
    mut s: *const std::ffi::c_char,
    mut p: *const std::ffi::c_char,
) -> *const std::ffi::c_char {
    let mut l: i32 = capture_to_close(ms);
    let mut res: *const std::ffi::c_char = 0 as *const std::ffi::c_char;
    (*ms).capture[l as usize].len = s.offset_from((*ms).capture[l as usize].init);
    res = match_0(ms, s, p);
    if res.is_null() {
        (*ms).capture[l as usize].len = -(1 as i32) as ptrdiff_t;
    }
    return res;
}
unsafe extern "C-unwind" fn match_capture(
    mut ms: *mut MatchState,
    mut s: *const std::ffi::c_char,
    mut l: i32,
) -> *const std::ffi::c_char {
    let mut len: size_t = 0;
    l = check_capture(ms, l);
    len = (*ms).capture[l as usize].len as size_t;
    if ((*ms).src_end).offset_from(s) as std::ffi::c_long as size_t >= len
        && memcmp(
            (*ms).capture[l as usize].init as *const c_void,
            s as *const c_void,
            len,
        ) == 0
    {
        return s.offset(len as isize);
    } else {
        return 0 as *const std::ffi::c_char;
    };
}
unsafe extern "C-unwind" fn match_0(
    mut ms: *mut MatchState,
    mut s: *const std::ffi::c_char,
    mut p: *const std::ffi::c_char,
) -> *const std::ffi::c_char {
    let mut ep_0: *const std::ffi::c_char = 0 as *const std::ffi::c_char;
    let mut current_block: u64;
    let fresh168 = (*ms).matchdepth;
    (*ms).matchdepth = (*ms).matchdepth - 1;
    if ((fresh168 == 0) as i32 != 0) as i32 as std::ffi::c_long != 0 {
        luaL_error((*ms).L, c"pattern too complex".as_ptr());
    }
    loop {
        if !(p != (*ms).p_end) {
            current_block = 6476622998065200121;
            break;
        }
        match *p as i32 {
            40 => {
                if *p.offset(1) as i32 == ')' as i32 {
                    s = start_capture(ms, s, p.offset(2), -(2 as i32));
                } else {
                    s = start_capture(ms, s, p.offset(1), -(1 as i32));
                }
                current_block = 6476622998065200121;
                break;
            }
            41 => {
                s = end_capture(ms, s, p.offset(1));
                current_block = 6476622998065200121;
                break;
            }
            36 => {
                if !(p.offset(1) != (*ms).p_end) {
                    s = if s == (*ms).src_end {
                        s
                    } else {
                        0 as *const std::ffi::c_char
                    };
                    current_block = 6476622998065200121;
                    break;
                }
            }
            37 => match *p.offset(1) as i32 {
                98 => {
                    current_block = 17965632435239708295;
                    match current_block {
                        17965632435239708295 => {
                            s = matchbalance(ms, s, p.offset(2));
                            if s.is_null() {
                                current_block = 6476622998065200121;
                                break;
                            }
                            p = p.offset(4);
                            continue;
                        }
                        8236137900636309791 => {
                            let mut ep: *const std::ffi::c_char = 0 as *const std::ffi::c_char;
                            let mut previous: std::ffi::c_char = 0;
                            p = p.offset(2);
                            if ((*p as i32 != '[' as i32) as i32 != 0) as i32 as std::ffi::c_long
                                != 0
                            {
                                luaL_error((*ms).L, c"missing '[' after '%%f' in pattern".as_ptr());
                            }
                            ep = classend(ms, p);
                            previous = (if s == (*ms).src_init {
                                '\0' as i32
                            } else {
                                *s.offset(-(1)) as i32
                            }) as std::ffi::c_char;
                            if matchbracketclass(previous as u8 as i32, p, ep.offset(-(1))) == 0
                                && matchbracketclass(*s as u8 as i32, p, ep.offset(-(1))) != 0
                            {
                                p = ep;
                                continue;
                            } else {
                                s = 0 as *const std::ffi::c_char;
                                current_block = 6476622998065200121;
                                break;
                            }
                        }
                        _ => {
                            s = match_capture(ms, s, *p.offset(1) as u8 as i32);
                            if s.is_null() {
                                current_block = 6476622998065200121;
                                break;
                            }
                            p = p.offset(2);
                            continue;
                        }
                    }
                }
                102 => {
                    current_block = 8236137900636309791;
                    match current_block {
                        17965632435239708295 => {
                            s = matchbalance(ms, s, p.offset(2));
                            if s.is_null() {
                                current_block = 6476622998065200121;
                                break;
                            }
                            p = p.offset(4);
                            continue;
                        }
                        8236137900636309791 => {
                            let mut ep: *const std::ffi::c_char = 0 as *const std::ffi::c_char;
                            let mut previous: std::ffi::c_char = 0;
                            p = p.offset(2);
                            if ((*p as i32 != '[' as i32) as i32 != 0) as i32 as std::ffi::c_long
                                != 0
                            {
                                luaL_error((*ms).L, c"missing '[' after '%%f' in pattern".as_ptr());
                            }
                            ep = classend(ms, p);
                            previous = (if s == (*ms).src_init {
                                '\0' as i32
                            } else {
                                *s.offset(-(1)) as i32
                            }) as std::ffi::c_char;
                            if matchbracketclass(previous as u8 as i32, p, ep.offset(-(1))) == 0
                                && matchbracketclass(*s as u8 as i32, p, ep.offset(-(1))) != 0
                            {
                                p = ep;
                                continue;
                            } else {
                                s = 0 as *const std::ffi::c_char;
                                current_block = 6476622998065200121;
                                break;
                            }
                        }
                        _ => {
                            s = match_capture(ms, s, *p.offset(1) as u8 as i32);
                            if s.is_null() {
                                current_block = 6476622998065200121;
                                break;
                            }
                            p = p.offset(2);
                            continue;
                        }
                    }
                }
                48 | 49 | 50 | 51 | 52 | 53 | 54 | 55 | 56 | 57 => {
                    current_block = 14576567515993809846;
                    match current_block {
                        17965632435239708295 => {
                            s = matchbalance(ms, s, p.offset(2));
                            if s.is_null() {
                                current_block = 6476622998065200121;
                                break;
                            }
                            p = p.offset(4);
                            continue;
                        }
                        8236137900636309791 => {
                            let mut ep: *const std::ffi::c_char = 0 as *const std::ffi::c_char;
                            let mut previous: std::ffi::c_char = 0;
                            p = p.offset(2);
                            if ((*p as i32 != '[' as i32) as i32 != 0) as i32 as std::ffi::c_long
                                != 0
                            {
                                luaL_error((*ms).L, c"missing '[' after '%%f' in pattern".as_ptr());
                            }
                            ep = classend(ms, p);
                            previous = (if s == (*ms).src_init {
                                '\0' as i32
                            } else {
                                *s.offset(-(1)) as i32
                            }) as std::ffi::c_char;
                            if matchbracketclass(previous as u8 as i32, p, ep.offset(-(1))) == 0
                                && matchbracketclass(*s as u8 as i32, p, ep.offset(-(1))) != 0
                            {
                                p = ep;
                                continue;
                            } else {
                                s = 0 as *const std::ffi::c_char;
                                current_block = 6476622998065200121;
                                break;
                            }
                        }
                        _ => {
                            s = match_capture(ms, s, *p.offset(1) as u8 as i32);
                            if s.is_null() {
                                current_block = 6476622998065200121;
                                break;
                            }
                            p = p.offset(2);
                            continue;
                        }
                    }
                }
                _ => {}
            },
            _ => {}
        }
        ep_0 = classend(ms, p);
        if singlematch(ms, s, p, ep_0) == 0 {
            if *ep_0 == '*' as std::ffi::c_char
                || *ep_0 == '?' as std::ffi::c_char
                || *ep_0 == '-' as std::ffi::c_char
            {
                p = ep_0.offset(1);
            } else {
                s = 0 as *const std::ffi::c_char;
                current_block = 6476622998065200121;
                break;
            }
        } else {
            match *ep_0 {
                63 => {
                    let mut res: *const std::ffi::c_char = 0 as *const std::ffi::c_char;
                    res = match_0(ms, s.offset(1), ep_0.offset(1));
                    if !res.is_null() {
                        s = res;
                        current_block = 6476622998065200121;
                        break;
                    } else {
                        p = ep_0.offset(1);
                    }
                }
                43 => {
                    s = s.offset(1);
                    s;
                    current_block = 121429310731562752;
                    break;
                }
                42 => {
                    current_block = 121429310731562752;
                    break;
                }
                45 => {
                    s = min_expand(ms, s, p, ep_0);
                    current_block = 6476622998065200121;
                    break;
                }
                _ => {
                    s = s.offset(1);
                    s;
                    p = ep_0;
                }
            }
        }
    }
    match current_block {
        121429310731562752 => {
            s = max_expand(ms, s, p, ep_0);
        }
        _ => {}
    }
    (*ms).matchdepth += 1;
    (*ms).matchdepth;
    return s;
}
unsafe extern "C-unwind" fn lmemfind(
    mut s1: *const std::ffi::c_char,
    mut l1: size_t,
    mut s2: *const std::ffi::c_char,
    mut l2: size_t,
) -> *const std::ffi::c_char {
    if l2 == 0 as size_t {
        return s1;
    } else if l2 > l1 {
        return 0 as *const std::ffi::c_char;
    } else {
        let mut init: *const std::ffi::c_char = 0 as *const std::ffi::c_char;
        l2 = l2.wrapping_sub(1);
        l2;
        l1 = l1.wrapping_sub(l2);
        while l1 > 0 as size_t && {
            init = memchr(s1 as *const c_void, *s2 as i32, l1) as *const std::ffi::c_char;
            !init.is_null()
        } {
            init = init.offset(1);
            init;
            if memcmp(init as *const c_void, s2.offset(1) as *const c_void, l2) == 0 {
                return init.offset(-(1));
            } else {
                l1 = l1.wrapping_sub(init.offset_from(s1) as std::ffi::c_long as size_t);
                s1 = init;
            }
        }
        return 0 as *const std::ffi::c_char;
    };
}
unsafe extern "C-unwind" fn get_onecapture(
    mut ms: *mut MatchState,
    mut i: i32,
    mut s: *const std::ffi::c_char,
    mut e: *const std::ffi::c_char,
    mut cap: *mut *const std::ffi::c_char,
) -> size_t {
    if i >= (*ms).level as i32 {
        if ((i != 0) as i32 != 0) as i32 as std::ffi::c_long != 0 {
            luaL_error(
                (*ms).L,
                c"invalid capture index %%%d".as_ptr(),
                i + 1 as i32,
            );
        }
        *cap = s;
        return e.offset_from(s) as std::ffi::c_long as size_t;
    } else {
        let mut capl: ptrdiff_t = (*ms).capture[i as usize].len;
        *cap = (*ms).capture[i as usize].init;
        if ((capl == -(1 as i32) as ptrdiff_t) as i32 != 0) as i32 as std::ffi::c_long != 0 {
            luaL_error((*ms).L, c"unfinished capture".as_ptr());
        } else if capl == -(2 as i32) as ptrdiff_t {
            lua_pushinteger(
                (*ms).L,
                (((*ms).capture[i as usize].init).offset_from((*ms).src_init) + 1) as lua_Integer,
            );
        }
        return capl as size_t;
    };
}
unsafe extern "C-unwind" fn push_onecapture(
    mut ms: *mut MatchState,
    mut i: i32,
    mut s: *const std::ffi::c_char,
    mut e: *const std::ffi::c_char,
) {
    let mut cap: *const std::ffi::c_char = 0 as *const std::ffi::c_char;
    let mut l: ptrdiff_t = get_onecapture(ms, i, s, e, &mut cap) as ptrdiff_t;
    if l != -(2 as i32) as ptrdiff_t {
        lua_pushlstring((*ms).L, cap, l as size_t);
    }
}
unsafe extern "C-unwind" fn push_captures(
    mut ms: *mut MatchState,
    mut s: *const std::ffi::c_char,
    mut e: *const std::ffi::c_char,
) -> i32 {
    let mut i: i32 = 0;
    let mut nlevels: i32 = if (*ms).level as i32 == 0 && !s.is_null() {
        1 as i32
    } else {
        (*ms).level as i32
    };
    luaL_checkstack((*ms).L, nlevels, c"too many captures".as_ptr());
    i = 0;
    while i < nlevels {
        push_onecapture(ms, i, s, e);
        i += 1;
        i;
    }
    return nlevels;
}
unsafe extern "C-unwind" fn nospecials(mut p: *const std::ffi::c_char, mut l: size_t) -> i32 {
    let mut upto: size_t = 0 as size_t;
    loop {
        if !(strpbrk(p.offset(upto as isize), c"^$*+?.([%-".as_ptr())).is_null() {
            return 0;
        }
        upto = (upto as usize).wrapping_add((strlen(p.offset(upto as isize))).wrapping_add(1))
            as size_t as size_t;
        if !(upto <= l) {
            break;
        }
    }
    return 1 as i32;
}
unsafe extern "C-unwind" fn prepstate(
    mut ms: *mut MatchState,
    mut L: *mut lua_State,
    mut s: *const std::ffi::c_char,
    mut ls: size_t,
    mut p: *const std::ffi::c_char,
    mut lp: size_t,
) {
    (*ms).L = L;
    (*ms).matchdepth = 200;
    (*ms).src_init = s;
    (*ms).src_end = s.offset(ls as isize);
    (*ms).p_end = p.offset(lp as isize);
}
unsafe extern "C-unwind" fn reprepstate(mut ms: *mut MatchState) {
    (*ms).level = 0 as u8;
}
unsafe extern "C-unwind" fn str_find_aux(mut L: *mut lua_State, mut find: i32) -> i32 {
    let mut ls: size_t = 0;
    let mut lp: size_t = 0;
    let mut s: *const std::ffi::c_char = luaL_checklstring(L, 1 as i32, &mut ls);
    let mut p: *const std::ffi::c_char = luaL_checklstring(L, 2 as i32, &mut lp);
    let mut init: size_t = (posrelatI(luaL_optinteger(L, 3 as i32, 1 as i32 as lua_Integer), ls))
        .wrapping_sub(1 as i32 as size_t);
    if init > ls {
        lua_pushnil(L);
        return 1 as i32;
    }
    if find != 0 && (lua_toboolean(L, 4 as i32) != 0 || nospecials(p, lp) != 0) {
        let mut s2: *const std::ffi::c_char =
            lmemfind(s.offset(init as isize), ls.wrapping_sub(init), p, lp);
        if !s2.is_null() {
            lua_pushinteger(L, (s2.offset_from(s) + 1) as lua_Integer);
            lua_pushinteger(
                L,
                (s2.offset_from(s) as std::ffi::c_long as size_t).wrapping_add(lp) as lua_Integer,
            );
            return 2 as i32;
        }
    } else {
        let mut ms: MatchState = MatchState {
            src_init: 0 as *const std::ffi::c_char,
            src_end: 0 as *const std::ffi::c_char,
            p_end: 0 as *const std::ffi::c_char,
            L: 0 as *mut lua_State,
            matchdepth: 0,
            level: 0,
            capture: [C2RustUnnamed_18 {
                init: 0 as *const std::ffi::c_char,
                len: 0,
            }; 32],
        };
        let mut s1: *const std::ffi::c_char = s.offset(init as isize);
        let mut anchor: i32 = (*p as i32 == '^' as i32) as i32;
        if anchor != 0 {
            p = p.offset(1);
            p;
            lp = lp.wrapping_sub(1);
            lp;
        }
        prepstate(&mut ms, L, s, ls, p, lp);
        loop {
            let mut res: *const std::ffi::c_char = 0 as *const std::ffi::c_char;
            reprepstate(&mut ms);
            res = match_0(&mut ms, s1, p);
            if !res.is_null() {
                if find != 0 {
                    lua_pushinteger(L, (s1.offset_from(s) + 1) as lua_Integer);
                    lua_pushinteger(L, res.offset_from(s) as std::ffi::c_long as lua_Integer);
                    return push_captures(
                        &mut ms,
                        0 as *const std::ffi::c_char,
                        0 as *const std::ffi::c_char,
                    ) + 2 as i32;
                } else {
                    return push_captures(&mut ms, s1, res);
                }
            }
            let fresh169 = s1;
            s1 = s1.offset(1);
            if !(fresh169 < ms.src_end && anchor == 0) {
                break;
            }
        }
    }
    lua_pushnil(L);
    return 1 as i32;
}
unsafe extern "C-unwind" fn str_find(mut L: *mut lua_State) -> i32 {
    return str_find_aux(L, 1 as i32);
}
unsafe extern "C-unwind" fn str_match(mut L: *mut lua_State) -> i32 {
    return str_find_aux(L, 0);
}
unsafe extern "C-unwind" fn gmatch_aux(mut L: *mut lua_State) -> i32 {
    let mut gm: *mut GMatchState =
        lua_touserdata(L, -(1000000) - 1000 - 3 as i32) as *mut GMatchState;
    let mut src: *const std::ffi::c_char = 0 as *const std::ffi::c_char;
    (*gm).ms.L = L;
    src = (*gm).src;
    while src <= (*gm).ms.src_end {
        let mut e: *const std::ffi::c_char = 0 as *const std::ffi::c_char;
        reprepstate(&mut (*gm).ms);
        e = match_0(&mut (*gm).ms, src, (*gm).p);
        if !e.is_null() && e != (*gm).lastmatch {
            (*gm).lastmatch = e;
            (*gm).src = (*gm).lastmatch;
            return push_captures(&mut (*gm).ms, src, e);
        }
        src = src.offset(1);
        src;
    }
    return 0;
}
unsafe extern "C-unwind" fn gmatch(mut L: *mut lua_State) -> i32 {
    let mut ls: size_t = 0;
    let mut lp: size_t = 0;
    let mut s: *const std::ffi::c_char = luaL_checklstring(L, 1 as i32, &mut ls);
    let mut p: *const std::ffi::c_char = luaL_checklstring(L, 2 as i32, &mut lp);
    let mut init: size_t = (posrelatI(luaL_optinteger(L, 3 as i32, 1 as i32 as lua_Integer), ls))
        .wrapping_sub(1 as i32 as size_t);
    let mut gm: *mut GMatchState = 0 as *mut GMatchState;
    lua_settop(L, 2 as i32);
    gm = lua_newuserdatauv(L, size_of::<GMatchState>() as usize, 0) as *mut GMatchState;
    if init > ls {
        init = ls.wrapping_add(1 as i32 as size_t);
    }
    prepstate(&mut (*gm).ms, L, s, ls, p, lp);
    (*gm).src = s.offset(init as isize);
    (*gm).p = p;
    (*gm).lastmatch = 0 as *const std::ffi::c_char;
    lua_pushcclosure(
        L,
        Some(gmatch_aux as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
        3 as i32,
    );
    return 1 as i32;
}
unsafe extern "C-unwind" fn add_s(
    mut ms: *mut MatchState,
    mut b: *mut luaL_Buffer,
    mut s: *const std::ffi::c_char,
    mut e: *const std::ffi::c_char,
) {
    let mut l: size_t = 0;
    let mut L: *mut lua_State = (*ms).L;
    let mut news: *const std::ffi::c_char = lua_tolstring(L, 3 as i32, &mut l);
    let mut p: *const std::ffi::c_char = 0 as *const std::ffi::c_char;
    loop {
        p = memchr(news as *const c_void, '%' as i32, l) as *mut std::ffi::c_char;
        if p.is_null() {
            break;
        }
        luaL_addlstring(b, news, p.offset_from(news) as std::ffi::c_long as size_t);
        p = p.offset(1);
        p;
        if *p as i32 == '%' as i32 {
            ((*b).n < (*b).size || !(luaL_prepbuffsize(b, 1 as i32 as size_t)).is_null()) as i32;
            let fresh170 = (*b).n;
            (*b).n = ((*b).n).wrapping_add(1);
            *((*b).b).offset(fresh170 as isize) = *p;
        } else if *p as i32 == '0' as i32 {
            luaL_addlstring(b, s, e.offset_from(s) as std::ffi::c_long as size_t);
        } else if *(*__ctype_b_loc()).offset(*p as u8 as i32 as isize) as i32
            & _ISdigit as i32 as u16 as i32
            != 0
        {
            let mut cap: *const std::ffi::c_char = 0 as *const std::ffi::c_char;
            let mut resl: ptrdiff_t =
                get_onecapture(ms, *p as i32 - '1' as i32, s, e, &mut cap) as ptrdiff_t;
            if resl == -(2 as i32) as ptrdiff_t {
                luaL_addvalue(b);
            } else {
                luaL_addlstring(b, cap, resl as size_t);
            }
        } else {
            luaL_error(
                L,
                c"invalid use of '%c' in replacement string".as_ptr(),
                '%' as i32,
            );
        }
        l = l.wrapping_sub(p.offset(1).offset_from(news) as std::ffi::c_long as size_t);
        news = p.offset(1);
    }
    luaL_addlstring(b, news, l);
}
unsafe extern "C-unwind" fn add_value(
    mut ms: *mut MatchState,
    mut b: *mut luaL_Buffer,
    mut s: *const std::ffi::c_char,
    mut e: *const std::ffi::c_char,
    mut tr: i32,
) -> i32 {
    let mut L: *mut lua_State = (*ms).L;
    match tr {
        6 => {
            let mut n: i32 = 0;
            lua_pushvalue(L, 3 as i32);
            n = push_captures(ms, s, e);
            lua_callk(L, n, 1 as i32, 0 as lua_KContext, None);
        }
        5 => {
            push_onecapture(ms, 0, s, e);
            lua_gettable(L, 3 as i32);
        }
        _ => {
            add_s(ms, b, s, e);
            return 1 as i32;
        }
    }
    if lua_toboolean(L, -(1 as i32)) == 0 {
        lua_settop(L, -(1 as i32) - 1 as i32);
        luaL_addlstring(b, s, e.offset_from(s) as std::ffi::c_long as size_t);
        return 0;
    } else if ((lua_isstring(L, -(1 as i32)) == 0) as i32 != 0) as i32 as std::ffi::c_long != 0 {
        return luaL_error(
            L,
            c"invalid replacement value (a %s)".as_ptr(),
            lua_typename(L, lua_type(L, -(1 as i32))),
        );
    } else {
        luaL_addvalue(b);
        return 1 as i32;
    };
}
unsafe extern "C-unwind" fn str_gsub(mut L: *mut lua_State) -> i32 {
    let mut srcl: size_t = 0;
    let mut lp: size_t = 0;
    let mut src: *const std::ffi::c_char = luaL_checklstring(L, 1 as i32, &mut srcl);
    let mut p: *const std::ffi::c_char = luaL_checklstring(L, 2 as i32, &mut lp);
    let mut lastmatch: *const std::ffi::c_char = 0 as *const std::ffi::c_char;
    let mut tr: i32 = lua_type(L, 3 as i32);
    let mut max_s: lua_Integer = luaL_optinteger(
        L,
        4 as i32,
        srcl.wrapping_add(1 as i32 as size_t) as lua_Integer,
    );
    let mut anchor: i32 = (*p as i32 == '^' as i32) as i32;
    let mut n: lua_Integer = 0 as lua_Integer;
    let mut changed: i32 = 0;
    let mut ms: MatchState = MatchState {
        src_init: 0 as *const std::ffi::c_char,
        src_end: 0 as *const std::ffi::c_char,
        p_end: 0 as *const std::ffi::c_char,
        L: 0 as *mut lua_State,
        matchdepth: 0,
        level: 0,
        capture: [C2RustUnnamed_18 {
            init: 0 as *const std::ffi::c_char,
            len: 0,
        }; 32],
    };
    let mut b: luaL_Buffer = luaL_Buffer {
        b: 0 as *mut std::ffi::c_char,
        size: 0,
        n: 0,
        L: 0 as *mut lua_State,
        init: C2RustUnnamed_15 { n: 0. },
    };
    (((tr == 3 as i32 || tr == 4 as i32 || tr == 6 as i32 || tr == 5 as i32) as i32 != 0) as i32
        as std::ffi::c_long
        != 0
        || luaL_typeerror(L, 3 as i32, c"string/function/table".as_ptr()) != 0) as i32;
    luaL_buffinit(L, &mut b);
    if anchor != 0 {
        p = p.offset(1);
        p;
        lp = lp.wrapping_sub(1);
        lp;
    }
    prepstate(&mut ms, L, src, srcl, p, lp);
    while n < max_s {
        let mut e: *const std::ffi::c_char = 0 as *const std::ffi::c_char;
        reprepstate(&mut ms);
        e = match_0(&mut ms, src, p);
        if !e.is_null() && e != lastmatch {
            n += 1;
            n;
            changed = add_value(&mut ms, &mut b, src, e, tr) | changed;
            lastmatch = e;
            src = lastmatch;
        } else {
            if !(src < ms.src_end) {
                break;
            }
            (b.n < b.size || !(luaL_prepbuffsize(&mut b, 1 as i32 as size_t)).is_null()) as i32;
            let fresh171 = src;
            src = src.offset(1);
            let fresh172 = b.n;
            b.n = (b.n).wrapping_add(1);
            *(b.b).offset(fresh172 as isize) = *fresh171;
        }
        if anchor != 0 {
            break;
        }
    }
    if changed == 0 {
        lua_pushvalue(L, 1 as i32);
    } else {
        luaL_addlstring(
            &mut b,
            src,
            (ms.src_end).offset_from(src) as std::ffi::c_long as size_t,
        );
        luaL_pushresult(&mut b);
    }
    lua_pushinteger(L, n);
    return 2 as i32;
}
unsafe extern "C-unwind" fn addquoted(
    mut b: *mut luaL_Buffer,
    mut s: *const std::ffi::c_char,
    mut len: size_t,
) {
    ((*b).n < (*b).size || !(luaL_prepbuffsize(b, 1 as i32 as size_t)).is_null()) as i32;
    let fresh173 = (*b).n;
    (*b).n = ((*b).n).wrapping_add(1);
    *((*b).b).offset(fresh173 as isize) = '"' as i32 as std::ffi::c_char;
    loop {
        let fresh174 = len;
        len = len.wrapping_sub(1);
        if !(fresh174 != 0) {
            break;
        }
        if *s as i32 == '"' as i32 || *s as i32 == '\\' as i32 || *s as i32 == '\n' as i32 {
            ((*b).n < (*b).size || !(luaL_prepbuffsize(b, 1 as i32 as size_t)).is_null()) as i32;
            let fresh175 = (*b).n;
            (*b).n = ((*b).n).wrapping_add(1);
            *((*b).b).offset(fresh175 as isize) = '\\' as i32 as std::ffi::c_char;
            ((*b).n < (*b).size || !(luaL_prepbuffsize(b, 1 as i32 as size_t)).is_null()) as i32;
            let fresh176 = (*b).n;
            (*b).n = ((*b).n).wrapping_add(1);
            *((*b).b).offset(fresh176 as isize) = *s;
        } else if *(*__ctype_b_loc()).offset(*s as u8 as i32 as isize) as i32
            & _IScntrl as i32 as u16 as i32
            != 0
        {
            let mut buff: [std::ffi::c_char; 10] = [0; 10];
            if *(*__ctype_b_loc()).offset(*s.offset(1) as u8 as i32 as isize) as i32
                & _ISdigit as i32 as u16 as i32
                == 0
            {
                snprintf(
                    buff.as_mut_ptr(),
                    size_of::<[std::ffi::c_char; 10]>() as usize,
                    c"\\%d".as_ptr(),
                    *s as u8 as i32,
                );
            } else {
                snprintf(
                    buff.as_mut_ptr(),
                    size_of::<[std::ffi::c_char; 10]>() as usize,
                    c"\\%03d".as_ptr(),
                    *s as u8 as i32,
                );
            }
            luaL_addstring(b, buff.as_mut_ptr());
        } else {
            ((*b).n < (*b).size || !(luaL_prepbuffsize(b, 1 as i32 as size_t)).is_null()) as i32;
            let fresh177 = (*b).n;
            (*b).n = ((*b).n).wrapping_add(1);
            *((*b).b).offset(fresh177 as isize) = *s;
        }
        s = s.offset(1);
        s;
    }
    ((*b).n < (*b).size || !(luaL_prepbuffsize(b, 1 as i32 as size_t)).is_null()) as i32;
    let fresh178 = (*b).n;
    (*b).n = ((*b).n).wrapping_add(1);
    *((*b).b).offset(fresh178 as isize) = '"' as i32 as std::ffi::c_char;
}
unsafe extern "C-unwind" fn quotefloat(
    mut L: *mut lua_State,
    mut buff: *mut std::ffi::c_char,
    mut n: lua_Number,
) -> i32 {
    let mut s: *const std::ffi::c_char = 0 as *const std::ffi::c_char;
    if n == ::core::f64::INFINITY {
        s = c"1e9999".as_ptr();
    } else if n == -::core::f64::INFINITY {
        s = c"-1e9999".as_ptr();
    } else if n != n {
        s = c"(0/0)".as_ptr();
    } else {
        let mut nb: i32 = snprintf(buff, 120 as usize, c"%a".as_ptr(), n);
        if (memchr(buff as *const c_void, '.' as i32, nb as usize)).is_null() {
            let mut point: std::ffi::c_char = *((*localeconv()).decimal_point).offset(0 as isize);
            let mut ppoint: *mut std::ffi::c_char =
                memchr(buff as *const c_void, point as i32, nb as usize) as *mut std::ffi::c_char;
            if !ppoint.is_null() {
                *ppoint = '.' as i32 as std::ffi::c_char;
            }
        }
        return nb;
    }
    return snprintf(buff, 120 as usize, c"%s".as_ptr(), s);
}
unsafe extern "C-unwind" fn addliteral(
    mut L: *mut lua_State,
    mut b: *mut luaL_Buffer,
    mut arg: i32,
) {
    match lua_type(L, arg) {
        4 => {
            let mut len: size_t = 0;
            let mut s: *const std::ffi::c_char = lua_tolstring(L, arg, &mut len);
            addquoted(b, s, len);
        }
        3 => {
            let mut buff: *mut std::ffi::c_char = luaL_prepbuffsize(b, 120 as size_t);
            let mut nb: i32 = 0;
            if lua_isinteger(L, arg) == 0 {
                nb = quotefloat(L, buff, lua_tonumberx(L, arg, 0 as *mut i32));
            } else {
                let mut n: lua_Integer = lua_tointegerx(L, arg, 0 as *mut i32);
                let mut format: *const std::ffi::c_char = if n
                    == -(9223372036854775807 as std::ffi::c_longlong) - 1 as std::ffi::c_longlong
                {
                    c"0x%llx".as_ptr()
                } else {
                    c"%lld".as_ptr()
                };
                nb = snprintf(buff, 120 as usize, format, n);
            }
            (*b).n = ((*b).n).wrapping_add(nb as size_t);
        }
        0 | 1 => {
            luaL_tolstring(L, arg, 0 as *mut size_t);
            luaL_addvalue(b);
        }
        _ => {
            luaL_argerror(L, arg, c"value has no literal form".as_ptr());
        }
    };
}
unsafe extern "C-unwind" fn get2digits(mut s: *const std::ffi::c_char) -> *const std::ffi::c_char {
    if *(*__ctype_b_loc()).offset(*s as u8 as i32 as isize) as i32 & _ISdigit as i32 as u16 as i32
        != 0
    {
        s = s.offset(1);
        s;
        if *(*__ctype_b_loc()).offset(*s as u8 as i32 as isize) as i32
            & _ISdigit as i32 as u16 as i32
            != 0
        {
            s = s.offset(1);
            s;
        }
    }
    return s;
}
unsafe extern "C-unwind" fn checkformat(
    mut L: *mut lua_State,
    mut form: *const std::ffi::c_char,
    mut flags: *const std::ffi::c_char,
    mut precision: i32,
) {
    let mut spec: *const std::ffi::c_char = form.offset(1);
    spec = spec.offset(strspn(spec, flags) as isize);
    if *spec as i32 != '0' as i32 {
        spec = get2digits(spec);
        if *spec as i32 == '.' as i32 && precision != 0 {
            spec = spec.offset(1);
            spec;
            spec = get2digits(spec);
        }
    }
    if *(*__ctype_b_loc()).offset(*spec as u8 as i32 as isize) as i32
        & _ISalpha as i32 as u16 as i32
        == 0
    {
        luaL_error(L, c"invalid conversion specification: '%s'".as_ptr(), form);
    }
}
unsafe extern "C-unwind" fn getformat(
    mut L: *mut lua_State,
    mut strfrmt: *const std::ffi::c_char,
    mut form: *mut std::ffi::c_char,
) -> *const std::ffi::c_char {
    let mut len: size_t = strspn(strfrmt, c"-+#0 123456789.".as_ptr());
    len = len.wrapping_add(1);
    len;
    if len >= (32 as i32 - 10) as size_t {
        luaL_error(L, c"invalid format (too long)".as_ptr());
    }
    let fresh179 = form;
    form = form.offset(1);
    *fresh179 = '%' as i32 as std::ffi::c_char;
    memcpy(
        form as *mut c_void,
        strfrmt as *const c_void,
        len.wrapping_mul(size_of::<std::ffi::c_char>() as usize),
    );
    *form.offset(len as isize) = '\0' as i32 as std::ffi::c_char;
    return strfrmt.offset(len as isize).offset(-(1));
}
unsafe extern "C-unwind" fn addlenmod(
    mut form: *mut std::ffi::c_char,
    mut lenmod: *const std::ffi::c_char,
) {
    let mut l: size_t = strlen(form);
    let mut lm: size_t = strlen(lenmod);
    let mut spec: std::ffi::c_char = *form.offset(l.wrapping_sub(1 as i32 as size_t) as isize);
    strcpy(form.offset(l as isize).offset(-(1)), lenmod);
    *form.offset(l.wrapping_add(lm).wrapping_sub(1 as i32 as size_t) as isize) = spec;
    *form.offset(l.wrapping_add(lm) as isize) = '\0' as i32 as std::ffi::c_char;
}
unsafe extern "C-unwind" fn str_format(mut L: *mut lua_State) -> i32 {
    let mut current_block: u64;
    let mut top: i32 = lua_gettop(L);
    let mut arg: i32 = 1 as i32;
    let mut sfl: size_t = 0;
    let mut strfrmt: *const std::ffi::c_char = luaL_checklstring(L, arg, &mut sfl);
    let mut strfrmt_end: *const std::ffi::c_char = strfrmt.offset(sfl as isize);
    let mut flags: *const std::ffi::c_char = 0 as *const std::ffi::c_char;
    let mut b: luaL_Buffer = luaL_Buffer {
        b: 0 as *mut std::ffi::c_char,
        size: 0,
        n: 0,
        L: 0 as *mut lua_State,
        init: C2RustUnnamed_15 { n: 0. },
    };
    luaL_buffinit(L, &mut b);
    while strfrmt < strfrmt_end {
        if *strfrmt as i32 != '%' as i32 {
            (b.n < b.size || !(luaL_prepbuffsize(&mut b, 1 as i32 as size_t)).is_null()) as i32;
            let fresh180 = strfrmt;
            strfrmt = strfrmt.offset(1);
            let fresh181 = b.n;
            b.n = (b.n).wrapping_add(1);
            *(b.b).offset(fresh181 as isize) = *fresh180;
        } else {
            strfrmt = strfrmt.offset(1);
            if *strfrmt as i32 == '%' as i32 {
                (b.n < b.size || !(luaL_prepbuffsize(&mut b, 1 as i32 as size_t)).is_null()) as i32;
                let fresh182 = strfrmt;
                strfrmt = strfrmt.offset(1);
                let fresh183 = b.n;
                b.n = (b.n).wrapping_add(1);
                *(b.b).offset(fresh183 as isize) = *fresh182;
            } else {
                let mut form: [std::ffi::c_char; 32] = [0; 32];
                let mut maxitem: i32 = 120;
                let mut buff: *mut std::ffi::c_char = luaL_prepbuffsize(&mut b, maxitem as size_t);
                let mut nb: i32 = 0;
                arg += 1;
                if arg > top {
                    return luaL_argerror(L, arg, c"no value".as_ptr());
                }
                strfrmt = getformat(L, strfrmt, form.as_mut_ptr());
                let fresh184 = strfrmt;
                strfrmt = strfrmt.offset(1);
                match *fresh184 as i32 {
                    99 => {
                        checkformat(L, form.as_mut_ptr(), c"-".as_ptr(), 0);
                        nb = snprintf(
                            buff,
                            maxitem as usize,
                            form.as_mut_ptr(),
                            luaL_checkinteger(L, arg) as i32,
                        );
                        current_block = 11793792312832361944;
                    }
                    100 | 105 => {
                        flags = c"-+0 ".as_ptr();
                        current_block = 5689001924483802034;
                    }
                    117 => {
                        flags = c"-0".as_ptr();
                        current_block = 5689001924483802034;
                    }
                    111 | 120 | 88 => {
                        flags = c"-#0".as_ptr();
                        current_block = 5689001924483802034;
                    }
                    97 | 65 => {
                        checkformat(L, form.as_mut_ptr(), c"-+#0 ".as_ptr(), 1 as i32);
                        addlenmod(form.as_mut_ptr(), c"".as_ptr());
                        nb = snprintf(
                            buff,
                            maxitem as usize,
                            form.as_mut_ptr(),
                            luaL_checknumber(L, arg),
                        );
                        current_block = 11793792312832361944;
                    }
                    102 => {
                        maxitem = 110 + 308 as i32;
                        buff = luaL_prepbuffsize(&mut b, maxitem as size_t);
                        current_block = 6669252993407410313;
                    }
                    101 | 69 | 103 | 71 => {
                        current_block = 6669252993407410313;
                    }
                    112 => {
                        let mut p: *const c_void = lua_topointer(L, arg);
                        checkformat(L, form.as_mut_ptr(), c"-".as_ptr(), 0);
                        if p.is_null() {
                            p = c"(null)".as_ptr() as *const c_void;
                            form[(strlen(form.as_mut_ptr())).wrapping_sub(1) as usize] =
                                's' as i32 as std::ffi::c_char;
                        }
                        nb = snprintf(buff, maxitem as usize, form.as_mut_ptr(), p);
                        current_block = 11793792312832361944;
                    }
                    113 => {
                        if form[2] as i32 != '\0' as i32 {
                            return luaL_error(
                                L,
                                c"specifier '%%q' cannot have modifiers".as_ptr(),
                            );
                        }
                        addliteral(L, &mut b, arg);
                        current_block = 11793792312832361944;
                    }
                    115 => {
                        let mut l: size_t = 0;
                        let mut s: *const std::ffi::c_char = luaL_tolstring(L, arg, &mut l);
                        if form[2] as i32 == '\0' as i32 {
                            luaL_addvalue(&mut b);
                        } else {
                            (((l == strlen(s)) as i32 != 0) as i32 as std::ffi::c_long != 0
                                || luaL_argerror(L, arg, c"string contains zeros".as_ptr()) != 0)
                                as i32;
                            checkformat(L, form.as_mut_ptr(), c"-".as_ptr(), 1 as i32);
                            if (strchr(form.as_mut_ptr(), '.' as i32)).is_null()
                                && l >= 100 as size_t
                            {
                                luaL_addvalue(&mut b);
                            } else {
                                nb = snprintf(buff, maxitem as usize, form.as_mut_ptr(), s);
                                lua_settop(L, -(1 as i32) - 1 as i32);
                            }
                        }
                        current_block = 11793792312832361944;
                    }
                    _ => {
                        return luaL_error(
                            L,
                            c"invalid conversion '%s' to 'format'".as_ptr(),
                            form.as_mut_ptr(),
                        );
                    }
                }
                match current_block {
                    5689001924483802034 => {
                        let mut n: lua_Integer = luaL_checkinteger(L, arg);
                        checkformat(L, form.as_mut_ptr(), flags, 1 as i32);
                        addlenmod(form.as_mut_ptr(), c"ll".as_ptr());
                        nb = snprintf(buff, maxitem as usize, form.as_mut_ptr(), n);
                    }
                    6669252993407410313 => {
                        let mut n_0: lua_Number = luaL_checknumber(L, arg);
                        checkformat(L, form.as_mut_ptr(), c"-+#0 ".as_ptr(), 1 as i32);
                        addlenmod(form.as_mut_ptr(), c"".as_ptr());
                        nb = snprintf(buff, maxitem as usize, form.as_mut_ptr(), n_0);
                    }
                    _ => {}
                }
                b.n = (b.n).wrapping_add(nb as size_t);
            }
        }
    }
    luaL_pushresult(&mut b);
    return 1 as i32;
}
static mut nativeendian: C2RustUnnamed_16 = C2RustUnnamed_16 { dummy: 1 as i32 };
unsafe extern "C-unwind" fn digit(mut c: i32) -> i32 {
    return ('0' as i32 <= c && c <= '9' as i32) as i32;
}
unsafe extern "C-unwind" fn getnum(mut fmt: *mut *const std::ffi::c_char, mut df: i32) -> i32 {
    if digit(**fmt as i32) == 0 {
        return df;
    } else {
        let mut a: i32 = 0;
        loop {
            let fresh185 = *fmt;
            *fmt = (*fmt).offset(1);
            a = a * 10 + (*fresh185 as i32 - '0' as i32);
            if !(digit(**fmt as i32) != 0
                && a <= ((if (size_of::<size_t>() as usize) < size_of::<i32>() as usize {
                    !(0 as size_t)
                } else {
                    2147483647 as i32 as size_t
                }) as i32
                    - 9 as i32)
                    / 10)
            {
                break;
            }
        }
        return a;
    };
}
unsafe extern "C-unwind" fn getnumlimit(
    mut h: *mut Header,
    mut fmt: *mut *const std::ffi::c_char,
    mut df: i32,
) -> i32 {
    let mut sz: i32 = getnum(fmt, df);
    if ((sz > 16 as i32 || sz <= 0) as i32 != 0) as i32 as std::ffi::c_long != 0 {
        return luaL_error(
            (*h).L,
            c"integral size (%d) out of limits [1,%d]".as_ptr(),
            sz,
            16 as i32,
        );
    }
    return sz;
}
unsafe extern "C-unwind" fn initheader(mut L: *mut lua_State, mut h: *mut Header) {
    (*h).L = L;
    (*h).islittle = nativeendian.little as i32;
    (*h).maxalign = 1 as i32;
}
unsafe extern "C-unwind" fn getoption(
    mut h: *mut Header,
    mut fmt: *mut *const std::ffi::c_char,
    mut size: *mut i32,
) -> KOption {
    let fresh186 = *fmt;
    *fmt = (*fmt).offset(1);
    let mut opt: i32 = *fresh186 as i32;
    *size = 0;
    match opt {
        98 => {
            *size = size_of::<std::ffi::c_char>() as usize as i32;
            return Kint;
        }
        66 => {
            *size = size_of::<std::ffi::c_char>() as usize as i32;
            return Kuint;
        }
        104 => {
            *size = size_of::<i16>() as usize as i32;
            return Kint;
        }
        72 => {
            *size = size_of::<i16>() as usize as i32;
            return Kuint;
        }
        108 => {
            *size = size_of::<std::ffi::c_long>() as usize as i32;
            return Kint;
        }
        76 => {
            *size = size_of::<std::ffi::c_long>() as usize as i32;
            return Kuint;
        }
        106 => {
            *size = size_of::<lua_Integer>() as usize as i32;
            return Kint;
        }
        74 => {
            *size = size_of::<lua_Integer>() as usize as i32;
            return Kuint;
        }
        84 => {
            *size = size_of::<size_t>() as usize as i32;
            return Kuint;
        }
        102 => {
            *size = size_of::<std::ffi::c_float>() as usize as i32;
            return Kfloat;
        }
        110 => {
            *size = size_of::<lua_Number>() as usize as i32;
            return Knumber;
        }
        100 => {
            *size = size_of::<std::ffi::c_double>() as usize as i32;
            return Kdouble;
        }
        105 => {
            *size = getnumlimit(h, fmt, size_of::<i32>() as usize as i32);
            return Kint;
        }
        73 => {
            *size = getnumlimit(h, fmt, size_of::<i32>() as usize as i32);
            return Kuint;
        }
        115 => {
            *size = getnumlimit(h, fmt, size_of::<size_t>() as usize as i32);
            return Kstring;
        }
        99 => {
            *size = getnum(fmt, -(1 as i32));
            if ((*size == -(1 as i32)) as i32 != 0) as i32 as std::ffi::c_long != 0 {
                luaL_error((*h).L, c"missing size for format option 'c'".as_ptr());
            }
            return Kchar;
        }
        122 => return Kzstr,
        120 => {
            *size = 1 as i32;
            return Kpadding;
        }
        88 => return Kpaddalign,
        32 => {}
        60 => {
            (*h).islittle = 1 as i32;
        }
        62 => {
            (*h).islittle = 0;
        }
        61 => {
            (*h).islittle = nativeendian.little as i32;
        }
        33 => {
            let maxalign: i32 = 8 as usize as i32;
            (*h).maxalign = getnumlimit(h, fmt, maxalign);
        }
        _ => {
            luaL_error((*h).L, c"invalid format option '%c'".as_ptr(), opt);
        }
    }
    return Knop;
}
unsafe extern "C-unwind" fn getdetails(
    mut h: *mut Header,
    mut totalsize: size_t,
    mut fmt: *mut *const std::ffi::c_char,
    mut psize: *mut i32,
    mut ntoalign: *mut i32,
) -> KOption {
    let mut opt: KOption = getoption(h, fmt, psize);
    let mut align: i32 = *psize;
    if opt as u32 == Kpaddalign as i32 as u32 {
        if **fmt as i32 == '\0' as i32
            || getoption(h, fmt, &mut align) as u32 == Kchar as i32 as u32
            || align == 0
        {
            luaL_argerror(
                (*h).L,
                1 as i32,
                c"invalid next option for option 'X'".as_ptr(),
            );
        }
    }
    if align <= 1 as i32 || opt as u32 == Kchar as i32 as u32 {
        *ntoalign = 0;
    } else {
        if align > (*h).maxalign {
            align = (*h).maxalign;
        }
        if ((align & align - 1 as i32 != 0) as i32 != 0) as i32 as std::ffi::c_long != 0 {
            luaL_argerror(
                (*h).L,
                1 as i32,
                c"format asks for alignment not power of 2".as_ptr(),
            );
        }
        *ntoalign = align - (totalsize & (align - 1 as i32) as size_t) as i32 & align - 1 as i32;
    }
    return opt;
}
unsafe extern "C-unwind" fn packint(
    mut b: *mut luaL_Buffer,
    mut n: lua_Unsigned,
    mut islittle: i32,
    mut size: i32,
    mut neg: i32,
) {
    let mut buff: *mut std::ffi::c_char = luaL_prepbuffsize(b, size as size_t);
    let mut i: i32 = 0;
    *buff.offset((if islittle != 0 { 0 } else { size - 1 as i32 }) as isize) =
        (n & (((1 as i32) << 8 as i32) - 1 as i32) as lua_Unsigned) as std::ffi::c_char;
    i = 1 as i32;
    while i < size {
        n >>= 8 as i32;
        *buff.offset(
            (if islittle != 0 {
                i
            } else {
                size - 1 as i32 - i
            }) as isize,
        ) = (n & (((1 as i32) << 8 as i32) - 1 as i32) as lua_Unsigned) as std::ffi::c_char;
        i += 1;
        i;
    }
    if neg != 0 && size > size_of::<lua_Integer>() as usize as i32 {
        i = size_of::<lua_Integer>() as usize as i32;
        while i < size {
            *buff.offset(
                (if islittle != 0 {
                    i
                } else {
                    size - 1 as i32 - i
                }) as isize,
            ) = (((1 as i32) << 8 as i32) - 1 as i32) as std::ffi::c_char;
            i += 1;
            i;
        }
    }
    (*b).n = ((*b).n).wrapping_add(size as size_t);
}
unsafe extern "C-unwind" fn copywithendian(
    mut dest: *mut std::ffi::c_char,
    mut src: *const std::ffi::c_char,
    mut size: i32,
    mut islittle: i32,
) {
    if islittle == nativeendian.little as i32 {
        memcpy(dest as *mut c_void, src as *const c_void, size as usize);
    } else {
        dest = dest.offset((size - 1 as i32) as isize);
        loop {
            let fresh187 = size;
            size = size - 1;
            if !(fresh187 != 0) {
                break;
            }
            let fresh188 = src;
            src = src.offset(1);
            let fresh189 = dest;
            dest = dest.offset(-1);
            *fresh189 = *fresh188;
        }
    };
}
unsafe extern "C-unwind" fn str_pack(mut L: *mut lua_State) -> i32 {
    let mut b: luaL_Buffer = luaL_Buffer {
        b: 0 as *mut std::ffi::c_char,
        size: 0,
        n: 0,
        L: 0 as *mut lua_State,
        init: C2RustUnnamed_15 { n: 0. },
    };
    let mut h: Header = Header {
        L: 0 as *mut lua_State,
        islittle: 0,
        maxalign: 0,
    };
    let mut fmt: *const std::ffi::c_char = luaL_checklstring(L, 1 as i32, 0 as *mut size_t);
    let mut arg: i32 = 1 as i32;
    let mut totalsize: size_t = 0 as size_t;
    initheader(L, &mut h);
    lua_pushnil(L);
    luaL_buffinit(L, &mut b);
    while *fmt as i32 != '\0' as i32 {
        let mut size: i32 = 0;
        let mut ntoalign: i32 = 0;
        let mut opt: KOption = getdetails(&mut h, totalsize, &mut fmt, &mut size, &mut ntoalign);
        totalsize = totalsize.wrapping_add((ntoalign + size) as size_t);
        loop {
            let fresh190 = ntoalign;
            ntoalign = ntoalign - 1;
            if !(fresh190 > 0) {
                break;
            }
            (b.n < b.size || !(luaL_prepbuffsize(&mut b, 1 as i32 as size_t)).is_null()) as i32;
            let fresh191 = b.n;
            b.n = (b.n).wrapping_add(1);
            *(b.b).offset(fresh191 as isize) = 0 as std::ffi::c_char;
        }
        arg += 1;
        arg;
        let mut current_block_33: u64;
        match opt as u32 {
            0 => {
                let mut n: lua_Integer = luaL_checkinteger(L, arg);
                if size < size_of::<lua_Integer>() as usize as i32 {
                    let mut lim: lua_Integer =
                        (1 as i32 as lua_Integer) << size * 8 as i32 - 1 as i32;
                    (((-lim <= n && n < lim) as i32 != 0) as i32 as std::ffi::c_long != 0
                        || luaL_argerror(L, arg, c"integer overflow".as_ptr()) != 0)
                        as i32;
                }
                packint(
                    &mut b,
                    n as lua_Unsigned,
                    h.islittle,
                    size,
                    (n < 0 as lua_Integer) as i32,
                );
                current_block_33 = 3222590281903869779;
            }
            1 => {
                let mut n_0: lua_Integer = luaL_checkinteger(L, arg);
                if size < size_of::<lua_Integer>() as usize as i32 {
                    ((((n_0 as lua_Unsigned) < (1 as i32 as lua_Unsigned) << size * 8 as i32)
                        as i32
                        != 0) as i32 as std::ffi::c_long
                        != 0
                        || luaL_argerror(L, arg, c"unsigned overflow".as_ptr()) != 0)
                        as i32;
                }
                packint(&mut b, n_0 as lua_Unsigned, h.islittle, size, 0);
                current_block_33 = 3222590281903869779;
            }
            2 => {
                let mut f: std::ffi::c_float = luaL_checknumber(L, arg) as std::ffi::c_float;
                let mut buff: *mut std::ffi::c_char =
                    luaL_prepbuffsize(&mut b, size_of::<std::ffi::c_float>() as usize);
                copywithendian(
                    buff,
                    &mut f as *mut std::ffi::c_float as *mut std::ffi::c_char,
                    size_of::<std::ffi::c_float>() as usize as i32,
                    h.islittle,
                );
                b.n = (b.n).wrapping_add(size as size_t);
                current_block_33 = 3222590281903869779;
            }
            3 => {
                let mut f_0: lua_Number = luaL_checknumber(L, arg);
                let mut buff_0: *mut std::ffi::c_char =
                    luaL_prepbuffsize(&mut b, size_of::<lua_Number>() as usize);
                copywithendian(
                    buff_0,
                    &mut f_0 as *mut lua_Number as *mut std::ffi::c_char,
                    size_of::<lua_Number>() as usize as i32,
                    h.islittle,
                );
                b.n = (b.n).wrapping_add(size as size_t);
                current_block_33 = 3222590281903869779;
            }
            4 => {
                let mut f_1: std::ffi::c_double = luaL_checknumber(L, arg);
                let mut buff_1: *mut std::ffi::c_char =
                    luaL_prepbuffsize(&mut b, size_of::<std::ffi::c_double>() as usize);
                copywithendian(
                    buff_1,
                    &mut f_1 as *mut std::ffi::c_double as *mut std::ffi::c_char,
                    size_of::<std::ffi::c_double>() as usize as i32,
                    h.islittle,
                );
                b.n = (b.n).wrapping_add(size as size_t);
                current_block_33 = 3222590281903869779;
            }
            5 => {
                let mut len: size_t = 0;
                let mut s: *const std::ffi::c_char = luaL_checklstring(L, arg, &mut len);
                (((len <= size as size_t) as i32 != 0) as i32 as std::ffi::c_long != 0
                    || luaL_argerror(L, arg, c"string longer than given size".as_ptr()) != 0)
                    as i32;
                luaL_addlstring(&mut b, s, len);
                loop {
                    let fresh192 = len;
                    len = len.wrapping_add(1);
                    if !(fresh192 < size as size_t) {
                        break;
                    }
                    (b.n < b.size || !(luaL_prepbuffsize(&mut b, 1 as i32 as size_t)).is_null())
                        as i32;
                    let fresh193 = b.n;
                    b.n = (b.n).wrapping_add(1);
                    *(b.b).offset(fresh193 as isize) = 0 as std::ffi::c_char;
                }
                current_block_33 = 3222590281903869779;
            }
            6 => {
                let mut len_0: size_t = 0;
                let mut s_0: *const std::ffi::c_char = luaL_checklstring(L, arg, &mut len_0);
                (((size >= size_of::<size_t>() as usize as i32
                    || len_0 < (1 as i32 as size_t) << size * 8 as i32) as i32
                    != 0) as i32 as std::ffi::c_long
                    != 0
                    || luaL_argerror(L, arg, c"string length does not fit in given size".as_ptr())
                        != 0) as i32;
                packint(&mut b, len_0 as lua_Unsigned, h.islittle, size, 0);
                luaL_addlstring(&mut b, s_0, len_0);
                totalsize = totalsize.wrapping_add(len_0);
                current_block_33 = 3222590281903869779;
            }
            7 => {
                let mut len_1: size_t = 0;
                let mut s_1: *const std::ffi::c_char = luaL_checklstring(L, arg, &mut len_1);
                (((strlen(s_1) == len_1) as i32 != 0) as i32 as std::ffi::c_long != 0
                    || luaL_argerror(L, arg, c"string contains zeros".as_ptr()) != 0)
                    as i32;
                luaL_addlstring(&mut b, s_1, len_1);
                (b.n < b.size || !(luaL_prepbuffsize(&mut b, 1 as i32 as size_t)).is_null()) as i32;
                let fresh194 = b.n;
                b.n = (b.n).wrapping_add(1);
                *(b.b).offset(fresh194 as isize) = '\0' as i32 as std::ffi::c_char;
                totalsize = totalsize.wrapping_add(len_1.wrapping_add(1 as i32 as size_t));
                current_block_33 = 3222590281903869779;
            }
            8 => {
                (b.n < b.size || !(luaL_prepbuffsize(&mut b, 1 as i32 as size_t)).is_null()) as i32;
                let fresh195 = b.n;
                b.n = (b.n).wrapping_add(1);
                *(b.b).offset(fresh195 as isize) = 0 as std::ffi::c_char;
                current_block_33 = 3092145676633685342;
            }
            9 | 10 => {
                current_block_33 = 3092145676633685342;
            }
            _ => {
                current_block_33 = 3222590281903869779;
            }
        }
        match current_block_33 {
            3092145676633685342 => {
                arg -= 1;
                arg;
            }
            _ => {}
        }
    }
    luaL_pushresult(&mut b);
    return 1 as i32;
}
unsafe extern "C-unwind" fn str_packsize(mut L: *mut lua_State) -> i32 {
    let mut h: Header = Header {
        L: 0 as *mut lua_State,
        islittle: 0,
        maxalign: 0,
    };
    let mut fmt: *const std::ffi::c_char = luaL_checklstring(L, 1 as i32, 0 as *mut size_t);
    let mut totalsize: size_t = 0 as size_t;
    initheader(L, &mut h);
    while *fmt as i32 != '\0' as i32 {
        let mut size: i32 = 0;
        let mut ntoalign: i32 = 0;
        let mut opt: KOption = getdetails(&mut h, totalsize, &mut fmt, &mut size, &mut ntoalign);
        (((opt as u32 != Kstring as i32 as u32 && opt as u32 != Kzstr as i32 as u32) as i32 != 0)
            as i32 as std::ffi::c_long
            != 0
            || luaL_argerror(L, 1 as i32, c"variable-length format".as_ptr()) != 0) as i32;
        size += ntoalign;
        (((totalsize
            <= (if (size_of::<size_t>() as usize) < size_of::<i32>() as usize {
                !(0 as size_t)
            } else {
                2147483647 as i32 as size_t
            })
            .wrapping_sub(size as size_t)) as i32
            != 0) as i32 as std::ffi::c_long
            != 0
            || luaL_argerror(L, 1 as i32, c"format result too large".as_ptr()) != 0) as i32;
        totalsize = totalsize.wrapping_add(size as size_t);
    }
    lua_pushinteger(L, totalsize as lua_Integer);
    return 1 as i32;
}
unsafe extern "C-unwind" fn unpackint(
    mut L: *mut lua_State,
    mut str: *const std::ffi::c_char,
    mut islittle: i32,
    mut size: i32,
    mut issigned: i32,
) -> lua_Integer {
    let mut res: lua_Unsigned = 0 as lua_Unsigned;
    let mut i: i32 = 0;
    let mut limit: i32 = if size <= size_of::<lua_Integer>() as usize as i32 {
        size
    } else {
        size_of::<lua_Integer>() as usize as i32
    };
    i = limit - 1 as i32;
    while i >= 0 {
        res <<= 8 as i32;
        res |= *str.offset(
            (if islittle != 0 {
                i
            } else {
                size - 1 as i32 - i
            }) as isize,
        ) as u8 as lua_Unsigned;
        i -= 1;
        i;
    }
    if size < size_of::<lua_Integer>() as usize as i32 {
        if issigned != 0 {
            let mut mask: lua_Unsigned = (1 as i32 as lua_Unsigned) << size * 8 as i32 - 1 as i32;
            res = (res ^ mask).wrapping_sub(mask);
        }
    } else if size > size_of::<lua_Integer>() as usize as i32 {
        let mut mask_0: i32 = if issigned == 0 || res as lua_Integer >= 0 as lua_Integer {
            0
        } else {
            ((1 as i32) << 8 as i32) - 1 as i32
        };
        i = limit;
        while i < size {
            if ((*str.offset(
                (if islittle != 0 {
                    i
                } else {
                    size - 1 as i32 - i
                }) as isize,
            ) as u8 as i32
                != mask_0) as i32
                != 0) as i32 as std::ffi::c_long
                != 0
            {
                luaL_error(
                    L,
                    c"%d-byte integer does not fit into Lua Integer".as_ptr(),
                    size,
                );
            }
            i += 1;
            i;
        }
    }
    return res as lua_Integer;
}
unsafe extern "C-unwind" fn str_unpack(mut L: *mut lua_State) -> i32 {
    let mut h: Header = Header {
        L: 0 as *mut lua_State,
        islittle: 0,
        maxalign: 0,
    };
    let mut fmt: *const std::ffi::c_char = luaL_checklstring(L, 1 as i32, 0 as *mut size_t);
    let mut ld: size_t = 0;
    let mut data: *const std::ffi::c_char = luaL_checklstring(L, 2 as i32, &mut ld);
    let mut pos: size_t = (posrelatI(luaL_optinteger(L, 3 as i32, 1 as i32 as lua_Integer), ld))
        .wrapping_sub(1 as i32 as size_t);
    let mut n: i32 = 0;
    (((pos <= ld) as i32 != 0) as i32 as std::ffi::c_long != 0
        || luaL_argerror(L, 3 as i32, c"initial position out of string".as_ptr()) != 0) as i32;
    initheader(L, &mut h);
    while *fmt as i32 != '\0' as i32 {
        let mut size: i32 = 0;
        let mut ntoalign: i32 = 0;
        let mut opt: KOption = getdetails(&mut h, pos, &mut fmt, &mut size, &mut ntoalign);
        ((((ntoalign as size_t).wrapping_add(size as size_t) <= ld.wrapping_sub(pos)) as i32 != 0)
            as i32 as std::ffi::c_long
            != 0
            || luaL_argerror(L, 2 as i32, c"data string too short".as_ptr()) != 0) as i32;
        pos = pos.wrapping_add(ntoalign as size_t);
        luaL_checkstack(L, 2 as i32, c"too many results".as_ptr());
        n += 1;
        n;
        match opt as u32 {
            0 | 1 => {
                let mut res: lua_Integer = unpackint(
                    L,
                    data.offset(pos as isize),
                    h.islittle,
                    size,
                    (opt as u32 == Kint as i32 as u32) as i32,
                );
                lua_pushinteger(L, res);
            }
            2 => {
                let mut f: std::ffi::c_float = 0.;
                copywithendian(
                    &mut f as *mut std::ffi::c_float as *mut std::ffi::c_char,
                    data.offset(pos as isize),
                    size_of::<std::ffi::c_float>() as usize as i32,
                    h.islittle,
                );
                lua_pushnumber(L, f as lua_Number);
            }
            3 => {
                let mut f_0: lua_Number = 0.;
                copywithendian(
                    &mut f_0 as *mut lua_Number as *mut std::ffi::c_char,
                    data.offset(pos as isize),
                    size_of::<lua_Number>() as usize as i32,
                    h.islittle,
                );
                lua_pushnumber(L, f_0);
            }
            4 => {
                let mut f_1: std::ffi::c_double = 0.;
                copywithendian(
                    &mut f_1 as *mut std::ffi::c_double as *mut std::ffi::c_char,
                    data.offset(pos as isize),
                    size_of::<std::ffi::c_double>() as usize as i32,
                    h.islittle,
                );
                lua_pushnumber(L, f_1);
            }
            5 => {
                lua_pushlstring(L, data.offset(pos as isize), size as size_t);
            }
            6 => {
                let mut len: size_t =
                    unpackint(L, data.offset(pos as isize), h.islittle, size, 0) as size_t;
                (((len <= ld.wrapping_sub(pos).wrapping_sub(size as size_t)) as i32 != 0) as i32
                    as std::ffi::c_long
                    != 0
                    || luaL_argerror(L, 2 as i32, c"data string too short".as_ptr()) != 0)
                    as i32;
                lua_pushlstring(L, data.offset(pos as isize).offset(size as isize), len);
                pos = pos.wrapping_add(len);
            }
            7 => {
                let mut len_0: size_t = strlen(data.offset(pos as isize));
                (((pos.wrapping_add(len_0) < ld) as i32 != 0) as i32 as std::ffi::c_long != 0
                    || luaL_argerror(L, 2 as i32, c"unfinished string for format 'z'".as_ptr())
                        != 0) as i32;
                lua_pushlstring(L, data.offset(pos as isize), len_0);
                pos = pos.wrapping_add(len_0.wrapping_add(1 as i32 as size_t));
            }
            9 | 8 | 10 => {
                n -= 1;
                n;
            }
            _ => {}
        }
        pos = pos.wrapping_add(size as size_t);
    }
    lua_pushinteger(L, pos.wrapping_add(1 as i32 as size_t) as lua_Integer);
    return n + 1 as i32;
}
static mut strlib: [luaL_Reg; 18] = unsafe {
    [
        {
            let mut init = luaL_Reg {
                name: c"byte".as_ptr(),
                func: Some(str_byte as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"char".as_ptr(),
                func: Some(str_char as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"dump".as_ptr(),
                func: Some(str_dump as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"find".as_ptr(),
                func: Some(str_find as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"format".as_ptr(),
                func: Some(str_format as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"gmatch".as_ptr(),
                func: Some(gmatch as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"gsub".as_ptr(),
                func: Some(str_gsub as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"len".as_ptr(),
                func: Some(str_len as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"lower".as_ptr(),
                func: Some(str_lower as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"match".as_ptr(),
                func: Some(str_match as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"rep".as_ptr(),
                func: Some(str_rep as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"reverse".as_ptr(),
                func: Some(str_reverse as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"sub".as_ptr(),
                func: Some(str_sub as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"upper".as_ptr(),
                func: Some(str_upper as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"pack".as_ptr(),
                func: Some(str_pack as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"packsize".as_ptr(),
                func: Some(str_packsize as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"unpack".as_ptr(),
                func: Some(str_unpack as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: 0 as *const std::ffi::c_char,
                func: None,
            };
            init
        },
    ]
};
unsafe extern "C-unwind" fn createmetatable(mut L: *mut lua_State) {
    lua_createtable(
        L,
        0,
        (size_of::<[luaL_Reg; 10]>() as usize)
            .wrapping_div(size_of::<luaL_Reg>() as usize)
            .wrapping_sub(1) as i32,
    );
    luaL_setfuncs(L, (&raw const stringmetamethods).cast(), 0);
    lua_pushstring(L, c"".as_ptr());
    lua_pushvalue(L, -(2 as i32));
    lua_setmetatable(L, -(2 as i32));
    lua_settop(L, -(1 as i32) - 1 as i32);
    lua_pushvalue(L, -(2 as i32));
    lua_setfield(L, -(2 as i32), c"__index".as_ptr());
    lua_settop(L, -(1 as i32) - 1 as i32);
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaopen_string(mut L: *mut lua_State) -> i32 {
    luaL_checkversion_(
        L,
        504 as i32 as lua_Number,
        (size_of::<lua_Integer>() as usize)
            .wrapping_mul(16)
            .wrapping_add(size_of::<lua_Number>() as usize),
    );
    lua_createtable(
        L,
        0,
        (size_of::<[luaL_Reg; 18]>() as usize)
            .wrapping_div(size_of::<luaL_Reg>() as usize)
            .wrapping_sub(1) as i32,
    );
    luaL_setfuncs(L, (&raw const strlib).cast(), 0);
    createmetatable(L);
    return 1 as i32;
}
unsafe extern "C-unwind" fn checkfield(
    mut L: *mut lua_State,
    mut key: *const std::ffi::c_char,
    mut n: i32,
) -> i32 {
    lua_pushstring(L, key);
    return (lua_rawget(L, -n) != 0) as i32;
}
unsafe extern "C-unwind" fn checktab(mut L: *mut lua_State, mut arg: i32, mut what: i32) {
    if lua_type(L, arg) != 5 as i32 {
        let mut n: i32 = 1 as i32;
        if lua_getmetatable(L, arg) != 0
            && (what & 1 as i32 == 0 || {
                n += 1;
                checkfield(L, c"__index".as_ptr(), n) != 0
            })
            && (what & 2 as i32 == 0 || {
                n += 1;
                checkfield(L, c"__newindex".as_ptr(), n) != 0
            })
            && (what & 4 as i32 == 0 || {
                n += 1;
                checkfield(L, c"__len".as_ptr(), n) != 0
            })
        {
            lua_settop(L, -n - 1 as i32);
        } else {
            luaL_checktype(L, arg, 5 as i32);
        }
    }
}
unsafe extern "C-unwind" fn tinsert(mut L: *mut lua_State) -> i32 {
    let mut pos: lua_Integer = 0;
    checktab(L, 1 as i32, 1 as i32 | 2 as i32 | 4 as i32);
    let mut e: lua_Integer = luaL_len(L, 1 as i32);
    e = (e as lua_Unsigned).wrapping_add(1 as i32 as lua_Unsigned) as lua_Integer;
    match lua_gettop(L) {
        2 => {
            pos = e;
        }
        3 => {
            let mut i: lua_Integer = 0;
            pos = luaL_checkinteger(L, 2 as i32);
            ((((pos as lua_Unsigned).wrapping_sub(1 as u32 as lua_Unsigned) < e as lua_Unsigned)
                as i32
                != 0) as i32 as std::ffi::c_long
                != 0
                || luaL_argerror(L, 2 as i32, c"position out of bounds".as_ptr()) != 0)
                as i32;
            i = e;
            while i > pos {
                lua_geti(L, 1 as i32, i - 1 as i32 as lua_Integer);
                lua_seti(L, 1 as i32, i);
                i -= 1;
                i;
            }
        }
        _ => {
            return luaL_error(L, c"wrong number of arguments to 'insert'".as_ptr());
        }
    }
    lua_seti(L, 1 as i32, pos);
    return 0;
}
unsafe extern "C-unwind" fn tremove(mut L: *mut lua_State) -> i32 {
    checktab(L, 1 as i32, 1 as i32 | 2 as i32 | 4 as i32);
    let mut size: lua_Integer = luaL_len(L, 1 as i32);
    let mut pos: lua_Integer = luaL_optinteger(L, 2 as i32, size);
    if pos != size {
        ((((pos as lua_Unsigned).wrapping_sub(1 as u32 as lua_Unsigned) <= size as lua_Unsigned)
            as i32
            != 0) as i32 as std::ffi::c_long
            != 0
            || luaL_argerror(L, 2 as i32, c"position out of bounds".as_ptr()) != 0) as i32;
    }
    lua_geti(L, 1 as i32, pos);
    while pos < size {
        lua_geti(L, 1 as i32, pos + 1 as i32 as lua_Integer);
        lua_seti(L, 1 as i32, pos);
        pos += 1;
        pos;
    }
    lua_pushnil(L);
    lua_seti(L, 1 as i32, pos);
    return 1 as i32;
}
unsafe extern "C-unwind" fn tmove(mut L: *mut lua_State) -> i32 {
    let mut f: lua_Integer = luaL_checkinteger(L, 2 as i32);
    let mut e: lua_Integer = luaL_checkinteger(L, 3 as i32);
    let mut t: lua_Integer = luaL_checkinteger(L, 4 as i32);
    let mut tt: i32 = if !(lua_type(L, 5 as i32) <= 0) {
        5 as i32
    } else {
        1 as i32
    };
    checktab(L, 1 as i32, 1 as i32);
    checktab(L, tt, 2 as i32);
    if e >= f {
        let mut n: lua_Integer = 0;
        let mut i: lua_Integer = 0;
        (((f > 0 as lua_Integer || e < 9223372036854775807 as std::ffi::c_longlong + f) as i32 != 0)
            as i32 as std::ffi::c_long
            != 0
            || luaL_argerror(L, 3 as i32, c"too many elements to move".as_ptr()) != 0)
            as i32;
        n = e - f + 1 as i32 as lua_Integer;
        (((t <= 9223372036854775807 as std::ffi::c_longlong - n + 1 as i32 as std::ffi::c_longlong)
            as i32
            != 0) as i32 as std::ffi::c_long
            != 0
            || luaL_argerror(L, 4 as i32, c"destination wrap around".as_ptr()) != 0) as i32;
        if t > e || t <= f || tt != 1 as i32 && lua_compare(L, 1 as i32, tt, 0) == 0 {
            i = 0 as lua_Integer;
            while i < n {
                lua_geti(L, 1 as i32, f + i);
                lua_seti(L, tt, t + i);
                i += 1;
                i;
            }
        } else {
            i = n - 1 as i32 as lua_Integer;
            while i >= 0 as lua_Integer {
                lua_geti(L, 1 as i32, f + i);
                lua_seti(L, tt, t + i);
                i -= 1;
                i;
            }
        }
    }
    lua_pushvalue(L, tt);
    return 1 as i32;
}
unsafe extern "C-unwind" fn addfield(
    mut L: *mut lua_State,
    mut b: *mut luaL_Buffer,
    mut i: lua_Integer,
) {
    lua_geti(L, 1 as i32, i);
    if ((lua_isstring(L, -(1 as i32)) == 0) as i32 != 0) as i32 as std::ffi::c_long != 0 {
        luaL_error(
            L,
            c"invalid value (%s) at index %I in table for 'concat'".as_ptr(),
            lua_typename(L, lua_type(L, -(1 as i32))),
            i,
        );
    }
    luaL_addvalue(b);
}
unsafe extern "C-unwind" fn tconcat(mut L: *mut lua_State) -> i32 {
    let mut b: luaL_Buffer = luaL_Buffer {
        b: 0 as *mut std::ffi::c_char,
        size: 0,
        n: 0,
        L: 0 as *mut lua_State,
        init: C2RustUnnamed_15 { n: 0. },
    };
    checktab(L, 1 as i32, 1 as i32 | 4 as i32);
    let mut last: lua_Integer = luaL_len(L, 1 as i32);
    let mut lsep: size_t = 0;
    let mut sep: *const std::ffi::c_char = luaL_optlstring(L, 2 as i32, c"".as_ptr(), &mut lsep);
    let mut i: lua_Integer = luaL_optinteger(L, 3 as i32, 1 as i32 as lua_Integer);
    last = luaL_optinteger(L, 4 as i32, last);
    luaL_buffinit(L, &mut b);
    while i < last {
        addfield(L, &mut b, i);
        luaL_addlstring(&mut b, sep, lsep);
        i += 1;
        i;
    }
    if i == last {
        addfield(L, &mut b, i);
    }
    luaL_pushresult(&mut b);
    return 1 as i32;
}
unsafe extern "C-unwind" fn tpack(mut L: *mut lua_State) -> i32 {
    let mut i: i32 = 0;
    let mut n: i32 = lua_gettop(L);
    lua_createtable(L, n, 1 as i32);
    lua_rotate(L, 1 as i32, 1 as i32);
    i = n;
    while i >= 1 as i32 {
        lua_seti(L, 1 as i32, i as lua_Integer);
        i -= 1;
        i;
    }
    lua_pushinteger(L, n as lua_Integer);
    lua_setfield(L, 1 as i32, c"n".as_ptr());
    return 1 as i32;
}
unsafe extern "C-unwind" fn tunpack(mut L: *mut lua_State) -> i32 {
    let mut n: lua_Unsigned = 0;
    let mut i: lua_Integer = luaL_optinteger(L, 2 as i32, 1 as i32 as lua_Integer);
    let mut e: lua_Integer = if lua_type(L, 3 as i32) <= 0 {
        luaL_len(L, 1 as i32)
    } else {
        luaL_checkinteger(L, 3 as i32)
    };
    if i > e {
        return 0;
    }
    n = (e as lua_Unsigned).wrapping_sub(i as lua_Unsigned);
    if ((n >= 2147483647 as lua_Unsigned || {
        n = n.wrapping_add(1);
        lua_checkstack(L, n as i32) == 0
    }) as i32
        != 0) as i32 as std::ffi::c_long
        != 0
    {
        return luaL_error(L, c"too many results to unpack".as_ptr());
    }
    while i < e {
        lua_geti(L, 1 as i32, i);
        i += 1;
        i;
    }
    lua_geti(L, 1 as i32, e);
    return n as i32;
}
unsafe extern "C-unwind" fn l_randomizePivot() -> u32 {
    let mut c: clock_t = clock();
    let mut t: time_t = time(0 as *mut time_t);
    let mut buff: [u32; 4] = [0; 4];
    let mut i: u32 = 0;
    let mut rnd: u32 = 0 as u32;
    memcpy(
        buff.as_mut_ptr() as *mut c_void,
        &mut c as *mut clock_t as *const c_void,
        (size_of::<clock_t>() as usize)
            .wrapping_div(size_of::<u32>() as usize)
            .wrapping_mul(size_of::<u32>() as usize),
    );
    memcpy(
        buff.as_mut_ptr().offset(
            (size_of::<clock_t>() as usize).wrapping_div(size_of::<u32>() as usize) as isize,
        ) as *mut c_void,
        &mut t as *mut time_t as *const c_void,
        (size_of::<time_t>() as usize)
            .wrapping_div(size_of::<u32>() as usize)
            .wrapping_mul(size_of::<u32>() as usize),
    );
    i = 0 as u32;
    while (i as usize) < (size_of::<[u32; 4]>() as usize).wrapping_div(size_of::<u32>() as usize) {
        rnd = rnd.wrapping_add(buff[i as usize]);
        i = i.wrapping_add(1);
        i;
    }
    return rnd;
}
unsafe extern "C-unwind" fn set2(mut L: *mut lua_State, mut i: IdxT, mut j: IdxT) {
    lua_seti(L, 1 as i32, i as lua_Integer);
    lua_seti(L, 1 as i32, j as lua_Integer);
}
unsafe extern "C-unwind" fn sort_comp(mut L: *mut lua_State, mut a: i32, mut b: i32) -> i32 {
    if lua_type(L, 2 as i32) == 0 {
        return lua_compare(L, a, b, 1 as i32);
    } else {
        let mut res: i32 = 0;
        lua_pushvalue(L, 2 as i32);
        lua_pushvalue(L, a - 1 as i32);
        lua_pushvalue(L, b - 2 as i32);
        lua_callk(L, 2 as i32, 1 as i32, 0 as lua_KContext, None);
        res = lua_toboolean(L, -(1 as i32));
        lua_settop(L, -(1 as i32) - 1 as i32);
        return res;
    };
}
unsafe extern "C-unwind" fn partition(mut L: *mut lua_State, mut lo: IdxT, mut up: IdxT) -> IdxT {
    let mut i: IdxT = lo;
    let mut j: IdxT = up.wrapping_sub(1 as i32 as IdxT);
    loop {
        loop {
            i = i.wrapping_add(1);
            lua_geti(L, 1 as i32, i as lua_Integer);
            if !(sort_comp(L, -(1 as i32), -(2 as i32)) != 0) {
                break;
            }
            if ((i == up.wrapping_sub(1 as i32 as IdxT)) as i32 != 0) as i32 as std::ffi::c_long
                != 0
            {
                luaL_error(L, c"invalid order function for sorting".as_ptr());
            }
            lua_settop(L, -(1 as i32) - 1 as i32);
        }
        loop {
            j = j.wrapping_sub(1);
            lua_geti(L, 1 as i32, j as lua_Integer);
            if !(sort_comp(L, -(3 as i32), -(1 as i32)) != 0) {
                break;
            }
            if ((j < i) as i32 != 0) as i32 as std::ffi::c_long != 0 {
                luaL_error(L, c"invalid order function for sorting".as_ptr());
            }
            lua_settop(L, -(1 as i32) - 1 as i32);
        }
        if j < i {
            lua_settop(L, -(1 as i32) - 1 as i32);
            set2(L, up.wrapping_sub(1 as i32 as IdxT), i);
            return i;
        }
        set2(L, i, j);
    }
}
unsafe extern "C-unwind" fn choosePivot(mut lo: IdxT, mut up: IdxT, mut rnd: u32) -> IdxT {
    let mut r4: IdxT = up.wrapping_sub(lo) / 4 as i32 as IdxT;
    let mut p: IdxT = rnd
        .wrapping_rem(r4 * 2 as i32 as IdxT)
        .wrapping_add(lo.wrapping_add(r4));
    return p;
}
unsafe extern "C-unwind" fn auxsort(
    mut L: *mut lua_State,
    mut lo: IdxT,
    mut up: IdxT,
    mut rnd: u32,
) {
    while lo < up {
        let mut p: IdxT = 0;
        let mut n: IdxT = 0;
        lua_geti(L, 1 as i32, lo as lua_Integer);
        lua_geti(L, 1 as i32, up as lua_Integer);
        if sort_comp(L, -(1 as i32), -(2 as i32)) != 0 {
            set2(L, lo, up);
        } else {
            lua_settop(L, -(2 as i32) - 1 as i32);
        }
        if up.wrapping_sub(lo) == 1 as i32 as IdxT {
            return;
        }
        if up.wrapping_sub(lo) < 100 as u32 || rnd == 0 as u32 {
            p = lo.wrapping_add(up) / 2 as i32 as IdxT;
        } else {
            p = choosePivot(lo, up, rnd);
        }
        lua_geti(L, 1 as i32, p as lua_Integer);
        lua_geti(L, 1 as i32, lo as lua_Integer);
        if sort_comp(L, -(2 as i32), -(1 as i32)) != 0 {
            set2(L, p, lo);
        } else {
            lua_settop(L, -(1 as i32) - 1 as i32);
            lua_geti(L, 1 as i32, up as lua_Integer);
            if sort_comp(L, -(1 as i32), -(2 as i32)) != 0 {
                set2(L, p, up);
            } else {
                lua_settop(L, -(2 as i32) - 1 as i32);
            }
        }
        if up.wrapping_sub(lo) == 2 as i32 as IdxT {
            return;
        }
        lua_geti(L, 1 as i32, p as lua_Integer);
        lua_pushvalue(L, -(1 as i32));
        lua_geti(
            L,
            1 as i32,
            up.wrapping_sub(1 as i32 as IdxT) as lua_Integer,
        );
        set2(L, p, up.wrapping_sub(1 as i32 as IdxT));
        p = partition(L, lo, up);
        if p.wrapping_sub(lo) < up.wrapping_sub(p) {
            auxsort(L, lo, p.wrapping_sub(1 as i32 as IdxT), rnd);
            n = p.wrapping_sub(lo);
            lo = p.wrapping_add(1 as i32 as IdxT);
        } else {
            auxsort(L, p.wrapping_add(1 as i32 as IdxT), up, rnd);
            n = up.wrapping_sub(p);
            up = p.wrapping_sub(1 as i32 as IdxT);
        }
        if up.wrapping_sub(lo) / 128 as i32 as IdxT > n {
            rnd = l_randomizePivot();
        }
    }
}
unsafe extern "C-unwind" fn sort(mut L: *mut lua_State) -> i32 {
    checktab(L, 1 as i32, 1 as i32 | 2 as i32 | 4 as i32);
    let mut n: lua_Integer = luaL_len(L, 1 as i32);
    if n > 1 as i32 as lua_Integer {
        (((n < 2147483647 as i32 as lua_Integer) as i32 != 0) as i32 as std::ffi::c_long != 0
            || luaL_argerror(L, 1 as i32, c"array too big".as_ptr()) != 0) as i32;
        if !(lua_type(L, 2 as i32) <= 0) {
            luaL_checktype(L, 2 as i32, 6 as i32);
        }
        lua_settop(L, 2 as i32);
        auxsort(L, 1 as i32 as IdxT, n as IdxT, 0 as u32);
    }
    return 0;
}
static mut tab_funcs: [luaL_Reg; 8] = unsafe {
    [
        {
            let mut init = luaL_Reg {
                name: c"concat".as_ptr(),
                func: Some(tconcat as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"insert".as_ptr(),
                func: Some(tinsert as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"pack".as_ptr(),
                func: Some(tpack as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"unpack".as_ptr(),
                func: Some(tunpack as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"remove".as_ptr(),
                func: Some(tremove as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"move".as_ptr(),
                func: Some(tmove as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"sort".as_ptr(),
                func: Some(sort as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: 0 as *const std::ffi::c_char,
                func: None,
            };
            init
        },
    ]
};
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaopen_table(mut L: *mut lua_State) -> i32 {
    luaL_checkversion_(
        L,
        504 as i32 as lua_Number,
        (size_of::<lua_Integer>() as usize)
            .wrapping_mul(16)
            .wrapping_add(size_of::<lua_Number>() as usize),
    );
    lua_createtable(
        L,
        0,
        (size_of::<[luaL_Reg; 8]>() as usize)
            .wrapping_div(size_of::<luaL_Reg>() as usize)
            .wrapping_sub(1) as i32,
    );
    luaL_setfuncs(L, (&raw const tab_funcs).cast(), 0);
    return 1 as i32;
}
unsafe extern "C-unwind" fn u_posrelat(mut pos: lua_Integer, mut len: size_t) -> lua_Integer {
    if pos >= 0 as lua_Integer {
        return pos;
    } else if (0 as u32 as size_t).wrapping_sub(pos as size_t) > len {
        return 0 as lua_Integer;
    } else {
        return len as lua_Integer + pos + 1 as i32 as lua_Integer;
    };
}
unsafe extern "C-unwind" fn utf8_decode(
    mut s: *const std::ffi::c_char,
    mut val: *mut utfint,
    mut strict: i32,
) -> *const std::ffi::c_char {
    static mut limits: [utfint; 6] = [
        !(0 as utfint),
        0x80 as utfint,
        0x800 as utfint,
        0x10000 as u32,
        0x200000 as u32,
        0x4000000 as u32,
    ];
    let mut c: u32 = *s.offset(0 as isize) as u8 as u32;
    let mut res: utfint = 0 as utfint;
    if c < 0x80 as u32 {
        res = c;
    } else {
        let mut count: i32 = 0;
        while c & 0x40 as u32 != 0 {
            count += 1;
            let mut cc: u32 = *s.offset(count as isize) as u8 as u32;
            if !(cc & 0xc0 as u32 == 0x80 as u32) {
                return 0 as *const std::ffi::c_char;
            }
            res = res << 6 as i32 | cc & 0x3f as i32 as u32;
            c <<= 1 as i32;
        }
        res |= (c & 0x7f as i32 as u32) << count * 5 as i32;
        if count > 5 as i32 || res > 0x7fffffff as u32 || res < limits[count as usize] {
            return 0 as *const std::ffi::c_char;
        }
        s = s.offset(count as isize);
    }
    if strict != 0 {
        if res > 0x10ffff as u32 || 0xd800 as u32 <= res && res <= 0xdfff as u32 {
            return 0 as *const std::ffi::c_char;
        }
    }
    if !val.is_null() {
        *val = res;
    }
    return s.offset(1);
}
unsafe extern "C-unwind" fn utflen(mut L: *mut lua_State) -> i32 {
    let mut n: lua_Integer = 0 as lua_Integer;
    let mut len: size_t = 0;
    let mut s: *const std::ffi::c_char = luaL_checklstring(L, 1 as i32, &mut len);
    let mut posi: lua_Integer =
        u_posrelat(luaL_optinteger(L, 2 as i32, 1 as i32 as lua_Integer), len);
    let mut posj: lua_Integer = u_posrelat(
        luaL_optinteger(L, 3 as i32, -(1 as i32) as lua_Integer),
        len,
    );
    let mut lax: i32 = lua_toboolean(L, 4 as i32);
    (((1 as i32 as lua_Integer <= posi && {
        posi -= 1;
        posi <= len as lua_Integer
    }) as i32
        != 0) as i32 as std::ffi::c_long
        != 0
        || luaL_argerror(L, 2 as i32, c"initial position out of bounds".as_ptr()) != 0) as i32;
    posj -= 1;
    (((posj < len as lua_Integer) as i32 != 0) as i32 as std::ffi::c_long != 0
        || luaL_argerror(L, 3 as i32, c"final position out of bounds".as_ptr()) != 0) as i32;
    while posi <= posj {
        let mut s1: *const std::ffi::c_char =
            utf8_decode(s.offset(posi as isize), 0 as *mut utfint, (lax == 0) as i32);
        if s1.is_null() {
            lua_pushnil(L);
            lua_pushinteger(L, posi + 1 as i32 as lua_Integer);
            return 2 as i32;
        }
        posi = s1.offset_from(s) as std::ffi::c_long as lua_Integer;
        n += 1;
        n;
    }
    lua_pushinteger(L, n);
    return 1 as i32;
}
unsafe extern "C-unwind" fn codepoint(mut L: *mut lua_State) -> i32 {
    let mut len: size_t = 0;
    let mut s: *const std::ffi::c_char = luaL_checklstring(L, 1 as i32, &mut len);
    let mut posi: lua_Integer =
        u_posrelat(luaL_optinteger(L, 2 as i32, 1 as i32 as lua_Integer), len);
    let mut pose: lua_Integer = u_posrelat(luaL_optinteger(L, 3 as i32, posi), len);
    let mut lax: i32 = lua_toboolean(L, 4 as i32);
    let mut n: i32 = 0;
    let mut se: *const std::ffi::c_char = 0 as *const std::ffi::c_char;
    (((posi >= 1 as i32 as lua_Integer) as i32 != 0) as i32 as std::ffi::c_long != 0
        || luaL_argerror(L, 2 as i32, c"out of bounds".as_ptr()) != 0) as i32;
    (((pose <= len as lua_Integer) as i32 != 0) as i32 as std::ffi::c_long != 0
        || luaL_argerror(L, 3 as i32, c"out of bounds".as_ptr()) != 0) as i32;
    if posi > pose {
        return 0;
    }
    if pose - posi >= 2147483647 as i32 as lua_Integer {
        return luaL_error(L, c"string slice too long".as_ptr());
    }
    n = (pose - posi) as i32 + 1 as i32;
    luaL_checkstack(L, n, c"string slice too long".as_ptr());
    n = 0;
    se = s.offset(pose as isize);
    s = s.offset((posi - 1 as i32 as lua_Integer) as isize);
    while s < se {
        let mut code: utfint = 0;
        s = utf8_decode(s, &mut code, (lax == 0) as i32);
        if s.is_null() {
            return luaL_error(L, c"invalid UTF-8 code".as_ptr());
        }
        lua_pushinteger(L, code as lua_Integer);
        n += 1;
        n;
    }
    return n;
}
unsafe extern "C-unwind" fn pushutfchar(mut L: *mut lua_State, mut arg: i32) {
    let mut code: lua_Unsigned = luaL_checkinteger(L, arg) as lua_Unsigned;
    (((code <= 0x7fffffff as u32 as lua_Unsigned) as i32 != 0) as i32 as std::ffi::c_long != 0
        || luaL_argerror(L, arg, c"value out of range".as_ptr()) != 0) as i32;
    lua_pushfstring(L, c"%U".as_ptr(), code as std::ffi::c_long);
}
unsafe extern "C-unwind" fn utfchar(mut L: *mut lua_State) -> i32 {
    let mut n: i32 = lua_gettop(L);
    if n == 1 as i32 {
        pushutfchar(L, 1 as i32);
    } else {
        let mut i: i32 = 0;
        let mut b: luaL_Buffer = luaL_Buffer {
            b: 0 as *mut std::ffi::c_char,
            size: 0,
            n: 0,
            L: 0 as *mut lua_State,
            init: C2RustUnnamed_15 { n: 0. },
        };
        luaL_buffinit(L, &mut b);
        i = 1 as i32;
        while i <= n {
            pushutfchar(L, i);
            luaL_addvalue(&mut b);
            i += 1;
            i;
        }
        luaL_pushresult(&mut b);
    }
    return 1 as i32;
}
unsafe extern "C-unwind" fn byteoffset(mut L: *mut lua_State) -> i32 {
    let mut len: size_t = 0;
    let mut s: *const std::ffi::c_char = luaL_checklstring(L, 1 as i32, &mut len);
    let mut n: lua_Integer = luaL_checkinteger(L, 2 as i32);
    let mut posi: lua_Integer = (if n >= 0 as lua_Integer {
        1 as i32 as size_t
    } else {
        len.wrapping_add(1 as i32 as size_t)
    }) as lua_Integer;
    posi = u_posrelat(luaL_optinteger(L, 3 as i32, posi), len);
    (((1 as i32 as lua_Integer <= posi && {
        posi -= 1;
        posi <= len as lua_Integer
    }) as i32
        != 0) as i32 as std::ffi::c_long
        != 0
        || luaL_argerror(L, 3 as i32, c"position out of bounds".as_ptr()) != 0) as i32;
    if n == 0 as lua_Integer {
        while posi > 0 as lua_Integer && *s.offset(posi as isize) as i32 & 0xc0 == 0x80 {
            posi -= 1;
            posi;
        }
    } else {
        if *s.offset(posi as isize) as i32 & 0xc0 == 0x80 {
            return luaL_error(L, c"initial position is a continuation byte".as_ptr());
        }
        if n < 0 as lua_Integer {
            while n < 0 as lua_Integer && posi > 0 as lua_Integer {
                loop {
                    posi -= 1;
                    posi;
                    if !(posi > 0 as lua_Integer && *s.offset(posi as isize) as i32 & 0xc0 == 0x80)
                    {
                        break;
                    }
                }
                n += 1;
                n;
            }
        } else {
            n -= 1;
            n;
            while n > 0 as lua_Integer && posi < len as lua_Integer {
                loop {
                    posi += 1;
                    posi;
                    if !(*s.offset(posi as isize) as i32 & 0xc0 == 0x80) {
                        break;
                    }
                }
                n -= 1;
                n;
            }
        }
    }
    if n == 0 as lua_Integer {
        lua_pushinteger(L, posi + 1 as i32 as lua_Integer);
    } else {
        lua_pushnil(L);
    }
    return 1 as i32;
}
unsafe extern "C-unwind" fn iter_aux(mut L: *mut lua_State, mut strict: i32) -> i32 {
    let mut len: size_t = 0;
    let mut s: *const std::ffi::c_char = luaL_checklstring(L, 1 as i32, &mut len);
    let mut n: lua_Unsigned = lua_tointegerx(L, 2 as i32, 0 as *mut i32) as lua_Unsigned;
    if n < len as lua_Unsigned {
        while *s.offset(n as isize) as i32 & 0xc0 == 0x80 {
            n = n.wrapping_add(1);
            n;
        }
    }
    if n >= len as lua_Unsigned {
        return 0;
    } else {
        let mut code: utfint = 0;
        let mut next: *const std::ffi::c_char =
            utf8_decode(s.offset(n as isize), &mut code, strict);
        if next.is_null() || *next as i32 & 0xc0 == 0x80 {
            return luaL_error(L, c"invalid UTF-8 code".as_ptr());
        }
        lua_pushinteger(L, n.wrapping_add(1 as i32 as lua_Unsigned) as lua_Integer);
        lua_pushinteger(L, code as lua_Integer);
        return 2 as i32;
    };
}
unsafe extern "C-unwind" fn iter_auxstrict(mut L: *mut lua_State) -> i32 {
    return iter_aux(L, 1 as i32);
}
unsafe extern "C-unwind" fn iter_auxlax(mut L: *mut lua_State) -> i32 {
    return iter_aux(L, 0);
}
unsafe extern "C-unwind" fn iter_codes(mut L: *mut lua_State) -> i32 {
    let mut lax: i32 = lua_toboolean(L, 2 as i32);
    let mut s: *const std::ffi::c_char = luaL_checklstring(L, 1 as i32, 0 as *mut size_t);
    ((!(*s as i32 & 0xc0 == 0x80) as i32 != 0) as i32 as std::ffi::c_long != 0
        || luaL_argerror(L, 1 as i32, c"invalid UTF-8 code".as_ptr()) != 0) as i32;
    lua_pushcclosure(
        L,
        if lax != 0 {
            Some(iter_auxlax as unsafe extern "C-unwind" fn(*mut lua_State) -> i32)
        } else {
            Some(iter_auxstrict as unsafe extern "C-unwind" fn(*mut lua_State) -> i32)
        },
        0,
    );
    lua_pushvalue(L, 1 as i32);
    lua_pushinteger(L, 0 as lua_Integer);
    return 3 as i32;
}
static mut funcs: [luaL_Reg; 7] = unsafe {
    [
        {
            let mut init = luaL_Reg {
                name: c"offset".as_ptr(),
                func: Some(byteoffset as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"codepoint".as_ptr(),
                func: Some(codepoint as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"char".as_ptr(),
                func: Some(utfchar as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"len".as_ptr(),
                func: Some(utflen as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"codes".as_ptr(),
                func: Some(iter_codes as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"charpattern".as_ptr(),
                func: None,
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: 0 as *const std::ffi::c_char,
                func: None,
            };
            init
        },
    ]
};
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaopen_utf8(mut L: *mut lua_State) -> i32 {
    luaL_checkversion_(
        L,
        504 as i32 as lua_Number,
        (size_of::<lua_Integer>() as usize)
            .wrapping_mul(16)
            .wrapping_add(size_of::<lua_Number>() as usize),
    );
    lua_createtable(
        L,
        0,
        (size_of::<[luaL_Reg; 7]>() as usize)
            .wrapping_div(size_of::<luaL_Reg>() as usize)
            .wrapping_sub(1) as i32,
    );
    luaL_setfuncs(L, (&raw const funcs).cast(), 0);
    lua_pushlstring(
        L,
        b"[\0-\x7F\xC2-\xFD][\x80-\xBF]*\0" as *const u8 as *const std::ffi::c_char,
        (size_of::<[std::ffi::c_char; 15]>() as usize)
            .wrapping_div(size_of::<std::ffi::c_char>() as usize)
            .wrapping_sub(1),
    );
    lua_setfield(L, -(2 as i32), c"charpattern".as_ptr());
    return 1 as i32;
}
static mut loadedlibs: [luaL_Reg; 11] = unsafe {
    [
        {
            let mut init = luaL_Reg {
                name: c"_G".as_ptr(),
                func: Some(luaopen_base as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"package".as_ptr(),
                func: Some(luaopen_package as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"coroutine".as_ptr(),
                func: Some(luaopen_coroutine as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"table".as_ptr(),
                func: Some(luaopen_table as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"io".as_ptr(),
                func: Some(luaopen_io as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"os".as_ptr(),
                func: Some(luaopen_os as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"string".as_ptr(),
                func: Some(luaopen_string as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"math".as_ptr(),
                func: Some(luaopen_math as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"utf8".as_ptr(),
                func: Some(luaopen_utf8 as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: c"debug".as_ptr(),
                func: Some(luaopen_debug as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
            };
            init
        },
        {
            let mut init = luaL_Reg {
                name: 0 as *const std::ffi::c_char,
                func: None,
            };
            init
        },
    ]
};
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaL_openlibs(mut L: *mut lua_State) {
    let mut lib: *const luaL_Reg = 0 as *const luaL_Reg;
    lib = (&raw const loadedlibs).cast();
    while ((*lib).func).is_some() {
        luaL_requiref(L, (*lib).name, (*lib).func, 1 as i32);
        lua_settop(L, -(1 as i32) - 1 as i32);
        lib = lib.offset(1);
        lib;
    }
}
