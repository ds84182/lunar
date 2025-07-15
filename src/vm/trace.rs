use std::ptr::{self, NonNull};

use crate::{
    CallInfo, Instruction, LClosure, LUA_VFALSE, LUA_VNIL, LUA_VNUMFLT, LUA_VNUMINT, LUA_VTRUE,
    StackValue, StkId,
    ldo::luaD_throw,
    lua_Integer, lua_Number, lua_State,
    opcodes::*,
    ttypetag,
    utils::{DropGuard, GlobalState, LVec32},
};

struct TracedInstr {
    pc: *const Instruction,
    // Types of instruction input arguments.
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

pub(crate) struct Trace {
    jit: cranelift::jit::JITModule,
    pub(super) entrypoint: unsafe extern "C" fn(
        base: StkId,
        L: *mut lua_State,
        ci: *const CallInfo,
        cl: *const LClosure,
    ) -> i32,
    pub(super) last_pc: *const Instruction,
}

pub(crate) unsafe fn compile_trace(L: *mut lua_State, tr: &TraceRecorder) -> Trace {
    let g = GlobalState::new_unchecked((*L).l_G);

    use cranelift::codegen::ir::UserFuncName;
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
        let mut bcx = FunctionBuilder::new(&mut ctx.func, &mut func_ctx);
        let block = bcx.create_block();

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

        struct TraceContext {
            L: *mut lua_State,
            g: GlobalState,
            arg_base: Value,
            regs: DropGuard<LVec32<Option<CgReg>>>,
        }

        impl TraceContext {
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
        }

        let mut cx = TraceContext {
            L,
            g,
            arg_base,
            regs: DropGuard::new(g, LVec32::new()),
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

            match get_opcode(i) {
                // Constant Loads:
                OP_LOADI => {
                    cx.set_reg(
                        a,
                        CgTValue {
                            value: bcx.ins().iconst(LUA_INT, sbx as i64),
                            tt: bcx.ins().iconst(TT, LUA_VNUMINT as i64),
                        },
                    );
                }
                OP_LOADF => {
                    cx.set_reg(
                        a,
                        CgTValue {
                            value: bcx.ins().f64const(sbx as f64),
                            tt: bcx.ins().iconst(TT, LUA_VNUMFLT as i64),
                        },
                    );
                }
                OP_LOADK => {
                    let kst = ksts.add(bx as usize);

                    cx.set_reg(
                        a,
                        CgTValue {
                            value: bcx.ins().iconst(PTR_TYPE, (*kst).value_.i),
                            tt: bcx.ins().iconst(TT, (*kst).tt_ as i64),
                        },
                    );
                }
                OP_LOADKX => {
                    todo!("inline constant")
                }
                OP_LOADFALSE | OP_LFALSESKIP => {
                    cx.set_reg(
                        a,
                        CgTValue {
                            // TODO: Allow "none" value (for undef)
                            value: bcx.ins().iconst(LUA_INT, 0),
                            tt: bcx.ins().iconst(TT, LUA_VFALSE as i64),
                        },
                    );
                }
                OP_LOADTRUE => {
                    cx.set_reg(
                        a,
                        CgTValue {
                            // TODO: Allow "none" value (for undef)
                            value: bcx.ins().iconst(LUA_INT, 0),
                            tt: bcx.ins().iconst(TT, LUA_VTRUE as i64),
                        },
                    );
                }
                OP_LOADNIL => {
                    for i in a..=(a + b) {
                        cx.set_reg(
                            i,
                            CgTValue {
                                // TODO: Allow "none" value (for undef)
                                value: bcx.ins().iconst(LUA_INT, 0),
                                tt: bcx.ins().iconst(TT, LUA_VNIL as i64),
                            },
                        );
                    }
                }
                // Unary-ish:
                OP_MOVE => {
                    let src = cx.get_reg(b, &mut bcx);
                    cx.set_reg(a, src);
                }
                OP_ADDI => {
                    let [tt, _] = ti.arg_tt;

                    // TODO: Check type and bail
                    let lhs = cx.get_reg(b, &mut bcx);

                    let lhs = match tt {
                        LUA_VNUMINT => lhs,
                        LUA_VNUMFLT => CgTValue {
                            value: bcx.ins().bitcast(LUA_NUM, MemFlags::new(), lhs.value),
                            tt: lhs.tt,
                        },
                        _ => unreachable!(),
                    };

                    cx.set_reg(
                        a,
                        match tt {
                            LUA_VNUMINT => CgTValue {
                                value: bcx.ins().iadd_imm(lhs.value, sc as i64),
                                tt: bcx.ins().iconst(TT, LUA_VNUMINT as i64),
                            },
                            LUA_VNUMFLT => {
                                let rhs = bcx.ins().f64const(sc as f64);
                                CgTValue {
                                    value: bcx.ins().fadd(lhs.value, rhs),
                                    tt: bcx.ins().iconst(TT, LUA_VNUMFLT as i64),
                                }
                            }
                            _ => unreachable!(),
                        },
                    );
                }
                OP_BANDK => {
                    let [tt, _] = ti.arg_tt;

                    // TODO: Check type and bail
                    let lhs = cx.get_reg(b, &mut bcx);

                    let lhs = match tt {
                        LUA_VNUMINT => lhs,
                        // TODO: This can bail if the number has no integer representation
                        LUA_VNUMFLT => todo!(),
                        _ => unreachable!(),
                    };

                    let kst = ksts.add(c as usize);

                    cx.set_reg(
                        a,
                        CgTValue {
                            value: bcx.ins().band_imm(lhs.value, (*kst).value_.i),
                            tt: bcx.ins().iconst(TT, LUA_VNUMINT as i64),
                        },
                    );
                }
                OP_BXORK => {
                    let [tt, _] = ti.arg_tt;

                    // TODO: Check type and bail
                    let lhs = cx.get_reg(b, &mut bcx);

                    let lhs = match tt {
                        LUA_VNUMINT => lhs,
                        // TODO: This can bail if the number has no integer representation
                        LUA_VNUMFLT => todo!(),
                        _ => unreachable!(),
                    };

                    let kst = ksts.add(c as usize);

                    cx.set_reg(
                        a,
                        CgTValue {
                            value: bcx.ins().bxor_imm(lhs.value, (*kst).value_.i),
                            tt: bcx.ins().iconst(TT, LUA_VNUMINT as i64),
                        },
                    );
                }
                OP_SHRI => {
                    let shl = sc.is_negative();
                    let shift = sc.unsigned_abs();

                    let [tt, _] = ti.arg_tt;

                    // TODO: Check type and bail
                    let lhs = cx.get_reg(b, &mut bcx);

                    let lhs = match tt {
                        LUA_VNUMINT => lhs,
                        // TODO: This can bail if the number has no integer representation
                        LUA_VNUMFLT => todo!(),
                        _ => unreachable!(),
                    };

                    cx.set_reg(
                        a,
                        CgTValue {
                            value: if shl {
                                bcx.ins().ishl_imm(lhs.value, shift as i64)
                            } else {
                                bcx.ins().sshr_imm(lhs.value, shift as i64)
                            },
                            tt: bcx.ins().iconst(TT, LUA_VNUMINT as i64),
                        },
                    );
                }
                OP_MULK => {
                    let [tt, _] = ti.arg_tt;

                    // TODO: Check type and bail
                    let lhs = cx.get_reg(b, &mut bcx);

                    let lhs = match tt {
                        LUA_VNUMINT => lhs,
                        LUA_VNUMFLT => CgTValue {
                            value: bcx.ins().bitcast(LUA_NUM, MemFlags::new(), lhs.value),
                            tt: lhs.tt,
                        },
                        _ => unreachable!(),
                    };

                    let kst = ksts.add(c as usize);

                    cx.set_reg(
                        a,
                        CgTValue {
                            value: match (tt, (*kst).tt_) {
                                (LUA_VNUMINT, LUA_VNUMINT) => {
                                    bcx.ins().imul_imm(lhs.value, (*kst).value_.i)
                                }
                                _ => todo!(),
                            },
                            tt: bcx.ins().iconst(TT, LUA_VNUMINT as i64),
                        },
                    );
                }
                OP_BXOR => {
                    let [tt_a, tt_b] = ti.arg_tt;

                    // TODO: Check type and bail
                    let lhs = cx.get_reg(b, &mut bcx);

                    let lhs = match tt_a {
                        LUA_VNUMINT => lhs,
                        // TODO: This can bail if the number has no integer representation
                        LUA_VNUMFLT => CgTValue {
                            value: bcx.ins().bitcast(LUA_NUM, MemFlags::new(), lhs.value),
                            tt: lhs.tt,
                        },
                        _ => unreachable!(),
                    };

                    // TODO: Check type and bail
                    let rhs = cx.get_reg(c, &mut bcx);

                    let rhs = match tt_b {
                        LUA_VNUMINT => rhs,
                        // TODO: This can bail if the number has no integer representation
                        LUA_VNUMFLT => todo!(),
                        _ => unreachable!(),
                    };

                    cx.set_reg(
                        a,
                        CgTValue {
                            value: match (tt_a, tt_b) {
                                (LUA_VNUMINT, LUA_VNUMINT) => bcx.ins().bxor(lhs.value, rhs.value),
                                _ => todo!(),
                            },
                            tt: bcx.ins().iconst(TT, LUA_VNUMINT as i64),
                        },
                    );
                }
                op => todo!("{op}"),
            }
        }

        // Flush all modified TValues
        for (i, reg) in cx.regs.iter().enumerate() {
            let Some(reg) = reg else { continue };

            if reg.writeback {
                let offset = (i as u32 * size_of::<StackValue>() as u32) as i32;
                bcx.ins().store(
                    MemFlags::trusted().with_can_move(),
                    reg.value.value,
                    cx.arg_base,
                    offset,
                );
                bcx.ins().store(
                    MemFlags::trusted().with_can_move(),
                    reg.value.tt,
                    cx.arg_base,
                    offset + (size_of::<crate::Value>() as i32),
                );
            }
        }

        // Return 0 (for successful execution)
        let result = bcx.ins().iconst(PTR_TYPE, 0);
        bcx.ins().return_(&[result]);

        bcx.seal_all_blocks();
        bcx.finalize();
    }
    ctx.set_disasm(true);
    module.define_function(trace_fn, &mut ctx).unwrap();
    eprintln!("{}", ctx.func);
    module.clear_context(&mut ctx);

    module.finalize_definitions().unwrap();

    let trace = module.get_finalized_function(trace_fn);

    Trace {
        jit: module,
        entrypoint: unsafe { std::mem::transmute(trace) },
        last_pc: tr.last_pc,
    }
}
