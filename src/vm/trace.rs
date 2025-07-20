use std::{
    mem::offset_of,
    ptr::{self, NonNull},
};

use crate::{
    BIT_ISCOLLECTABLE, CallInfo, Instruction, LClosure, LUA_VFALSE, LUA_VNIL, LUA_VNUMFLT,
    LUA_VNUMINT, LUA_VTABLE, LUA_VTRUE, StackValue, StkId, TValue, Table, ctb,
    gc::luaC_barrierback_,
    lua_Integer, lua_Number, lua_State,
    opcodes::*,
    table::{luaH_get, luaH_getint},
    ttypetag,
    utils::{AllocError, DropGuard, GlobalState, LVec32, LuaDrop, RBTree},
};

struct TracedInstr {
    pc: *const Instruction,
    // Types of instruction input arguments.
    // TODO: Check L->openupval for aliasing upvalues (including result reg) and record them here
    arg_tt: [u8; 2],
    // TODO: Record (some) values. For ex CALL instructions would insert guards based on function prototype address.
    // TODO: For conditional instructions, record whether the branch was taken or not.
}

impl LuaDrop for TracedInstr {
    fn drop_with_state(&mut self, _: GlobalState) {}
}

struct BitSet {
    bits: LVec32<u32>,
}

impl LuaDrop for BitSet {
    fn drop_with_state(&mut self, g: GlobalState) {
        self.bits.drop_with_state(g);
    }
}

impl BitSet {
    fn new() -> Self {
        Self {
            bits: LVec32::new(),
        }
    }

    fn contains(&self, i: u32) -> bool {
        let idx = i / 32;
        let bit = 1 << (i & 31);
        (self.bits.get(idx as usize).copied().unwrap_or(0) & bit) != 0
    }

    fn set(&mut self, g: GlobalState, i: u32, v: bool) -> Result<(), AllocError> {
        let idx = i / 32;
        let bit = 1 << (i & 31);

        if self.bits.len() <= idx as usize {
            if !v {
                return Ok(());
            }
            self.bits.resize(g, idx + 1, 0)?;
        }

        let Some(bits) = self.bits.get_mut(idx as usize) else {
            return Ok(());
        };

        if v {
            *bits |= bit;
        } else {
            *bits &= !bit;
        }

        Ok(())
    }

    fn clear(&mut self) {
        self.bits.clear();
    }

    fn iter(&self) -> impl Iterator<Item = u32> {
        self.bits
            .iter()
            .copied()
            .enumerate()
            .flat_map(|(i, x)| {
                if x != 0 {
                    Some(((i * 32) as u32, x))
                } else {
                    None
                }
            })
            .map(|(base, bits)| {
                (0..32).flat_map(move |i| ((bits & (1 << i)) != 0).then_some(base + i))
            })
            .flatten()
    }
}

// TODO: Loop retracing.
// When we trace the next iteration of the loop, cross reference the instructions in the buffer
// with current execution. During which we update arg_tt (polymorphic inline caching...).
// When diverging, create two new basic blocks: One splits the current block, and the other diverges.
// Skip creating the divergent basic block if the block is already present (instead retrace it).
// TODO: If a jump to the middle of a block occurs, we need to split the block.
// Maintain a btree on the start of blocks in the trace buffer so we can find a block for the jump.
// TODO: Because of basic block splitting we need a separate pass (after recording) for use & defs.

pub(crate) struct TraceRecorder {
    pub(super) recording: bool,
    looping: bool,
    /// Buffer of recorded instructions.
    inst_buffer: LVec32<TracedInstr>,
    // Start buffer position -> closure.
    // Searched via binary search.
    // closure_span: NonNull<[(u32, *const LClosure)]>,
    closure: Option<NonNull<LClosure>>,
    // entry_block: Option<NonNull<BlockData>>,
    // current_block: Option<NonNull<BlockData>>,
    // last_alloc_block: Option<NonNull<BlockData>>,
    // // *const Instruction (light ud) -> *mut BlockData (light ud)
    // block_table: NonNull<Table>,
    def_use: DefUse,
    last_pc: *const Instruction,
    trace_out: *mut Option<NonNull<Trace>>,
    entrypoint_out: *mut Option<TraceEntrypoint>,
    depth: u32,
    type_info: Option<NonNull<[RegTypeInfo]>>,
}

impl LuaDrop for TraceRecorder {
    fn drop_with_state(&mut self, g: GlobalState) {
        let Self {
            recording: _,
            looping: _,
            inst_buffer,
            closure: _,
            def_use,
            last_pc: _,
            trace_out: _,
            entrypoint_out: _,
            depth: _,
            type_info: _,
        } = self;

        inst_buffer.drop_with_state(g);
        def_use.drop_with_state(g);
    }
}

impl TraceRecorder {
    pub(crate) fn new() -> Self {
        Self {
            recording: false,
            looping: false,
            inst_buffer: LVec32::new(),
            closure: None,
            def_use: DefUse::new(),
            last_pc: ptr::null(),
            trace_out: ptr::null_mut(),
            entrypoint_out: ptr::null_mut(),
            depth: 0,
            type_info: None,
        }
    }

    pub(crate) fn begin_recording(
        &mut self,
        trace_out: *mut Option<NonNull<Trace>>,
        entrypoint_out: *mut Option<TraceEntrypoint>,
    ) {
        assert!(!self.recording);

        self.recording = true;
        self.looping = false;
        self.inst_buffer.clear();
        self.last_pc = ptr::null();
        self.closure = None;
        self.trace_out = trace_out;
        self.entrypoint_out = entrypoint_out;
        self.depth = 0;
        self.type_info = None;
    }

    pub(crate) unsafe fn setup_for_side_trace(&mut self, trace: NonNull<SideTrace>) {
        self.depth = trace.as_ref().depth;
        self.type_info = Some(NonNull::from(&*trace.as_ref().type_info));
    }

    pub(crate) unsafe fn end_recording(&mut self, L: *mut lua_State) -> Option<()> {
        if !self.recording {
            return None;
        }

        self.recording = false;

        if self.inst_buffer.is_empty() {
            return None;
        }

        eprint!("uses: [");
        for r in self.def_use.uses.iter() {
            eprint!(" {r},")
        }
        eprintln!(" ]");

        let trace = compile_trace(
            L,
            &self,
            self.type_info.map(|n| n.as_ref()).unwrap_or(&[]),
            self.depth,
        );

        let g = GlobalState::new_unchecked((*L).l_G);

        self.def_use.clear(g);

        if !self.entrypoint_out.is_null() {
            self.entrypoint_out.write(Some(trace.entrypoint));
        }

        let trace_ptr = g.alloc()?;
        trace_ptr.write(trace);
        self.trace_out.write(Some(trace_ptr));

        Some(())
    }

    pub(crate) unsafe fn record_start(
        &mut self,
        L: *mut lua_State,
        pc: *const Instruction,
        ci: *const CallInfo,
        cl: NonNull<LClosure>,
    ) -> bool {
        if !self.recording {
            return false;
        }

        let g = GlobalState::new_unchecked((*L).l_G);

        // TODO: Loop unrolling

        if self.looping {
            return false;
        }

        // TODO: Associate closure with instructions via spans
        self.closure = Some(cl);
        self.last_pc = pc;

        if self.inst_buffer.first().is_some_and(|f| f.pc == pc) {
            // Reached first instruction again. Bail.
            self.looping = true;
        }

        let stack = unsafe { (*ci).func.p.add(1) };

        let i = unsafe { *pc };

        let mut traced = TracedInstr { pc, arg_tt: [0; 2] };

        match get_opcode(i) {
            // No need to record trace arguments for constant load instructions.
            OP_LOADI | OP_LOADF | OP_LOADK | OP_LOADKX | OP_LOADFALSE | OP_LFALSESKIP
            | OP_LOADTRUE | OP_LOADNIL => {}

            // Unary-ish. Single register argument. Constants are inferred later (if any).
            OP_MOVE | OP_ADDI | OP_ADDK | OP_SUBK | OP_MULK | OP_MODK | OP_POWK | OP_DIVK
            | OP_IDIVK | OP_BANDK | OP_BORK | OP_BXORK | OP_SHRI | OP_SHLI | OP_UNM | OP_BNOT
            | OP_NOT => {
                traced.arg_tt[0] = unsafe { ttypetag(stack.add(getarg_b(i) as usize)) };
            }

            // Binary.
            OP_ADD | OP_SUB | OP_MUL | OP_MOD | OP_POW | OP_DIV | OP_IDIV | OP_BAND | OP_BOR
            | OP_BXOR | OP_SHL | OP_SHR => {
                traced.arg_tt[0] = unsafe { ttypetag(stack.add(getarg_b(i) as usize)) };
                traced.arg_tt[1] = unsafe { ttypetag(stack.add(getarg_c(i) as usize)) };
            }

            // TODO: Check for metatable?
            OP_GETTABLE if unsafe { ttypetag(stack.add(getarg_b(i) as usize)) == LUA_VTABLE } => {
                traced.arg_tt[0] = LUA_VTABLE;
                traced.arg_tt[1] = unsafe { ttypetag(stack.add(getarg_c(i) as usize)) };
            }

            OP_GETI if unsafe { ttypetag(stack.add(getarg_b(i) as usize)) == LUA_VTABLE } => {
                traced.arg_tt[0] = LUA_VTABLE;
            }

            OP_SETI if unsafe { ttypetag(stack.add(getarg_a(i) as usize)) == LUA_VTABLE } => {
                traced.arg_tt[0] = LUA_VTABLE;
                traced.arg_tt[1] = if getarg_k(i) {
                    // Will be pulled from constant
                    0
                } else {
                    // TODO: This is not really used, as the collectable bit check is small
                    unsafe { ttypetag(stack.add(getarg_c(i) as usize)) }
                };
            }

            // Only trace integer for loops
            OP_FORLOOP
                if unsafe { ttypetag(stack.add(getarg_a(i) as usize)) == LUA_VNUMINT }
                    && unsafe { ttypetag(stack.add(getarg_a(i) as usize + 2)) == LUA_VNUMINT } =>
            {
                traced.arg_tt[0] = LUA_VNUMINT;
                traced.arg_tt[1] = LUA_VNUMINT;

                self.looping = true;
                self.last_pc = self.last_pc.offset((-(getarg_bx(i) as isize)) + 1);
            }

            // TODO: FORPREP is causing miscompilations because we aren't following the control flow correctly.
            OP_FORPREP => {
                traced.arg_tt[0] = LUA_VNUMINT;
                traced.arg_tt[1] = LUA_VNUMINT;
            }

            // OP_GETUPVAL => {
            //     let upval = unsafe {
            //         (&raw const (*cl).upvals)
            //             .cast::<*mut UpVal>()
            //             .add(getarg_b(i) as usize)
            //             .read()
            //     };

            //     traced.arg_tt[0] = unsafe { ttypetag((*upval).v.p) };
            // }

            // Bail trace for the following instructions:
            // OP_GETUPVAL (NYI)
            // OP_SETUPVAL (NYI)
            // OP_GETTABUP (NYI)
            // OP_GETTABLE (NYI)
            // OP_GETI (NYI)
            // OP_GETFIELD (NYI)
            // OP_SETTABUP (NYI)
            // OP_SETTABLE (NYI)
            // OP_SETI (NYI)
            // OP_SETFIELD (NYI)
            // OP_NEWTABLE (NYI)
            // OP_SELF (NYI)
            // OP_MMBIN (NYI)
            // OP_MMBINI (NYI)
            // OP_MMBINK (NYI)
            // OP_LEN (NYI)
            // OP_CONCAT (NYI)
            // OP_CLOSE (NYI)
            // OP_TBC (NYI)
            // OP_JMP (NYI)
            // OP_EQ (NYI)
            // OP_LT (NYI)
            // OP_LE (NYI)
            // OP_EQK (NYI)
            // OP_EQI (NYI)
            // OP_LTI (NYI)
            // OP_LEI (NYI)
            // OP_GTI (NYI)
            // OP_GEI (NYI)
            // OP_TEST (NYI)
            // OP_TESTSET (NYI)
            // OP_CALL (NYI)
            // OP_TAILCALL (NYI)
            // OP_RETURN (NYI)
            // OP_RETURN0 (NYI)
            // OP_RETURN1 (NYI)
            // OP_FORLOOP (NYI)
            // OP_FORPREP (NYI)
            // OP_TFORPREP (NYI)
            // OP_TFORCALL (NYI)
            // OP_TFORLOOP (NYI)
            // OP_SETLIST (NYI)
            // OP_CLOSURE (NYI)
            // OP_VARARG (NYI)
            // OP_VARARGPREP (NYI)
            OP_GETUPVAL | OP_SETUPVAL | OP_GETTABUP | OP_GETTABLE | OP_GETI | OP_GETFIELD
            | OP_SETTABUP | OP_SETTABLE | OP_SETI | OP_SETFIELD | OP_NEWTABLE | OP_SELF
            | OP_MMBIN | OP_MMBINI | OP_MMBINK | OP_LEN | OP_CONCAT | OP_CLOSE | OP_TBC
            | OP_JMP | OP_EQ | OP_LT | OP_LE | OP_EQK | OP_EQI | OP_LTI | OP_LEI | OP_GTI
            | OP_GEI | OP_TEST | OP_TESTSET | OP_CALL | OP_TAILCALL | OP_RETURN | OP_RETURN0
            | OP_RETURN1 | OP_FORLOOP | OP_FORPREP | OP_TFORPREP | OP_TFORCALL | OP_TFORLOOP
            | OP_SETLIST | OP_CLOSURE | OP_VARARG | OP_VARARGPREP | _ => {
                // TODO: Bail, end trace
                return false;
            }
        }

        if let Err(err) = inst_def_use(g, i, &mut self.def_use) {
            unsafe { err.throw(L) };
        }

        self.def_use.next();

        if let Err((err, _)) = self.inst_buffer.push(g, traced) {
            unsafe { err.throw(L) };
        }

        true
    }

