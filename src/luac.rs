use ::libc;
use ::c2rust_bitfields;
extern "C" {
    pub type _IO_wide_data;
    pub type _IO_codecvt;
    pub type _IO_marker;
    pub type lua_longjmp;
    fn __ctype_b_loc() -> *mut *const std::ffi::c_ushort;
    fn __errno_location() -> *mut std::ffi::c_int;
    static mut stdout: *mut FILE;
    static mut stderr: *mut FILE;
    fn fclose(__stream: *mut FILE) -> std::ffi::c_int;
    fn fopen(_: *const std::ffi::c_char, _: *const std::ffi::c_char) -> *mut FILE;
    fn fprintf(_: *mut FILE, _: *const std::ffi::c_char, _: ...) -> std::ffi::c_int;
    fn printf(_: *const std::ffi::c_char, _: ...) -> std::ffi::c_int;
    fn sprintf(
        _: *mut std::ffi::c_char,
        _: *const std::ffi::c_char,
        _: ...
    ) -> std::ffi::c_int;
    fn fwrite(
        _: *const std::ffi::c_void,
        _: std::ffi::c_ulong,
        _: std::ffi::c_ulong,
        _: *mut FILE,
    ) -> std::ffi::c_ulong;
    fn ferror(__stream: *mut FILE) -> std::ffi::c_int;
    fn exit(_: std::ffi::c_int) -> !;
    fn strcmp(_: *const std::ffi::c_char, _: *const std::ffi::c_char) -> std::ffi::c_int;
    fn strspn(
        _: *const std::ffi::c_char,
        _: *const std::ffi::c_char,
    ) -> std::ffi::c_ulong;
    fn strerror(_: std::ffi::c_int) -> *mut std::ffi::c_char;
    fn lua_close(L: *mut lua_State);
    fn lua_checkstack(L: *mut lua_State, n: std::ffi::c_int) -> std::ffi::c_int;
    fn lua_tointegerx(
        L: *mut lua_State,
        idx: std::ffi::c_int,
        isnum: *mut std::ffi::c_int,
    ) -> lua_Integer;
    fn lua_tolstring(
        L: *mut lua_State,
        idx: std::ffi::c_int,
        len: *mut size_t,
    ) -> *const std::ffi::c_char;
    fn lua_touserdata(L: *mut lua_State, idx: std::ffi::c_int) -> *mut std::ffi::c_void;
    fn lua_pushinteger(L: *mut lua_State, n: lua_Integer);
    fn lua_pushcclosure(L: *mut lua_State, fn_0: lua_CFunction, n: std::ffi::c_int);
    fn lua_pushlightuserdata(L: *mut lua_State, p: *mut std::ffi::c_void);
    fn lua_pcallk(
        L: *mut lua_State,
        nargs: std::ffi::c_int,
        nresults: std::ffi::c_int,
        errfunc: std::ffi::c_int,
        ctx: lua_KContext,
        k: lua_KFunction,
    ) -> std::ffi::c_int;
    fn lua_load(
        L: *mut lua_State,
        reader_0: lua_Reader,
        dt: *mut std::ffi::c_void,
        chunkname: *const std::ffi::c_char,
        mode: *const std::ffi::c_char,
    ) -> std::ffi::c_int;
    fn luaL_loadfilex(
        L: *mut lua_State,
        filename: *const std::ffi::c_char,
        mode: *const std::ffi::c_char,
    ) -> std::ffi::c_int;
    fn luaL_newstate() -> *mut lua_State;
    fn luaG_getfuncline(f: *const Proto, pc: std::ffi::c_int) -> std::ffi::c_int;
    fn luaU_dump(
        L: *mut lua_State,
        f: *const Proto,
        w: lua_Writer,
        data: *mut std::ffi::c_void,
        strip: std::ffi::c_int,
    ) -> std::ffi::c_int;
}
pub type __off_t = std::ffi::c_long;
pub type __off64_t = std::ffi::c_long;
pub type __sig_atomic_t = std::ffi::c_int;
pub type C2RustUnnamed = std::ffi::c_uint;
pub const _ISalnum: C2RustUnnamed = 8;
pub const _ISpunct: C2RustUnnamed = 4;
pub const _IScntrl: C2RustUnnamed = 2;
pub const _ISblank: C2RustUnnamed = 1;
pub const _ISgraph: C2RustUnnamed = 32768;
pub const _ISprint: C2RustUnnamed = 16384;
pub const _ISspace: C2RustUnnamed = 8192;
pub const _ISxdigit: C2RustUnnamed = 4096;
pub const _ISdigit: C2RustUnnamed = 2048;
pub const _ISalpha: C2RustUnnamed = 1024;
pub const _ISlower: C2RustUnnamed = 512;
pub const _ISupper: C2RustUnnamed = 256;
pub type size_t = std::ffi::c_ulong;
#[derive(Copy, Clone, BitfieldStruct)]
#[repr(C)]
pub struct _IO_FILE {
    pub _flags: std::ffi::c_int,
    pub _IO_read_ptr: *mut std::ffi::c_char,
    pub _IO_read_end: *mut std::ffi::c_char,
    pub _IO_read_base: *mut std::ffi::c_char,
    pub _IO_write_base: *mut std::ffi::c_char,
    pub _IO_write_ptr: *mut std::ffi::c_char,
    pub _IO_write_end: *mut std::ffi::c_char,
    pub _IO_buf_base: *mut std::ffi::c_char,
    pub _IO_buf_end: *mut std::ffi::c_char,
    pub _IO_save_base: *mut std::ffi::c_char,
    pub _IO_backup_base: *mut std::ffi::c_char,
    pub _IO_save_end: *mut std::ffi::c_char,
    pub _markers: *mut _IO_marker,
    pub _chain: *mut _IO_FILE,
    pub _fileno: std::ffi::c_int,
    #[bitfield(name = "_flags2", ty = "std::ffi::c_int", bits = "0..=23")]
    pub _flags2: [u8; 3],
    pub _short_backupbuf: [std::ffi::c_char; 1],
    pub _old_offset: __off_t,
    pub _cur_column: std::ffi::c_ushort,
    pub _vtable_offset: std::ffi::c_schar,
    pub _shortbuf: [std::ffi::c_char; 1],
    pub _lock: *mut std::ffi::c_void,
    pub _offset: __off64_t,
    pub _codecvt: *mut _IO_codecvt,
    pub _wide_data: *mut _IO_wide_data,
    pub _freeres_list: *mut _IO_FILE,
    pub _freeres_buf: *mut std::ffi::c_void,
    pub _prevchain: *mut *mut _IO_FILE,
    pub _mode: std::ffi::c_int,
    pub _unused2: [std::ffi::c_char; 20],
}
pub type _IO_lock_t = ();
pub type FILE = _IO_FILE;
pub type ptrdiff_t = std::ffi::c_long;
pub type intptr_t = std::ffi::c_long;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct lua_State {
    pub next: *mut GCObject,
    pub tt: lu_byte,
    pub marked: lu_byte,
    pub status: lu_byte,
    pub allowhook: lu_byte,
    pub nci: std::ffi::c_ushort,
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
    pub oldpc: std::ffi::c_int,
    pub basehookcount: std::ffi::c_int,
    pub hookcount: std::ffi::c_int,
    pub hookmask: sig_atomic_t,
}
pub type sig_atomic_t = __sig_atomic_t;
pub type l_uint32 = std::ffi::c_uint;
pub type lua_Hook = Option::<unsafe extern "C" fn(*mut lua_State, *mut lua_Debug) -> ()>;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct lua_Debug {
    pub event: std::ffi::c_int,
    pub name: *const std::ffi::c_char,
    pub namewhat: *const std::ffi::c_char,
    pub what: *const std::ffi::c_char,
    pub source: *const std::ffi::c_char,
    pub srclen: size_t,
    pub currentline: std::ffi::c_int,
    pub linedefined: std::ffi::c_int,
    pub lastlinedefined: std::ffi::c_int,
    pub nups: std::ffi::c_uchar,
    pub nparams: std::ffi::c_uchar,
    pub isvararg: std::ffi::c_char,
    pub istailcall: std::ffi::c_char,
    pub ftransfer: std::ffi::c_ushort,
    pub ntransfer: std::ffi::c_ushort,
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
    pub u: C2RustUnnamed_2,
    pub u2: C2RustUnnamed_0,
    pub nresults: std::ffi::c_short,
    pub callstatus: std::ffi::c_ushort,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed_0 {
    pub funcidx: std::ffi::c_int,
    pub nyield: std::ffi::c_int,
    pub nres: std::ffi::c_int,
    pub transferinfo: C2RustUnnamed_1,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_1 {
    pub ftransfer: std::ffi::c_ushort,
    pub ntransfer: std::ffi::c_ushort,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed_2 {
    pub l: C2RustUnnamed_4,
    pub c: C2RustUnnamed_3,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_3 {
    pub k: lua_KFunction,
    pub old_errfunc: ptrdiff_t,
    pub ctx: lua_KContext,
}
pub type lua_KContext = intptr_t;
pub type lua_KFunction = Option::<
    unsafe extern "C" fn(
        *mut lua_State,
        std::ffi::c_int,
        lua_KContext,
    ) -> std::ffi::c_int,
>;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_4 {
    pub savedpc: *const Instruction,
    pub trap: sig_atomic_t,
    pub nextraargs: std::ffi::c_int,
}
pub type Instruction = l_uint32;
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
    pub tbclist: C2RustUnnamed_5,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_5 {
    pub value_: Value,
    pub tt_: lu_byte,
    pub delta: std::ffi::c_ushort,
}
pub type lu_byte = std::ffi::c_uchar;
#[derive(Copy, Clone)]
#[repr(C)]
pub union Value {
    pub gc: *mut GCObject,
    pub p: *mut std::ffi::c_void,
    pub f: lua_CFunction,
    pub i: lua_Integer,
    pub n: lua_Number,
    pub ub: lu_byte,
}
pub type lua_Number = std::ffi::c_double;
pub type lua_Integer = std::ffi::c_longlong;
pub type lua_CFunction = Option::<
    unsafe extern "C" fn(*mut lua_State) -> std::ffi::c_int,
>;
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
pub struct UpVal {
    pub next: *mut GCObject,
    pub tt: lu_byte,
    pub marked: lu_byte,
    pub v: C2RustUnnamed_8,
    pub u: C2RustUnnamed_6,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed_6 {
    pub open: C2RustUnnamed_7,
    pub value: TValue,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_7 {
    pub next: *mut UpVal,
    pub previous: *mut *mut UpVal,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed_8 {
    pub p: *mut TValue,
    pub offset: ptrdiff_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct global_State {
    pub frealloc: lua_Alloc,
    pub ud: *mut std::ffi::c_void,
    pub totalbytes: l_mem,
    pub GCdebt: l_mem,
    pub GCestimate: lu_mem,
    pub lastatomic: lu_mem,
    pub strt: stringtable,
    pub l_registry: TValue,
    pub nilvalue: TValue,
    pub seed: std::ffi::c_uint,
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
    pub ud_warn: *mut std::ffi::c_void,
}
pub type lua_WarnFunction = Option::<
    unsafe extern "C" fn(
        *mut std::ffi::c_void,
        *const std::ffi::c_char,
        std::ffi::c_int,
    ) -> (),
>;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct TString {
    pub next: *mut GCObject,
    pub tt: lu_byte,
    pub marked: lu_byte,
    pub extra: lu_byte,
    pub shrlen: lu_byte,
    pub hash: std::ffi::c_uint,
    pub u: C2RustUnnamed_9,
    pub contents: [std::ffi::c_char; 1],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed_9 {
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
    pub alimit: std::ffi::c_uint,
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
    pub next: std::ffi::c_int,
    pub key_val: Value,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct stringtable {
    pub hash: *mut *mut TString,
    pub nuse: std::ffi::c_int,
    pub size: std::ffi::c_int,
}
pub type lu_mem = size_t;
pub type l_mem = ptrdiff_t;
pub type lua_Alloc = Option::<
    unsafe extern "C" fn(
        *mut std::ffi::c_void,
        *mut std::ffi::c_void,
        size_t,
        size_t,
    ) -> *mut std::ffi::c_void,
>;
pub type lua_Reader = Option::<
    unsafe extern "C" fn(
        *mut lua_State,
        *mut std::ffi::c_void,
        *mut size_t,
    ) -> *const std::ffi::c_char,
>;
pub type lua_Writer = Option::<
    unsafe extern "C" fn(
        *mut lua_State,
        *const std::ffi::c_void,
        size_t,
        *mut std::ffi::c_void,
    ) -> std::ffi::c_int,
>;
pub type ls_byte = std::ffi::c_schar;
#[derive(Copy, Clone)]
#[repr(C)]
pub union UValue {
    pub uv: TValue,
    pub n: lua_Number,
    pub u: std::ffi::c_double,
    pub s: *mut std::ffi::c_void,
    pub i: lua_Integer,
    pub l: std::ffi::c_long,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Udata {
    pub next: *mut GCObject,
    pub tt: lu_byte,
    pub marked: lu_byte,
    pub nuvalue: std::ffi::c_ushort,
    pub len: size_t,
    pub metatable: *mut Table,
    pub gclist: *mut GCObject,
    pub uv: [UValue; 1],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Upvaldesc {
    pub name: *mut TString,
    pub instack: lu_byte,
    pub idx: lu_byte,
    pub kind: lu_byte,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct LocVar {
    pub varname: *mut TString,
    pub startpc: std::ffi::c_int,
    pub endpc: std::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct AbsLineInfo {
    pub pc: std::ffi::c_int,
    pub line: std::ffi::c_int,
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
    pub sizeupvalues: std::ffi::c_int,
    pub sizek: std::ffi::c_int,
    pub sizecode: std::ffi::c_int,
    pub sizelineinfo: std::ffi::c_int,
    pub sizep: std::ffi::c_int,
    pub sizelocvars: std::ffi::c_int,
    pub sizeabslineinfo: std::ffi::c_int,
    pub linedefined: std::ffi::c_int,
    pub lastlinedefined: std::ffi::c_int,
    pub k: *mut TValue,
    pub code: *mut Instruction,
    pub p: *mut *mut Proto,
    pub upvalues: *mut Upvaldesc,
    pub lineinfo: *mut ls_byte,
    pub abslineinfo: *mut AbsLineInfo,
    pub locvars: *mut LocVar,
    pub source: *mut TString,
    pub gclist: *mut GCObject,
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
pub union Closure {
    pub c: CClosure,
    pub l: LClosure,
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
pub type OpCode = std::ffi::c_uint;
pub const OP_EXTRAARG: OpCode = 82;
pub const OP_VARARGPREP: OpCode = 81;
pub const OP_VARARG: OpCode = 80;
pub const OP_CLOSURE: OpCode = 79;
pub const OP_SETLIST: OpCode = 78;
pub const OP_TFORLOOP: OpCode = 77;
pub const OP_TFORCALL: OpCode = 76;
pub const OP_TFORPREP: OpCode = 75;
pub const OP_FORPREP: OpCode = 74;
pub const OP_FORLOOP: OpCode = 73;
pub const OP_RETURN1: OpCode = 72;
pub const OP_RETURN0: OpCode = 71;
pub const OP_RETURN: OpCode = 70;
pub const OP_TAILCALL: OpCode = 69;
pub const OP_CALL: OpCode = 68;
pub const OP_TESTSET: OpCode = 67;
pub const OP_TEST: OpCode = 66;
pub const OP_GEI: OpCode = 65;
pub const OP_GTI: OpCode = 64;
pub const OP_LEI: OpCode = 63;
pub const OP_LTI: OpCode = 62;
pub const OP_EQI: OpCode = 61;
pub const OP_EQK: OpCode = 60;
pub const OP_LE: OpCode = 59;
pub const OP_LT: OpCode = 58;
pub const OP_EQ: OpCode = 57;
pub const OP_JMP: OpCode = 56;
pub const OP_TBC: OpCode = 55;
pub const OP_CLOSE: OpCode = 54;
pub const OP_CONCAT: OpCode = 53;
pub const OP_LEN: OpCode = 52;
pub const OP_NOT: OpCode = 51;
pub const OP_BNOT: OpCode = 50;
pub const OP_UNM: OpCode = 49;
pub const OP_MMBINK: OpCode = 48;
pub const OP_MMBINI: OpCode = 47;
pub const OP_MMBIN: OpCode = 46;
pub const OP_SHR: OpCode = 45;
pub const OP_SHL: OpCode = 44;
pub const OP_BXOR: OpCode = 43;
pub const OP_BOR: OpCode = 42;
pub const OP_BAND: OpCode = 41;
pub const OP_IDIV: OpCode = 40;
pub const OP_DIV: OpCode = 39;
pub const OP_POW: OpCode = 38;
pub const OP_MOD: OpCode = 37;
pub const OP_MUL: OpCode = 36;
pub const OP_SUB: OpCode = 35;
pub const OP_ADD: OpCode = 34;
pub const OP_SHLI: OpCode = 33;
pub const OP_SHRI: OpCode = 32;
pub const OP_BXORK: OpCode = 31;
pub const OP_BORK: OpCode = 30;
pub const OP_BANDK: OpCode = 29;
pub const OP_IDIVK: OpCode = 28;
pub const OP_DIVK: OpCode = 27;
pub const OP_POWK: OpCode = 26;
pub const OP_MODK: OpCode = 25;
pub const OP_MULK: OpCode = 24;
pub const OP_SUBK: OpCode = 23;
pub const OP_ADDK: OpCode = 22;
pub const OP_ADDI: OpCode = 21;
pub const OP_SELF: OpCode = 20;
pub const OP_NEWTABLE: OpCode = 19;
pub const OP_SETFIELD: OpCode = 18;
pub const OP_SETI: OpCode = 17;
pub const OP_SETTABLE: OpCode = 16;
pub const OP_SETTABUP: OpCode = 15;
pub const OP_GETFIELD: OpCode = 14;
pub const OP_GETI: OpCode = 13;
pub const OP_GETTABLE: OpCode = 12;
pub const OP_GETTABUP: OpCode = 11;
pub const OP_SETUPVAL: OpCode = 10;
pub const OP_GETUPVAL: OpCode = 9;
pub const OP_LOADNIL: OpCode = 8;
pub const OP_LOADTRUE: OpCode = 7;
pub const OP_LFALSESKIP: OpCode = 6;
pub const OP_LOADFALSE: OpCode = 5;
pub const OP_LOADKX: OpCode = 4;
pub const OP_LOADK: OpCode = 3;
pub const OP_LOADF: OpCode = 2;
pub const OP_LOADI: OpCode = 1;
pub const OP_MOVE: OpCode = 0;
static mut opnames: [*const std::ffi::c_char; 84] = [
    b"MOVE\0" as *const u8 as *const std::ffi::c_char,
    b"LOADI\0" as *const u8 as *const std::ffi::c_char,
    b"LOADF\0" as *const u8 as *const std::ffi::c_char,
    b"LOADK\0" as *const u8 as *const std::ffi::c_char,
    b"LOADKX\0" as *const u8 as *const std::ffi::c_char,
    b"LOADFALSE\0" as *const u8 as *const std::ffi::c_char,
    b"LFALSESKIP\0" as *const u8 as *const std::ffi::c_char,
    b"LOADTRUE\0" as *const u8 as *const std::ffi::c_char,
    b"LOADNIL\0" as *const u8 as *const std::ffi::c_char,
    b"GETUPVAL\0" as *const u8 as *const std::ffi::c_char,
    b"SETUPVAL\0" as *const u8 as *const std::ffi::c_char,
    b"GETTABUP\0" as *const u8 as *const std::ffi::c_char,
    b"GETTABLE\0" as *const u8 as *const std::ffi::c_char,
    b"GETI\0" as *const u8 as *const std::ffi::c_char,
    b"GETFIELD\0" as *const u8 as *const std::ffi::c_char,
    b"SETTABUP\0" as *const u8 as *const std::ffi::c_char,
    b"SETTABLE\0" as *const u8 as *const std::ffi::c_char,
    b"SETI\0" as *const u8 as *const std::ffi::c_char,
    b"SETFIELD\0" as *const u8 as *const std::ffi::c_char,
    b"NEWTABLE\0" as *const u8 as *const std::ffi::c_char,
    b"SELF\0" as *const u8 as *const std::ffi::c_char,
    b"ADDI\0" as *const u8 as *const std::ffi::c_char,
    b"ADDK\0" as *const u8 as *const std::ffi::c_char,
    b"SUBK\0" as *const u8 as *const std::ffi::c_char,
    b"MULK\0" as *const u8 as *const std::ffi::c_char,
    b"MODK\0" as *const u8 as *const std::ffi::c_char,
    b"POWK\0" as *const u8 as *const std::ffi::c_char,
    b"DIVK\0" as *const u8 as *const std::ffi::c_char,
    b"IDIVK\0" as *const u8 as *const std::ffi::c_char,
    b"BANDK\0" as *const u8 as *const std::ffi::c_char,
    b"BORK\0" as *const u8 as *const std::ffi::c_char,
    b"BXORK\0" as *const u8 as *const std::ffi::c_char,
    b"SHRI\0" as *const u8 as *const std::ffi::c_char,
    b"SHLI\0" as *const u8 as *const std::ffi::c_char,
    b"ADD\0" as *const u8 as *const std::ffi::c_char,
    b"SUB\0" as *const u8 as *const std::ffi::c_char,
    b"MUL\0" as *const u8 as *const std::ffi::c_char,
    b"MOD\0" as *const u8 as *const std::ffi::c_char,
    b"POW\0" as *const u8 as *const std::ffi::c_char,
    b"DIV\0" as *const u8 as *const std::ffi::c_char,
    b"IDIV\0" as *const u8 as *const std::ffi::c_char,
    b"BAND\0" as *const u8 as *const std::ffi::c_char,
    b"BOR\0" as *const u8 as *const std::ffi::c_char,
    b"BXOR\0" as *const u8 as *const std::ffi::c_char,
    b"SHL\0" as *const u8 as *const std::ffi::c_char,
    b"SHR\0" as *const u8 as *const std::ffi::c_char,
    b"MMBIN\0" as *const u8 as *const std::ffi::c_char,
    b"MMBINI\0" as *const u8 as *const std::ffi::c_char,
    b"MMBINK\0" as *const u8 as *const std::ffi::c_char,
    b"UNM\0" as *const u8 as *const std::ffi::c_char,
    b"BNOT\0" as *const u8 as *const std::ffi::c_char,
    b"NOT\0" as *const u8 as *const std::ffi::c_char,
    b"LEN\0" as *const u8 as *const std::ffi::c_char,
    b"CONCAT\0" as *const u8 as *const std::ffi::c_char,
    b"CLOSE\0" as *const u8 as *const std::ffi::c_char,
    b"TBC\0" as *const u8 as *const std::ffi::c_char,
    b"JMP\0" as *const u8 as *const std::ffi::c_char,
    b"EQ\0" as *const u8 as *const std::ffi::c_char,
    b"LT\0" as *const u8 as *const std::ffi::c_char,
    b"LE\0" as *const u8 as *const std::ffi::c_char,
    b"EQK\0" as *const u8 as *const std::ffi::c_char,
    b"EQI\0" as *const u8 as *const std::ffi::c_char,
    b"LTI\0" as *const u8 as *const std::ffi::c_char,
    b"LEI\0" as *const u8 as *const std::ffi::c_char,
    b"GTI\0" as *const u8 as *const std::ffi::c_char,
    b"GEI\0" as *const u8 as *const std::ffi::c_char,
    b"TEST\0" as *const u8 as *const std::ffi::c_char,
    b"TESTSET\0" as *const u8 as *const std::ffi::c_char,
    b"CALL\0" as *const u8 as *const std::ffi::c_char,
    b"TAILCALL\0" as *const u8 as *const std::ffi::c_char,
    b"RETURN\0" as *const u8 as *const std::ffi::c_char,
    b"RETURN0\0" as *const u8 as *const std::ffi::c_char,
    b"RETURN1\0" as *const u8 as *const std::ffi::c_char,
    b"FORLOOP\0" as *const u8 as *const std::ffi::c_char,
    b"FORPREP\0" as *const u8 as *const std::ffi::c_char,
    b"TFORPREP\0" as *const u8 as *const std::ffi::c_char,
    b"TFORCALL\0" as *const u8 as *const std::ffi::c_char,
    b"TFORLOOP\0" as *const u8 as *const std::ffi::c_char,
    b"SETLIST\0" as *const u8 as *const std::ffi::c_char,
    b"CLOSURE\0" as *const u8 as *const std::ffi::c_char,
    b"VARARG\0" as *const u8 as *const std::ffi::c_char,
    b"VARARGPREP\0" as *const u8 as *const std::ffi::c_char,
    b"EXTRAARG\0" as *const u8 as *const std::ffi::c_char,
    0 as *const std::ffi::c_char,
];
static mut listing: std::ffi::c_int = 0 as std::ffi::c_int;
static mut dumping: std::ffi::c_int = 1 as std::ffi::c_int;
static mut stripping: std::ffi::c_int = 0 as std::ffi::c_int;
static mut Output: [std::ffi::c_char; 9] = unsafe {
    *::core::mem::transmute::<&[u8; 9], &mut [std::ffi::c_char; 9]>(b"luac.out\0")
};
static mut output: *const std::ffi::c_char = unsafe { Output.as_ptr() as *mut _ };
static mut progname: *const std::ffi::c_char = b"luac\0" as *const u8
    as *const std::ffi::c_char;
static mut tmname: *mut *mut TString = 0 as *const *mut TString as *mut *mut TString;
unsafe extern "C" fn fatal(mut message: *const std::ffi::c_char) {
    fprintf(
        stderr,
        b"%s: %s\n\0" as *const u8 as *const std::ffi::c_char,
        progname,
        message,
    );
    exit(1 as std::ffi::c_int);
}
unsafe extern "C" fn cannot(mut what: *const std::ffi::c_char) {
    fprintf(
        stderr,
        b"%s: cannot %s %s: %s\n\0" as *const u8 as *const std::ffi::c_char,
        progname,
        what,
        output,
        strerror(*__errno_location()),
    );
    exit(1 as std::ffi::c_int);
}
unsafe extern "C" fn usage(mut message: *const std::ffi::c_char) {
    if *message as std::ffi::c_int == '-' as i32 {
        fprintf(
            stderr,
            b"%s: unrecognized option '%s'\n\0" as *const u8 as *const std::ffi::c_char,
            progname,
            message,
        );
    } else {
        fprintf(
            stderr,
            b"%s: %s\n\0" as *const u8 as *const std::ffi::c_char,
            progname,
            message,
        );
    }
    fprintf(
        stderr,
        b"usage: %s [options] [filenames]\nAvailable options are:\n  -l       list (use -l -l for full listing)\n  -o name  output to file 'name' (default is \"%s\")\n  -p       parse only\n  -s       strip debug information\n  -v       show version information\n  --       stop handling options\n  -        stop handling options and process stdin\n\0"
            as *const u8 as *const std::ffi::c_char,
        progname,
        Output.as_mut_ptr(),
    );
    exit(1 as std::ffi::c_int);
}
unsafe extern "C" fn doargs(
    mut argc: std::ffi::c_int,
    mut argv: *mut *mut std::ffi::c_char,
) -> std::ffi::c_int {
    let mut i: std::ffi::c_int = 0;
    let mut version: std::ffi::c_int = 0 as std::ffi::c_int;
    if !(*argv.offset(0 as std::ffi::c_int as isize)).is_null()
        && **argv.offset(0 as std::ffi::c_int as isize) as std::ffi::c_int
            != 0 as std::ffi::c_int
    {
        progname = *argv.offset(0 as std::ffi::c_int as isize);
    }
    i = 1 as std::ffi::c_int;
    while i < argc {
        if **argv.offset(i as isize) as std::ffi::c_int != '-' as i32 {
            break;
        } else if strcmp(
            *argv.offset(i as isize),
            b"--\0" as *const u8 as *const std::ffi::c_char,
        ) == 0 as std::ffi::c_int
        {
            i += 1;
            i;
            if version != 0 {
                version += 1;
                version;
            }
            break;
        } else {
            if strcmp(
                *argv.offset(i as isize),
                b"-\0" as *const u8 as *const std::ffi::c_char,
            ) == 0 as std::ffi::c_int
            {
                break;
            }
            if strcmp(
                *argv.offset(i as isize),
                b"-l\0" as *const u8 as *const std::ffi::c_char,
            ) == 0 as std::ffi::c_int
            {
                listing += 1;
                listing;
            } else if strcmp(
                *argv.offset(i as isize),
                b"-o\0" as *const u8 as *const std::ffi::c_char,
            ) == 0 as std::ffi::c_int
            {
                i += 1;
                output = *argv.offset(i as isize);
                if output.is_null() || *output as std::ffi::c_int == 0 as std::ffi::c_int
                    || *output as std::ffi::c_int == '-' as i32
                        && *output.offset(1 as std::ffi::c_int as isize)
                            as std::ffi::c_int != 0 as std::ffi::c_int
                {
                    usage(
                        b"'-o' needs argument\0" as *const u8 as *const std::ffi::c_char,
                    );
                }
                if strcmp(
                    *argv.offset(i as isize),
                    b"-\0" as *const u8 as *const std::ffi::c_char,
                ) == 0 as std::ffi::c_int
                {
                    output = 0 as *const std::ffi::c_char;
                }
            } else if strcmp(
                *argv.offset(i as isize),
                b"-p\0" as *const u8 as *const std::ffi::c_char,
            ) == 0 as std::ffi::c_int
            {
                dumping = 0 as std::ffi::c_int;
            } else if strcmp(
                *argv.offset(i as isize),
                b"-s\0" as *const u8 as *const std::ffi::c_char,
            ) == 0 as std::ffi::c_int
            {
                stripping = 1 as std::ffi::c_int;
            } else if strcmp(
                *argv.offset(i as isize),
                b"-v\0" as *const u8 as *const std::ffi::c_char,
            ) == 0 as std::ffi::c_int
            {
                version += 1;
                version;
            } else {
                usage(*argv.offset(i as isize));
            }
            i += 1;
            i;
        }
    }
    if i == argc && (listing != 0 || dumping == 0) {
        dumping = 0 as std::ffi::c_int;
        i -= 1;
        let ref mut fresh0 = *argv.offset(i as isize);
        *fresh0 = Output.as_mut_ptr();
    }
    if version != 0 {
        printf(
            b"%s\n\0" as *const u8 as *const std::ffi::c_char,
            b"Lua 5.4.8  Copyright (C) 1994-2025 Lua.org, PUC-Rio\0" as *const u8
                as *const std::ffi::c_char,
        );
        if version == argc - 1 as std::ffi::c_int {
            exit(0 as std::ffi::c_int);
        }
    }
    return i;
}
unsafe extern "C" fn reader(
    mut L: *mut lua_State,
    mut ud: *mut std::ffi::c_void,
    mut size: *mut size_t,
) -> *const std::ffi::c_char {
    let ref mut fresh1 = *(ud as *mut std::ffi::c_int);
    let fresh2 = *fresh1;
    *fresh1 = *fresh1 - 1;
    if fresh2 != 0 {
        *size = (::core::mem::size_of::<[std::ffi::c_char; 20]>() as std::ffi::c_ulong)
            .wrapping_sub(1 as std::ffi::c_int as std::ffi::c_ulong);
        return b"(function()end)();\n\0" as *const u8 as *const std::ffi::c_char;
    } else {
        *size = 0 as std::ffi::c_int as size_t;
        return 0 as *const std::ffi::c_char;
    };
}
unsafe extern "C" fn combine(
    mut L: *mut lua_State,
    mut n: std::ffi::c_int,
) -> *const Proto {
    if n == 1 as std::ffi::c_int {
        return (*&mut (*((*((*L).top.p).offset(-(1 as std::ffi::c_int) as isize))
            .val
            .value_
            .gc as *mut GCUnion))
            .cl
            .l)
            .p
    } else {
        let mut f: *mut Proto = 0 as *mut Proto;
        let mut i: std::ffi::c_int = n;
        if lua_load(
            L,
            Some(
                reader
                    as unsafe extern "C" fn(
                        *mut lua_State,
                        *mut std::ffi::c_void,
                        *mut size_t,
                    ) -> *const std::ffi::c_char,
            ),
            &mut i as *mut std::ffi::c_int as *mut std::ffi::c_void,
            b"=(luac)\0" as *const u8 as *const std::ffi::c_char,
            0 as *const std::ffi::c_char,
        ) != 0 as std::ffi::c_int
        {
            fatal(lua_tolstring(L, -(1 as std::ffi::c_int), 0 as *mut size_t));
        }
        f = (*&mut (*((*((*L).top.p).offset(-(1 as std::ffi::c_int) as isize))
            .val
            .value_
            .gc as *mut GCUnion))
            .cl
            .l)
            .p;
        i = 0 as std::ffi::c_int;
        while i < n {
            let ref mut fresh3 = *((*f).p).offset(i as isize);
            *fresh3 = (*&mut (*((*((*L).top.p)
                .offset((i - n - 1 as std::ffi::c_int) as isize))
                .val
                .value_
                .gc as *mut GCUnion))
                .cl
                .l)
                .p;
            if (**((*f).p).offset(i as isize)).sizeupvalues > 0 as std::ffi::c_int {
                (*((**((*f).p).offset(i as isize)).upvalues)
                    .offset(0 as std::ffi::c_int as isize))
                    .instack = 0 as std::ffi::c_int as lu_byte;
            }
            i += 1;
            i;
        }
        return f;
    };
}
unsafe extern "C" fn writer(
    mut L: *mut lua_State,
    mut p: *const std::ffi::c_void,
    mut size: size_t,
    mut u: *mut std::ffi::c_void,
) -> std::ffi::c_int {
    return (fwrite(p, size, 1 as std::ffi::c_int as std::ffi::c_ulong, u as *mut FILE)
        != 1 as std::ffi::c_int as std::ffi::c_ulong
        && size != 0 as std::ffi::c_int as size_t) as std::ffi::c_int;
}
unsafe extern "C" fn pmain(mut L: *mut lua_State) -> std::ffi::c_int {
    let mut argc: std::ffi::c_int = lua_tointegerx(
        L,
        1 as std::ffi::c_int,
        0 as *mut std::ffi::c_int,
    ) as std::ffi::c_int;
    let mut argv: *mut *mut std::ffi::c_char = lua_touserdata(L, 2 as std::ffi::c_int)
        as *mut *mut std::ffi::c_char;
    let mut f: *const Proto = 0 as *const Proto;
    let mut i: std::ffi::c_int = 0;
    tmname = ((*(*L).l_G).tmname).as_mut_ptr();
    if lua_checkstack(L, argc) == 0 {
        fatal(b"too many input files\0" as *const u8 as *const std::ffi::c_char);
    }
    i = 0 as std::ffi::c_int;
    while i < argc {
        let mut filename: *const std::ffi::c_char = if strcmp(
            *argv.offset(i as isize),
            b"-\0" as *const u8 as *const std::ffi::c_char,
        ) == 0 as std::ffi::c_int
        {
            0 as *mut std::ffi::c_char
        } else {
            *argv.offset(i as isize)
        };
        if luaL_loadfilex(L, filename, 0 as *const std::ffi::c_char)
            != 0 as std::ffi::c_int
        {
            fatal(lua_tolstring(L, -(1 as std::ffi::c_int), 0 as *mut size_t));
        }
        i += 1;
        i;
    }
    f = combine(L, argc);
    if listing != 0 {
        PrintFunction(f, (listing > 1 as std::ffi::c_int) as std::ffi::c_int);
    }
    if dumping != 0 {
        let mut D: *mut FILE = if output.is_null() {
            stdout
        } else {
            fopen(output, b"wb\0" as *const u8 as *const std::ffi::c_char)
        };
        if D.is_null() {
            cannot(b"open\0" as *const u8 as *const std::ffi::c_char);
        }
        luaU_dump(
            L,
            f,
            Some(
                writer
                    as unsafe extern "C" fn(
                        *mut lua_State,
                        *const std::ffi::c_void,
                        size_t,
                        *mut std::ffi::c_void,
                    ) -> std::ffi::c_int,
            ),
            D as *mut std::ffi::c_void,
            stripping,
        );
        if ferror(D) != 0 {
            cannot(b"write\0" as *const u8 as *const std::ffi::c_char);
        }
        if fclose(D) != 0 {
            cannot(b"close\0" as *const u8 as *const std::ffi::c_char);
        }
    }
    return 0 as std::ffi::c_int;
}
unsafe fn main_0(
    mut argc: std::ffi::c_int,
    mut argv: *mut *mut std::ffi::c_char,
) -> std::ffi::c_int {
    let mut L: *mut lua_State = 0 as *mut lua_State;
    let mut i: std::ffi::c_int = doargs(argc, argv);
    argc -= i;
    argv = argv.offset(i as isize);
    if argc <= 0 as std::ffi::c_int {
        usage(b"no input files given\0" as *const u8 as *const std::ffi::c_char);
    }
    L = luaL_newstate();
    if L.is_null() {
        fatal(
            b"cannot create state: not enough memory\0" as *const u8
                as *const std::ffi::c_char,
        );
    }
    lua_pushcclosure(
        L,
        Some(pmain as unsafe extern "C" fn(*mut lua_State) -> std::ffi::c_int),
        0 as std::ffi::c_int,
    );
    lua_pushinteger(L, argc as lua_Integer);
    lua_pushlightuserdata(L, argv as *mut std::ffi::c_void);
    if lua_pcallk(
        L,
        2 as std::ffi::c_int,
        0 as std::ffi::c_int,
        0 as std::ffi::c_int,
        0 as std::ffi::c_int as lua_KContext,
        None,
    ) != 0 as std::ffi::c_int
    {
        fatal(lua_tolstring(L, -(1 as std::ffi::c_int), 0 as *mut size_t));
    }
    lua_close(L);
    return 0 as std::ffi::c_int;
}
unsafe extern "C" fn PrintString(mut ts: *const TString) {
    let mut s: *const std::ffi::c_char = ((*ts).contents).as_ptr();
    let mut i: size_t = 0;
    let mut n: size_t = if (*ts).shrlen as std::ffi::c_int != 0xff as std::ffi::c_int {
        (*ts).shrlen as size_t
    } else {
        (*ts).u.lnglen
    };
    printf(b"\"\0" as *const u8 as *const std::ffi::c_char);
    i = 0 as std::ffi::c_int as size_t;
    while i < n {
        let mut c: std::ffi::c_int = *s.offset(i as isize) as std::ffi::c_uchar
            as std::ffi::c_int;
        match c {
            34 => {
                printf(b"\\\"\0" as *const u8 as *const std::ffi::c_char);
            }
            92 => {
                printf(b"\\\\\0" as *const u8 as *const std::ffi::c_char);
            }
            7 => {
                printf(b"\\a\0" as *const u8 as *const std::ffi::c_char);
            }
            8 => {
                printf(b"\\b\0" as *const u8 as *const std::ffi::c_char);
            }
            12 => {
                printf(b"\\f\0" as *const u8 as *const std::ffi::c_char);
            }
            10 => {
                printf(b"\\n\0" as *const u8 as *const std::ffi::c_char);
            }
            13 => {
                printf(b"\\r\0" as *const u8 as *const std::ffi::c_char);
            }
            9 => {
                printf(b"\\t\0" as *const u8 as *const std::ffi::c_char);
            }
            11 => {
                printf(b"\\v\0" as *const u8 as *const std::ffi::c_char);
            }
            _ => {
                if *(*__ctype_b_loc()).offset(c as isize) as std::ffi::c_int
                    & _ISprint as std::ffi::c_int as std::ffi::c_ushort
                        as std::ffi::c_int != 0
                {
                    printf(b"%c\0" as *const u8 as *const std::ffi::c_char, c);
                } else {
                    printf(b"\\%03d\0" as *const u8 as *const std::ffi::c_char, c);
                }
            }
        }
        i = i.wrapping_add(1);
        i;
    }
    printf(b"\"\0" as *const u8 as *const std::ffi::c_char);
}
unsafe extern "C" fn PrintType(mut f: *const Proto, mut i: std::ffi::c_int) {
    let mut o: *const TValue = &mut *((*f).k).offset(i as isize) as *mut TValue;
    match (*o).tt_ as std::ffi::c_int & 0x3f as std::ffi::c_int {
        0 => {
            printf(b"N\0" as *const u8 as *const std::ffi::c_char);
        }
        1 | 17 => {
            printf(b"B\0" as *const u8 as *const std::ffi::c_char);
        }
        19 => {
            printf(b"F\0" as *const u8 as *const std::ffi::c_char);
        }
        3 => {
            printf(b"I\0" as *const u8 as *const std::ffi::c_char);
        }
        4 | 20 => {
            printf(b"S\0" as *const u8 as *const std::ffi::c_char);
        }
        _ => {
            printf(
                b"?%d\0" as *const u8 as *const std::ffi::c_char,
                (*o).tt_ as std::ffi::c_int & 0x3f as std::ffi::c_int,
            );
        }
    }
    printf(b"\t\0" as *const u8 as *const std::ffi::c_char);
}
unsafe extern "C" fn PrintConstant(mut f: *const Proto, mut i: std::ffi::c_int) {
    let mut o: *const TValue = &mut *((*f).k).offset(i as isize) as *mut TValue;
    match (*o).tt_ as std::ffi::c_int & 0x3f as std::ffi::c_int {
        0 => {
            printf(b"nil\0" as *const u8 as *const std::ffi::c_char);
        }
        1 => {
            printf(b"false\0" as *const u8 as *const std::ffi::c_char);
        }
        17 => {
            printf(b"true\0" as *const u8 as *const std::ffi::c_char);
        }
        19 => {
            let mut buff: [std::ffi::c_char; 100] = [0; 100];
            sprintf(
                buff.as_mut_ptr(),
                b"%.14g\0" as *const u8 as *const std::ffi::c_char,
                (*o).value_.n,
            );
            printf(b"%s\0" as *const u8 as *const std::ffi::c_char, buff.as_mut_ptr());
            if buff[strspn(
                buff.as_mut_ptr(),
                b"-0123456789\0" as *const u8 as *const std::ffi::c_char,
            ) as usize] as std::ffi::c_int == '\0' as i32
            {
                printf(b".0\0" as *const u8 as *const std::ffi::c_char);
            }
        }
        3 => {
            printf(b"%lld\0" as *const u8 as *const std::ffi::c_char, (*o).value_.i);
        }
        4 | 20 => {
            PrintString(&mut (*((*o).value_.gc as *mut GCUnion)).ts);
        }
        _ => {
            printf(
                b"?%d\0" as *const u8 as *const std::ffi::c_char,
                (*o).tt_ as std::ffi::c_int & 0x3f as std::ffi::c_int,
            );
        }
    };
}
unsafe extern "C" fn PrintCode(mut f: *const Proto) {
    let mut code: *const Instruction = (*f).code;
    let mut pc: std::ffi::c_int = 0;
    let mut n: std::ffi::c_int = (*f).sizecode;
    pc = 0 as std::ffi::c_int;
    while pc < n {
        let mut i: Instruction = *code.offset(pc as isize);
        let mut o: OpCode = (i >> 0 as std::ffi::c_int
            & !(!(0 as std::ffi::c_int as Instruction) << 7 as std::ffi::c_int)
                << 0 as std::ffi::c_int) as OpCode;
        let mut a: std::ffi::c_int = (i >> 0 as std::ffi::c_int + 7 as std::ffi::c_int
            & !(!(0 as std::ffi::c_int as Instruction) << 8 as std::ffi::c_int)
                << 0 as std::ffi::c_int) as std::ffi::c_int;
        let mut b: std::ffi::c_int = (i
            >> 0 as std::ffi::c_int + 7 as std::ffi::c_int + 8 as std::ffi::c_int
                + 1 as std::ffi::c_int
            & !(!(0 as std::ffi::c_int as Instruction) << 8 as std::ffi::c_int)
                << 0 as std::ffi::c_int) as std::ffi::c_int;
        let mut c: std::ffi::c_int = (i
            >> 0 as std::ffi::c_int + 7 as std::ffi::c_int + 8 as std::ffi::c_int
                + 1 as std::ffi::c_int + 8 as std::ffi::c_int
            & !(!(0 as std::ffi::c_int as Instruction) << 8 as std::ffi::c_int)
                << 0 as std::ffi::c_int) as std::ffi::c_int;
        let mut ax: std::ffi::c_int = (i >> 0 as std::ffi::c_int + 7 as std::ffi::c_int
            & !(!(0 as std::ffi::c_int as Instruction)
                << 8 as std::ffi::c_int + 8 as std::ffi::c_int + 1 as std::ffi::c_int
                    + 8 as std::ffi::c_int) << 0 as std::ffi::c_int) as std::ffi::c_int;
        let mut bx: std::ffi::c_int = (i
            >> 0 as std::ffi::c_int + 7 as std::ffi::c_int + 8 as std::ffi::c_int
            & !(!(0 as std::ffi::c_int as Instruction)
                << 8 as std::ffi::c_int + 8 as std::ffi::c_int + 1 as std::ffi::c_int)
                << 0 as std::ffi::c_int) as std::ffi::c_int;
        let mut sb: std::ffi::c_int = (i
            >> 0 as std::ffi::c_int + 7 as std::ffi::c_int + 8 as std::ffi::c_int
                + 1 as std::ffi::c_int
            & !(!(0 as std::ffi::c_int as Instruction) << 8 as std::ffi::c_int)
                << 0 as std::ffi::c_int) as std::ffi::c_int
            - (((1 as std::ffi::c_int) << 8 as std::ffi::c_int) - 1 as std::ffi::c_int
                >> 1 as std::ffi::c_int);
        let mut sc: std::ffi::c_int = (i
            >> 0 as std::ffi::c_int + 7 as std::ffi::c_int + 8 as std::ffi::c_int
                + 1 as std::ffi::c_int + 8 as std::ffi::c_int
            & !(!(0 as std::ffi::c_int as Instruction) << 8 as std::ffi::c_int)
                << 0 as std::ffi::c_int) as std::ffi::c_int
            - (((1 as std::ffi::c_int) << 8 as std::ffi::c_int) - 1 as std::ffi::c_int
                >> 1 as std::ffi::c_int);
        let mut sbx: std::ffi::c_int = (i
            >> 0 as std::ffi::c_int + 7 as std::ffi::c_int + 8 as std::ffi::c_int
            & !(!(0 as std::ffi::c_int as Instruction)
                << 8 as std::ffi::c_int + 8 as std::ffi::c_int + 1 as std::ffi::c_int)
                << 0 as std::ffi::c_int) as std::ffi::c_int
            - (((1 as std::ffi::c_int)
                << 8 as std::ffi::c_int + 8 as std::ffi::c_int + 1 as std::ffi::c_int)
                - 1 as std::ffi::c_int >> 1 as std::ffi::c_int);
        let mut isk: std::ffi::c_int = (i
            >> 0 as std::ffi::c_int + 7 as std::ffi::c_int + 8 as std::ffi::c_int
            & !(!(0 as std::ffi::c_int as Instruction) << 1 as std::ffi::c_int)
                << 0 as std::ffi::c_int) as std::ffi::c_int;
        let mut line: std::ffi::c_int = luaG_getfuncline(f, pc);
        printf(
            b"\t%d\t\0" as *const u8 as *const std::ffi::c_char,
            pc + 1 as std::ffi::c_int,
        );
        if line > 0 as std::ffi::c_int {
            printf(b"[%d]\t\0" as *const u8 as *const std::ffi::c_char, line);
        } else {
            printf(b"[-]\t\0" as *const u8 as *const std::ffi::c_char);
        }
        printf(b"%-9s\t\0" as *const u8 as *const std::ffi::c_char, opnames[o as usize]);
        match o as std::ffi::c_uint {
            0 => {
                printf(b"%d %d\0" as *const u8 as *const std::ffi::c_char, a, b);
            }
            1 => {
                printf(b"%d %d\0" as *const u8 as *const std::ffi::c_char, a, sbx);
            }
            2 => {
                printf(b"%d %d\0" as *const u8 as *const std::ffi::c_char, a, sbx);
            }
            3 => {
                printf(b"%d %d\0" as *const u8 as *const std::ffi::c_char, a, bx);
                printf(b"\t; \0" as *const u8 as *const std::ffi::c_char);
                PrintConstant(f, bx);
            }
            4 => {
                printf(b"%d\0" as *const u8 as *const std::ffi::c_char, a);
                printf(b"\t; \0" as *const u8 as *const std::ffi::c_char);
                PrintConstant(
                    f,
                    (*code.offset((pc + 1 as std::ffi::c_int) as isize)
                        >> 0 as std::ffi::c_int + 7 as std::ffi::c_int
                        & !(!(0 as std::ffi::c_int as Instruction)
                            << 8 as std::ffi::c_int + 8 as std::ffi::c_int
                                + 1 as std::ffi::c_int + 8 as std::ffi::c_int)
                            << 0 as std::ffi::c_int) as std::ffi::c_int,
                );
            }
            5 => {
                printf(b"%d\0" as *const u8 as *const std::ffi::c_char, a);
            }
            6 => {
                printf(b"%d\0" as *const u8 as *const std::ffi::c_char, a);
            }
            7 => {
                printf(b"%d\0" as *const u8 as *const std::ffi::c_char, a);
            }
            8 => {
                printf(b"%d %d\0" as *const u8 as *const std::ffi::c_char, a, b);
                printf(
                    b"\t; %d out\0" as *const u8 as *const std::ffi::c_char,
                    b + 1 as std::ffi::c_int,
                );
            }
            9 => {
                printf(b"%d %d\0" as *const u8 as *const std::ffi::c_char, a, b);
                printf(
                    b"\t; %s\0" as *const u8 as *const std::ffi::c_char,
                    if !((*((*f).upvalues).offset(b as isize)).name).is_null() {
                        ((*(*((*f).upvalues).offset(b as isize)).name).contents)
                            .as_mut_ptr() as *const std::ffi::c_char
                    } else {
                        b"-\0" as *const u8 as *const std::ffi::c_char
                    },
                );
            }
            10 => {
                printf(b"%d %d\0" as *const u8 as *const std::ffi::c_char, a, b);
                printf(
                    b"\t; %s\0" as *const u8 as *const std::ffi::c_char,
                    if !((*((*f).upvalues).offset(b as isize)).name).is_null() {
                        ((*(*((*f).upvalues).offset(b as isize)).name).contents)
                            .as_mut_ptr() as *const std::ffi::c_char
                    } else {
                        b"-\0" as *const u8 as *const std::ffi::c_char
                    },
                );
            }
            11 => {
                printf(b"%d %d %d\0" as *const u8 as *const std::ffi::c_char, a, b, c);
                printf(
                    b"\t; %s\0" as *const u8 as *const std::ffi::c_char,
                    if !((*((*f).upvalues).offset(b as isize)).name).is_null() {
                        ((*(*((*f).upvalues).offset(b as isize)).name).contents)
                            .as_mut_ptr() as *const std::ffi::c_char
                    } else {
                        b"-\0" as *const u8 as *const std::ffi::c_char
                    },
                );
                printf(b" \0" as *const u8 as *const std::ffi::c_char);
                PrintConstant(f, c);
            }
            12 => {
                printf(b"%d %d %d\0" as *const u8 as *const std::ffi::c_char, a, b, c);
            }
            13 => {
                printf(b"%d %d %d\0" as *const u8 as *const std::ffi::c_char, a, b, c);
            }
            14 => {
                printf(b"%d %d %d\0" as *const u8 as *const std::ffi::c_char, a, b, c);
                printf(b"\t; \0" as *const u8 as *const std::ffi::c_char);
                PrintConstant(f, c);
            }
            15 => {
                printf(
                    b"%d %d %d%s\0" as *const u8 as *const std::ffi::c_char,
                    a,
                    b,
                    c,
                    if isk != 0 {
                        b"k\0" as *const u8 as *const std::ffi::c_char
                    } else {
                        b"\0" as *const u8 as *const std::ffi::c_char
                    },
                );
                printf(
                    b"\t; %s\0" as *const u8 as *const std::ffi::c_char,
                    if !((*((*f).upvalues).offset(a as isize)).name).is_null() {
                        ((*(*((*f).upvalues).offset(a as isize)).name).contents)
                            .as_mut_ptr() as *const std::ffi::c_char
                    } else {
                        b"-\0" as *const u8 as *const std::ffi::c_char
                    },
                );
                printf(b" \0" as *const u8 as *const std::ffi::c_char);
                PrintConstant(f, b);
                if isk != 0 {
                    printf(b" \0" as *const u8 as *const std::ffi::c_char);
                    PrintConstant(f, c);
                }
            }
            16 => {
                printf(
                    b"%d %d %d%s\0" as *const u8 as *const std::ffi::c_char,
                    a,
                    b,
                    c,
                    if isk != 0 {
                        b"k\0" as *const u8 as *const std::ffi::c_char
                    } else {
                        b"\0" as *const u8 as *const std::ffi::c_char
                    },
                );
                if isk != 0 {
                    printf(b"\t; \0" as *const u8 as *const std::ffi::c_char);
                    PrintConstant(f, c);
                }
            }
            17 => {
                printf(
                    b"%d %d %d%s\0" as *const u8 as *const std::ffi::c_char,
                    a,
                    b,
                    c,
                    if isk != 0 {
                        b"k\0" as *const u8 as *const std::ffi::c_char
                    } else {
                        b"\0" as *const u8 as *const std::ffi::c_char
                    },
                );
                if isk != 0 {
                    printf(b"\t; \0" as *const u8 as *const std::ffi::c_char);
                    PrintConstant(f, c);
                }
            }
            18 => {
                printf(
                    b"%d %d %d%s\0" as *const u8 as *const std::ffi::c_char,
                    a,
                    b,
                    c,
                    if isk != 0 {
                        b"k\0" as *const u8 as *const std::ffi::c_char
                    } else {
                        b"\0" as *const u8 as *const std::ffi::c_char
                    },
                );
                printf(b"\t; \0" as *const u8 as *const std::ffi::c_char);
                PrintConstant(f, b);
                if isk != 0 {
                    printf(b" \0" as *const u8 as *const std::ffi::c_char);
                    PrintConstant(f, c);
                }
            }
            19 => {
                printf(b"%d %d %d\0" as *const u8 as *const std::ffi::c_char, a, b, c);
                printf(
                    b"\t; %d\0" as *const u8 as *const std::ffi::c_char,
                    c
                        + (*code.offset((pc + 1 as std::ffi::c_int) as isize)
                            >> 0 as std::ffi::c_int + 7 as std::ffi::c_int
                            & !(!(0 as std::ffi::c_int as Instruction)
                                << 8 as std::ffi::c_int + 8 as std::ffi::c_int
                                    + 1 as std::ffi::c_int + 8 as std::ffi::c_int)
                                << 0 as std::ffi::c_int) as std::ffi::c_int
                            * (((1 as std::ffi::c_int) << 8 as std::ffi::c_int)
                                - 1 as std::ffi::c_int + 1 as std::ffi::c_int),
                );
            }
            20 => {
                printf(
                    b"%d %d %d%s\0" as *const u8 as *const std::ffi::c_char,
                    a,
                    b,
                    c,
                    if isk != 0 {
                        b"k\0" as *const u8 as *const std::ffi::c_char
                    } else {
                        b"\0" as *const u8 as *const std::ffi::c_char
                    },
                );
                if isk != 0 {
                    printf(b"\t; \0" as *const u8 as *const std::ffi::c_char);
                    PrintConstant(f, c);
                }
            }
            21 => {
                printf(b"%d %d %d\0" as *const u8 as *const std::ffi::c_char, a, b, sc);
            }
            22 => {
                printf(b"%d %d %d\0" as *const u8 as *const std::ffi::c_char, a, b, c);
                printf(b"\t; \0" as *const u8 as *const std::ffi::c_char);
                PrintConstant(f, c);
            }
            23 => {
                printf(b"%d %d %d\0" as *const u8 as *const std::ffi::c_char, a, b, c);
                printf(b"\t; \0" as *const u8 as *const std::ffi::c_char);
                PrintConstant(f, c);
            }
            24 => {
                printf(b"%d %d %d\0" as *const u8 as *const std::ffi::c_char, a, b, c);
                printf(b"\t; \0" as *const u8 as *const std::ffi::c_char);
                PrintConstant(f, c);
            }
            25 => {
                printf(b"%d %d %d\0" as *const u8 as *const std::ffi::c_char, a, b, c);
                printf(b"\t; \0" as *const u8 as *const std::ffi::c_char);
                PrintConstant(f, c);
            }
            26 => {
                printf(b"%d %d %d\0" as *const u8 as *const std::ffi::c_char, a, b, c);
                printf(b"\t; \0" as *const u8 as *const std::ffi::c_char);
                PrintConstant(f, c);
            }
            27 => {
                printf(b"%d %d %d\0" as *const u8 as *const std::ffi::c_char, a, b, c);
                printf(b"\t; \0" as *const u8 as *const std::ffi::c_char);
                PrintConstant(f, c);
            }
            28 => {
                printf(b"%d %d %d\0" as *const u8 as *const std::ffi::c_char, a, b, c);
                printf(b"\t; \0" as *const u8 as *const std::ffi::c_char);
                PrintConstant(f, c);
            }
            29 => {
                printf(b"%d %d %d\0" as *const u8 as *const std::ffi::c_char, a, b, c);
                printf(b"\t; \0" as *const u8 as *const std::ffi::c_char);
                PrintConstant(f, c);
            }
            30 => {
                printf(b"%d %d %d\0" as *const u8 as *const std::ffi::c_char, a, b, c);
                printf(b"\t; \0" as *const u8 as *const std::ffi::c_char);
                PrintConstant(f, c);
            }
            31 => {
                printf(b"%d %d %d\0" as *const u8 as *const std::ffi::c_char, a, b, c);
                printf(b"\t; \0" as *const u8 as *const std::ffi::c_char);
                PrintConstant(f, c);
            }
            32 => {
                printf(b"%d %d %d\0" as *const u8 as *const std::ffi::c_char, a, b, sc);
            }
            33 => {
                printf(b"%d %d %d\0" as *const u8 as *const std::ffi::c_char, a, b, sc);
            }
            34 => {
                printf(b"%d %d %d\0" as *const u8 as *const std::ffi::c_char, a, b, c);
            }
            35 => {
                printf(b"%d %d %d\0" as *const u8 as *const std::ffi::c_char, a, b, c);
            }
            36 => {
                printf(b"%d %d %d\0" as *const u8 as *const std::ffi::c_char, a, b, c);
            }
            37 => {
                printf(b"%d %d %d\0" as *const u8 as *const std::ffi::c_char, a, b, c);
            }
            38 => {
                printf(b"%d %d %d\0" as *const u8 as *const std::ffi::c_char, a, b, c);
            }
            39 => {
                printf(b"%d %d %d\0" as *const u8 as *const std::ffi::c_char, a, b, c);
            }
            40 => {
                printf(b"%d %d %d\0" as *const u8 as *const std::ffi::c_char, a, b, c);
            }
            41 => {
                printf(b"%d %d %d\0" as *const u8 as *const std::ffi::c_char, a, b, c);
            }
            42 => {
                printf(b"%d %d %d\0" as *const u8 as *const std::ffi::c_char, a, b, c);
            }
            43 => {
                printf(b"%d %d %d\0" as *const u8 as *const std::ffi::c_char, a, b, c);
            }
            44 => {
                printf(b"%d %d %d\0" as *const u8 as *const std::ffi::c_char, a, b, c);
            }
            45 => {
                printf(b"%d %d %d\0" as *const u8 as *const std::ffi::c_char, a, b, c);
            }
            46 => {
                printf(b"%d %d %d\0" as *const u8 as *const std::ffi::c_char, a, b, c);
                printf(
                    b"\t; %s\0" as *const u8 as *const std::ffi::c_char,
                    ((**tmname.offset(c as isize)).contents).as_mut_ptr(),
                );
            }
            47 => {
                printf(
                    b"%d %d %d %d\0" as *const u8 as *const std::ffi::c_char,
                    a,
                    sb,
                    c,
                    isk,
                );
                printf(
                    b"\t; %s\0" as *const u8 as *const std::ffi::c_char,
                    ((**tmname.offset(c as isize)).contents).as_mut_ptr(),
                );
                if isk != 0 {
                    printf(b" flip\0" as *const u8 as *const std::ffi::c_char);
                }
            }
            48 => {
                printf(
                    b"%d %d %d %d\0" as *const u8 as *const std::ffi::c_char,
                    a,
                    b,
                    c,
                    isk,
                );
                printf(
                    b"\t; %s \0" as *const u8 as *const std::ffi::c_char,
                    ((**tmname.offset(c as isize)).contents).as_mut_ptr(),
                );
                PrintConstant(f, b);
                if isk != 0 {
                    printf(b" flip\0" as *const u8 as *const std::ffi::c_char);
                }
            }
            49 => {
                printf(b"%d %d\0" as *const u8 as *const std::ffi::c_char, a, b);
            }
            50 => {
                printf(b"%d %d\0" as *const u8 as *const std::ffi::c_char, a, b);
            }
            51 => {
                printf(b"%d %d\0" as *const u8 as *const std::ffi::c_char, a, b);
            }
            52 => {
                printf(b"%d %d\0" as *const u8 as *const std::ffi::c_char, a, b);
            }
            53 => {
                printf(b"%d %d\0" as *const u8 as *const std::ffi::c_char, a, b);
            }
            54 => {
                printf(b"%d\0" as *const u8 as *const std::ffi::c_char, a);
            }
            55 => {
                printf(b"%d\0" as *const u8 as *const std::ffi::c_char, a);
            }
            56 => {
                printf(
                    b"%d\0" as *const u8 as *const std::ffi::c_char,
                    (i >> 0 as std::ffi::c_int + 7 as std::ffi::c_int
                        & !(!(0 as std::ffi::c_int as Instruction)
                            << 8 as std::ffi::c_int + 8 as std::ffi::c_int
                                + 1 as std::ffi::c_int + 8 as std::ffi::c_int)
                            << 0 as std::ffi::c_int) as std::ffi::c_int
                        - (((1 as std::ffi::c_int)
                            << 8 as std::ffi::c_int + 8 as std::ffi::c_int
                                + 1 as std::ffi::c_int + 8 as std::ffi::c_int)
                            - 1 as std::ffi::c_int >> 1 as std::ffi::c_int),
                );
                printf(
                    b"\t; to %d\0" as *const u8 as *const std::ffi::c_char,
                    (i >> 0 as std::ffi::c_int + 7 as std::ffi::c_int
                        & !(!(0 as std::ffi::c_int as Instruction)
                            << 8 as std::ffi::c_int + 8 as std::ffi::c_int
                                + 1 as std::ffi::c_int + 8 as std::ffi::c_int)
                            << 0 as std::ffi::c_int) as std::ffi::c_int
                        - (((1 as std::ffi::c_int)
                            << 8 as std::ffi::c_int + 8 as std::ffi::c_int
                                + 1 as std::ffi::c_int + 8 as std::ffi::c_int)
                            - 1 as std::ffi::c_int >> 1 as std::ffi::c_int) + pc
                        + 2 as std::ffi::c_int,
                );
            }
            57 => {
                printf(b"%d %d %d\0" as *const u8 as *const std::ffi::c_char, a, b, isk);
            }
            58 => {
                printf(b"%d %d %d\0" as *const u8 as *const std::ffi::c_char, a, b, isk);
            }
            59 => {
                printf(b"%d %d %d\0" as *const u8 as *const std::ffi::c_char, a, b, isk);
            }
            60 => {
                printf(b"%d %d %d\0" as *const u8 as *const std::ffi::c_char, a, b, isk);
                printf(b"\t; \0" as *const u8 as *const std::ffi::c_char);
                PrintConstant(f, b);
            }
            61 => {
                printf(
                    b"%d %d %d\0" as *const u8 as *const std::ffi::c_char,
                    a,
                    sb,
                    isk,
                );
            }
            62 => {
                printf(
                    b"%d %d %d\0" as *const u8 as *const std::ffi::c_char,
                    a,
                    sb,
                    isk,
                );
            }
            63 => {
                printf(
                    b"%d %d %d\0" as *const u8 as *const std::ffi::c_char,
                    a,
                    sb,
                    isk,
                );
            }
            64 => {
                printf(
                    b"%d %d %d\0" as *const u8 as *const std::ffi::c_char,
                    a,
                    sb,
                    isk,
                );
            }
            65 => {
                printf(
                    b"%d %d %d\0" as *const u8 as *const std::ffi::c_char,
                    a,
                    sb,
                    isk,
                );
            }
            66 => {
                printf(b"%d %d\0" as *const u8 as *const std::ffi::c_char, a, isk);
            }
            67 => {
                printf(b"%d %d %d\0" as *const u8 as *const std::ffi::c_char, a, b, isk);
            }
            68 => {
                printf(b"%d %d %d\0" as *const u8 as *const std::ffi::c_char, a, b, c);
                printf(b"\t; \0" as *const u8 as *const std::ffi::c_char);
                if b == 0 as std::ffi::c_int {
                    printf(b"all in \0" as *const u8 as *const std::ffi::c_char);
                } else {
                    printf(
                        b"%d in \0" as *const u8 as *const std::ffi::c_char,
                        b - 1 as std::ffi::c_int,
                    );
                }
                if c == 0 as std::ffi::c_int {
                    printf(b"all out\0" as *const u8 as *const std::ffi::c_char);
                } else {
                    printf(
                        b"%d out\0" as *const u8 as *const std::ffi::c_char,
                        c - 1 as std::ffi::c_int,
                    );
                }
            }
            69 => {
                printf(
                    b"%d %d %d%s\0" as *const u8 as *const std::ffi::c_char,
                    a,
                    b,
                    c,
                    if isk != 0 {
                        b"k\0" as *const u8 as *const std::ffi::c_char
                    } else {
                        b"\0" as *const u8 as *const std::ffi::c_char
                    },
                );
                printf(
                    b"\t; %d in\0" as *const u8 as *const std::ffi::c_char,
                    b - 1 as std::ffi::c_int,
                );
            }
            70 => {
                printf(
                    b"%d %d %d%s\0" as *const u8 as *const std::ffi::c_char,
                    a,
                    b,
                    c,
                    if isk != 0 {
                        b"k\0" as *const u8 as *const std::ffi::c_char
                    } else {
                        b"\0" as *const u8 as *const std::ffi::c_char
                    },
                );
                printf(b"\t; \0" as *const u8 as *const std::ffi::c_char);
                if b == 0 as std::ffi::c_int {
                    printf(b"all out\0" as *const u8 as *const std::ffi::c_char);
                } else {
                    printf(
                        b"%d out\0" as *const u8 as *const std::ffi::c_char,
                        b - 1 as std::ffi::c_int,
                    );
                }
            }
            72 => {
                printf(b"%d\0" as *const u8 as *const std::ffi::c_char, a);
            }
            73 => {
                printf(b"%d %d\0" as *const u8 as *const std::ffi::c_char, a, bx);
                printf(
                    b"\t; to %d\0" as *const u8 as *const std::ffi::c_char,
                    pc - bx + 2 as std::ffi::c_int,
                );
            }
            74 => {
                printf(b"%d %d\0" as *const u8 as *const std::ffi::c_char, a, bx);
                printf(
                    b"\t; exit to %d\0" as *const u8 as *const std::ffi::c_char,
                    pc + bx + 3 as std::ffi::c_int,
                );
            }
            75 => {
                printf(b"%d %d\0" as *const u8 as *const std::ffi::c_char, a, bx);
                printf(
                    b"\t; to %d\0" as *const u8 as *const std::ffi::c_char,
                    pc + bx + 2 as std::ffi::c_int,
                );
            }
            76 => {
                printf(b"%d %d\0" as *const u8 as *const std::ffi::c_char, a, c);
            }
            77 => {
                printf(b"%d %d\0" as *const u8 as *const std::ffi::c_char, a, bx);
                printf(
                    b"\t; to %d\0" as *const u8 as *const std::ffi::c_char,
                    pc - bx + 2 as std::ffi::c_int,
                );
            }
            78 => {
                printf(b"%d %d %d\0" as *const u8 as *const std::ffi::c_char, a, b, c);
                if isk != 0 {
                    printf(
                        b"\t; %d\0" as *const u8 as *const std::ffi::c_char,
                        c
                            + (*code.offset((pc + 1 as std::ffi::c_int) as isize)
                                >> 0 as std::ffi::c_int + 7 as std::ffi::c_int
                                & !(!(0 as std::ffi::c_int as Instruction)
                                    << 8 as std::ffi::c_int + 8 as std::ffi::c_int
                                        + 1 as std::ffi::c_int + 8 as std::ffi::c_int)
                                    << 0 as std::ffi::c_int) as std::ffi::c_int
                                * (((1 as std::ffi::c_int) << 8 as std::ffi::c_int)
                                    - 1 as std::ffi::c_int + 1 as std::ffi::c_int),
                    );
                }
            }
            79 => {
                printf(b"%d %d\0" as *const u8 as *const std::ffi::c_char, a, bx);
                printf(
                    b"\t; %p\0" as *const u8 as *const std::ffi::c_char,
                    *((*f).p).offset(bx as isize) as *const std::ffi::c_void,
                );
            }
            80 => {
                printf(b"%d %d\0" as *const u8 as *const std::ffi::c_char, a, c);
                printf(b"\t; \0" as *const u8 as *const std::ffi::c_char);
                if c == 0 as std::ffi::c_int {
                    printf(b"all out\0" as *const u8 as *const std::ffi::c_char);
                } else {
                    printf(
                        b"%d out\0" as *const u8 as *const std::ffi::c_char,
                        c - 1 as std::ffi::c_int,
                    );
                }
            }
            81 => {
                printf(b"%d\0" as *const u8 as *const std::ffi::c_char, a);
            }
            82 => {
                printf(b"%d\0" as *const u8 as *const std::ffi::c_char, ax);
            }
            71 | _ => {}
        }
        printf(b"\n\0" as *const u8 as *const std::ffi::c_char);
        pc += 1;
        pc;
    }
}
unsafe extern "C" fn PrintHeader(mut f: *const Proto) {
    let mut s: *const std::ffi::c_char = if !((*f).source).is_null() {
        ((*(*f).source).contents).as_mut_ptr() as *const std::ffi::c_char
    } else {
        b"=?\0" as *const u8 as *const std::ffi::c_char
    };
    if *s as std::ffi::c_int == '@' as i32 || *s as std::ffi::c_int == '=' as i32 {
        s = s.offset(1);
        s;
    } else if *s as std::ffi::c_int
        == (*::core::mem::transmute::<
            &[u8; 5],
            &[std::ffi::c_char; 5],
        >(b"\x1BLua\0"))[0 as std::ffi::c_int as usize] as std::ffi::c_int
    {
        s = b"(bstring)\0" as *const u8 as *const std::ffi::c_char;
    } else {
        s = b"(string)\0" as *const u8 as *const std::ffi::c_char;
    }
    printf(
        b"\n%s <%s:%d,%d> (%d instruction%s at %p)\n\0" as *const u8
            as *const std::ffi::c_char,
        if (*f).linedefined == 0 as std::ffi::c_int {
            b"main\0" as *const u8 as *const std::ffi::c_char
        } else {
            b"function\0" as *const u8 as *const std::ffi::c_char
        },
        s,
        (*f).linedefined,
        (*f).lastlinedefined,
        (*f).sizecode,
        if (*f).sizecode == 1 as std::ffi::c_int {
            b"\0" as *const u8 as *const std::ffi::c_char
        } else {
            b"s\0" as *const u8 as *const std::ffi::c_char
        },
        f as *const std::ffi::c_void,
    );
    printf(
        b"%d%s param%s, %d slot%s, %d upvalue%s, \0" as *const u8
            as *const std::ffi::c_char,
        (*f).numparams as std::ffi::c_int,
        if (*f).is_vararg as std::ffi::c_int != 0 {
            b"+\0" as *const u8 as *const std::ffi::c_char
        } else {
            b"\0" as *const u8 as *const std::ffi::c_char
        },
        if (*f).numparams as std::ffi::c_int == 1 as std::ffi::c_int {
            b"\0" as *const u8 as *const std::ffi::c_char
        } else {
            b"s\0" as *const u8 as *const std::ffi::c_char
        },
        (*f).maxstacksize as std::ffi::c_int,
        if (*f).maxstacksize as std::ffi::c_int == 1 as std::ffi::c_int {
            b"\0" as *const u8 as *const std::ffi::c_char
        } else {
            b"s\0" as *const u8 as *const std::ffi::c_char
        },
        (*f).sizeupvalues,
        if (*f).sizeupvalues == 1 as std::ffi::c_int {
            b"\0" as *const u8 as *const std::ffi::c_char
        } else {
            b"s\0" as *const u8 as *const std::ffi::c_char
        },
    );
    printf(
        b"%d local%s, %d constant%s, %d function%s\n\0" as *const u8
            as *const std::ffi::c_char,
        (*f).sizelocvars,
        if (*f).sizelocvars == 1 as std::ffi::c_int {
            b"\0" as *const u8 as *const std::ffi::c_char
        } else {
            b"s\0" as *const u8 as *const std::ffi::c_char
        },
        (*f).sizek,
        if (*f).sizek == 1 as std::ffi::c_int {
            b"\0" as *const u8 as *const std::ffi::c_char
        } else {
            b"s\0" as *const u8 as *const std::ffi::c_char
        },
        (*f).sizep,
        if (*f).sizep == 1 as std::ffi::c_int {
            b"\0" as *const u8 as *const std::ffi::c_char
        } else {
            b"s\0" as *const u8 as *const std::ffi::c_char
        },
    );
}
unsafe extern "C" fn PrintDebug(mut f: *const Proto) {
    let mut i: std::ffi::c_int = 0;
    let mut n: std::ffi::c_int = 0;
    n = (*f).sizek;
    printf(
        b"constants (%d) for %p:\n\0" as *const u8 as *const std::ffi::c_char,
        n,
        f as *const std::ffi::c_void,
    );
    i = 0 as std::ffi::c_int;
    while i < n {
        printf(b"\t%d\t\0" as *const u8 as *const std::ffi::c_char, i);
        PrintType(f, i);
        PrintConstant(f, i);
        printf(b"\n\0" as *const u8 as *const std::ffi::c_char);
        i += 1;
        i;
    }
    n = (*f).sizelocvars;
    printf(
        b"locals (%d) for %p:\n\0" as *const u8 as *const std::ffi::c_char,
        n,
        f as *const std::ffi::c_void,
    );
    i = 0 as std::ffi::c_int;
    while i < n {
        printf(
            b"\t%d\t%s\t%d\t%d\n\0" as *const u8 as *const std::ffi::c_char,
            i,
            ((*(*((*f).locvars).offset(i as isize)).varname).contents).as_mut_ptr(),
            (*((*f).locvars).offset(i as isize)).startpc + 1 as std::ffi::c_int,
            (*((*f).locvars).offset(i as isize)).endpc + 1 as std::ffi::c_int,
        );
        i += 1;
        i;
    }
    n = (*f).sizeupvalues;
    printf(
        b"upvalues (%d) for %p:\n\0" as *const u8 as *const std::ffi::c_char,
        n,
        f as *const std::ffi::c_void,
    );
    i = 0 as std::ffi::c_int;
    while i < n {
        printf(
            b"\t%d\t%s\t%d\t%d\n\0" as *const u8 as *const std::ffi::c_char,
            i,
            if !((*((*f).upvalues).offset(i as isize)).name).is_null() {
                ((*(*((*f).upvalues).offset(i as isize)).name).contents).as_mut_ptr()
                    as *const std::ffi::c_char
            } else {
                b"-\0" as *const u8 as *const std::ffi::c_char
            },
            (*((*f).upvalues).offset(i as isize)).instack as std::ffi::c_int,
            (*((*f).upvalues).offset(i as isize)).idx as std::ffi::c_int,
        );
        i += 1;
        i;
    }
}
unsafe extern "C" fn PrintFunction(mut f: *const Proto, mut full: std::ffi::c_int) {
    let mut i: std::ffi::c_int = 0;
    let mut n: std::ffi::c_int = (*f).sizep;
    PrintHeader(f);
    PrintCode(f);
    if full != 0 {
        PrintDebug(f);
    }
    i = 0 as std::ffi::c_int;
    while i < n {
        PrintFunction(*((*f).p).offset(i as isize), full);
        i += 1;
        i;
    }
}
pub fn main() {
    let mut args: Vec::<*mut std::ffi::c_char> = Vec::new();
    for arg in ::std::env::args() {
        args.push(
            (::std::ffi::CString::new(arg))
                .expect("Failed to convert argument into CString.")
                .into_raw(),
        );
    }
    args.push(::core::ptr::null_mut());
    unsafe {
        ::std::process::exit(
            main_0(
                (args.len() - 1) as std::ffi::c_int,
                args.as_mut_ptr() as *mut *mut std::ffi::c_char,
            ) as i32,
        )
    }
}
