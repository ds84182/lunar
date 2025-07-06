use crate::*;

pub type OpCode = u32;

pub const OP_MOVE: OpCode = 0;
pub const OP_LOADI: OpCode = 1;
pub const OP_LOADF: OpCode = 2;
pub const OP_LOADK: OpCode = 3;
pub const OP_LOADKX: OpCode = 4;
pub const OP_LOADFALSE: OpCode = 5;
pub const OP_LFALSESKIP: OpCode = 6;
pub const OP_LOADTRUE: OpCode = 7;
pub const OP_LOADNIL: OpCode = 8;
pub const OP_GETUPVAL: OpCode = 9;
pub const OP_SETUPVAL: OpCode = 10;
pub const OP_GETTABUP: OpCode = 11;
pub const OP_GETTABLE: OpCode = 12;
pub const OP_GETI: OpCode = 13;
pub const OP_GETFIELD: OpCode = 14;
pub const OP_SETTABUP: OpCode = 15;
pub const OP_SETTABLE: OpCode = 16;
pub const OP_SETI: OpCode = 17;
pub const OP_SETFIELD: OpCode = 18;
pub const OP_NEWTABLE: OpCode = 19;
pub const OP_SELF: OpCode = 20;
pub const OP_ADDI: OpCode = 21;
pub const OP_ADDK: OpCode = 22;
pub const OP_SUBK: OpCode = 23;
pub const OP_MULK: OpCode = 24;
pub const OP_MODK: OpCode = 25;
pub const OP_POWK: OpCode = 26;
pub const OP_DIVK: OpCode = 27;
pub const OP_IDIVK: OpCode = 28;
pub const OP_BANDK: OpCode = 29;
pub const OP_BORK: OpCode = 30;
pub const OP_BXORK: OpCode = 31;
pub const OP_SHRI: OpCode = 32;
pub const OP_SHLI: OpCode = 33;
pub const OP_ADD: OpCode = 34;
pub const OP_SUB: OpCode = 35;
pub const OP_MUL: OpCode = 36;
pub const OP_MOD: OpCode = 37;
pub const OP_POW: OpCode = 38;
pub const OP_DIV: OpCode = 39;
pub const OP_IDIV: OpCode = 40;
pub const OP_BAND: OpCode = 41;
pub const OP_BOR: OpCode = 42;
pub const OP_BXOR: OpCode = 43;
pub const OP_SHL: OpCode = 44;
pub const OP_SHR: OpCode = 45;
pub const OP_MMBIN: OpCode = 46;
pub const OP_MMBINI: OpCode = 47;
pub const OP_MMBINK: OpCode = 48;
pub const OP_UNM: OpCode = 49;
pub const OP_BNOT: OpCode = 50;
pub const OP_NOT: OpCode = 51;
pub const OP_LEN: OpCode = 52;
pub const OP_CONCAT: OpCode = 53;
pub const OP_CLOSE: OpCode = 54;
pub const OP_TBC: OpCode = 55;
pub const OP_JMP: OpCode = 56;
pub const OP_EQ: OpCode = 57;
pub const OP_LT: OpCode = 58;
pub const OP_LE: OpCode = 59;
pub const OP_EQK: OpCode = 60;
pub const OP_EQI: OpCode = 61;
pub const OP_LTI: OpCode = 62;
pub const OP_LEI: OpCode = 63;
pub const OP_GTI: OpCode = 64;
pub const OP_GEI: OpCode = 65;
pub const OP_TEST: OpCode = 66;
pub const OP_TESTSET: OpCode = 67;
pub const OP_CALL: OpCode = 68;
pub const OP_TAILCALL: OpCode = 69;
pub const OP_RETURN: OpCode = 70;
pub const OP_RETURN0: OpCode = 71;
pub const OP_RETURN1: OpCode = 72;
pub const OP_FORLOOP: OpCode = 73;
pub const OP_FORPREP: OpCode = 74;
pub const OP_TFORPREP: OpCode = 75;
pub const OP_TFORCALL: OpCode = 76;
pub const OP_TFORLOOP: OpCode = 77;
pub const OP_SETLIST: OpCode = 78;
pub const OP_CLOSURE: OpCode = 79;
pub const OP_VARARG: OpCode = 80;
pub const OP_VARARGPREP: OpCode = 81;
pub const OP_EXTRAARG: OpCode = 82;

pub const iABC: OpMode = 0;
pub const iABx: OpMode = 1;
pub const iAsBx: OpMode = 2;
pub const iAx: OpMode = 3;
pub const isJ: OpMode = 4;

const fn opmode(mm: u8, ot: u8, it: u8, t: u8, a: u8, m: u8) -> u8 {
    ((mm) << 7) | ((ot) << 6) | ((it) << 5) | ((t) << 4) | ((a) << 3) | (m)
}

const SIZE_C: u32 = 8;
const SIZE_B: u32 = 8;
const SIZE_Bx: u32 = SIZE_C + SIZE_B + 1;
const SIZE_A: u32 = 8;
const SIZE_Ax: u32 = SIZE_Bx + SIZE_A;
const SIZE_sJ: u32 = SIZE_Bx + SIZE_A;