    fn killset(&self, g: GlobalState) -> Result<RBTree<(u32, u32), ()>, AllocError> {
        let mut killset = RBTree::new();

        for ((reg, _start), end) in self.def_use.reg_span.iter() {
            killset.insert(g, (*end, *reg), ())?;
        }

        Ok(killset)
    }

    /// Cancel trace recording due to a thrown error.
    pub(crate) fn cancel(&mut self) {
        self.inst_buffer.clear();
        self.recording = false;
    }
}

struct DefUse {
    /// Index in the trace buffer for the current instruction.
    tb_idx: u32,
    /// Registers used from other basic blocks (use before def).
    uses: BitSet,
    /// Registers written by this basic block.
    defs: BitSet,
    /// Last place where a register was defined and used.
    ///
    /// reg -> (def_idx, last_use_idx)
    last_def_use: RBTree<u32, (u32, u32)>,
    /// Span starting at the definition of the register and ending at the last use.
    ///
    /// Last use index is `u32::MAX` if the last usage has not been found yet.
    ///
    /// (reg, tb_start_index) -> tb_end_index
    reg_span: RBTree<(u32, u32), u32>,
}

impl LuaDrop for DefUse {
    fn drop_with_state(&mut self, g: GlobalState) {
        let Self {
            tb_idx: _,
            uses,
            defs,
            last_def_use,
            reg_span,
        } = self;

        uses.drop_with_state(g);
        defs.drop_with_state(g);
        last_def_use.drop_with_state(g);
        reg_span.drop_with_state(g);
    }
}

impl DefUse {
    fn new() -> DefUse {
        DefUse {
            tb_idx: 0,
            uses: BitSet::new(),
            defs: BitSet::new(),
            last_def_use: RBTree::new(),
            reg_span: RBTree::new(),
        }
    }

    fn next(&mut self) {
        self.tb_idx += 1;
    }

    fn clear(&mut self, g: GlobalState) {
        self.tb_idx = 0;
        self.uses.clear();
        self.defs.clear();
        self.last_def_use.clear(g);
        self.reg_span.clear(g);
    }

    fn write(&mut self, g: GlobalState, reg: u32) -> Result<(), AllocError> {
        let was_def = self.defs.contains(reg);

        self.defs.set(g, reg, true)?;

        if was_def {
            // Get the location of the last definition
            if let Some((prev_def, last_use)) = self.last_def_use.get(&reg) {
                // Set the range of the last definition
                self.reg_span.insert(g, (reg, *prev_def), *last_use)?;
            }
        }

        let tb_idx = self.tb_idx;

        self.last_def_use.insert(g, reg, (tb_idx, tb_idx))?;
        self.reg_span.insert(g, (reg, tb_idx), u32::MAX)?;

        Ok(())
    }

    fn read(&mut self, g: GlobalState, reg: u32) -> Result<(), AllocError> {
        if !self.defs.contains(reg) {
            // Incoming phi from outside of the trace
            self.uses.set(g, reg, true)?;
        } else {
            // Update last usage
            if let Some((_, last_use)) = self.last_def_use.get_mut(&reg) {
                *last_use = self.tb_idx;
            }
        }

        Ok(())
    }

    fn read_write(&mut self, g: GlobalState, reg: u32) -> Result<(), AllocError> {
        self.read(g, reg)?;
        self.write(g, reg)
    }

    /// Undefine all defs above base, terminating their register spans.
    fn undef_all(&mut self, g: GlobalState, base: u32) -> Result<(), AllocError> {
        let top = (self.defs.bits.len() * 32) as u32;
        for reg in base..top {
            let was_def = self.defs.contains(reg);

            let _ = self.defs.set(g, reg, false);

            if was_def {
                // Get the location of the last definition
                if let Some((prev_def, last_use)) = self.last_def_use.get(&reg) {
                    // Set the range of the last definition
                    self.reg_span.insert(g, (reg, *prev_def), *last_use)?;
                }
            }
        }
        Ok(())
    }
}

fn inst_def_use(g: GlobalState, i: Instruction, def_use: &mut DefUse) -> Result<(), AllocError> {
    let a = getarg_a(i);
    let b = getarg_b(i);
    let c = getarg_c(i);
    let k = getarg_k(i);

    match get_opcode(i) {
        // Nulary. `R[A]` is set, no registers read.
        OP_LOADI | OP_LOADF | OP_LOADK | OP_LOADKX | OP_LOADFALSE | OP_LFALSESKIP | OP_LOADTRUE
        | OP_LOADNIL | OP_GETUPVAL | OP_GETTABUP | OP_NEWTABLE => {
            def_use.write(g, a)?;
        }

        // Unary-ish. `R[A] := ... R[B]`
        OP_MOVE | OP_ADDI | OP_ADDK | OP_SUBK | OP_MULK | OP_MODK | OP_POWK | OP_DIVK
        | OP_IDIVK | OP_BANDK | OP_BORK | OP_BXORK | OP_SHRI | OP_SHLI | OP_UNM | OP_BNOT
        | OP_NOT | OP_GETI | OP_GETFIELD => {
            def_use.read(g, b)?;
            def_use.write(g, a)?;
        }

        // Binary. `R[A] := R[B] ... R[C]`
        OP_ADD | OP_SUB | OP_MUL | OP_MOD | OP_POW | OP_DIV | OP_IDIV | OP_BAND | OP_BOR
        | OP_BXOR | OP_SHL | OP_SHR | OP_GETTABLE => {
            def_use.read(g, b)?;
            def_use.read(g, c)?;

            def_use.write(g, a)?;
        }

        // `R[A] ... RK[C]`
        OP_SETI | OP_SETFIELD => {
            if !k {
                def_use.read(g, c)?;
            }

            def_use.read(g, a)?;
        }

        // `... R[A] ...`
        OP_SETUPVAL => {
            def_use.read(g, a)?;
        }

        // `... RK[C] ...`
        OP_SETTABUP => {
            if !k {
                def_use.read(g, c)?;
            }
        }

        // `R[A] R[B] RK[C]`
        OP_SETTABLE => {
            def_use.read(g, a)?;
            def_use.read(g, b)?;
            if !k {
                def_use.read(g, c)?;
            }
        }

        // Only trace integer for loops
        OP_FORLOOP => {
            let idx = a;
            let count = idx + 1;
            let step = idx + 2;
            let out_idx = idx + 3;

            def_use.undef_all(g, idx + 4)?;

            def_use.read_write(g, idx)?;
            def_use.read_write(g, count)?;
            def_use.read(g, step)?;
            def_use.write(g, out_idx)?;
        }

        OP_FORPREP => {
            let idx = a;
            let count = idx + 1;
            let step = idx + 2;
            let out_idx = idx + 3;

            def_use.undef_all(g, idx + 4)?;

            def_use.read(g, idx)?;
            def_use.read_write(g, count)?;
            def_use.read(g, step)?;
            def_use.write(g, out_idx)?;
        }

        // TODO: implement the following instructions
        // OP_SELF (NYI)
        // OP_MMBIN (NYI)
        // OP_MMBINI (NYI)
        // OP_MMBINK (NYI)
        // OP_LEN (NYI)
        // OP_CONCAT (NYI)
        // OP_CLOSE (NYI)
        // OP_TBC (NYI)
        // OP_JMP (NYI)
        // OP_EQ (NYI)
        // OP_LT (NYI)
        // OP_LE (NYI)
        // OP_EQK (NYI)
        // OP_EQI (NYI)
        // OP_LTI (NYI)
        // OP_LEI (NYI)
        // OP_GTI (NYI)
        // OP_GEI (NYI)
        // OP_TEST (NYI)
        // OP_TESTSET (NYI)
        // OP_CALL (NYI)
        // OP_TAILCALL (NYI)
        // OP_RETURN (NYI)
        // OP_RETURN0 (NYI)
        // OP_RETURN1 (NYI)
        // OP_FORLOOP (NYI)
        // OP_FORPREP (NYI)
        // OP_TFORPREP (NYI)
        // OP_TFORCALL (NYI)
        // OP_TFORLOOP (NYI)
        // OP_SETLIST (NYI)
        // OP_CLOSURE (NYI)
        // OP_VARARG (NYI)
        // OP_VARARGPREP (NYI)
        OP_SELF | OP_MMBIN | OP_MMBINI | OP_MMBINK | OP_LEN | OP_CONCAT | OP_CLOSE | OP_TBC
        | OP_JMP | OP_EQ | OP_LT | OP_LE | OP_EQK | OP_EQI | OP_LTI | OP_LEI | OP_GTI | OP_GEI
        | OP_TEST | OP_TESTSET | OP_CALL | OP_TAILCALL | OP_RETURN | OP_RETURN0 | OP_RETURN1
        | OP_TFORPREP | OP_TFORCALL | OP_TFORLOOP | OP_SETLIST | OP_CLOSURE | OP_VARARG
        | OP_VARARGPREP | _ => {
            todo!()
        }
    }
    Ok(())
}

