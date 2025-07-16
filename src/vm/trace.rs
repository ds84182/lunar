use std::{
    mem::offset_of,
    ptr::{self, NonNull},
};

use crate::{
    CallInfo, Instruction, LClosure, LUA_VFALSE, LUA_VNIL, LUA_VNUMFLT, LUA_VNUMINT, LUA_VTABLE,
    LUA_VTRUE, StackValue, StkId, TValue, Table, ctb,
    ldo::luaD_throw,
    lua_Integer, lua_Number, lua_State,
    opcodes::*,
    table::{luaH_get, luaH_getint},
    ttypetag,
    utils::{DropGuard, GlobalState, LVec32},
};

struct TracedInstr {
    pc: *const Instruction,
    // Types of instruction input arguments.
    // TODO: Check L->openupval for aliasing upvalues (including result reg) and record them here
    arg_tt: [u8; 2],
    // TODO: Record (some) values. For ex CALL instructions would insert guards based on function prototype address.
    // TODO: For conditional instructions, record whether the branch was taken or not.
}

pub(crate) struct TraceRecorder {
    pub(super) recording: bool,
    /// Buffer of recorded instructions.
    inst_buffer: LVec32<TracedInstr>,
    // Start buffer position -> closure.
    // Searched via binary search.
    // closure_span: NonNull<[(u32, *const LClosure)]>,
    closure: Option<NonNull<LClosure>>,
    last_pc: *const Instruction,
    trace_out: *mut Option<NonNull<Trace>>,
}

impl TraceRecorder {
    pub(crate) fn new() -> Self {
        Self {
            recording: false,
            inst_buffer: LVec32::new(),
            closure: None,
            last_pc: ptr::null(),
            trace_out: ptr::null_mut(),
        }
    }

    pub(crate) fn begin_recording(&mut self, trace_out: *mut Option<NonNull<Trace>>) {
        self.recording = true;
        self.trace_out = trace_out;
    }

    pub(crate) unsafe fn end_recording(&mut self, L: *mut lua_State) -> Option<()> {
        if !self.recording {
            return None;
        }

        self.recording = false;

        let trace = compile_trace(L, &self);

        self.inst_buffer.clear();
        self.last_pc = ptr::null();
        self.closure = None;
        let trace_ptr = GlobalState::new_unchecked((*L).l_G).alloc();
        if let Some(ptr) = trace_ptr {
            ptr.write(trace);
        }
        self.trace_out.write(trace_ptr);

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

        // TODO: Associate closure with instructions via spans
        self.closure = Some(cl);
        self.last_pc = pc;

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

        if self
            .inst_buffer
            .push(unsafe { GlobalState::new_unchecked((*L).l_G) }, traced)
            .is_err()
        {
            unsafe { luaD_throw(L, 4) };
        }

        true
    }

    // pub(crate) unsafe fn record_end(
    //     &mut self,
    //     L: *mut lua_State,
    //     pc: *const Instruction,
    //     next_pc: *const Instruction,
    //     ci: *const CallInfo,
    //     cl: NonNull<LClosure>,
    // ) {
    //     if !self.recording {
    //         return;
    //     }
    // }

    /// Cancel trace recording due to a thrown error.
    pub(crate) fn cancel(&mut self) {
        self.inst_buffer.clear();
        self.recording = false;
    }
}

// Simple trace compiler:
// Creates a cranelift JIT module, then compiles a simple linear function based off of the trace.
// Value types are checked, and bails back to the interpreter on mismatch.

#[derive(Copy, Clone)]
struct TraceBail {
    pc: *const Instruction,
}

pub(crate) struct Trace {
    jit: cranelift::jit::JITModule,
    pub(super) entrypoint: unsafe extern "C" fn(
        base: StkId,
        L: *mut lua_State,
        ci: *const CallInfo,
        cl: *const LClosure,
    ) -> isize,
    pub(super) last_pc: *const Instruction,
    bail: NonNull<[TraceBail]>,
}