const SIZE_OP: u32 = 7;

const POS_OP: u32 = 0;

const POS_A: u32 = POS_OP + SIZE_OP;
const POS_k: u32 = POS_A + SIZE_A;
const POS_B: u32 = POS_k + 1;
const POS_C: u32 = POS_B + SIZE_B;

const POS_Bx: u32 = POS_k;
const POS_Ax: u32 = POS_A;
const POS_sJ: u32 = POS_A;

const OFFSET_sBx: i32 = (((1 << SIZE_Bx) - 1) >> 1) as i32;
const OFFSET_sJ: i32 = (((1 << SIZE_sJ) - 1) >> 1) as i32;
const OFFSET_sC: i32 = (((1 << SIZE_C) - 1) >> 1) as i32;

#[inline]
const fn get<const S: u32, const P: u32>(i: u32) -> u32 {
    (i >> P) & const { !(u32::MAX << S) }
}

#[inline]
const fn set<const S: u32, const P: u32>(i: &mut u32, o: u32) {
    let mask = const { (!(u32::MAX << S)) << P };
    *i = ((o << P) & mask) | (*i & !mask);
}

#[inline]
pub(crate) fn get_opcode(i: u32) -> u32 {
    get::<SIZE_OP, POS_OP>(i)
}

#[inline]
pub(crate) fn getarg_a(i: u32) -> u32 {
    get::<SIZE_A, POS_A>(i)
}

#[inline]
pub(crate) fn getarg_b(i: u32) -> u32 {
    get::<SIZE_B, POS_B>(i)
}

#[inline]
pub(crate) fn getarg_sb(i: u32) -> i32 {
    (get::<SIZE_B, POS_B>(i) as i32) - OFFSET_sC
}

#[inline]
pub(crate) fn getarg_c(i: u32) -> u32 {
    get::<SIZE_C, POS_C>(i)
}

#[inline]
pub(crate) fn getarg_sc(i: u32) -> i32 {
    (get::<SIZE_C, POS_C>(i) as i32) - OFFSET_sC
}

#[inline]
pub(crate) fn getarg_k(i: u32) -> u32 {
    get::<1, POS_k>(i)
}

#[inline]
pub(crate) fn getarg_bx(i: u32) -> u32 {
    get::<SIZE_Bx, POS_Bx>(i)
}

#[inline]
pub(crate) fn getarg_ax(i: u32) -> u32 {
    get::<SIZE_Ax, POS_Ax>(i)
}

#[inline]
pub(crate) fn getarg_sbx(i: u32) -> i32 {
    get::<SIZE_Bx, POS_Bx>(i) as i32 - OFFSET_sBx
}

#[inline]
pub(crate) fn getarg_sj(i: u32) -> i32 {
    get::<SIZE_sJ, POS_sJ>(i) as i32 - OFFSET_sJ
}

