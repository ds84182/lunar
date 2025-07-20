use crate::*;

pub type OpCode = u32;

/// `R[A] := R[B]`
pub const OP_MOVE: OpCode = 0;
/// `R[A] := sBx`
pub const OP_LOADI: OpCode = 1;
/// `R[A] := (lua_Number)sBx`
pub const OP_LOADF: OpCode = 2;
/// `R[A] := K[Bx]`
pub const OP_LOADK: OpCode = 3;
/// `R[A] := K[extra arg]`
///
/// Next instruction is always [`OP_EXTRAARG`].
pub const OP_LOADKX: OpCode = 4;
/// `R[A] := false`
pub const OP_LOADFALSE: OpCode = 5;
/// `R[A] := false; pc++`
///
/// Used to convert a condition to a boolean value, in a code equivalent to (not cond ? false : true).
/// (It produces false and skips the next instruction producing true.)
pub const OP_LFALSESKIP: OpCode = 6;
/// `R[A] := true`
pub const OP_LOADTRUE: OpCode = 7;
/// `R[A], R[A+1], ..., R[A+B] := nil`
pub const OP_LOADNIL: OpCode = 8;
/// `R[A] := UpValue[B]`
pub const OP_GETUPVAL: OpCode = 9;
/// `UpValue[B] := R[A]`
pub const OP_SETUPVAL: OpCode = 10;
/// `R[A] := UpValue[B][K[C]:shortstring]`
pub const OP_GETTABUP: OpCode = 11;
/// `R[A] := R[B][R[C]]`
pub const OP_GETTABLE: OpCode = 12;
/// `R[A] := R[B][C]`
pub const OP_GETI: OpCode = 13;
/// `R[A] := R[B][K[C]:shortstring]`
pub const OP_GETFIELD: OpCode = 14;
/// `UpValue[A][K[B]:shortstring] := RK(C)`
pub const OP_SETTABUP: OpCode = 15;
/// `R[A][R[B]] := RK(C)`
pub const OP_SETTABLE: OpCode = 16;
/// `R[A][B] := RK(C)`
pub const OP_SETI: OpCode = 17;
/// `R[A][K[B]:shortstring] := RK(C)`
pub const OP_SETFIELD: OpCode = 18;
/// `R[A] := {}`
pub const OP_NEWTABLE: OpCode = 19;
/// `R[A+1] := R[B]; R[A] := R[B][RK(C):string]`
pub const OP_SELF: OpCode = 20;
/// `R[A] := R[B] + sC`
pub const OP_ADDI: OpCode = 21;
/// `R[A] := R[B] + K[C]:number`
pub const OP_ADDK: OpCode = 22;
/// `R[A] := R[B] - K[C]:number`
pub const OP_SUBK: OpCode = 23;
/// `R[A] := R[B] * K[C]:number`
pub const OP_MULK: OpCode = 24;
/// `R[A] := R[B] % K[C]:number`
pub const OP_MODK: OpCode = 25;
/// `R[A] := R[B] ^ K[C]:number`
pub const OP_POWK: OpCode = 26;
/// `R[A] := R[B] / K[C]:number`
pub const OP_DIVK: OpCode = 27;
/// `R[A] := R[B] // K[C]:number`
pub const OP_IDIVK: OpCode = 28;
/// `R[A] := R[B] & K[C]:integer`
pub const OP_BANDK: OpCode = 29;
/// `R[A] := R[B] | K[C]:integer`
pub const OP_BORK: OpCode = 30;
/// `R[A] := R[B] ~ K[C]:integer`
pub const OP_BXORK: OpCode = 31;
/// `R[A] := R[B] >> sC`
pub const OP_SHRI: OpCode = 32;
/// `R[A] := sC << R[B]`
pub const OP_SHLI: OpCode = 33;
/// `R[A] := R[B] + R[C]`
pub const OP_ADD: OpCode = 34;
/// `R[A] := R[B] - R[C]`
pub const OP_SUB: OpCode = 35;
/// `R[A] := R[B] * R[C]`
pub const OP_MUL: OpCode = 36;
/// `R[A] := R[B] % R[C]`
pub const OP_MOD: OpCode = 37;
/// `R[A] := R[B] ^ R[C]`
pub const OP_POW: OpCode = 38;
/// `R[A] := R[B] / R[C]`
pub const OP_DIV: OpCode = 39;
/// `R[A] := R[B] // R[C]`
pub const OP_IDIV: OpCode = 40;
/// `R[A] := R[B] & R[C]`
pub const OP_BAND: OpCode = 41;
/// `R[A] := R[B] | R[C]`
pub const OP_BOR: OpCode = 42;
/// `R[A] := R[B] ~ R[C]`
pub const OP_BXOR: OpCode = 43;
/// `R[A] := R[B] << R[C]`
pub const OP_SHL: OpCode = 44;
/// `R[A] := R[B] >> R[C]`
pub const OP_SHR: OpCode = 45;
/// call C metamethod over R[A] and R[B]
///
/// Follows each arithmetic and bitwise opcode. If the operation succeeds, it skips this next
/// opcode. Otherwise, this opcode calls the corresponding metamethod.
pub const OP_MMBIN: OpCode = 46;
/// call C metamethod over R[A] and sB
///
/// Follows each arithmetic and bitwise opcode. If the operation succeeds, it skips this next
/// opcode. Otherwise, this opcode calls the corresponding metamethod.
pub const OP_MMBINI: OpCode = 47;
/// call C metamethod over R[A] and K[B]
///
/// Follows each arithmetic and bitwise opcode. If the operation succeeds, it skips this next
/// opcode. Otherwise, this opcode calls the corresponding metamethod.
pub const OP_MMBINK: OpCode = 48;
/// `R[A] := -R[B]`
pub const OP_UNM: OpCode = 49;
/// `R[A] := ~R[B]`
pub const OP_BNOT: OpCode = 50;
/// `R[A] := not R[B]`
pub const OP_NOT: OpCode = 51;
/// `R[A] := #R[B] (length operator)`
pub const OP_LEN: OpCode = 52;
/// `R[A] := R[A].. ... ..R[A + B - 1]`
pub const OP_CONCAT: OpCode = 53;
/// close all upvalues `>= R[A]`
pub const OP_CLOSE: OpCode = 54;
/// mark variable A "to be closed"
pub const OP_TBC: OpCode = 55;
/// `pc += sJ`
pub const OP_JMP: OpCode = 56;
/// `if ((R[A] == R[B]) ~= k) then pc++`
///
/// Assumes that the next instruction is a jump.
pub const OP_EQ: OpCode = 57;
/// `if ((R[A] <  R[B]) ~= k) then pc++`
///
/// Assumes that the next instruction is a jump.
pub const OP_LT: OpCode = 58;
/// `if ((R[A] <= R[B]) ~= k) then pc++`
///
/// Assumes that the next instruction is a jump.
pub const OP_LE: OpCode = 59;
/// `if ((R[A] == K[B]) ~= k) then pc++`
///
/// Assumes that the next instruction is a jump.
pub const OP_EQK: OpCode = 60;
/// `if ((R[A] == sB) ~= k) then pc++`
///
/// Assumes that the next instruction is a jump.
///
/// `C` signals whether the original operand was a float.
/// It must be corrected in case of metamethods.
pub const OP_EQI: OpCode = 61;
/// `if ((R[A] < sB) ~= k) then pc++`
///
/// Assumes that the next instruction is a jump.
///
/// `C` signals whether the original operand was a float.
/// It must be corrected in case of metamethods.
pub const OP_LTI: OpCode = 62;
/// `if ((R[A] <= sB) ~= k) then pc++`
///
/// Assumes that the next instruction is a jump.
///
/// `C` signals whether the original operand was a float.
/// It must be corrected in case of metamethods.
pub const OP_LEI: OpCode = 63;
/// `if ((R[A] > sB) ~= k) then pc++`
///
/// Assumes that the next instruction is a jump.
///
/// `C` signals whether the original operand was a float.
/// It must be corrected in case of metamethods.
pub const OP_GTI: OpCode = 64;
/// `if ((R[A] >= sB) ~= k) then pc++`
///
/// Assumes that the next instruction is a jump.
///
/// `C` signals whether the original operand was a float.
/// It must be corrected in case of metamethods.
pub const OP_GEI: OpCode = 65;
/// `if (not R[A] == k) then pc++`
///
/// Assumes that the next instruction is a jump.
pub const OP_TEST: OpCode = 66;
/// `if (not R[B] == k) then pc++ else R[A] := R[B]`
///
/// Assumes that the next instruction is a jump.
///
/// Used in short-circuit expressions that need both to jump and produce a value, such as
/// (a = b or c).
pub const OP_TESTSET: OpCode = 67;
/// `R[A], ... ,R[A+C-2] := R[A](R[A+1], ... ,R[A+B-1])`
///
/// If (B == 0) then B = top - A. If (C == 0), then `top` is set to last_result+1, so next open
/// instruction (OP_CALL, OP_RETURN*, OP_SETLIST) may use `top`.
pub const OP_CALL: OpCode = 68;
/// `return R[A](R[A+1], ... ,R[A+B-1])`
///
/// `k` specifies that the function builds upvalues, which may need to be closed.
/// `C > 0` means the function is vararg, so that its `func` must be corrected before returning;
/// in this case, `C - 1` is its number of fixed parameters.
pub const OP_TAILCALL: OpCode = 69;
/// `return R[A], ... ,R[A+B-2]`
///
/// If (B == 0) then return up to `top`.
///
/// `k` specifies that the function builds upvalues, which may need to be closed.
/// `C > 0` means the function is vararg, so that its `func` must be corrected before returning;
/// in this case, `C - 1` is its number of fixed parameters.
pub const OP_RETURN: OpCode = 70;
/// `return`
pub const OP_RETURN0: OpCode = 71;
/// `return R[A]`
pub const OP_RETURN1: OpCode = 72;
/// update counters; if loop continues then pc-=Bx;
pub const OP_FORLOOP: OpCode = 73;
/// check values and prepare counters; if not to run then pc+=Bx+1;
pub const OP_FORPREP: OpCode = 74;
/// `create upvalue for R[A + 3]; pc+=Bx`
pub const OP_TFORPREP: OpCode = 75;
/// `R[A+4], ... ,R[A+3+C] := R[A](R[A+1], R[A+2]);`
pub const OP_TFORCALL: OpCode = 76;
/// `if R[A+2] ~= nil then { R[A]=R[A+2]; pc -= Bx }`
pub const OP_TFORLOOP: OpCode = 77;
/// `R[A][C+i] := R[A+i], 1 <= i <= B`
pub const OP_SETLIST: OpCode = 78;
/// `R[A] := closure(KPROTO[Bx])`
pub const OP_CLOSURE: OpCode = 79;
/// `R[A], R[A+1], ..., R[A+C-2] = vararg`
///
/// If (C == 0) then use actual number of varargs and set `top` (like in OP_CALL with C == 0).
pub const OP_VARARG: OpCode = 80;
/// adjust vararg parameters
pub const OP_VARARGPREP: OpCode = 81;
/// extra (larger) argument for previous opcode.
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