// Simple trace compiler:
// Creates a cranelift JIT module, then compiles a simple linear function based off of the trace.
// Value types are checked, and bails back to the interpreter on mismatch.

type TraceEntrypoint = unsafe extern "C" fn(
    base: StkId,
    L: *mut lua_State,
    ci: *const CallInfo,
    cl: *const LClosure,
) -> isize;

// TODO: Need a depth limit for side traces, since side traces themselves can have side traces.
// TODO: Move type info to Trace, and avoid creating new side traces for loops with the same
// type info (which would compile the same exact loop again, but as a nested side trace).
// The main trace would have empty type info.

#[derive(Copy, Clone, PartialEq, Eq)]
pub(crate) struct RegTypeInfo {
    reg: u32,
    tt: u8,
}

impl std::fmt::Debug for RegTypeInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "R[{}] as {}", self.reg, self.tt)
    }
}

/// A side trace, which branches off of the main trace.
#[repr(C)]
pub(crate) struct SideTrace {
    /// The entrypoint of the side trace.
    ///
    /// Specifically placed as the first field in the side trace so the owning trace can
    /// tailcall instead of bailing.
    pub(super) entrypoint: Option<TraceEntrypoint>,
    /// Next side trace point in the chain of side trace points.
    next: Option<NonNull<SideTrace>>,
    /// Depth of this side trace in the trace tree, where 1 is branched off of the main trace.
    pub(super) depth: u32,
    /// PC the side trace starts at.
    pub(super) pc: *const Instruction,
    /// Known register types at the site of the trace bail branch.
    ///
    /// They do not need type tests.
    ///
    /// After trace compilation, registers that were not used are removed.
    type_info: LVec32<RegTypeInfo>,
    pub(super) visited: bool,
    /// Compiled side trace.
    pub(super) trace: Option<NonNull<Trace>>,
}

pub(crate) struct Trace {
    // TODO: Reuse JIT module?
    jit: cranelift::jit::JITModule,
    pub(super) entrypoint: TraceEntrypoint,
    pub(super) last_pc: *const Instruction,
    depth: u32,
    side_traces: Option<NonNull<SideTrace>>,
}