impl Trace {
    pub(super) fn bail(&self, result: isize) -> *const Instruction {
        let index = (-(result + 1)) as usize;
        let bail = unsafe { self.bail.cast::<TraceBail>().add(index).as_ref() };
        bail.pc
    }
}

pub(crate) unsafe fn compile_trace(L: *mut lua_State, tr: &TraceRecorder) -> Trace {
    let g = GlobalState::new_unchecked((*L).l_G);

    use cranelift::codegen::ir::{BlockArg, UserFuncName};
    use cranelift::jit::{JITBuilder, JITModule};
    use cranelift::prelude::*;
    use cranelift_module::{Module, default_libcall_names};

    let mut flag_builder = settings::builder();
    flag_builder.set("use_colocated_libcalls", "false").unwrap();
    // FIXME set back to true once the x64 backend supports it.
    flag_builder.set("is_pic", "false").unwrap();
    flag_builder.set("opt_level", "speed").unwrap();
    let isa_builder = cranelift_native::builder().unwrap_or_else(|msg| {
        panic!("host machine is not supported: {msg}");
    });
    let isa = isa_builder
        .finish(settings::Flags::new(flag_builder))
        .unwrap();
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

    let mut bail = DropGuard::new(g, LVec32::new());

    // TODO: 32-bit support
    let mut sig = module.make_signature();
    sig.params.push(AbiParam::new(PTR_TYPE)); // base
    sig.params.push(AbiParam::new(PTR_TYPE)); // L
    sig.params.push(AbiParam::new(PTR_TYPE)); // ci
    sig.params.push(AbiParam::new(PTR_TYPE)); // cl
    sig.returns.push(AbiParam::new(PTR_TYPE)); // result (0 = ok, negative for bailout)

    let mut ctx = module.make_context();
    let mut func_ctx = FunctionBuilderContext::new();

    let trace_fn = module
        .declare_function("trace", cranelift_module::Linkage::Local, &sig)
        .unwrap();

    ctx.func.signature = sig;
    ctx.func.name = UserFuncName::user(0, trace_fn.as_u32());

    {
        let mut ext_luaH_get = ExtFuncRef::Uninit(&ext_luaH_get);
        let mut ext_luaH_getint = ExtFuncRef::Uninit(&ext_luaH_getint);
        let mut ext_powf = ExtFuncRef::Uninit(&ext_powf);

        let mut bcx = FunctionBuilder::new(&mut ctx.func, &mut func_ctx);
        let mut block = bcx.create_block();

        bcx.switch_to_block(block);
        bcx.append_block_params_for_function_params(block);

        let arg_base = bcx.block_params(block)[0];

        #[derive(Copy, Clone)]
        struct CgTValue {
            value: Value,
            tt: Value,
        }

        #[derive(Copy, Clone)]
        struct CgReg {
            value: CgTValue,
            writeback: bool,
            known_tt: u8,
        }

        struct TraceContext<'a> {
            L: *mut lua_State,
            g: GlobalState,
            arg_base: Value,
            regs: DropGuard<LVec32<Option<CgReg>>>,
            bail: &'a mut LVec32<TraceBail>,
        }

        impl<'a> TraceContext<'a> {
            fn set_reg(&mut self, r: u32, value: CgTValue) {
                if (self.regs.len() as u32) < r + 1 {
                    if let Err(err) = self.regs.resize(self.g, r + 1, None) {
                        unsafe { err.throw(self.L) };
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

                self.regs[r as usize] = Some(CgReg {
                    value: CgTValue {
                        value,
                        tt: bcx.ins().iconst(TT, tt as i64),
                    },
                    writeback: true,
                    known_tt: tt,
                });
            }

            fn get_reg(&mut self, r: u32, bcx: &mut FunctionBuilder<'_>) -> CgTValue {
                if (self.regs.len() as u32) < r + 1 {
                    if let Err(err) = self.regs.resize(self.g, r + 1, None) {
                        unsafe { err.throw(self.L) };
                    }
                }

                let src = &mut self.regs[r as usize];

                src.get_or_insert_with(|| {
                    let offset = (r * size_of::<StackValue>() as u32) as i32;
                    CgReg {
                        value: CgTValue {
                            value: bcx.ins().load(
                                PTR_TYPE,
                                MemFlags::trusted().with_can_move(),
                                self.arg_base,
                                offset,
                            ),
                            tt: bcx.ins().load(
                                types::I8,
                                MemFlags::trusted().with_can_move(),
                                self.arg_base,
                                offset + (size_of::<crate::Value>() as i32),
                            ),
                        },
                        writeback: false,
                        known_tt: 255,
                    }
                })
                .value
            }

            fn set_reg_static_type(&mut self, r: u32, tt: u8) {
                let Some(Some(reg)) = self.regs.get_mut(r as usize) else {
                    return;
                };
                reg.known_tt = tt;
            }

            fn reg_static_type(&self, r: u32) -> u8 {
                let Some(Some(reg)) = self.regs.get(r as usize) else {
                    return 255;
                };
                reg.known_tt
            }

            fn bail_id(&mut self, bail: &mut Option<TraceBail>) -> i64 {
                let Some(bail) = bail.take() else {
                    return -(self.bail.len() as i64);
                };

                if let Err((err, _)) = self.bail.push(self.g, bail) {
                    unsafe { err.throw(self.L) }
                }

                -(self.bail.len() as i64)
            }

            fn reg_offset(&self, r: u32) -> i32 {
                // NOTE: This would change based on the current inlined function, once inlining is implemented.
                (r as u32 * size_of::<StackValue>() as u32) as i32
            }

            /// Flushes all modified TValues to the backing registers.
            fn writeback_regs(&self, bcx: &mut FunctionBuilder<'_>) {
                for (i, reg) in self.regs.iter().enumerate() {
                    let Some(reg) = reg else { continue };

                    if reg.writeback {
                        let offset = (i as u32 * size_of::<StackValue>() as u32) as i32;
                        bcx.ins().store(
                            MemFlags::trusted().with_can_move(),
                            reg.value.value,
                            self.arg_base,
                            offset,
                        );
                        bcx.ins().store(
                            MemFlags::trusted().with_can_move(),
                            reg.value.tt,
                            self.arg_base,
                            offset + (size_of::<crate::Value>() as i32),
                        );
                    }
                }
            }

            /// Writeback a single register to memory, if it needs writeback.
            fn writeback_reg(&self, bcx: &mut FunctionBuilder<'_>, r: u32) {
                let Some(Some(reg)) = self.regs.get(r as usize) else {
                    return;
                };

                if reg.writeback {
                    let offset = self.reg_offset(r);
                    bcx.ins().store(
                        MemFlags::trusted().with_can_move(),
                        reg.value.value,
                        self.arg_base,
                        offset,
                    );
                    bcx.ins().store(
                        MemFlags::trusted().with_can_move(),
                        reg.value.tt,
                        self.arg_base,
                        offset + (size_of::<crate::Value>() as i32),
                    );
                }
            }

            /// Creates a type guard, which bails out to the interpreter on type mismatch.
            fn type_guard(
                &mut self,
                bcx: &mut FunctionBuilder<'_>,
                tt: Value,
                expected_tt: u8,
                bail: &mut Option<TraceBail>,
            ) -> Block {
                let bail_block = bcx.create_block();
                bcx.set_cold_block(bail_block);
                let continue_block = bcx.create_block();

                let cond = bcx.ins().icmp_imm(IntCC::Equal, tt, expected_tt as i64);

                bcx.ins().brif(cond, continue_block, [], bail_block, []);

                bcx.switch_to_block(bail_block);

                self.writeback_regs(bcx);

                let bail_id = self.bail_id(bail);
                let bail_id = bcx.ins().iconst(PTR_TYPE, bail_id);
                bcx.ins().return_(&[bail_id]);

                bcx.switch_to_block(continue_block);
                continue_block
            }

            /// Int or float arithmetic between a register and a constant.
            fn arith_k<'f>(
                &mut self,
                bcx: &mut FunctionBuilder<'f>,
                bail: &mut Option<TraceBail>,
                block: &mut Block,
                ksts: *mut TValue,
                a: u32,
                b: u32,
                b_tt: u8,
                c: u32,
                int_op: impl FnOnce(&mut FunctionBuilder<'f>, Value, i64) -> Value,
                flt_op: impl FnOnce(&mut FunctionBuilder<'f>, Value, Value) -> Value,
            ) {
                let lhs = self.get_reg(b, bcx);

                if self.reg_static_type(b) == 255 {
                    // Register type not known statically, insert type guard.
                    *block = self.type_guard(bcx, lhs.tt, b_tt, bail);
                    self.set_reg_static_type(b, b_tt);
                }

                let lhs = match b_tt {
                    LUA_VNUMINT => lhs,
                    LUA_VNUMFLT => CgTValue {
                        value: bcx.ins().bitcast(LUA_NUM, MemFlags::new(), lhs.value),
                        tt: lhs.tt,
                    },
                    _ => unreachable!(),
                };

                let kst = unsafe { ksts.add(c as usize) };

                let (value, result_tt) = match (b_tt, unsafe { (*kst).tt_ }) {
                    (LUA_VNUMINT, LUA_VNUMINT) => (
                        int_op(bcx, lhs.value, unsafe { (*kst).value_.i }),
                        LUA_VNUMINT,
                    ),
                    (LUA_VNUMINT, LUA_VNUMFLT) => {
                        let x = bcx.ins().fcvt_from_sint(LUA_NUM, lhs.value);
                        let y = bcx.ins().f64const(unsafe { (*kst).value_.n });
                        (flt_op(bcx, x, y), LUA_VNUMFLT)
                    }
                    (LUA_VNUMFLT, LUA_VNUMINT) => {
                        let x = lhs.value;
                        let y = bcx.ins().f64const(unsafe { (*kst).value_.i } as f64);
                        (flt_op(bcx, x, y), LUA_VNUMFLT)
                    }
                    (LUA_VNUMFLT, LUA_VNUMFLT) => {
                        let x = lhs.value;
                        let y = bcx.ins().f64const(unsafe { (*kst).value_.n });
                        (flt_op(bcx, x, y), LUA_VNUMFLT)
                    }
                    _ => todo!(),
                };

                self.set_reg_typed(a, value, result_tt, bcx);
            }

            /// Int or float arithmetic between two registers.
            fn arith<'f>(
                &mut self,
                bcx: &mut FunctionBuilder<'f>,
                bail: &mut Option<TraceBail>,
                block: &mut Block,
                a: u32,
                b: u32,
                b_tt: u8,
                c: u32,
                c_tt: u8,
                int_op: impl FnOnce(&mut FunctionBuilder<'f>, Value, Value) -> Value,
                flt_op: impl FnOnce(&mut FunctionBuilder<'f>, Value, Value) -> Value,
            ) {
                let lhs = self.get_reg(b, bcx);

                if self.reg_static_type(b) == 255 {
                    // Register type not known statically, insert type guard.
                    *block = self.type_guard(bcx, lhs.tt, b_tt, bail);
                    self.set_reg_static_type(b, b_tt);
                }

                let lhs = match b_tt {
                    LUA_VNUMINT => lhs,
                    LUA_VNUMFLT => CgTValue {
                        value: bcx.ins().bitcast(LUA_NUM, MemFlags::new(), lhs.value),
                        tt: lhs.tt,
                    },
                    _ => unreachable!(),
                };

                let rhs = self.get_reg(c, bcx);

                if self.reg_static_type(c) == 255 {
                    // Register type not known statically, insert type guard.
                    *block = self.type_guard(bcx, rhs.tt, c_tt, bail);
                    self.set_reg_static_type(c, c_tt);
                }

                let rhs = match c_tt {
                    LUA_VNUMINT => rhs,
                    LUA_VNUMFLT => CgTValue {
                        value: bcx.ins().bitcast(LUA_NUM, MemFlags::new(), rhs.value),
                        tt: rhs.tt,
                    },
                    _ => unreachable!(),
                };

                let (value, result_tt) = match (b_tt, c_tt) {
                    (LUA_VNUMINT, LUA_VNUMINT) => (int_op(bcx, lhs.value, rhs.value), LUA_VNUMINT),
                    (LUA_VNUMINT, LUA_VNUMFLT) => {
                        let x = bcx.ins().fcvt_from_sint(LUA_NUM, lhs.value);
                        let y = rhs.value;
                        (flt_op(bcx, x, y), LUA_VNUMFLT)
                    }
                    (LUA_VNUMFLT, LUA_VNUMINT) => {
                        let x = lhs.value;
                        let y = bcx.ins().fcvt_from_sint(LUA_NUM, rhs.value);
                        (flt_op(bcx, x, y), LUA_VNUMFLT)
                    }
                    (LUA_VNUMFLT, LUA_VNUMFLT) => {
                        let x = lhs.value;
                        let y = rhs.value;
                        (flt_op(bcx, x, y), LUA_VNUMFLT)
                    }
                    _ => todo!(),
                };

                self.set_reg_typed(a, value, result_tt, bcx);
            }

            /// Float-only arithmetic between two registers.
            fn arith_f<'f>(
                &mut self,
                bcx: &mut FunctionBuilder<'f>,
                bail: &mut Option<TraceBail>,
                block: &mut Block,
                a: u32,
                b: u32,
                b_tt: u8,
                c: u32,
                c_tt: u8,
                flt_op: impl FnOnce(&mut FunctionBuilder<'f>, Value, Value) -> Value,
            ) {
                let lhs = self.get_reg(b, bcx);

                if self.reg_static_type(b) == 255 {
                    // Register type not known statically, insert type guard.
                    *block = self.type_guard(bcx, lhs.tt, b_tt, bail);
                    self.set_reg_static_type(b, b_tt);
                }

                let lhs = match b_tt {
                    LUA_VNUMINT => bcx.ins().fcvt_from_sint(LUA_NUM, lhs.value),
                    LUA_VNUMFLT => bcx.ins().bitcast(LUA_NUM, MemFlags::new(), lhs.value),
                    _ => unreachable!(),
                };

                let rhs = self.get_reg(c, bcx);

                if self.reg_static_type(c) == 255 {
                    // Register type not known statically, insert type guard.
                    *block = self.type_guard(bcx, rhs.tt, c_tt, bail);
                    self.set_reg_static_type(c, c_tt);
                }

                let rhs = match c_tt {
                    LUA_VNUMINT => bcx.ins().fcvt_from_sint(LUA_NUM, rhs.value),
                    LUA_VNUMFLT => bcx.ins().bitcast(LUA_NUM, MemFlags::new(), rhs.value),
                    _ => unreachable!(),
                };

                let value = flt_op(bcx, lhs, rhs);

                self.set_reg_typed(a, value, LUA_VNUMFLT, bcx);
            }

            fn bitwise_k<'f>(
                &mut self,
                bcx: &mut FunctionBuilder<'f>,
                bail: &mut Option<TraceBail>,
                block: &mut Block,
                ksts: *mut TValue,
                a: u32,
                b: u32,
                b_tt: u8,
                c: u32,
                int_op: impl FnOnce(&mut FunctionBuilder<'f>, Value, i64) -> Value,
            ) {
                let lhs = self.get_reg(b, bcx);

                if self.reg_static_type(b) == 255 {
                    // Register type not known statically, insert type guard.
                    *block = self.type_guard(bcx, lhs.tt, b_tt, bail);
                    self.set_reg_static_type(b, b_tt);
                }

                let lhs = match b_tt {
                    LUA_VNUMINT => lhs,
                    // TODO: This can bail if the number has no integer representation
                    LUA_VNUMFLT => todo!("generate float to exact int codepath"),
                    _ => unreachable!(),
                };

                let kst = unsafe { ksts.add(c as usize) };

                self.set_reg_typed(
                    a,
                    int_op(bcx, lhs.value, unsafe { (*kst).value_.i }),
                    LUA_VNUMINT,
                    bcx,
                );
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
            regs: DropGuard::new(g, LVec32::new()),
            bail: &mut bail,
        };

        // Replay trace into block
        for ti in tr.inst_buffer.iter() {
            let closure = tr.closure.unwrap().as_ptr();
            let ksts = (*(*closure).p).k;

            let i = *ti.pc;

            let a = getarg_a(i);
            let b = getarg_b(i);
            let c = getarg_c(i);
            let sc = getarg_sc(i);
            let bx = getarg_bx(i);
            let sbx = getarg_sbx(i);

            let mut bail = Some(TraceBail { pc: ti.pc });

            match get_opcode(i) {
                // Constant Loads:
                OP_LOADI => {
                    cx.set_reg_typed(
                        a,
                        bcx.ins().iconst(LUA_INT, sbx as i64),
                        LUA_VNUMINT,
                        &mut bcx,
                    );
                }
                OP_LOADF => {
                    cx.set_reg_typed(a, bcx.ins().f64const(sbx as f64), LUA_VNUMFLT, &mut bcx);
                }
                OP_LOADK => {
                    let kst = ksts.add(bx as usize);

                    cx.set_reg_typed(
                        a,
                        bcx.ins().iconst(PTR_TYPE, (*kst).value_.i),
                        (*kst).tt_,
                        &mut bcx,
                    );
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
                }
                OP_LOADTRUE => {
                    cx.set_reg_typed(
                        a,
                        // TODO: Allow "none" value (for undef)
                        bcx.ins().iconst(LUA_INT, 0),
                        LUA_VTRUE,
                        &mut bcx,
                    );
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
                    }
                }
                // Unary-ish:
                OP_MOVE => {
                    // TODO: Propagate static type
                    let src = cx.get_reg(b, &mut bcx);
                    cx.set_reg(a, src);
                }
                OP_ADDI => {
                    let [tt, _] = ti.arg_tt;

                    // TODO: Check type and bail
                    let lhs = cx.get_reg(b, &mut bcx);

                    if cx.reg_static_type(b) == 255 {
                        // Register type not known statically, insert type guard.
                        block = cx.type_guard(&mut bcx, lhs.tt, tt, &mut bail);
                        cx.set_reg_static_type(b, tt);
                    }

                    let lhs = match tt {
                        LUA_VNUMINT => lhs,
                        LUA_VNUMFLT => CgTValue {
                            value: bcx.ins().bitcast(LUA_NUM, MemFlags::new(), lhs.value),
                            tt: lhs.tt,
                        },
                        _ => unreachable!(),
                    };

                    let (value, result_tt) = match tt {
                        LUA_VNUMINT => (bcx.ins().iadd_imm(lhs.value, sc as i64), LUA_VNUMINT),
                        LUA_VNUMFLT => {
                            let rhs = bcx.ins().f64const(sc as f64);
                            (bcx.ins().fadd(lhs.value, rhs), LUA_VNUMFLT)
                        }
                        _ => unreachable!(),
                    };

                    cx.set_reg_typed(a, value, result_tt, &mut bcx);
                }
                OP_ADDK => {
                    let [tt, _] = ti.arg_tt;

                    cx.arith_k(
                        &mut bcx,
                        &mut bail,
                        &mut block,
                        ksts,
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
                        &mut bail,
                        &mut block,
                        ksts,
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
                        &mut bail,
                        &mut block,
                        ksts,
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

                    let lhs = cx.get_reg(b, &mut bcx);

                    if cx.reg_static_type(b) == 255 {
                        // Register type not known statically, insert type guard.
                        block = cx.type_guard(&mut bcx, lhs.tt, tt, &mut bail);
                        cx.set_reg_static_type(b, tt);
                    }

                    let lhs = match tt {
                        LUA_VNUMINT => bcx.ins().fcvt_from_sint(LUA_NUM, lhs.value),
                        LUA_VNUMFLT => bcx.ins().bitcast(LUA_NUM, MemFlags::new(), lhs.value),
                        _ => unreachable!(),
                    };

                    let kst = unsafe { ksts.add(c as usize) };

                    let rhs = bcx.ins().f64const(if (*kst).tt_ == LUA_VNUMINT {
                        (*kst).value_.i as lua_Number
                    } else {
                        (*kst).value_.n
                    });

                    let call = ext_powf.call(&mut bcx, &[lhs, rhs]);
                    let value = bcx.inst_results(call)[0];

                    cx.set_reg_typed(a, value, LUA_VNUMFLT, &mut bcx);
                }
                // TODO: OP_DIVK
                // TODO: OP_IDIVK
                OP_BANDK => {
                    let [tt, _] = ti.arg_tt;

                    cx.bitwise_k(
                        &mut bcx,
                        &mut bail,
                        &mut block,
                        ksts,
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
                        &mut bail,
                        &mut block,
                        ksts,
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
                        &mut bail,
                        &mut block,
                        ksts,
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

                    let lhs = cx.get_reg(b, &mut bcx);

                    if cx.reg_static_type(b) == 255 {
                        // Register type not known statically, insert type guard.
                        block = cx.type_guard(&mut bcx, lhs.tt, tt, &mut bail);
                        cx.set_reg_static_type(b, tt);
                    }

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
                            bcx.ins().ishl_imm(lhs.value, shift as i64)
                        } else {
                            bcx.ins().sshr_imm(lhs.value, shift as i64)
                        },
                        LUA_VNUMINT,
                        &mut bcx,
                    );
                }
                OP_ADD => {
                    let [b_tt, c_tt] = ti.arg_tt;

                    cx.arith(
                        &mut bcx,
                        &mut bail,
                        &mut block,
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
                        &mut bail,
                        &mut block,
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
                        &mut bail,
                        &mut block,
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
                        &mut bail,
                        &mut block,
                        a,
                        b,
                        b_tt,
                        c,
                        c_tt,
                        |bcx, x, y| bcx.ins().fdiv(x, y),
                    );
                }
                OP_BXOR => {
                    let [tt_a, tt_b] = ti.arg_tt;

                    let lhs = cx.get_reg(b, &mut bcx);

                    if cx.reg_static_type(b) == 255 {
                        // Register type not known statically, insert type guard.
                        block = cx.type_guard(&mut bcx, lhs.tt, tt_a, &mut bail);
                        cx.set_reg_static_type(b, tt_a);
                    }

                    let lhs = match tt_a {
                        LUA_VNUMINT => lhs,
                        // TODO: This can bail if the number has no integer representation
                        LUA_VNUMFLT => todo!(),
                        _ => unreachable!(),
                    };

                    let rhs = cx.get_reg(c, &mut bcx);

                    if cx.reg_static_type(c) == 255 {
                        // Register type not known statically, insert type guard.
                        block = cx.type_guard(&mut bcx, rhs.tt, tt_b, &mut bail);
                        cx.set_reg_static_type(c, tt_b);
                    }

                    let rhs = match tt_b {
                        LUA_VNUMINT => rhs,
                        // TODO: This can bail if the number has no integer representation
                        LUA_VNUMFLT => todo!(),
                        _ => unreachable!(),
                    };

                    let (value, result_tt) = match (tt_a, tt_b) {
                        (LUA_VNUMINT, LUA_VNUMINT) => {
                            (bcx.ins().bxor(lhs.value, rhs.value), LUA_VNUMINT)
                        }
                        _ => todo!(),
                    };

                    cx.set_reg_typed(a, value, result_tt, &mut bcx);
                }
                // Tables
                OP_GETI => {
                    let table = cx.get_reg(b, &mut bcx);

                    if cx.reg_static_type(b) == 255 {
                        // Register type not known statically, insert type guard.
                        block = cx.type_guard(&mut bcx, table.tt, ctb(LUA_VTABLE), &mut bail);
                        cx.set_reg_static_type(b, ctb(LUA_VTABLE));
                    }

                    let key = bcx.ins().iconst(LUA_INT, c as i64);

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
                        table.value,
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

                    {
                        bcx.switch_to_block(bail_block);

                        cx.writeback_regs(&mut bcx);

                        let bail_id = cx.bail_id(&mut bail);
                        let bail_id = bcx.ins().iconst(PTR_TYPE, bail_id);
                        bcx.ins().return_(&[bail_id]);
                    }

                    bcx.switch_to_block(continue_block);
                    block = continue_block;

                    let value = bcx.ins().load(PTR_TYPE, MemFlags::trusted(), value_ptr, 0);

                    cx.set_reg(a, CgTValue { value, tt });
                }
                OP_GETTABLE => {
                    let [_, tt_c] = ti.arg_tt;

                    let table = cx.get_reg(b, &mut bcx);

                    if cx.reg_static_type(b) == 255 {
                        // Register type not known statically, insert type guard.
                        block = cx.type_guard(&mut bcx, table.tt, ctb(LUA_VTABLE), &mut bail);
                        cx.set_reg_static_type(b, ctb(LUA_VTABLE));
                    }

                    let key = cx.get_reg(c, &mut bcx);

                    if cx.reg_static_type(c) == 255 {
                        // Register type not known statically, insert type guard.
                        block = cx.type_guard(&mut bcx, key.tt, tt_c, &mut bail);
                        cx.set_reg_static_type(c, tt_c);
                    }

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
                                table.value,
                                key.value,
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

                            {
                                bcx.switch_to_block(bail_block);

                                cx.writeback_regs(&mut bcx);

                                let bail_id = cx.bail_id(&mut bail);
                                let bail_id = bcx.ins().iconst(PTR_TYPE, bail_id);
                                bcx.ins().return_(&[bail_id]);
                            }

                            bcx.switch_to_block(continue_block);
                            block = continue_block;

                            let value = bcx.ins().load(PTR_TYPE, MemFlags::trusted(), value_ptr, 0);

                            cx.set_reg(a, CgTValue { value, tt });
                        }
                        // TODO: Branch between table int get fast path and luaH_get
                        _ => todo!(),
                    }
                }

                op => todo!("{op}"),
            }
        }

        // Flush all modified TValues
        cx.writeback_regs(&mut bcx);

        // Return 0 (for successful execution)
        let result = bcx.ins().iconst(PTR_TYPE, 0);
        bcx.ins().return_(&[result]);

        bcx.seal_all_blocks();
        bcx.finalize();
    }
    eprintln!("Pre-opt:\n{}", ctx.func);
    module.define_function(trace_fn, &mut ctx).unwrap();
    eprintln!("Opt:\n{}", ctx.func);
    module.clear_context(&mut ctx);

    module.finalize_definitions().unwrap();

    let trace = module.get_finalized_function(trace_fn);

    let Ok(bail) = DropGuard::into_inner(bail).into_boxed_slice(g) else {
        unsafe { luaD_throw(L, 4) }
    };

    eprintln!("Trace has {} bail outs", bail.as_ptr().len());
    eprintln!();

    Trace {
        jit: module,
        entrypoint: unsafe { std::mem::transmute(trace) },
        last_pc: tr.last_pc,
        bail: bail.into_ptr(),
    }
}