pub(super) const MAXARG_A: u32 = (1 << SIZE_A) - 1;
pub(super) const MAXARG_Ax: u32 = (1 << SIZE_Ax) - 1;
pub(super) const MAXARG_B: u32 = (1 << SIZE_B) - 1;
pub(super) const MAXARG_Bx: u32 = (1 << SIZE_Bx) - 1;
pub(super) const MAXARG_C: u32 = (1 << SIZE_C) - 1;

#[inline]
const fn get<const S: u32, const P: u32>(i: u32) -> u32 {
    (i >> P) & const { !(u32::MAX << S) }
}

#[inline]
const fn set<const S: u32, const P: u32>(i: &mut u32, o: u32) {
    let mask = const { (!(u32::MAX << S)) << P };
    *i = ((o << P) & mask) | (*i & !mask);
}

#[inline(always)]
pub(crate) fn get_opcode(i: u32) -> u32 {
    get::<SIZE_OP, POS_OP>(i)
}

#[inline(always)]
pub(crate) fn getarg_a(i: u32) -> u32 {
    get::<SIZE_A, POS_A>(i)
}

#[inline(always)]
pub(crate) fn getarg_b(i: u32) -> u32 {
    get::<SIZE_B, POS_B>(i)
}

#[inline(always)]
pub(crate) fn getarg_sb(i: u32) -> i32 {
    (get::<SIZE_B, POS_B>(i) as i32) - OFFSET_sC
}