pub(crate) unsafe fn compile_trace(
    L: *mut lua_State,
    tr: &TraceRecorder,
    type_info: &[RegTypeInfo],
    trace_depth: u32,
) -> Trace {
    let g = GlobalState::new_unchecked((*L).l_G);

    let killset = DropGuard::new(
        g,
        match tr.killset(g) {
            Ok(ks) => ks,
            Err(err) => err.throw(L),
        },
    );

    use cranelift::codegen::ir::{BlockArg, SigRef, UserFuncName};
    use cranelift::jit::{JITBuilder, JITModule};
    use cranelift::prelude::*;
    use cranelift_module::{Module, default_libcall_names};

    let mut flag_builder = settings::builder();
    flag_builder.set("use_colocated_libcalls", "false").unwrap();
    // FIXME set back to true once the x64 backend supports it.
    flag_builder.set("is_pic", "false").unwrap();
    flag_builder.set("opt_level", "speed").unwrap();
    flag_builder.set("preserve_frame_pointers", "true").unwrap();
    // flag_builder.set("enable_verifier", "false").unwrap();
    let isa_builder = cranelift_native::builder().unwrap_or_else(|msg| {
        panic!("host machine is not supported: {msg}");
    });
    let mut flags = settings::Flags::new(flag_builder);
    flags.enable_alias_analysis();
    let isa = isa_builder.finish(flags).unwrap();
    let mut module = JITModule::new(JITBuilder::with_isa(isa, default_libcall_names()));

    #[cfg(target_pointer_width = "64")]
    const PTR_TYPE: Type = types::I64;
    #[cfg(target_pointer_width = "32")]
    const PTR_TYPE: Type = types::I32;

    const LUA_INT: Type = {
        match size_of::<lua_Integer>() {
            8 => types::I64,
            4 => types::I32,
            2 => types::I16,
            1 => types::I8,
            _ => panic!("cannot select integer type"),
        }
    };

    const LUA_NUM: Type = {
        match size_of::<lua_Number>() {
            8 => types::F64,
            4 => types::F32,
            _ => panic!("cannot select number type"),
        }
    };

    const TT: Type = types::I8;

    struct ExtFunc {
        name: &'static str,
        sig: Signature,
        addr: i64,
    }

    enum ExtFuncRef<'a> {
        Uninit(&'a ExtFunc),
        Init { sig: codegen::ir::SigRef, addr: i64 },
    }

    impl ExtFuncRef<'_> {
        fn call(&mut self, bcx: &mut FunctionBuilder<'_>, args: &[Value]) -> codegen::ir::Inst {
            let (sig, addr) = match self {
                ExtFuncRef::Uninit(func) => {
                    let (sig, addr) = (bcx.import_signature(func.sig.clone()), func.addr);
                    *self = ExtFuncRef::Init { sig, addr };
                    (sig, addr)
                }
                ExtFuncRef::Init { sig, addr } => (*sig, *addr),
            };

            let addr = bcx.ins().iconst(PTR_TYPE, addr);
            bcx.ins().call_indirect(sig, addr, args)
        }
    }

    // External Functions:
    // luaH_get(*mut Table, *const TValue) -> *mut TValue (non-null)
    let ext_luaH_get = {
        let mut sig = module.make_signature();
        sig.params.push(AbiParam::new(PTR_TYPE));
        sig.params.push(AbiParam::new(PTR_TYPE));
        sig.returns.push(AbiParam::new(PTR_TYPE));
        ExtFunc {
            name: "luaH_get",
            sig,
            addr: unsafe {
                std::mem::transmute::<unsafe extern "C-unwind" fn(_, _) -> _, usize>(luaH_get)
                    as i64
            },
        }
    };

    // luaH_getint(*mut Table, lua_Integer) -> *mut TValue (non-null)
    let ext_luaH_getint = {
        let mut sig = module.make_signature();
        sig.params.push(AbiParam::new(PTR_TYPE));
        sig.params.push(AbiParam::new(LUA_INT));
        sig.returns.push(AbiParam::new(PTR_TYPE));
        ExtFunc {
            name: "luaH_getint",
            sig,
            addr: unsafe {
                std::mem::transmute::<unsafe extern "C-unwind" fn(_, _) -> _, usize>(luaH_getint)
                    as i64
            },
        }
    };

    // luaC_barrierback_(*mut lua_State, *mut GCObject)
    let ext_luaC_barrierback = {
        let mut sig = module.make_signature();
        sig.params.push(AbiParam::new(PTR_TYPE));
        sig.params.push(AbiParam::new(PTR_TYPE));
        ExtFunc {
            name: "luaC_barrierback_",
            sig,
            addr: unsafe {
                std::mem::transmute::<unsafe extern "C-unwind" fn(_, _) -> _, usize>(
                    luaC_barrierback_,
                ) as i64
            },
        }
    };

    // powf(lua_Number, lua_Number) -> lua_Number
    let ext_powf = {
        unsafe extern "C" fn powf(a: lua_Number, b: lua_Number) -> lua_Number {
            a.powf(b)
        }

        let mut sig = module.make_signature();
        sig.params.push(AbiParam::new(LUA_NUM));
        sig.params.push(AbiParam::new(LUA_NUM));
        sig.returns.push(AbiParam::new(LUA_NUM));
        ExtFunc {
            name: "powf",
            sig,
            addr: unsafe {
                std::mem::transmute::<unsafe extern "C" fn(_, _) -> _, usize>(powf) as i64
            },
        }
    };

    // TODO: 32-bit support
    let mut sig = module.make_signature();
    sig.params.push(AbiParam::new(PTR_TYPE)); // base
    sig.params.push(AbiParam::new(PTR_TYPE)); // L
    sig.params.push(AbiParam::new(PTR_TYPE)); // ci
    sig.params.push(AbiParam::new(PTR_TYPE)); // cl
    sig.returns.push(AbiParam::new(PTR_TYPE)); // result (0 = ok, negative for bailout)

    let mut ctx = module.make_context();
    let mut func_ctx = FunctionBuilderContext::new();

    let trampoline_fn = module
        .declare_function("trampoline", cranelift_module::Linkage::Local, &sig)
        .unwrap();

    let default_call_conv = sig.call_conv;

    sig.call_conv = isa::CallConv::Tail;

    let trace_fn = module
        .declare_function("trace", cranelift_module::Linkage::Local, &sig)
        .unwrap();

    {
        // Compile the trampoline to call sig
        ctx.func.signature = sig.clone();
        ctx.func.signature.call_conv = default_call_conv;
        ctx.func.name = UserFuncName::user(0, trampoline_fn.as_u32());

        let mut bcx = FunctionBuilder::new(&mut ctx.func, &mut func_ctx);
        let mut block = bcx.create_block();

        bcx.switch_to_block(block);
        bcx.append_block_params_for_function_params(block);

        let &[arg_base, arg_state, arg_call_info, arg_closure] = bcx.block_params(block) else {
            panic!()
        };
        let trace_fn = module.declare_func_in_func(trace_fn, &mut bcx.func);

        let inst = bcx
            .ins()
            .call(trace_fn, &[arg_base, arg_state, arg_call_info, arg_closure]);
        let result = bcx.inst_results(inst)[0];
        bcx.ins().return_(&[result]);
        bcx.seal_all_blocks();
        bcx.finalize();
    }
    module.define_function(trampoline_fn, &mut ctx).unwrap();
    module.clear_context(&mut ctx);

    ctx.func.signature = sig;
    ctx.func.name = UserFuncName::user(0, trace_fn.as_u32());

    let side_traces;

    {
        let mut ext_luaH_get = ExtFuncRef::Uninit(&ext_luaH_get);
        let mut ext_luaH_getint = ExtFuncRef::Uninit(&ext_luaH_getint);
        let mut ext_luaC_barrierback = ExtFuncRef::Uninit(&ext_luaC_barrierback);
        let mut ext_powf = ExtFuncRef::Uninit(&ext_powf);

        let mut bcx = FunctionBuilder::new(&mut ctx.func, &mut func_ctx);
        let entry_block = bcx.create_block();
        let mut block = bcx.create_block();

        bcx.switch_to_block(entry_block);
        bcx.append_block_params_for_function_params(entry_block);
        bcx.ins().jump(block, &[]);

        bcx.switch_to_block(block);

        let &[arg_base, arg_state, arg_call_info, arg_closure] = bcx.block_params(entry_block)
        else {
            panic!()
        };

        let entry_block = block;

        let tailcall_sig = bcx.import_signature(bcx.func.signature.clone());

        #[derive(Copy, Clone, PartialEq, Eq)]
        enum CgValueType {
            Any,
            Int,
            Float,
        }

        #[derive(Copy, Clone)]
        struct CgTValue {
            value: Option<Value>,
            tt: Option<Value>,
        }

        #[derive(Copy, Clone)]
        struct CgReg {
            value: CgTValue,
            writeback: bool,
            known_tt: u8,
        }

        struct TraceContext<'tr> {
            L: *mut lua_State,
            g: GlobalState,
            arg_base: Value,
            arg_state: Value,
            arg_call_info: Value,
            arg_closure: Value,
            regs: DropGuard<LVec32<Option<CgReg>>>,
            last_def_use: &'tr RBTree<u32, (u32, u32)>,
            side_trace_dirty: bool,
            trace_depth: u32,
            last_side_trace: Option<NonNull<SideTrace>>,
            pc: *const Instruction,
            tailcall_sig: SigRef,
        }

        impl<'tr> TraceContext<'tr> {
            fn set_reg(&mut self, r: u32, value: CgTValue) {
                if (self.regs.len() as u32) < r + 1 {
                    if let Err(err) = self.regs.resize(self.g, r + 1, None) {
                        unsafe { err.throw(self.L) };
                    }
                }

                if let Some(Some(reg)) = self.regs.get(r as usize) {
                    if reg.known_tt != 255 {
                        self.side_trace_dirty = true;
                    }
                }

                self.regs[r as usize] = Some(CgReg {
                    value,
                    writeback: true,
                    known_tt: 255,
                });
            }

            fn set_reg_typed(
                &mut self,
                r: u32,
                value: Value,
                tt: u8,
                bcx: &mut FunctionBuilder<'_>,
            ) {
                if (self.regs.len() as u32) < r + 1 {
                    if let Err(err) = self.regs.resize(self.g, r + 1, None) {
                        unsafe { err.throw(self.L) };
                    }
                }

                if let Some(Some(reg)) = self.regs.get(r as usize) {
                    if reg.known_tt != tt {
                        self.side_trace_dirty = true;
                    }
                }

                self.regs[r as usize] = Some(CgReg {
                    value: CgTValue {
                        value: Some(value),
                        tt: None,
                    },
                    writeback: true,
                    known_tt: tt,
                });
            }

            fn get_reg_value(
                &mut self,
                reg: u32,
                ty: CgValueType,
                bcx: &mut FunctionBuilder<'_>,
            ) -> Value {
                if (self.regs.len() as u32) < reg + 1 {
                    if let Err(err) = self.regs.resize(self.g, reg + 1, None) {
                        unsafe { err.throw(self.L) };
                    }
                }

                let src = &mut self.regs[reg as usize];

                let src = src.get_or_insert_with(|| CgReg {
                    value: CgTValue {
                        value: None,
                        tt: None,
                    },
                    writeback: false,
                    known_tt: 255,
                });

                let value = *src.value.value.get_or_insert_with(|| {
                    let offset = Self::reg_offset(reg);

                    bcx.ins().load(
                        match ty {
                            CgValueType::Float => LUA_NUM,
                            CgValueType::Any | CgValueType::Int => LUA_INT,
                        },
                        MemFlags::trusted().with_can_move(),
                        self.arg_base,
                        offset,
                    )
                });

                match (ty, bcx.func.dfg.value_type(value)) {
                    (CgValueType::Any, _) => value,
                    (CgValueType::Int, LUA_INT) => value,
                    (CgValueType::Float, LUA_NUM) => value,
                    (CgValueType::Int, LUA_NUM) => {
                        bcx.ins().bitcast(LUA_INT, MemFlags::new(), value)
                    }
                    (CgValueType::Float, LUA_INT) => {
                        bcx.ins().bitcast(LUA_NUM, MemFlags::new(), value)
                    }
                    _ => unreachable!(),
                }
            }

            fn get_reg_tt(&mut self, reg: u32, bcx: &mut FunctionBuilder<'_>) -> Value {
                if (self.regs.len() as u32) < reg + 1 {
                    if let Err(err) = self.regs.resize(self.g, reg + 1, None) {
                        unsafe { err.throw(self.L) };
                    }
                }

                let src = &mut self.regs[reg as usize];

                let src = src.get_or_insert_with(|| CgReg {
                    value: CgTValue {
                        value: None,
                        tt: None,
                    },
                    writeback: false,
                    known_tt: 255,
                });

                if src.known_tt != 255 {
                    bcx.ins().iconst(TT, src.known_tt as i64)
                } else {
                    *src.value.tt.get_or_insert_with(|| {
                        let offset = Self::reg_offset(reg);

                        bcx.ins().load(
                            TT,
                            MemFlags::trusted().with_can_move(),
                            self.arg_base,
                            offset + (size_of::<crate::Value>() as i32),
                        )
                    })
                }
            }

            fn get_reg_value_and_tt(
                &mut self,
                reg: u32,
                ty: CgValueType,
                bcx: &mut FunctionBuilder<'_>,
            ) -> (Value, Value) {
                (self.get_reg_value(reg, ty, bcx), self.get_reg_tt(reg, bcx))
            }

            fn get_reg_type_guarded(
                &mut self,
                r: u32,
                tt: u8,
                bcx: &mut FunctionBuilder<'_>,
                block: &mut Block,
            ) -> Value {
                if self.reg_static_type(r) != tt {
                    // Register type not known statically, insert type guard.
                    let value_tt = self.get_reg_tt(r, bcx);
                    *block = self.type_guard(bcx, value_tt, tt);
                    self.set_reg_static_type(r, tt);

                    if let Some(Some(reg)) = self.regs.get_mut(r as usize) {
                        // Will create constants at use-site instead.
                        reg.value.tt = None;
                    }
                }

                self.get_reg_value(r, CgValueType::Int, bcx)
            }

            fn set_reg_static_type(&mut self, r: u32, tt: u8) {
                if (self.regs.len() as u32) < r + 1 {
                    if let Err(err) = self.regs.resize(self.g, r + 1, None) {
                        unsafe { err.throw(self.L) };
                    }
                }

                let Some(reg) = self.regs.get_mut(r as usize) else {
                    return;
                };

                let reg = reg.get_or_insert_with(|| CgReg {
                    value: CgTValue {
                        value: None,
                        tt: None,
                    },
                    writeback: false,
                    known_tt: tt,
                });

                if reg.known_tt != tt {
                    self.side_trace_dirty = true;
                }

                reg.known_tt = tt;
            }

            fn reg_static_type(&self, r: u32) -> u8 {
                let Some(Some(reg)) = self.regs.get(r as usize) else {
                    return 255;
                };
                reg.known_tt
            }

            fn side_trace(&mut self) -> Result<NonNull<SideTrace>, AllocError> {
                if let (Some(last_side_trace), false) =
                    (self.last_side_trace, self.side_trace_dirty)
                {
                    return Ok(last_side_trace);
                }

                let mut type_info = DropGuard::new(self.g, LVec32::new());
                for (i, reg) in self.regs.iter().enumerate() {
                    let Some(reg) = reg else { continue };
                    if reg.known_tt != 255 {
                        type_info
                            .push(
                                self.g,
                                RegTypeInfo {
                                    reg: i as u32,
                                    tt: reg.known_tt,
                                },
                            )
                            .map_err(|(err, _)| err)?;
                    }
                }

                let mut out_ptr = self.g.alloc().ok_or(AllocError)?;

                let side_trace = SideTrace {
                    entrypoint: None,
                    next: self.last_side_trace,
                    pc: self.pc,
                    depth: self.trace_depth + 1,
                    type_info: type_info.into_inner(),
                    visited: false,
                    trace: None,
                };

                unsafe {
                    out_ptr.write(side_trace);
                }

                self.last_side_trace = Some(out_ptr);
                self.side_trace_dirty = false;

                Ok(out_ptr)
            }

            fn emit_side_exit(&mut self, bcx: &mut FunctionBuilder<'_>) {
                let side_trace = match self.side_trace() {
                    Ok(trace) => trace,
                    Err(err) => unsafe { err.throw(self.L) },
                };

                self.writeback_regs(bcx);

                // Either tailcall to the side trace, or return if the side trace hasn't been compiled.
                let side_block = bcx.create_block();
                let return_block = bcx.create_block();

                let side_trace = bcx.ins().iconst(PTR_TYPE, side_trace.addr().get() as i64);
                let entrypoint = bcx.ins().load(PTR_TYPE, MemFlags::trusted(), side_trace, 0);
                let entrypoint_not_null = bcx.ins().icmp_imm(IntCC::NotEqual, entrypoint, 0);

                bcx.ins()
                    .brif(entrypoint_not_null, side_block, [], return_block, []);

                bcx.switch_to_block(side_block);
                bcx.ins().return_call_indirect(
                    self.tailcall_sig,
                    entrypoint,
                    &[
                        self.arg_base,
                        self.arg_state,
                        self.arg_call_info,
                        self.arg_closure,
                    ],
                );

                bcx.switch_to_block(return_block);
                bcx.ins().return_(&[side_trace]);
            }

            fn reg_offset(r: u32) -> i32 {
                // NOTE: This would change based on the current inlined function, once inlining is implemented.
                (r as u32 * size_of::<StackValue>() as u32) as i32
            }

            /// Flushes all modified TValues to the backing registers.
            fn writeback_regs(&self, bcx: &mut FunctionBuilder<'_>) {
                for (i, reg) in self.regs.iter().enumerate() {
                    let Some(reg) = reg else { continue };

                    if reg.writeback {
                        let offset = Self::reg_offset(i as u32);
                        if let Some(value) = reg.value.value {
                            bcx.ins().store(
                                MemFlags::trusted().with_can_move(),
                                value,
                                self.arg_base,
                                offset,
                            );
                        }
                        if let Some(tt) = reg.value.tt {
                            bcx.ins().store(
                                MemFlags::trusted().with_can_move(),
                                tt,
                                self.arg_base,
                                offset + (size_of::<crate::Value>() as i32),
                            );
                        } else if reg.known_tt != 255 {
                            let tt = bcx.ins().iconst(TT, reg.known_tt as i64);
                            bcx.ins().store(
                                MemFlags::trusted().with_can_move(),
                                tt,
                                self.arg_base,
                                offset + (size_of::<crate::Value>() as i32),
                            );
                        }
                    }
                }
            }

            /// Writeback a single register to memory, if it needs writeback.
            fn writeback_reg(&self, bcx: &mut FunctionBuilder<'_>, r: u32) {
                let Some(Some(reg)) = self.regs.get(r as usize) else {
                    return;
                };

                if reg.writeback {
                    let offset = Self::reg_offset(r);
                    if let Some(value) = reg.value.value {
                        bcx.ins().store(
                            MemFlags::trusted().with_can_move(),
                            value,
                            self.arg_base,
                            offset,
                        );
                    }
                    if let Some(tt) = reg.value.tt {
                        bcx.ins().store(
                            MemFlags::trusted().with_can_move(),
                            tt,
                            self.arg_base,
                            offset + (size_of::<crate::Value>() as i32),
                        );
                    } else if reg.known_tt != 255 {
                        let tt = bcx.ins().iconst(TT, reg.known_tt as i64);
                        bcx.ins().store(
                            MemFlags::trusted().with_can_move(),
                            tt,
                            self.arg_base,
                            offset + (size_of::<crate::Value>() as i32),
                        );
                    }
                }
            }

            fn writeback_reg_if_last_def(
                &mut self,
                bcx: &mut FunctionBuilder<'_>,
                tr_idx: u32,
                reg_id: u32,
            ) {
                // TODO: Reenable this codepath once the performance degredation is found.
                if true {
                    return;
                }

                let Some((last_def, _)) = self.last_def_use.get(&reg_id) else {
                    return;
                };

                if *last_def != tr_idx {
                    return;
                }

                let offset = Self::reg_offset(reg_id);

                let Some(Some(reg)) = self.regs.get_mut(reg_id as usize) else {
                    return;
                };

                if reg.writeback {
                    if let Some(value) = reg.value.value {
                        bcx.ins().store(
                            MemFlags::trusted().with_can_move(),
                            value,
                            self.arg_base,
                            offset,
                        );
                    }
                    if let Some(tt) = reg.value.tt {
                        bcx.ins().store(
                            MemFlags::trusted().with_can_move(),
                            tt,
                            self.arg_base,
                            offset + (size_of::<crate::Value>() as i32),
                        );
                    } else if reg.known_tt != 255 {
                        let tt = bcx.ins().iconst(TT, reg.known_tt as i64);
                        bcx.ins().store(
                            MemFlags::trusted().with_can_move(),
                            tt,
                            self.arg_base,
                            offset + (size_of::<crate::Value>() as i32),
                        );
                    }

                    reg.writeback = false;
                }
            }

            /// Creates a type guard, which bails out to the interpreter on type mismatch.
            fn type_guard(
                &mut self,
                bcx: &mut FunctionBuilder<'_>,
                tt: Value,
                expected_tt: u8,
            ) -> Block {
                let bail_block = bcx.create_block();
                bcx.set_cold_block(bail_block);
                let continue_block = bcx.create_block();

                let cond = bcx.ins().icmp_imm(IntCC::Equal, tt, expected_tt as i64);

                bcx.ins().brif(cond, continue_block, [], bail_block, []);

                bcx.switch_to_block(bail_block);
                self.emit_side_exit(bcx);

                bcx.switch_to_block(continue_block);
                continue_block
            }

            fn arith_value(
                &mut self,
                bcx: &mut FunctionBuilder<'_>,
                block: &mut Block,
                reg: u32,
                reg_tt: u8,
            ) -> Value {
                if self.reg_static_type(reg) == 255 {
                    // Register type not known statically, insert type guard.
                    let lhs_tt = self.get_reg_tt(reg, bcx);
                    *block = self.type_guard(bcx, lhs_tt, reg_tt);
                    self.set_reg_static_type(reg, reg_tt);
                }

                match reg_tt {
                    LUA_VNUMINT => self.get_reg_value(reg, CgValueType::Int, bcx),
                    LUA_VNUMFLT => self.get_reg_value(reg, CgValueType::Float, bcx),
                    _ => unreachable!(),
                }
            }

            /// Int or float arithmetic between a register and a constant.
            fn arith_k<'f>(
                &mut self,
                bcx: &mut FunctionBuilder<'f>,
                block: &mut Block,
                ksts: *mut TValue,
                tr_idx: u32,
                a: u32,
                b: u32,
                b_tt: u8,
                c: u32,
                int_op: impl FnOnce(&mut FunctionBuilder<'f>, Value, i64) -> Value,
                flt_op: impl FnOnce(&mut FunctionBuilder<'f>, Value, Value) -> Value,
            ) {
                let lhs = self.arith_value(bcx, block, b, b_tt);

                let kst = unsafe { ksts.add(c as usize) };

                let (value, result_tt) = match (b_tt, unsafe { (*kst).tt_ }) {
                    (LUA_VNUMINT, LUA_VNUMINT) => {
                        (int_op(bcx, lhs, unsafe { (*kst).value_.i }), LUA_VNUMINT)
                    }
                    (LUA_VNUMINT, LUA_VNUMFLT) => {
                        let x = bcx.ins().fcvt_from_sint(LUA_NUM, lhs);
                        let y = bcx.ins().f64const(unsafe { (*kst).value_.n });
                        (flt_op(bcx, x, y), LUA_VNUMFLT)
                    }
                    (LUA_VNUMFLT, LUA_VNUMINT) => {
                        let x = lhs;
                        let y = bcx.ins().f64const(unsafe { (*kst).value_.i } as f64);
                        (flt_op(bcx, x, y), LUA_VNUMFLT)
                    }
                    (LUA_VNUMFLT, LUA_VNUMFLT) => {
                        let x = lhs;
                        let y = bcx.ins().f64const(unsafe { (*kst).value_.n });
                        (flt_op(bcx, x, y), LUA_VNUMFLT)
                    }
                    _ => todo!(),
                };

                self.set_reg_typed(a, value, result_tt, bcx);
                self.writeback_reg_if_last_def(bcx, tr_idx, a);
            }

            /// Int or float arithmetic between two registers.
            fn arith<'f>(
                &mut self,
                bcx: &mut FunctionBuilder<'f>,
                block: &mut Block,
                tr_idx: u32,
                a: u32,
                b: u32,
                b_tt: u8,
                c: u32,
                c_tt: u8,
                int_op: impl FnOnce(&mut FunctionBuilder<'f>, Value, Value) -> Value,
                flt_op: impl FnOnce(&mut FunctionBuilder<'f>, Value, Value) -> Value,
            ) {
                let lhs = self.arith_value(bcx, block, b, b_tt);
                let rhs = self.arith_value(bcx, block, c, c_tt);

                let (value, result_tt) = match (b_tt, c_tt) {
                    (LUA_VNUMINT, LUA_VNUMINT) => (int_op(bcx, lhs, rhs), LUA_VNUMINT),
                    (LUA_VNUMINT, LUA_VNUMFLT) => {
                        let x = bcx.ins().fcvt_from_sint(LUA_NUM, lhs);
                        let y = rhs;
                        (flt_op(bcx, x, y), LUA_VNUMFLT)
                    }
                    (LUA_VNUMFLT, LUA_VNUMINT) => {
                        let x = lhs;
                        let y = bcx.ins().fcvt_from_sint(LUA_NUM, rhs);
                        (flt_op(bcx, x, y), LUA_VNUMFLT)
                    }
                    (LUA_VNUMFLT, LUA_VNUMFLT) => {
                        let x = lhs;
                        let y = rhs;
                        (flt_op(bcx, x, y), LUA_VNUMFLT)
                    }
                    _ => todo!(),
                };

                self.set_reg_typed(a, value, result_tt, bcx);
                self.writeback_reg_if_last_def(bcx, tr_idx, a);
            }

            /// Float-only arithmetic between two registers.
            fn arith_f<'f>(
                &mut self,
                bcx: &mut FunctionBuilder<'f>,
                block: &mut Block,
                tr_idx: u32,
                a: u32,
                b: u32,
                b_tt: u8,
                c: u32,
                c_tt: u8,
                flt_op: impl FnOnce(&mut FunctionBuilder<'f>, Value, Value) -> Value,
            ) {
                let lhs = self.arith_value(bcx, block, b, b_tt);
                let lhs = if b_tt == LUA_VNUMINT {
                    bcx.ins().fcvt_from_sint(LUA_NUM, lhs)
                } else {
                    lhs
                };

                let rhs = self.arith_value(bcx, block, c, c_tt);
                let rhs = if b_tt == LUA_VNUMINT {
                    bcx.ins().fcvt_from_sint(LUA_NUM, rhs)
                } else {
                    rhs
                };

                let value = flt_op(bcx, lhs, rhs);

                self.set_reg_typed(a, value, LUA_VNUMFLT, bcx);
                self.writeback_reg_if_last_def(bcx, tr_idx, a);
            }

            fn bitwise_k<'f>(
                &mut self,
                bcx: &mut FunctionBuilder<'f>,
                block: &mut Block,
                ksts: *mut TValue,
                tr_idx: u32,
                a: u32,
                b: u32,
                b_tt: u8,
                c: u32,
                int_op: impl FnOnce(&mut FunctionBuilder<'f>, Value, i64) -> Value,
            ) {
                let lhs = self.arith_value(bcx, block, b, b_tt);

                let lhs = match b_tt {
                    LUA_VNUMINT => lhs,
                    // TODO: This can bail if the number has no integer representation
                    LUA_VNUMFLT => todo!("generate float to exact int codepath"),
                    _ => unreachable!(),
                };

                let kst = unsafe { ksts.add(c as usize) };

                self.set_reg_typed(
                    a,
                    int_op(bcx, lhs, unsafe { (*kst).value_.i }),
                    LUA_VNUMINT,
                    bcx,
                );
                self.writeback_reg_if_last_def(bcx, tr_idx, a);
            }

            /// Get a value ptr from the table via integer key.
            ///
            /// `slot_check_block` must accept a single ptr as a block argument.
            ///
            /// Must switch to another block after this call.
            fn table_getint(
                &mut self,
                bcx: &mut FunctionBuilder<'_>,
                ext_luaH_getint: &mut ExtFuncRef<'_>,
                slot_check_block: Block,
                table: Value,
                key: Value,
            ) {
                let fastget_block = bcx.create_block();
                let getint_block = bcx.create_block();

                // luaV_fastgeti fast path first:
                let alimit = bcx.ins().load(
                    types::I32,
                    MemFlags::trusted(),
                    table,
                    offset_of!(Table, alimit) as i32,
                );
                let alimit = bcx.ins().uextend(LUA_INT, alimit);

                let key_index = bcx.ins().iadd_imm(key, -1);

                let cond = bcx.ins().icmp(IntCC::UnsignedLessThan, key_index, alimit);

                bcx.ins().brif(cond, fastget_block, [], getint_block, []);

                {
                    bcx.switch_to_block(fastget_block);

                    let key_offset = bcx.ins().imul_imm(key_index, size_of::<TValue>() as i64);

                    let array = bcx.ins().load(
                        PTR_TYPE,
                        MemFlags::trusted(),
                        table,
                        offset_of!(Table, array) as i32,
                    );
                    let fastget_value_ptr = bcx.ins().iadd(array, key_offset);

                    bcx.ins()
                        .jump(slot_check_block, &[BlockArg::Value(fastget_value_ptr)]);
                }

                {
                    bcx.switch_to_block(getint_block);

                    // self.writeback_reg(&mut bcx, c);
                    // let reg_offset = self.reg_offset(c);

                    // let reg = bcx.ins().iadd_imm(cx.arg_base, reg_offset as i64);

                    let call_inst = ext_luaH_getint.call(bcx, &[table, key]);
                    let getint_value_ptr = bcx.inst_results(call_inst)[0];

                    bcx.ins()
                        .jump(slot_check_block, &[BlockArg::Value(getint_value_ptr)]);
                }
            }
        }

        let mut cx = TraceContext {
            L,
            g,
            arg_base,
            arg_state,
            arg_call_info,
            arg_closure,
            regs: DropGuard::new(g, LVec32::new()),
            last_def_use: &tr.def_use.last_def_use,
            side_trace_dirty: true,
            trace_depth,
            last_side_trace: None,
            pc: ptr::null(),
            tailcall_sig,
        };

        let mut emit_epilogue = true;

        for reg_tt in type_info.iter().rev() {
            cx.set_reg_static_type(reg_tt.reg, reg_tt.tt);
        }

        // Replay trace into block
        for (tr_idx, ti) in tr.inst_buffer.iter().enumerate() {
            let tr_idx = tr_idx as u32;
            let closure = tr.closure.unwrap().as_ptr();
            let ksts = (*(*closure).p).k;

            let i = *ti.pc;

            let k = getarg_k(i);
            let a = getarg_a(i);
            let b = getarg_b(i);
            let c = getarg_c(i);
            let sc = getarg_sc(i);
            let bx = getarg_bx(i);
            let sbx = getarg_sbx(i);

            cx.side_trace_dirty = true;
            cx.pc = ti.pc;

            match get_opcode(i) {
                // Constant Loads:
                OP_LOADI => {
                    cx.set_reg_typed(
                        a,
                        bcx.ins().iconst(LUA_INT, sbx as i64),
                        LUA_VNUMINT,
                        &mut bcx,
                    );
                    cx.writeback_reg_if_last_def(&mut bcx, tr_idx, a);
                }
                OP_LOADF => {
                    cx.set_reg_typed(a, bcx.ins().f64const(sbx as f64), LUA_VNUMFLT, &mut bcx);
                    cx.writeback_reg_if_last_def(&mut bcx, tr_idx, a);
                }
                OP_LOADK => {
                    let kst = ksts.add(bx as usize);

                    cx.set_reg_typed(
                        a,
                        bcx.ins().iconst(PTR_TYPE, (*kst).value_.i),
                        (*kst).tt_,
                        &mut bcx,
                    );
                    cx.writeback_reg_if_last_def(&mut bcx, tr_idx, a);
                }
                OP_LOADKX => {
                    todo!("inline constant")
                }
                OP_LOADFALSE | OP_LFALSESKIP => {
                    cx.set_reg_typed(
                        a,
                        // TODO: Allow "none" value (for undef)
                        bcx.ins().iconst(LUA_INT, 0),
                        LUA_VFALSE,
                        &mut bcx,
                    );
                    cx.writeback_reg_if_last_def(&mut bcx, tr_idx, a);
                }
                OP_LOADTRUE => {
                    cx.set_reg_typed(
                        a,
                        // TODO: Allow "none" value (for undef)
                        bcx.ins().iconst(LUA_INT, 0),
                        LUA_VTRUE,
                        &mut bcx,
                    );
                    cx.writeback_reg_if_last_def(&mut bcx, tr_idx, a);
                }
                OP_LOADNIL => {
                    for i in a..=(a + b) {
                        cx.set_reg_typed(
                            i,
                            // TODO: Allow "none" value (for undef)
                            bcx.ins().iconst(LUA_INT, 0),
                            LUA_VNIL,
                            &mut bcx,
                        );
                        cx.writeback_reg_if_last_def(&mut bcx, tr_idx, i);
                    }
                }
                // Unary-ish:
                OP_MOVE => {
                    // TODO: Lazy load `R[a]` when overwriting `R[b]` by tracking register aliases
                    let (value, tt) = cx.get_reg_value_and_tt(b, CgValueType::Any, &mut bcx);
                    let static_tt = cx.reg_static_type(b);
                    cx.set_reg(
                        a,
                        CgTValue {
                            value: Some(value),
                            tt: Some(tt),
                        },
                    );
                    cx.set_reg_static_type(a, static_tt);
                    cx.writeback_reg_if_last_def(&mut bcx, tr_idx, a);
                }
                OP_ADDI => {
                    let [tt, _] = ti.arg_tt;

                    let lhs = cx.arith_value(&mut bcx, &mut block, b, tt);

                    let (value, result_tt) = match tt {
                        LUA_VNUMINT => (bcx.ins().iadd_imm(lhs, sc as i64), LUA_VNUMINT),
                        LUA_VNUMFLT => {
                            let rhs = bcx.ins().f64const(sc as f64);
                            (bcx.ins().fadd(lhs, rhs), LUA_VNUMFLT)
                        }
                        _ => unreachable!(),
                    };

                    cx.set_reg_typed(a, value, result_tt, &mut bcx);
                    cx.writeback_reg_if_last_def(&mut bcx, tr_idx, a);
                }
                OP_ADDK => {
                    let [tt, _] = ti.arg_tt;

                    cx.arith_k(
                        &mut bcx,
                        &mut block,
                        ksts,
                        tr_idx,
                        a,
                        b,
                        tt,
                        c,
                        |bcx, x, y| bcx.ins().iadd_imm(x, y),
                        |bcx, x, y| bcx.ins().fadd(x, y),
                    );
                }
                OP_SUBK => {
                    let [tt, _] = ti.arg_tt;

                    cx.arith_k(
                        &mut bcx,
                        &mut block,
                        ksts,
                        tr_idx,
                        a,
                        b,
                        tt,
                        c,
                        |bcx, x, y| {
                            let y = bcx.ins().iconst(LUA_INT, y);
                            bcx.ins().isub(x, y)
                        },
                        |bcx, x, y| bcx.ins().fsub(x, y),
                    );
                }
                OP_MULK => {
                    let [tt, _] = ti.arg_tt;

                    cx.arith_k(
                        &mut bcx,
                        &mut block,
                        ksts,
                        tr_idx,
                        a,
                        b,
                        tt,
                        c,
                        |bcx, x, y| bcx.ins().imul_imm(x, y),
                        |bcx, x, y| bcx.ins().fmul(x, y),
                    );
                }
                // TODO: OP_MODK
                OP_POWK => {
                    let [tt, _] = ti.arg_tt;

                    let lhs = cx.arith_value(&mut bcx, &mut block, b, tt);

                    let lhs = match tt {
                        LUA_VNUMINT => bcx.ins().fcvt_from_sint(LUA_NUM, lhs),
                        LUA_VNUMFLT => lhs,
                        _ => unreachable!(),
                    };

                    let kst = unsafe { ksts.add(c as usize) };
                    let exp = if (*kst).tt_ == LUA_VNUMINT {
                        (*kst).value_.i as lua_Number
                    } else {
                        (*kst).value_.n
                    };

                    let value = if exp == 0.5 {
                        bcx.ins().sqrt(lhs)
                    } else {
                        let rhs = bcx.ins().f64const(exp);

                        let call = ext_powf.call(&mut bcx, &[lhs, rhs]);
                        bcx.inst_results(call)[0]
                    };

                    cx.set_reg_typed(a, value, LUA_VNUMFLT, &mut bcx);
                    cx.writeback_reg_if_last_def(&mut bcx, tr_idx, a);
                }
                // TODO: OP_DIVK
                // TODO: OP_IDIVK
                OP_BANDK => {
                    let [tt, _] = ti.arg_tt;

                    cx.bitwise_k(
                        &mut bcx,
                        &mut block,
                        ksts,
                        tr_idx,
                        a,
                        b,
                        tt,
                        c,
                        |bcx, x, y| bcx.ins().band_imm(x, y),
                    );
                }
                OP_BORK => {
                    let [tt, _] = ti.arg_tt;

                    cx.bitwise_k(
                        &mut bcx,
                        &mut block,
                        ksts,
                        tr_idx,
                        a,
                        b,
                        tt,
                        c,
                        |bcx, x, y| bcx.ins().bor_imm(x, y),
                    );
                }
                OP_BXORK => {
                    let [tt, _] = ti.arg_tt;

                    cx.bitwise_k(
                        &mut bcx,
                        &mut block,
                        ksts,
                        tr_idx,
                        a,
                        b,
                        tt,
                        c,
                        |bcx, x, y| bcx.ins().bxor_imm(x, y),
                    );
                }
                OP_SHRI => {
                    let shl = sc.is_negative();
                    let shift = sc.unsigned_abs();

                    let [tt, _] = ti.arg_tt;

                    let lhs = cx.arith_value(&mut bcx, &mut block, b, tt);

                    let lhs = match tt {
                        LUA_VNUMINT => lhs,
                        // TODO: This can bail if the number has no integer representation
                        LUA_VNUMFLT => todo!(),
                        _ => unreachable!(),
                    };

                    cx.set_reg_typed(
                        a,
                        // TODO: If the shift is larger than the size of the type, load const zero.
                        if shl {
                            bcx.ins().ishl_imm(lhs, shift as i64)
                        } else {
                            bcx.ins().sshr_imm(lhs, shift as i64)
                        },
                        LUA_VNUMINT,
                        &mut bcx,
                    );
                    cx.writeback_reg_if_last_def(&mut bcx, tr_idx, a);
                }
                OP_ADD => {
                    let [b_tt, c_tt] = ti.arg_tt;

                    cx.arith(
                        &mut bcx,
                        &mut block,
                        tr_idx,
                        a,
                        b,
                        b_tt,
                        c,
                        c_tt,
                        |bcx, x, y| bcx.ins().iadd(x, y),
                        |bcx, x, y| bcx.ins().fadd(x, y),
                    );
                }
                OP_SUB => {
                    let [b_tt, c_tt] = ti.arg_tt;

                    cx.arith(
                        &mut bcx,
                        &mut block,
                        tr_idx,
                        a,
                        b,
                        b_tt,
                        c,
                        c_tt,
                        |bcx, x, y| bcx.ins().isub(x, y),
                        |bcx, x, y| bcx.ins().fsub(x, y),
                    );
                }
                OP_MUL => {
                    let [b_tt, c_tt] = ti.arg_tt;

                    cx.arith(
                        &mut bcx,
                        &mut block,
                        tr_idx,
                        a,
                        b,
                        b_tt,
                        c,
                        c_tt,
                        |bcx, x, y| bcx.ins().imul(x, y),
                        |bcx, x, y| bcx.ins().fmul(x, y),
                    );
                }
                // TODO: OP_MOD
                // TODO: OP_POW
                OP_DIV => {
                    let [b_tt, c_tt] = ti.arg_tt;

                    cx.arith_f(
                        &mut bcx,
                        &mut block,
                        tr_idx,
                        a,
                        b,
                        b_tt,
                        c,
                        c_tt,
                        |bcx, x, y| bcx.ins().fdiv(x, y),
                    );
                }
                OP_BXOR => {
                    let [b_tt, c_tt] = ti.arg_tt;

                    let lhs = cx.arith_value(&mut bcx, &mut block, b, b_tt);

                    let lhs = match b_tt {
                        LUA_VNUMINT => lhs,
                        // TODO: This can bail if the number has no integer representation
                        LUA_VNUMFLT => todo!(),
                        _ => unreachable!(),
                    };

                    let rhs = cx.arith_value(&mut bcx, &mut block, c, c_tt);

                    let rhs = match c_tt {
                        LUA_VNUMINT => rhs,
                        // TODO: This can bail if the number has no integer representation
                        LUA_VNUMFLT => todo!(),
                        _ => unreachable!(),
                    };

                    let (value, result_tt) = match (b_tt, c_tt) {
                        (LUA_VNUMINT, LUA_VNUMINT) => (bcx.ins().bxor(lhs, rhs), LUA_VNUMINT),
                        _ => todo!(),
                    };

                    cx.set_reg_typed(a, value, result_tt, &mut bcx);
                    cx.writeback_reg_if_last_def(&mut bcx, tr_idx, a);
                }
                // Tables
                OP_GETI => {
                    let table = cx.get_reg_type_guarded(b, ctb(LUA_VTABLE), &mut bcx, &mut block);

                    let key = bcx.ins().iconst(LUA_INT, c as i64);

                    let slot_check_block = bcx.create_block();
                    let bail_block = bcx.create_block();
                    bcx.set_cold_block(bail_block);
                    let continue_block = bcx.create_block();

                    // Phi coming from `fastget_block` and `getint_block`
                    bcx.append_block_param(slot_check_block, PTR_TYPE);

                    cx.table_getint(&mut bcx, &mut ext_luaH_getint, slot_check_block, table, key);

                    let value_ptr;
                    let tt;

                    {
                        bcx.switch_to_block(slot_check_block);

                        value_ptr = bcx.block_params(slot_check_block)[0];

                        tt = bcx.ins().load(
                            types::I8,
                            MemFlags::trusted(),
                            value_ptr,
                            size_of::<crate::Value>() as i32,
                        );

                        let tt_no_variant = bcx.ins().band_imm(tt, 0x0F);

                        let cond = bcx.ins().icmp_imm(IntCC::Equal, tt_no_variant, 0);

                        bcx.ins().brif(cond, bail_block, [], continue_block, []);
                    }

                    bcx.switch_to_block(bail_block);
                    cx.emit_side_exit(&mut bcx);

                    bcx.switch_to_block(continue_block);
                    block = continue_block;

                    let value = bcx.ins().load(PTR_TYPE, MemFlags::trusted(), value_ptr, 0);

                    cx.set_reg(
                        a,
                        CgTValue {
                            value: Some(value),
                            tt: Some(tt),
                        },
                    );
                    cx.writeback_reg_if_last_def(&mut bcx, tr_idx, a);
                }
                OP_GETTABLE => {
                    let [_, tt_c] = ti.arg_tt;

                    let table = cx.get_reg_type_guarded(b, ctb(LUA_VTABLE), &mut bcx, &mut block);

                    let key = cx.get_reg_type_guarded(c, tt_c, &mut bcx, &mut block);

                    match tt_c {
                        LUA_VNUMINT => {
                            // local slot
                            // if key-1 < table.alimit then
                            //   slot = table.array[key]
                            // else
                            //   slot = luaH_getint(table, key)
                            // end
                            // local tt = slot.tt
                            // if tt == nil then
                            //   bail
                            // end
                            // load slot.value
                            // TODO: Don't load the whole slot immediately?

                            let slot_check_block = bcx.create_block();
                            let bail_block = bcx.create_block();
                            bcx.set_cold_block(bail_block);
                            let continue_block = bcx.create_block();

                            // Phi coming from `fastget_block` and `getint_block`
                            bcx.append_block_param(slot_check_block, PTR_TYPE);

                            cx.table_getint(
                                &mut bcx,
                                &mut ext_luaH_getint,
                                slot_check_block,
                                table,
                                key,
                            );

                            let value_ptr;
                            let tt;

                            {
                                bcx.switch_to_block(slot_check_block);

                                value_ptr = bcx.block_params(slot_check_block)[0];

                                tt = bcx.ins().load(
                                    types::I8,
                                    MemFlags::trusted(),
                                    value_ptr,
                                    size_of::<crate::Value>() as i32,
                                );

                                let tt_no_variant = bcx.ins().band_imm(tt, 0x0F);

                                let cond = bcx.ins().icmp_imm(IntCC::Equal, tt_no_variant, 0);

                                bcx.ins().brif(cond, bail_block, [], continue_block, []);
                            }

                            bcx.switch_to_block(bail_block);
                            cx.emit_side_exit(&mut bcx);

                            bcx.switch_to_block(continue_block);
                            block = continue_block;

                            let value = bcx.ins().load(PTR_TYPE, MemFlags::trusted(), value_ptr, 0);

                            cx.set_reg(
                                a,
                                CgTValue {
                                    value: Some(value),
                                    tt: Some(tt),
                                },
                            );
                            cx.writeback_reg_if_last_def(&mut bcx, tr_idx, a);
                        }
                        // TODO: Branch between table int get fast path and luaH_get
                        _ => todo!(),
                    }
                }

                OP_SETI => {
                    let table = cx.get_reg_type_guarded(a, ctb(LUA_VTABLE), &mut bcx, &mut block);

                    // Lookup the slot, bailing if the current slot value is nil.
                    let value_ptr;
                    {
                        let key = bcx.ins().iconst(LUA_INT, b as i64);

                        let slot_check_block = bcx.create_block();
                        let bail_block = bcx.create_block();
                        bcx.set_cold_block(bail_block);
                        let continue_block = bcx.create_block();

                        // Phi coming from `fastget_block` and `getint_block`
                        bcx.append_block_param(slot_check_block, PTR_TYPE);

                        cx.table_getint(
                            &mut bcx,
                            &mut ext_luaH_getint,
                            slot_check_block,
                            table,
                            key,
                        );

                        let tt;

                        {
                            bcx.switch_to_block(slot_check_block);

                            value_ptr = bcx.block_params(slot_check_block)[0];

                            tt = bcx.ins().load(
                                types::I8,
                                MemFlags::trusted(),
                                value_ptr,
                                size_of::<crate::Value>() as i32,
                            );

                            let tt_no_variant = bcx.ins().band_imm(tt, 0x0F);

                            let cond = bcx.ins().icmp_imm(IntCC::Equal, tt_no_variant, 0);

                            bcx.ins().brif(cond, bail_block, [], continue_block, []);
                        }

                        bcx.switch_to_block(bail_block);
                        cx.emit_side_exit(&mut bcx);

                        bcx.switch_to_block(continue_block);
                        block = continue_block;
                    }

                    let value;
                    let value_tt;

                    let static_tt = if k {
                        // Read constant's type
                        let kst = ksts.add(c as usize);
                        let tt = (*kst).tt_;

                        value = bcx.ins().iconst(PTR_TYPE, (*kst).value_.i);
                        value_tt = bcx.ins().iconst(TT, tt as i64);

                        tt
                    } else {
                        (value, value_tt) = cx.get_reg_value_and_tt(c, CgValueType::Any, &mut bcx);
                        let tt = cx.reg_static_type(c);
                        tt
                    };

                    // Write the value out to value_ptr
                    bcx.ins().store(MemFlags::trusted(), value, value_ptr, 0);
                    bcx.ins().store(
                        MemFlags::trusted(),
                        value_tt,
                        value_ptr,
                        size_of::<crate::Value>() as i32,
                    );

                    if static_tt == 255 {
                        // Unknown type, check if collectable and emit barrierback
                        todo!()
                    } else if (static_tt & BIT_ISCOLLECTABLE) != 0 {
                        // Statically known type is always collectable, emit unconditional barrierback
                        todo!()
                    }

                    // Otherwise don't emit barrierback when the type is known to be non-collectable
                }

                OP_FORLOOP => {
                    // float loops NYI and should not be traced
                    assert_eq!(ti.arg_tt[0], LUA_VNUMINT);

                    // All registers above are dead. No need to writeback.
                    for i in (a + 4)..cx.regs.len() as u32 {
                        if let Some(reg) = &mut cx.regs[i as usize] {
                            reg.writeback = false;
                            reg.known_tt = 255;
                        }
                    }

                    let count = cx.get_reg_value(a + 1, CgValueType::Int, &mut bcx);

                    let cond = bcx.ins().icmp_imm(IntCC::NotEqual, count, 0);

                    let bail_block = bcx.create_block();
                    let continue_block = bcx.create_block();

                    bcx.ins().brif(cond, continue_block, [], bail_block, []);

                    {
                        bcx.switch_to_block(bail_block);

                        cx.pc = ti.pc.add(1);
                        cx.side_trace_dirty = true;
                        cx.emit_side_exit(&mut bcx);
                    }

                    bcx.switch_to_block(continue_block);
                    block = continue_block;

                    cx.set_reg_typed(a + 1, bcx.ins().iadd_imm(count, -1), LUA_VNUMINT, &mut bcx);

                    let idx = cx.get_reg_value(a, CgValueType::Int, &mut bcx);
                    let step = cx.get_reg_value(a + 2, CgValueType::Int, &mut bcx);

                    let next_idx = bcx.ins().iadd(idx, step);

                    cx.set_reg_typed(a, next_idx, LUA_VNUMINT, &mut bcx);
                    cx.set_reg_typed(a + 3, next_idx, LUA_VNUMINT, &mut bcx);

                    cx.writeback_reg_if_last_def(&mut bcx, tr_idx, a + 1);
                    cx.writeback_reg_if_last_def(&mut bcx, tr_idx, a);
                    cx.writeback_reg_if_last_def(&mut bcx, tr_idx, a + 3);

                    // Unconditional side exit :)

                    // Follow the loop to its beginning
                    cx.pc = ti.pc.add(1).offset(-(getarg_bx(i) as isize));
                    cx.side_trace_dirty = true;

                    let side_trace = match cx.side_trace() {
                        Ok(st) => st,
                        Err(err) => err.throw(L),
                    };

                    if cx.pc == tr.inst_buffer[0].pc
                        && trace_depth > 0
                        && &*side_trace.as_ref().type_info == type_info
                    {
                        cx.writeback_regs(&mut bcx);
                        bcx.ins().jump(entry_block, &[]);
                    } else {
                        cx.emit_side_exit(&mut bcx);
                    }

                    emit_epilogue = false;
                }

                OP_FORPREP => {
                    // Only integer loops supported.
                    assert_eq!(ti.arg_tt, [LUA_VNUMINT, LUA_VNUMINT]);

                    // All registers above are dead. No need to writeback.
                    for i in (a + 4)..cx.regs.len() as u32 {
                        if let Some(reg) = &mut cx.regs[i as usize] {
                            reg.writeback = false;
                            reg.known_tt = 255;
                        }
                    }

                    let loop_body_skipped = tr
                        .inst_buffer
                        .get(tr_idx as usize + 1)
                        .is_some_and(|next_ti| ti.pc.add(1) != next_ti.pc);

                    // TODO: This creates FOUR bail blocks, but we only need one.

                    let init = cx.get_reg_type_guarded(a, LUA_VNUMINT, &mut bcx, &mut block);
                    let limit = cx.get_reg_type_guarded(a + 1, LUA_VNUMINT, &mut bcx, &mut block);
                    let step = cx.get_reg_type_guarded(a + 2, LUA_VNUMINT, &mut bcx, &mut block);

                    let bail_block = bcx.create_block();
                    bcx.set_cold_block(bail_block);

                    {
                        bcx.switch_to_block(bail_block);
                        cx.emit_side_exit(&mut bcx);
                        bcx.switch_to_block(block);
                    }

                    let step_zero_cond = bcx.ins().icmp_imm(IntCC::Equal, step, 0);

                    let continue_block = bcx.create_block();
                    bcx.ins()
                        .brif(step_zero_cond, bail_block, [], continue_block, []);
                    block = continue_block;
                    bcx.switch_to_block(block);

                    let skip_loop_block = bcx.create_block();

                    // Assumes for limit is an integer. Otherwise the logic is complex.

                    // Bail depending on the opposite of the loop result.
                    let skip_loop_gt_cond = bcx.ins().icmp(IntCC::SignedGreaterThan, init, limit);
                    let skip_loop_lt_cond = bcx.ins().icmp(IntCC::SignedLessThan, init, limit);

                    let is_step_positive = bcx.ins().icmp_imm(IntCC::SignedGreaterThan, step, 0);

                    let skip_loop_cond =
                        bcx.ins()
                            .select(is_step_positive, skip_loop_gt_cond, skip_loop_lt_cond);

                    let continue_block = bcx.create_block();
                    bcx.ins()
                        .brif(skip_loop_cond, skip_loop_block, [], continue_block, []);

                    if loop_body_skipped {
                        // Loop body was skipped.
                        bcx.switch_to_block(continue_block);
                        // TODO: Emit count and other initialization and side trace to the start of the loop
                        // cx.pc = ti.pc.add(1);
                        cx.pc = ti.pc;
                        cx.side_trace_dirty = true;
                        cx.emit_side_exit(&mut bcx);

                        block = skip_loop_block;
                    } else {
                        bcx.switch_to_block(skip_loop_block);
                        cx.pc = ti.pc.add(1 + bx as usize);
                        cx.side_trace_dirty = true;
                        cx.emit_side_exit(&mut bcx);

                        block = continue_block;
                    }
                    bcx.switch_to_block(block);

                    if !loop_body_skipped {
                        // Compute count
                        let count = {
                            let pos_count_block = bcx.create_block();
                            let neg_count_block = bcx.create_block();
                            let continue_block = bcx.create_block();
                            bcx.append_block_param(continue_block, LUA_INT);

                            bcx.ins().brif(
                                is_step_positive,
                                pos_count_block,
                                [],
                                neg_count_block,
                                [],
                            );

                            {
                                bcx.switch_to_block(pos_count_block);
                                let pos_count = bcx.ins().isub(limit, init);
                                // TODO: Constant propagation to avoid division when step == 1 (which is common)
                                let pos_count = bcx.ins().udiv(pos_count, step);
                                bcx.ins()
                                    .jump(continue_block, [&BlockArg::Value(pos_count)]);
                            }

                            {
                                bcx.switch_to_block(neg_count_block);
                                let neg_count = bcx.ins().isub(init, limit);
                                let neg_step = {
                                    let step = bcx.ins().iadd_imm(step, 1);
                                    let step = bcx.ins().ineg(step);
                                    bcx.ins().iadd_imm(step, 1)
                                };
                                // TODO: Constant propagation to avoid division when step == 1 (which is common)
                                let neg_count = bcx.ins().udiv(neg_count, neg_step);
                                bcx.ins()
                                    .jump(continue_block, [&BlockArg::Value(neg_count)]);
                            }

                            block = continue_block;
                            bcx.switch_to_block(block);
                            bcx.block_params(block)[0]
                        };

                        cx.set_reg_typed(a + 3, init, LUA_VNUMINT, &mut bcx);

                        // Overwrite limit with count
                        cx.set_reg_typed(a + 1, count, LUA_VNUMINT, &mut bcx);

                        cx.writeback_reg_if_last_def(&mut bcx, tr_idx, a + 1);
                        cx.writeback_reg_if_last_def(&mut bcx, tr_idx, a + 3);
                    }
                }

                op => todo!("{op}"),
            }

            // Clear writeback for killed registers
            let mut cursor = killset.upper_bound(std::ops::Bound::Included(&(tr_idx, 0)));
            while let Some((cidx, reg)) = cursor.key() {
                if *cidx != tr_idx {
                    break;
                }
                // Kill register
                if let Some(Some(reg)) = cx.regs.get_mut(*reg as usize) {
                    reg.writeback = false;
                    reg.known_tt = 255;
                }
                cursor.move_next();
            }
        }

        if emit_epilogue {
            // // Flush all modified TValues
            // cx.writeback_regs(&mut bcx);

            // // Return 0 (for successful execution)
            // let result = bcx.ins().iconst(PTR_TYPE, 0);

            // bcx.ins().return_(&[result]);

            cx.pc = tr.last_pc;
            cx.side_trace_dirty = true;
            cx.emit_side_exit(&mut bcx);
        }

        bcx.seal_all_blocks();
        bcx.finalize();

        side_traces = cx.last_side_trace;
    }
    // eprintln!("Pre-opt:\n{}", ctx.func);
    module.define_function(trace_fn, &mut ctx).unwrap();
    eprintln!("Opt:\n{}", ctx.func);
    module.clear_context(&mut ctx);

    module.finalize_definitions().unwrap();

    let trace = module.get_finalized_function(if trace_depth == 0 {
        trampoline_fn
    } else {
        trace_fn
    });

    eprintln!("Trace Depth: {}", trace_depth);
    eprint!("killset: [ ");
    for ((idx, reg), ()) in killset.iter() {
        eprint!("r{reg}@{idx} ");
    }
    eprintln!("]");
    for ti in tr.inst_buffer.iter() {
        eprintln!("{:?}", Instr(*ti.pc));
    }

    let side_trace_count = {
        let mut count = 0;
        let mut side_trace = side_traces;
        while let Some(trace) = side_trace {
            let trace = unsafe { trace.as_ref() };
            eprintln!("Side Trace N-{count}: {{");
            eprintln!("  pc: {:?}", Instr(*trace.pc));
            eprintln!("  type_info: {:?}", trace.type_info);
            eprintln!("}}");
            count += 1;
            side_trace = trace.next;
        }
        count
    };

    eprintln!("Side Traces: {side_trace_count}");
    eprintln!("ITO: {type_info:?}");

    eprintln!();

    Trace {
        jit: module,
        entrypoint: unsafe { std::mem::transmute(trace) },
        last_pc: tr.last_pc,
        depth: trace_depth,
        side_traces,
    }
}