pub const luaP_opmodes: &'static [lu_byte; 83] = &[
    /*       MM OT IT T  A  mode		   opcode  */
    opmode(0, 0, 0, 0, 1, iABC), /* OP_MOVE */
    opmode(0, 0, 0, 0, 1, iAsBx), /* OP_LOADI */
    opmode(0, 0, 0, 0, 1, iAsBx), /* OP_LOADF */
    opmode(0, 0, 0, 0, 1, iABx), /* OP_LOADK */
    opmode(0, 0, 0, 0, 1, iABx), /* OP_LOADKX */
    opmode(0, 0, 0, 0, 1, iABC), /* OP_LOADFALSE */
    opmode(0, 0, 0, 0, 1, iABC), /* OP_LFALSESKIP */
    opmode(0, 0, 0, 0, 1, iABC), /* OP_LOADTRUE */
    opmode(0, 0, 0, 0, 1, iABC), /* OP_LOADNIL */
    opmode(0, 0, 0, 0, 1, iABC), /* OP_GETUPVAL */
    opmode(0, 0, 0, 0, 0, iABC), /* OP_SETUPVAL */
    opmode(0, 0, 0, 0, 1, iABC), /* OP_GETTABUP */
    opmode(0, 0, 0, 0, 1, iABC), /* OP_GETTABLE */
    opmode(0, 0, 0, 0, 1, iABC), /* OP_GETI */
    opmode(0, 0, 0, 0, 1, iABC), /* OP_GETFIELD */
    opmode(0, 0, 0, 0, 0, iABC), /* OP_SETTABUP */
    opmode(0, 0, 0, 0, 0, iABC), /* OP_SETTABLE */
    opmode(0, 0, 0, 0, 0, iABC), /* OP_SETI */
    opmode(0, 0, 0, 0, 0, iABC), /* OP_SETFIELD */
    opmode(0, 0, 0, 0, 1, iABC), /* OP_NEWTABLE */
    opmode(0, 0, 0, 0, 1, iABC), /* OP_SELF */
    opmode(0, 0, 0, 0, 1, iABC), /* OP_ADDI */
    opmode(0, 0, 0, 0, 1, iABC), /* OP_ADDK */
    opmode(0, 0, 0, 0, 1, iABC), /* OP_SUBK */
    opmode(0, 0, 0, 0, 1, iABC), /* OP_MULK */
    opmode(0, 0, 0, 0, 1, iABC), /* OP_MODK */
    opmode(0, 0, 0, 0, 1, iABC), /* OP_POWK */
    opmode(0, 0, 0, 0, 1, iABC), /* OP_DIVK */
    opmode(0, 0, 0, 0, 1, iABC), /* OP_IDIVK */
    opmode(0, 0, 0, 0, 1, iABC), /* OP_BANDK */
    opmode(0, 0, 0, 0, 1, iABC), /* OP_BORK */
    opmode(0, 0, 0, 0, 1, iABC), /* OP_BXORK */
    opmode(0, 0, 0, 0, 1, iABC), /* OP_SHRI */
    opmode(0, 0, 0, 0, 1, iABC), /* OP_SHLI */
    opmode(0, 0, 0, 0, 1, iABC), /* OP_ADD */
    opmode(0, 0, 0, 0, 1, iABC), /* OP_SUB */
    opmode(0, 0, 0, 0, 1, iABC), /* OP_MUL */
    opmode(0, 0, 0, 0, 1, iABC), /* OP_MOD */
    opmode(0, 0, 0, 0, 1, iABC), /* OP_POW */
    opmode(0, 0, 0, 0, 1, iABC), /* OP_DIV */
    opmode(0, 0, 0, 0, 1, iABC), /* OP_IDIV */
    opmode(0, 0, 0, 0, 1, iABC), /* OP_BAND */
    opmode(0, 0, 0, 0, 1, iABC), /* OP_BOR */
    opmode(0, 0, 0, 0, 1, iABC), /* OP_BXOR */
    opmode(0, 0, 0, 0, 1, iABC), /* OP_SHL */
    opmode(0, 0, 0, 0, 1, iABC), /* OP_SHR */
    opmode(1, 0, 0, 0, 0, iABC), /* OP_MMBIN */
    opmode(1, 0, 0, 0, 0, iABC), /* OP_MMBINI*/
    opmode(1, 0, 0, 0, 0, iABC), /* OP_MMBINK*/
    opmode(0, 0, 0, 0, 1, iABC), /* OP_UNM */
    opmode(0, 0, 0, 0, 1, iABC), /* OP_BNOT */
    opmode(0, 0, 0, 0, 1, iABC), /* OP_NOT */
    opmode(0, 0, 0, 0, 1, iABC), /* OP_LEN */
    opmode(0, 0, 0, 0, 1, iABC), /* OP_CONCAT */
    opmode(0, 0, 0, 0, 0, iABC), /* OP_CLOSE */
    opmode(0, 0, 0, 0, 0, iABC), /* OP_TBC */
    opmode(0, 0, 0, 0, 0, isJ),  /* OP_JMP */
    opmode(0, 0, 0, 1, 0, iABC), /* OP_EQ */
    opmode(0, 0, 0, 1, 0, iABC), /* OP_LT */
    opmode(0, 0, 0, 1, 0, iABC), /* OP_LE */
    opmode(0, 0, 0, 1, 0, iABC), /* OP_EQK */
    opmode(0, 0, 0, 1, 0, iABC), /* OP_EQI */
    opmode(0, 0, 0, 1, 0, iABC), /* OP_LTI */
    opmode(0, 0, 0, 1, 0, iABC), /* OP_LEI */
    opmode(0, 0, 0, 1, 0, iABC), /* OP_GTI */
    opmode(0, 0, 0, 1, 0, iABC), /* OP_GEI */
    opmode(0, 0, 0, 1, 0, iABC), /* OP_TEST */
    opmode(0, 0, 0, 1, 1, iABC), /* OP_TESTSET */
    opmode(0, 1, 1, 0, 1, iABC), /* OP_CALL */
    opmode(0, 1, 1, 0, 1, iABC), /* OP_TAILCALL */
    opmode(0, 0, 1, 0, 0, iABC), /* OP_RETURN */
    opmode(0, 0, 0, 0, 0, iABC), /* OP_RETURN0 */
    opmode(0, 0, 0, 0, 0, iABC), /* OP_RETURN1 */
    opmode(0, 0, 0, 0, 1, iABx), /* OP_FORLOOP */
    opmode(0, 0, 0, 0, 1, iABx), /* OP_FORPREP */
    opmode(0, 0, 0, 0, 0, iABx), /* OP_TFORPREP */
    opmode(0, 0, 0, 0, 0, iABC), /* OP_TFORCALL */
    opmode(0, 0, 0, 0, 1, iABx), /* OP_TFORLOOP */
    opmode(0, 0, 1, 0, 0, iABC), /* OP_SETLIST */
    opmode(0, 0, 0, 0, 1, iABx), /* OP_CLOSURE */
    opmode(0, 1, 0, 0, 1, iABC), /* OP_VARARG */
    opmode(0, 0, 1, 0, 1, iABC), /* OP_VARARGPREP */
    opmode(0, 0, 0, 0, 0, iAx),  /* OP_EXTRAARG */
];