#[inline(always)]
pub(crate) fn getarg_c(i: u32) -> u32 {
    get::<SIZE_C, POS_C>(i)
}

#[inline(always)]
pub(crate) fn getarg_sc(i: u32) -> i32 {
    (get::<SIZE_C, POS_C>(i) as i32) - OFFSET_sC
}

#[inline(always)]
pub(crate) fn getarg_k(i: u32) -> bool {
    get::<1, POS_k>(i) != 0
}

#[inline(always)]
pub(crate) fn getarg_bx(i: u32) -> u32 {
    get::<SIZE_Bx, POS_Bx>(i)
}

#[inline(always)]
pub(crate) fn getarg_ax(i: u32) -> u32 {
    get::<SIZE_Ax, POS_Ax>(i)
}

#[inline(always)]
pub(crate) fn getarg_sbx(i: u32) -> i32 {
    get::<SIZE_Bx, POS_Bx>(i) as i32 - OFFSET_sBx
}

#[inline(always)]
pub(crate) fn getarg_sj(i: u32) -> i32 {
    get::<SIZE_sJ, POS_sJ>(i) as i32 - OFFSET_sJ
}

/// Wrapper for an instruction.
#[derive(Copy, Clone)]
#[repr(transparent)]
pub(crate) struct Instr(pub(crate) Instruction);

impl std::fmt::Debug for Instr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let op = self.opcode();
        if let Some(op_name) = luaP_opnames.get(op as usize) {
            write!(f, "{op_name} ")?;
        } else {
            write!(f, "{op} ")?;
        }

        match self.props().mode() {
            iABC => {
                write!(f, "{:?}", self.abc())
            }
            iABx => {
                write!(f, "{:?}", self.a_bx())
            }
            iAsBx => {
                write!(f, "{:?}", self.a_sbx())
            }
            iAx => {
                write!(f, "{:?}", self.ax())
            }
            isJ => {
                write!(f, "{:?}", self.sj())
            }
            _ => Ok(()),
        }
    }
}

impl Instr {
    #[inline]
    pub(crate) fn opcode(self) -> OpCode {
        get_opcode(self.0)
    }

    #[inline]
    pub(crate) fn props(self) -> OpProps {
        OpProps(
            luaP_opmodes
                .get(self.opcode() as usize)
                .copied()
                .unwrap_or(0),
        )
    }

    #[inline]
    pub(crate) fn abc(self) -> (u32, u32, u32) {
        (getarg_a(self.0), getarg_b(self.0), getarg_c(self.0))
    }

    #[inline]
    pub(crate) fn a_sbx(self) -> (u32, i32) {
        (getarg_a(self.0), getarg_sbx(self.0))
    }

    #[inline]
    pub(crate) fn a_bx(self) -> (u32, u32) {
        (getarg_a(self.0), getarg_bx(self.0))
    }

    #[inline]
    pub(crate) fn ax(self) -> u32 {
        getarg_ax(self.0)
    }

    #[inline]
    pub(crate) fn sj(self) -> i32 {
        getarg_sj(self.0)
    }
}

/// Wrapper for [`OpMode`].
#[derive(Copy, Clone)]
#[repr(transparent)]
pub(crate) struct OpProps(pub(crate) OpMode);

impl OpProps {
    pub(crate) fn mode(self) -> OpMode {
        self.0 & 0b111
    }

    pub(crate) fn sets_reg_a(self) -> bool {
        self.0 & (1 << 3) != 0
    }

    pub(crate) fn is_test(self) -> bool {
        self.0 & (1 << 4) != 0
    }

    pub(crate) fn uses_top(self) -> bool {
        self.0 & (1 << 5) != 0
    }

    pub(crate) fn sets_top(self) -> bool {
        self.0 & (1 << 6) != 0
    }

    pub(crate) fn is_metamethod(self) -> bool {
        self.0 & (1 << 7) != 0
    }
}

pub const luaP_opnames: &'static [&'static str; 83] = &[
    "MOVE",
    "LOADI",
    "LOADF",
    "LOADK",
    "LOADKX",
    "LOADFALSE",
    "LFALSESKIP",
    "LOADTRUE",
    "LOADNIL",
    "GETUPVAL",
    "SETUPVAL",
    "GETTABUP",
    "GETTABLE",
    "GETI",
    "GETFIELD",
    "SETTABUP",
    "SETTABLE",
    "SETI",
    "SETFIELD",
    "NEWTABLE",
    "SELF",
    "ADDI",
    "ADDK",
    "SUBK",
    "MULK",
    "MODK",
    "POWK",
    "DIVK",
    "IDIVK",
    "BANDK",
    "BORK",
    "BXORK",
    "SHRI",
    "SHLI",
    "ADD",
    "SUB",
    "MUL",
    "MOD",
    "POW",
    "DIV",
    "IDIV",
    "BAND",
    "BOR",
    "BXOR",
    "SHL",
    "SHR",
    "MMBIN",
    "MMBINI",
    "MMBINK",
    "UNM",
    "BNOT",
    "NOT",
    "LEN",
    "CONCAT",
    "CLOSE",
    "TBC",
    "JMP",
    "EQ",
    "LT",
    "LE",
    "EQK",
    "EQI",
    "LTI",
    "LEI",
    "GTI",
    "GEI",
    "TEST",
    "TESTSET",
    "CALL",
    "TAILCALL",
    "RETURN",
    "RETURN0",
    "RETURN1",
    "FORLOOP",
    "FORPREP",
    "TFORPREP",
    "TFORCALL",
    "TFORLOOP",
    "SETLIST",
    "CLOSURE",
    "VARARG",
    "VARARGPREP",
    "EXTRAARG",
];

pub const luaP_opmodes: &'static [lu_byte; 83] = &[
    /*       MM OT IT T  A  mode		   opcode  */
    opmode(0, 0, 0, 0, 1, iABC),  /* OP_MOVE */
    opmode(0, 0, 0, 0, 1, iAsBx), /* OP_LOADI */
    opmode(0, 0, 0, 0, 1, iAsBx), /* OP_LOADF */
    opmode(0, 0, 0, 0, 1, iABx),  /* OP_LOADK */
    opmode(0, 0, 0, 0, 1, iABx),  /* OP_LOADKX */
    opmode(0, 0, 0, 0, 1, iABC),  /* OP_LOADFALSE */
    opmode(0, 0, 0, 0, 1, iABC),  /* OP_LFALSESKIP */
    opmode(0, 0, 0, 0, 1, iABC),  /* OP_LOADTRUE */
    opmode(0, 0, 0, 0, 1, iABC),  /* OP_LOADNIL */
    opmode(0, 0, 0, 0, 1, iABC),  /* OP_GETUPVAL */
    opmode(0, 0, 0, 0, 0, iABC),  /* OP_SETUPVAL */
    opmode(0, 0, 0, 0, 1, iABC),  /* OP_GETTABUP */
    opmode(0, 0, 0, 0, 1, iABC),  /* OP_GETTABLE */
    opmode(0, 0, 0, 0, 1, iABC),  /* OP_GETI */
    opmode(0, 0, 0, 0, 1, iABC),  /* OP_GETFIELD */
    opmode(0, 0, 0, 0, 0, iABC),  /* OP_SETTABUP */
    opmode(0, 0, 0, 0, 0, iABC),  /* OP_SETTABLE */
    opmode(0, 0, 0, 0, 0, iABC),  /* OP_SETI */
    opmode(0, 0, 0, 0, 0, iABC),  /* OP_SETFIELD */
    opmode(0, 0, 0, 0, 1, iABC),  /* OP_NEWTABLE */
    opmode(0, 0, 0, 0, 1, iABC),  /* OP_SELF */
    opmode(0, 0, 0, 0, 1, iABC),  /* OP_ADDI */
    opmode(0, 0, 0, 0, 1, iABC),  /* OP_ADDK */
    opmode(0, 0, 0, 0, 1, iABC),  /* OP_SUBK */
    opmode(0, 0, 0, 0, 1, iABC),  /* OP_MULK */
    opmode(0, 0, 0, 0, 1, iABC),  /* OP_MODK */
    opmode(0, 0, 0, 0, 1, iABC),  /* OP_POWK */
    opmode(0, 0, 0, 0, 1, iABC),  /* OP_DIVK */
    opmode(0, 0, 0, 0, 1, iABC),  /* OP_IDIVK */
    opmode(0, 0, 0, 0, 1, iABC),  /* OP_BANDK */
    opmode(0, 0, 0, 0, 1, iABC),  /* OP_BORK */
    opmode(0, 0, 0, 0, 1, iABC),  /* OP_BXORK */
    opmode(0, 0, 0, 0, 1, iABC),  /* OP_SHRI */
    opmode(0, 0, 0, 0, 1, iABC),  /* OP_SHLI */
    opmode(0, 0, 0, 0, 1, iABC),  /* OP_ADD */
    opmode(0, 0, 0, 0, 1, iABC),  /* OP_SUB */
    opmode(0, 0, 0, 0, 1, iABC),  /* OP_MUL */
    opmode(0, 0, 0, 0, 1, iABC),  /* OP_MOD */
    opmode(0, 0, 0, 0, 1, iABC),  /* OP_POW */
    opmode(0, 0, 0, 0, 1, iABC),  /* OP_DIV */
    opmode(0, 0, 0, 0, 1, iABC),  /* OP_IDIV */
    opmode(0, 0, 0, 0, 1, iABC),  /* OP_BAND */
    opmode(0, 0, 0, 0, 1, iABC),  /* OP_BOR */
    opmode(0, 0, 0, 0, 1, iABC),  /* OP_BXOR */
    opmode(0, 0, 0, 0, 1, iABC),  /* OP_SHL */
    opmode(0, 0, 0, 0, 1, iABC),  /* OP_SHR */
    opmode(1, 0, 0, 0, 0, iABC),  /* OP_MMBIN */
    opmode(1, 0, 0, 0, 0, iABC),  /* OP_MMBINI*/
    opmode(1, 0, 0, 0, 0, iABC),  /* OP_MMBINK*/
    opmode(0, 0, 0, 0, 1, iABC),  /* OP_UNM */
    opmode(0, 0, 0, 0, 1, iABC),  /* OP_BNOT */
    opmode(0, 0, 0, 0, 1, iABC),  /* OP_NOT */
    opmode(0, 0, 0, 0, 1, iABC),  /* OP_LEN */
    opmode(0, 0, 0, 0, 1, iABC),  /* OP_CONCAT */
    opmode(0, 0, 0, 0, 0, iABC),  /* OP_CLOSE */
    opmode(0, 0, 0, 0, 0, iABC),  /* OP_TBC */
    opmode(0, 0, 0, 0, 0, isJ),   /* OP_JMP */
    opmode(0, 0, 0, 1, 0, iABC),  /* OP_EQ */
    opmode(0, 0, 0, 1, 0, iABC),  /* OP_LT */
    opmode(0, 0, 0, 1, 0, iABC),  /* OP_LE */
    opmode(0, 0, 0, 1, 0, iABC),  /* OP_EQK */
    opmode(0, 0, 0, 1, 0, iABC),  /* OP_EQI */
    opmode(0, 0, 0, 1, 0, iABC),  /* OP_LTI */
    opmode(0, 0, 0, 1, 0, iABC),  /* OP_LEI */
    opmode(0, 0, 0, 1, 0, iABC),  /* OP_GTI */
    opmode(0, 0, 0, 1, 0, iABC),  /* OP_GEI */
    opmode(0, 0, 0, 1, 0, iABC),  /* OP_TEST */
    opmode(0, 0, 0, 1, 1, iABC),  /* OP_TESTSET */
    opmode(0, 1, 1, 0, 1, iABC),  /* OP_CALL */
    opmode(0, 1, 1, 0, 1, iABC),  /* OP_TAILCALL */
    opmode(0, 0, 1, 0, 0, iABC),  /* OP_RETURN */
    opmode(0, 0, 0, 0, 0, iABC),  /* OP_RETURN0 */
    opmode(0, 0, 0, 0, 0, iABC),  /* OP_RETURN1 */
    opmode(0, 0, 0, 0, 1, iABx),  /* OP_FORLOOP */
    opmode(0, 0, 0, 0, 1, iABx),  /* OP_FORPREP */
    opmode(0, 0, 0, 0, 0, iABx),  /* OP_TFORPREP */
    opmode(0, 0, 0, 0, 0, iABC),  /* OP_TFORCALL */
    opmode(0, 0, 0, 0, 1, iABx),  /* OP_TFORLOOP */
    opmode(0, 0, 1, 0, 0, iABC),  /* OP_SETLIST */
    opmode(0, 0, 0, 0, 1, iABx),  /* OP_CLOSURE */
    opmode(0, 1, 0, 0, 1, iABC),  /* OP_VARARG */
    opmode(0, 0, 1, 0, 1, iABC),  /* OP_VARARGPREP */
    opmode(0, 0, 0, 0, 0, iAx),   /* OP_EXTRAARG */
];
