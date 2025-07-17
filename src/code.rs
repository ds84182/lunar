use crate::*;

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaK_semerror(
    mut ls: *mut LexState,
    mut msg: *const std::ffi::c_char,
) -> ! {
    (*ls).t.token = 0;
    luaX_syntaxerror(ls, msg);
}
unsafe extern "C-unwind" fn tonumeral(mut e: *const ExpDesc, mut v: *mut TValue) -> i32 {
    if (*e).t != (*e).f {
        return 0;
    }
    match (*e).k as u32 {
        6 => {
            if !v.is_null() {
                let mut io: *mut TValue = v;
                (*io).value_.i = (*e).u.ival;
                (*io).tt_ = (3 as i32 | (0) << 4 as i32) as lu_byte;
            }
            return 1 as i32;
        }
        5 => {
            if !v.is_null() {
                let mut io_0: *mut TValue = v;
                (*io_0).value_.n = (*e).u.nval;
                (*io_0).tt_ = (3 as i32 | (1 as i32) << 4 as i32) as lu_byte;
            }
            return 1 as i32;
        }
        _ => return 0,
    };
}
unsafe extern "C-unwind" fn const2val(
    mut fs: *mut FuncState,
    mut e: *const ExpDesc,
) -> *mut TValue {
    return &mut (*((*(*(*fs).ls).dyd).actvar.arr).offset((*e).u.info as isize)).k;
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaK_exp2const(
    mut fs: *mut FuncState,
    mut e: *const ExpDesc,
    mut v: *mut TValue,
) -> i32 {
    if (*e).t != (*e).f {
        return 0;
    }
    match (*e).k as u32 {
        3 => {
            (*v).tt_ = (1 as i32 | (0) << 4 as i32) as lu_byte;
            return 1 as i32;
        }
        2 => {
            (*v).tt_ = (1 as i32 | (1 as i32) << 4 as i32) as lu_byte;
            return 1 as i32;
        }
        1 => {
            (*v).tt_ = (0 | (0) << 4 as i32) as lu_byte;
            return 1 as i32;
        }
        7 => {
            let mut io: *mut TValue = v;
            let mut x_: *mut TString = (*e).u.strval;
            (*io).value_.gc = &mut (*(x_ as *mut GCUnion)).gc;
            (*io).tt_ = ((*x_).tt as i32 | (1 as i32) << 6 as i32) as lu_byte;
            if (*io).tt_ as i32 & (1 as i32) << 6 as i32 == 0
                || (*io).tt_ as i32 & 0x3f as i32 == (*(*io).value_.gc).tt as i32
                    && (((*(*fs).ls).L).is_null()
                        || (*(*io).value_.gc).marked as i32
                            & ((*(*(*(*fs).ls).L).l_G).currentwhite as i32
                                ^ ((1 as i32) << 3 as i32 | (1 as i32) << 4 as i32))
                            == 0)
            {
            } else {
            };
            return 1 as i32;
        }
        11 => {
            let mut io1: *mut TValue = v;
            let mut io2: *const TValue = const2val(fs, e);
            (*io1).value_ = (*io2).value_;
            (*io1).tt_ = (*io2).tt_;
            if (*io1).tt_ as i32 & (1 as i32) << 6 as i32 == 0
                || (*io1).tt_ as i32 & 0x3f as i32 == (*(*io1).value_.gc).tt as i32
                    && (((*(*fs).ls).L).is_null()
                        || (*(*io1).value_.gc).marked as i32
                            & ((*(*(*(*fs).ls).L).l_G).currentwhite as i32
                                ^ ((1 as i32) << 3 as i32 | (1 as i32) << 4 as i32))
                            == 0)
            {
            } else {
            };
            return 1 as i32;
        }
        _ => return tonumeral(e, v),
    };
}
unsafe extern "C-unwind" fn previousinstruction(mut fs: *mut FuncState) -> *mut Instruction {
    static mut invalidinstruction: Instruction = !(0 as Instruction);
    if (*fs).pc > (*fs).lasttarget {
        return &mut *((*(*fs).f).code).offset(((*fs).pc - 1 as i32) as isize) as *mut Instruction;
    } else {
        return &raw const invalidinstruction as *const Instruction as *mut Instruction;
    };
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaK_nil(mut fs: *mut FuncState, mut from: i32, mut n: i32) {
    let mut l: i32 = from + n - 1 as i32;
    let mut previous: *mut Instruction = previousinstruction(fs);
    if (*previous >> 0 & !(!(0 as Instruction) << 7 as i32) << 0) as OpCode as u32
        == OP_LOADNIL as i32 as u32
    {
        let mut pfrom: i32 =
            (*previous >> 0 + 7 as i32 & !(!(0 as Instruction) << 8 as i32) << 0) as i32;
        let mut pl: i32 = pfrom
            + (*previous >> 0 + 7 as i32 + 8 as i32 + 1 as i32
                & !(!(0 as Instruction) << 8 as i32) << 0) as i32;
        if pfrom <= from && from <= pl + 1 as i32 || from <= pfrom && pfrom <= l + 1 as i32 {
            if pfrom < from {
                from = pfrom;
            }
            if pl > l {
                l = pl;
            }
            *previous = *previous & !(!(!(0 as Instruction) << 8 as i32) << 0 + 7 as i32)
                | (from as Instruction) << 0 + 7 as i32
                    & !(!(0 as Instruction) << 8 as i32) << 0 + 7 as i32;
            *previous = *previous
                & !(!(!(0 as Instruction) << 8 as i32) << 0 + 7 as i32 + 8 as i32 + 1 as i32)
                | ((l - from) as Instruction) << 0 + 7 as i32 + 8 as i32 + 1 as i32
                    & !(!(0 as Instruction) << 8 as i32) << 0 + 7 as i32 + 8 as i32 + 1 as i32;
            return;
        }
    }
    luaK_codeABCk(fs, OP_LOADNIL, from, n - 1 as i32, 0, 0);
}
unsafe extern "C-unwind" fn getjump(mut fs: *mut FuncState, mut pc: i32) -> i32 {
    let mut offset: i32 = (*((*(*fs).f).code).offset(pc as isize) >> 0 + 7 as i32
        & !(!(0 as Instruction) << 8 as i32 + 8 as i32 + 1 as i32 + 8 as i32) << 0)
        as i32
        - (((1 as i32) << 8 as i32 + 8 as i32 + 1 as i32 + 8 as i32) - 1 as i32 >> 1 as i32);
    if offset == -(1 as i32) {
        return -(1 as i32);
    } else {
        return pc + 1 as i32 + offset;
    };
}
unsafe extern "C-unwind" fn fixjump(mut fs: *mut FuncState, mut pc: i32, mut dest: i32) {
    let mut jmp: *mut Instruction = &mut *((*(*fs).f).code).offset(pc as isize) as *mut Instruction;
    let mut offset: i32 = dest - (pc + 1 as i32);
    if !(-(((1 as i32) << 8 as i32 + 8 as i32 + 1 as i32 + 8 as i32) - 1 as i32 >> 1 as i32)
        <= offset
        && offset
            <= ((1 as i32) << 8 as i32 + 8 as i32 + 1 as i32 + 8 as i32)
                - 1 as i32
                - (((1 as i32) << 8 as i32 + 8 as i32 + 1 as i32 + 8 as i32) - 1 as i32
                    >> 1 as i32))
    {
        luaX_syntaxerror((*fs).ls, c"control structure too long".as_ptr());
    }
    *jmp = *jmp
        & !(!(!(0 as Instruction) << 8 as i32 + 8 as i32 + 1 as i32 + 8 as i32) << 0 + 7 as i32)
        | ((offset
            + (((1 as i32) << 8 as i32 + 8 as i32 + 1 as i32 + 8 as i32) - 1 as i32 >> 1 as i32))
            as u32)
            << 0 + 7 as i32
            & !(!(0 as Instruction) << 8 as i32 + 8 as i32 + 1 as i32 + 8 as i32) << 0 + 7 as i32;
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaK_concat(mut fs: *mut FuncState, mut l1: *mut i32, mut l2: i32) {
    if l2 == -(1 as i32) {
        return;
    } else if *l1 == -(1 as i32) {
        *l1 = l2;
    } else {
        let mut list: i32 = *l1;
        let mut next: i32 = 0;
        loop {
            next = getjump(fs, list);
            if !(next != -(1 as i32)) {
                break;
            }
            list = next;
        }
        fixjump(fs, list, l2);
    };
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaK_jump(mut fs: *mut FuncState) -> i32 {
    return codesJ(fs, OP_JMP, -(1 as i32), 0);
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaK_ret(mut fs: *mut FuncState, mut first: i32, mut nret: i32) {
    let mut op: OpCode = OP_MOVE;
    match nret {
        0 => {
            op = OP_RETURN0;
        }
        1 => {
            op = OP_RETURN1;
        }
        _ => {
            op = OP_RETURN;
        }
    }
    luaK_codeABCk(fs, op, first, nret + 1 as i32, 0, 0);
}
unsafe extern "C-unwind" fn condjump(
    mut fs: *mut FuncState,
    mut op: OpCode,
    mut A: i32,
    mut B: i32,
    mut C: i32,
    mut k: i32,
) -> i32 {
    luaK_codeABCk(fs, op, A, B, C, k);
    return luaK_jump(fs);
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaK_getlabel(mut fs: *mut FuncState) -> i32 {
    (*fs).lasttarget = (*fs).pc;
    return (*fs).pc;
}
unsafe extern "C-unwind" fn getjumpcontrol(
    mut fs: *mut FuncState,
    mut pc: i32,
) -> *mut Instruction {
    let mut pi: *mut Instruction = &mut *((*(*fs).f).code).offset(pc as isize) as *mut Instruction;
    if pc >= 1 as i32
        && luaP_opmodes
            [(*pi.offset(-(1)) >> 0 & !(!(0 as Instruction) << 7 as i32) << 0) as OpCode as usize]
            as i32
            & (1 as i32) << 4 as i32
            != 0
    {
        return pi.offset(-(1));
    } else {
        return pi;
    };
}
unsafe extern "C-unwind" fn patchtestreg(
    mut fs: *mut FuncState,
    mut node: i32,
    mut reg: i32,
) -> i32 {
    let mut i: *mut Instruction = getjumpcontrol(fs, node);
    if (*i >> 0 & !(!(0 as Instruction) << 7 as i32) << 0) as OpCode as u32
        != OP_TESTSET as i32 as u32
    {
        return 0;
    }
    if reg != ((1 as i32) << 8 as i32) - 1 as i32
        && reg
            != (*i >> 0 + 7 as i32 + 8 as i32 + 1 as i32 & !(!(0 as Instruction) << 8 as i32) << 0)
                as i32
    {
        *i = *i & !(!(!(0 as Instruction) << 8 as i32) << 0 + 7 as i32)
            | (reg as Instruction) << 0 + 7 as i32
                & !(!(0 as Instruction) << 8 as i32) << 0 + 7 as i32;
    } else {
        *i = (OP_TEST as i32 as Instruction) << 0
            | ((*i >> 0 + 7 as i32 + 8 as i32 + 1 as i32 & !(!(0 as Instruction) << 8 as i32) << 0)
                as i32 as Instruction)
                << 0 + 7 as i32
            | (0 as Instruction) << 0 + 7 as i32 + 8 as i32 + 1 as i32
            | (0 as Instruction) << 0 + 7 as i32 + 8 as i32 + 1 as i32 + 8 as i32
            | ((*i >> 0 + 7 as i32 + 8 as i32 & !(!(0 as Instruction) << 1 as i32) << 0) as i32
                as Instruction)
                << 0 + 7 as i32 + 8 as i32;
    }
    return 1 as i32;
}
unsafe extern "C-unwind" fn removevalues(mut fs: *mut FuncState, mut list: i32) {
    while list != -(1 as i32) {
        patchtestreg(fs, list, ((1 as i32) << 8 as i32) - 1 as i32);
        list = getjump(fs, list);
    }
}
unsafe extern "C-unwind" fn patchlistaux(
    mut fs: *mut FuncState,
    mut list: i32,
    mut vtarget: i32,
    mut reg: i32,
    mut dtarget: i32,
) {
    while list != -(1 as i32) {
        let mut next: i32 = getjump(fs, list);
        if patchtestreg(fs, list, reg) != 0 {
            fixjump(fs, list, vtarget);
        } else {
            fixjump(fs, list, dtarget);
        }
        list = next;
    }
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaK_patchlist(
    mut fs: *mut FuncState,
    mut list: i32,
    mut target: i32,
) {
    patchlistaux(
        fs,
        list,
        target,
        ((1 as i32) << 8 as i32) - 1 as i32,
        target,
    );
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaK_patchtohere(mut fs: *mut FuncState, mut list: i32) {
    let mut hr: i32 = luaK_getlabel(fs);
    luaK_patchlist(fs, list, hr);
}
unsafe extern "C-unwind" fn savelineinfo(mut fs: *mut FuncState, mut f: *mut Proto, mut line: i32) {
    let mut linedif: i32 = line - (*fs).previousline;
    let mut pc: i32 = (*fs).pc - 1 as i32;
    if abs(linedif) >= 0x80 || {
        let fresh91 = (*fs).iwthabs;
        (*fs).iwthabs = ((*fs).iwthabs).wrapping_add(1);
        fresh91 as i32 >= 128 as i32
    } {
        (*f).abslineinfo = luaM_growaux_(
            (*(*fs).ls).L,
            (*f).abslineinfo as *mut c_void,
            (*fs).nabslineinfo,
            &mut (*f).sizeabslineinfo,
            size_of::<AbsLineInfo>() as usize as i32,
            (if 2147483647 as i32 as size_t
                <= (!(0 as size_t)).wrapping_div(size_of::<AbsLineInfo>() as usize)
            {
                2147483647
            } else {
                (!(0 as size_t)).wrapping_div(size_of::<AbsLineInfo>() as usize) as u32
            }) as i32,
            c"lines".as_ptr(),
        ) as *mut AbsLineInfo;
        (*((*f).abslineinfo).offset((*fs).nabslineinfo as isize)).pc = pc;
        let fresh92 = (*fs).nabslineinfo;
        (*fs).nabslineinfo = (*fs).nabslineinfo + 1;
        (*((*f).abslineinfo).offset(fresh92 as isize)).line = line;
        linedif = -(0x80);
        (*fs).iwthabs = 1 as i32 as lu_byte;
    }
    (*f).lineinfo = luaM_growaux_(
        (*(*fs).ls).L,
        (*f).lineinfo as *mut c_void,
        pc,
        &mut (*f).sizelineinfo,
        size_of::<ls_byte>() as usize as i32,
        (if 2147483647 as i32 as size_t
            <= (!(0 as size_t)).wrapping_div(size_of::<ls_byte>() as usize)
        {
            2147483647
        } else {
            (!(0 as size_t)).wrapping_div(size_of::<ls_byte>() as usize) as u32
        }) as i32,
        c"opcodes".as_ptr(),
    ) as *mut ls_byte;
    *((*f).lineinfo).offset(pc as isize) = linedif as ls_byte;
    (*fs).previousline = line;
}
unsafe extern "C-unwind" fn removelastlineinfo(mut fs: *mut FuncState) {
    let mut f: *mut Proto = (*fs).f;
    let mut pc: i32 = (*fs).pc - 1 as i32;
    if *((*f).lineinfo).offset(pc as isize) as i32 != -(0x80) {
        (*fs).previousline -= *((*f).lineinfo).offset(pc as isize) as i32;
        (*fs).iwthabs = ((*fs).iwthabs).wrapping_sub(1);
        (*fs).iwthabs;
    } else {
        (*fs).nabslineinfo -= 1;
        (*fs).nabslineinfo;
        (*fs).iwthabs = (128 as i32 + 1 as i32) as lu_byte;
    };
}
unsafe extern "C-unwind" fn removelastinstruction(mut fs: *mut FuncState) {
    removelastlineinfo(fs);
    (*fs).pc -= 1;
    (*fs).pc;
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaK_code(mut fs: *mut FuncState, mut i: Instruction) -> i32 {
    let mut f: *mut Proto = (*fs).f;
    (*f).code = luaM_growaux_(
        (*(*fs).ls).L,
        (*f).code as *mut c_void,
        (*fs).pc,
        &mut (*f).sizecode,
        size_of::<Instruction>() as usize as i32,
        (if 2147483647 as i32 as size_t
            <= (!(0 as size_t)).wrapping_div(size_of::<Instruction>() as usize)
        {
            2147483647
        } else {
            (!(0 as size_t)).wrapping_div(size_of::<Instruction>() as usize) as u32
        }) as i32,
        c"opcodes".as_ptr(),
    ) as *mut Instruction;
    let fresh93 = (*fs).pc;
    (*fs).pc = (*fs).pc + 1;
    *((*f).code).offset(fresh93 as isize) = i;
    savelineinfo(fs, f, (*(*fs).ls).lastline);
    return (*fs).pc - 1 as i32;
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaK_codeABCk(
    mut fs: *mut FuncState,
    mut o: OpCode,
    mut a: i32,
    mut b: i32,
    mut c: i32,
    mut k: i32,
) -> i32 {
    return luaK_code(
        fs,
        (o as Instruction) << 0
            | (a as Instruction) << 0 + 7 as i32
            | (b as Instruction) << 0 + 7 as i32 + 8 as i32 + 1 as i32
            | (c as Instruction) << 0 + 7 as i32 + 8 as i32 + 1 as i32 + 8 as i32
            | (k as Instruction) << 0 + 7 as i32 + 8 as i32,
    );
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaK_codeABx(
    mut fs: *mut FuncState,
    mut o: OpCode,
    mut a: i32,
    mut bc: u32,
) -> i32 {
    return luaK_code(
        fs,
        (o as Instruction) << 0
            | (a as Instruction) << 0 + 7 as i32
            | bc << 0 + 7 as i32 + 8 as i32,
    );
}
unsafe extern "C-unwind" fn codeAsBx(
    mut fs: *mut FuncState,
    mut o: OpCode,
    mut a: i32,
    mut bc: i32,
) -> i32 {
    let mut b: u32 =
        (bc + (((1 as i32) << 8 as i32 + 8 as i32 + 1 as i32) - 1 as i32 >> 1 as i32)) as u32;
    return luaK_code(
        fs,
        (o as Instruction) << 0 | (a as Instruction) << 0 + 7 as i32 | b << 0 + 7 as i32 + 8 as i32,
    );
}
unsafe extern "C-unwind" fn codesJ(
    mut fs: *mut FuncState,
    mut o: OpCode,
    mut sj: i32,
    mut k: i32,
) -> i32 {
    let mut j: u32 = (sj
        + (((1 as i32) << 8 as i32 + 8 as i32 + 1 as i32 + 8 as i32) - 1 as i32 >> 1 as i32))
        as u32;
    return luaK_code(
        fs,
        (o as Instruction) << 0 | j << 0 + 7 as i32 | (k as Instruction) << 0 + 7 as i32 + 8 as i32,
    );
}
unsafe extern "C-unwind" fn codeextraarg(mut fs: *mut FuncState, mut a: i32) -> i32 {
    return luaK_code(
        fs,
        (OP_EXTRAARG as i32 as Instruction) << 0 | (a as Instruction) << 0 + 7 as i32,
    );
}
unsafe extern "C-unwind" fn luaK_codek(mut fs: *mut FuncState, mut reg: i32, mut k: i32) -> i32 {
    if k <= ((1 as i32) << 8 as i32 + 8 as i32 + 1 as i32) - 1 as i32 {
        return luaK_codeABx(fs, OP_LOADK, reg, k as u32);
    } else {
        let mut p: i32 = luaK_codeABx(fs, OP_LOADKX, reg, 0 as u32);
        codeextraarg(fs, k);
        return p;
    };
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaK_checkstack(mut fs: *mut FuncState, mut n: i32) {
    let mut newstack: i32 = (*fs).freereg as i32 + n;
    if newstack > (*(*fs).f).maxstacksize as i32 {
        if newstack >= 255 as i32 {
            luaX_syntaxerror(
                (*fs).ls,
                c"function or expression needs too many registers".as_ptr(),
            );
        }
        (*(*fs).f).maxstacksize = newstack as lu_byte;
    }
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaK_reserveregs(mut fs: *mut FuncState, mut n: i32) {
    luaK_checkstack(fs, n);
    (*fs).freereg = ((*fs).freereg as i32 + n) as lu_byte;
}
unsafe extern "C-unwind" fn freereg(mut fs: *mut FuncState, mut reg: i32) {
    if reg >= luaY_nvarstack(fs) {
        (*fs).freereg = ((*fs).freereg).wrapping_sub(1);
        (*fs).freereg;
    }
}
unsafe extern "C-unwind" fn freeregs(mut fs: *mut FuncState, mut r1: i32, mut r2: i32) {
    if r1 > r2 {
        freereg(fs, r1);
        freereg(fs, r2);
    } else {
        freereg(fs, r2);
        freereg(fs, r1);
    };
}
unsafe extern "C-unwind" fn freeexp(mut fs: *mut FuncState, mut e: *mut ExpDesc) {
    if (*e).k as u32 == VNONRELOC as i32 as u32 {
        freereg(fs, (*e).u.info);
    }
}
unsafe extern "C-unwind" fn freeexps(
    mut fs: *mut FuncState,
    mut e1: *mut ExpDesc,
    mut e2: *mut ExpDesc,
) {
    let mut r1: i32 = if (*e1).k as u32 == VNONRELOC as i32 as u32 {
        (*e1).u.info
    } else {
        -(1 as i32)
    };
    let mut r2: i32 = if (*e2).k as u32 == VNONRELOC as i32 as u32 {
        (*e2).u.info
    } else {
        -(1 as i32)
    };
    freeregs(fs, r1, r2);
}
unsafe extern "C-unwind" fn addk(
    mut fs: *mut FuncState,
    mut key: *mut TValue,
    mut v: *mut TValue,
) -> i32 {
    let mut val: TValue = TValue {
        value_: Value {
            gc: 0 as *mut GCObject,
        },
        tt_: 0,
    };
    let mut L: *mut lua_State = (*(*fs).ls).L;
    let mut f: *mut Proto = (*fs).f;
    let mut idx: *const TValue = luaH_get((*(*fs).ls).h, key);
    let mut k: i32 = 0;
    let mut oldsize: i32 = 0;
    if (*idx).tt_ as i32 == 3 as i32 | (0) << 4 as i32 {
        k = (*idx).value_.i as i32;
        if k < (*fs).nk
            && (*((*f).k).offset(k as isize)).tt_ as i32 & 0x3f as i32
                == (*v).tt_ as i32 & 0x3f as i32
            && luaV_equalobj(0 as *mut lua_State, &mut *((*f).k).offset(k as isize), v) != 0
        {
            return k;
        }
    }
    oldsize = (*f).sizek;
    k = (*fs).nk;
    let mut io: *mut TValue = &mut val;
    (*io).value_.i = k as lua_Integer;
    (*io).tt_ = (3 as i32 | (0) << 4 as i32) as lu_byte;
    luaH_finishset(L, (*(*fs).ls).h, key, idx, &mut val);
    (*f).k = luaM_growaux_(
        L,
        (*f).k as *mut c_void,
        k,
        &mut (*f).sizek,
        size_of::<TValue>() as usize as i32,
        (if (((1 as i32) << 8 as i32 + 8 as i32 + 1 as i32 + 8 as i32) - 1 as i32) as size_t
            <= (!(0 as size_t)).wrapping_div(size_of::<TValue>() as usize)
        {
            (((1 as i32) << 8 as i32 + 8 as i32 + 1 as i32 + 8 as i32) - 1 as i32) as u32
        } else {
            (!(0 as size_t)).wrapping_div(size_of::<TValue>() as usize) as u32
        }) as i32,
        c"constants".as_ptr(),
    ) as *mut TValue;
    while oldsize < (*f).sizek {
        let fresh94 = oldsize;
        oldsize = oldsize + 1;
        (*((*f).k).offset(fresh94 as isize)).tt_ = (0 | (0) << 4 as i32) as lu_byte;
    }
    let mut io1: *mut TValue = &mut *((*f).k).offset(k as isize) as *mut TValue;
    let mut io2: *const TValue = v;
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
    (*fs).nk += 1;
    (*fs).nk;
    if (*v).tt_ as i32 & (1 as i32) << 6 as i32 != 0 {
        if (*f).marked as i32 & (1 as i32) << 5 as i32 != 0
            && (*(*v).value_.gc).marked as i32 & ((1 as i32) << 3 as i32 | (1 as i32) << 4 as i32)
                != 0
        {
            luaC_barrier_(
                L,
                &mut (*(f as *mut GCUnion)).gc,
                &mut (*((*v).value_.gc as *mut GCUnion)).gc,
            );
        } else {
        };
    } else {
    };
    return k;
}
unsafe extern "C-unwind" fn stringK(mut fs: *mut FuncState, mut s: *mut TString) -> i32 {
    let mut o: TValue = TValue {
        value_: Value {
            gc: 0 as *mut GCObject,
        },
        tt_: 0,
    };
    let mut io: *mut TValue = &mut o;
    let mut x_: *mut TString = s;
    (*io).value_.gc = &mut (*(x_ as *mut GCUnion)).gc;
    (*io).tt_ = ((*x_).tt as i32 | (1 as i32) << 6 as i32) as lu_byte;
    if (*io).tt_ as i32 & (1 as i32) << 6 as i32 == 0
        || (*io).tt_ as i32 & 0x3f as i32 == (*(*io).value_.gc).tt as i32
            && (((*(*fs).ls).L).is_null()
                || (*(*io).value_.gc).marked as i32
                    & ((*(*(*(*fs).ls).L).l_G).currentwhite as i32
                        ^ ((1 as i32) << 3 as i32 | (1 as i32) << 4 as i32))
                    == 0)
    {
    } else {
    };
    return addk(fs, &mut o, &mut o);
}
unsafe extern "C-unwind" fn luaK_intK(mut fs: *mut FuncState, mut n: lua_Integer) -> i32 {
    let mut o: TValue = TValue {
        value_: Value {
            gc: 0 as *mut GCObject,
        },
        tt_: 0,
    };
    let mut io: *mut TValue = &mut o;
    (*io).value_.i = n;
    (*io).tt_ = (3 as i32 | (0) << 4 as i32) as lu_byte;
    return addk(fs, &mut o, &mut o);
}
unsafe extern "C-unwind" fn luaK_numberK(mut fs: *mut FuncState, mut r: lua_Number) -> i32 {
    let mut o: TValue = TValue {
        value_: Value {
            gc: 0 as *mut GCObject,
        },
        tt_: 0,
    };
    let mut ik: lua_Integer = 0;
    let mut io: *mut TValue = &mut o;
    (*io).value_.n = r;
    (*io).tt_ = (3 as i32 | (1 as i32) << 4 as i32) as lu_byte;
    if luaV_flttointeger::<F2Ieq>(r, &mut ik) == 0 {
        return addk(fs, &mut o, &mut o);
    } else {
        let nbm: i32 = 53 as i32;
        let q: lua_Number = ldexp(1.0f64, -nbm + 1 as i32);
        let k: lua_Number = if ik == 0 as lua_Integer { q } else { r + r * q };
        let mut kv: TValue = TValue {
            value_: Value {
                gc: 0 as *mut GCObject,
            },
            tt_: 0,
        };
        let mut io_0: *mut TValue = &mut kv;
        (*io_0).value_.n = k;
        (*io_0).tt_ = (3 as i32 | (1 as i32) << 4 as i32) as lu_byte;
        return addk(fs, &mut kv, &mut o);
    };
}
unsafe extern "C-unwind" fn boolF(mut fs: *mut FuncState) -> i32 {
    let mut o: TValue = TValue {
        value_: Value {
            gc: 0 as *mut GCObject,
        },
        tt_: 0,
    };
    o.tt_ = (1 as i32 | (0) << 4 as i32) as lu_byte;
    return addk(fs, &mut o, &mut o);
}
unsafe extern "C-unwind" fn boolT(mut fs: *mut FuncState) -> i32 {
    let mut o: TValue = TValue {
        value_: Value {
            gc: 0 as *mut GCObject,
        },
        tt_: 0,
    };
    o.tt_ = (1 as i32 | (1 as i32) << 4 as i32) as lu_byte;
    return addk(fs, &mut o, &mut o);
}
unsafe extern "C-unwind" fn nilK(mut fs: *mut FuncState) -> i32 {
    let mut k: TValue = TValue {
        value_: Value {
            gc: 0 as *mut GCObject,
        },
        tt_: 0,
    };
    let mut v: TValue = TValue {
        value_: Value {
            gc: 0 as *mut GCObject,
        },
        tt_: 0,
    };
    v.tt_ = (0 | (0) << 4 as i32) as lu_byte;
    let mut io: *mut TValue = &mut k;
    let mut x_: *mut Table = (*(*fs).ls).h;
    (*io).value_.gc = &mut (*(x_ as *mut GCUnion)).gc;
    (*io).tt_ = (5 as i32 | (0) << 4 as i32 | (1 as i32) << 6 as i32) as lu_byte;
    if (*io).tt_ as i32 & (1 as i32) << 6 as i32 == 0
        || (*io).tt_ as i32 & 0x3f as i32 == (*(*io).value_.gc).tt as i32
            && (((*(*fs).ls).L).is_null()
                || (*(*io).value_.gc).marked as i32
                    & ((*(*(*(*fs).ls).L).l_G).currentwhite as i32
                        ^ ((1 as i32) << 3 as i32 | (1 as i32) << 4 as i32))
                    == 0)
    {
    } else {
    };
    return addk(fs, &mut k, &mut v);
}
unsafe extern "C-unwind" fn fitsC(mut i: lua_Integer) -> i32 {
    return ((i as lua_Unsigned)
        .wrapping_add((((1 as i32) << 8 as i32) - 1 as i32 >> 1 as i32) as lua_Unsigned)
        <= (((1 as i32) << 8 as i32) - 1 as i32) as u32 as lua_Unsigned) as i32;
}
unsafe extern "C-unwind" fn fitsBx(mut i: lua_Integer) -> i32 {
    return (-(((1 as i32) << 8 as i32 + 8 as i32 + 1 as i32) - 1 as i32 >> 1 as i32) as lua_Integer
        <= i
        && i <= (((1 as i32) << 8 as i32 + 8 as i32 + 1 as i32)
            - 1 as i32
            - (((1 as i32) << 8 as i32 + 8 as i32 + 1 as i32) - 1 as i32 >> 1 as i32))
            as lua_Integer) as i32;
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaK_int(mut fs: *mut FuncState, mut reg: i32, mut i: lua_Integer) {
    if fitsBx(i) != 0 {
        codeAsBx(fs, OP_LOADI, reg, i as i32);
    } else {
        luaK_codek(fs, reg, luaK_intK(fs, i));
    };
}
unsafe extern "C-unwind" fn luaK_float(mut fs: *mut FuncState, mut reg: i32, mut f: lua_Number) {
    let mut fi: lua_Integer = 0;
    if luaV_flttointeger::<F2Ieq>(f, &mut fi) != 0 && fitsBx(fi) != 0 {
        codeAsBx(fs, OP_LOADF, reg, fi as i32);
    } else {
        luaK_codek(fs, reg, luaK_numberK(fs, f));
    };
}
unsafe extern "C-unwind" fn const2exp(mut v: *mut TValue, mut e: *mut ExpDesc) {
    match (*v).tt_ as i32 & 0x3f as i32 {
        3 => {
            (*e).k = VKINT;
            (*e).u.ival = (*v).value_.i;
        }
        19 => {
            (*e).k = VKFLT;
            (*e).u.nval = (*v).value_.n;
        }
        1 => {
            (*e).k = VFALSE;
        }
        17 => {
            (*e).k = VTRUE;
        }
        0 => {
            (*e).k = VNIL;
        }
        4 | 20 => {
            (*e).k = VKSTR;
            (*e).u.strval = &mut (*((*v).value_.gc as *mut GCUnion)).ts;
        }
        _ => {}
    };
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaK_setreturns(
    mut fs: *mut FuncState,
    mut e: *mut ExpDesc,
    mut nresults: i32,
) {
    let mut pc: *mut Instruction =
        &mut *((*(*fs).f).code).offset((*e).u.info as isize) as *mut Instruction;
    if (*e).k as u32 == VCALL as i32 as u32 {
        *pc = *pc
            & !(!(!(0 as Instruction) << 8 as i32)
                << 0 + 7 as i32 + 8 as i32 + 1 as i32 + 8 as i32)
            | ((nresults + 1 as i32) as Instruction)
                << 0 + 7 as i32 + 8 as i32 + 1 as i32 + 8 as i32
                & !(!(0 as Instruction) << 8 as i32)
                    << 0 + 7 as i32 + 8 as i32 + 1 as i32 + 8 as i32;
    } else {
        *pc = *pc
            & !(!(!(0 as Instruction) << 8 as i32)
                << 0 + 7 as i32 + 8 as i32 + 1 as i32 + 8 as i32)
            | ((nresults + 1 as i32) as Instruction)
                << 0 + 7 as i32 + 8 as i32 + 1 as i32 + 8 as i32
                & !(!(0 as Instruction) << 8 as i32)
                    << 0 + 7 as i32 + 8 as i32 + 1 as i32 + 8 as i32;
        *pc = *pc & !(!(!(0 as Instruction) << 8 as i32) << 0 + 7 as i32)
            | ((*fs).freereg as Instruction) << 0 + 7 as i32
                & !(!(0 as Instruction) << 8 as i32) << 0 + 7 as i32;
        luaK_reserveregs(fs, 1 as i32);
    };
}
unsafe extern "C-unwind" fn str2K(mut fs: *mut FuncState, mut e: *mut ExpDesc) {
    (*e).u.info = stringK(fs, (*e).u.strval);
    (*e).k = VK;
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaK_setoneret(mut fs: *mut FuncState, mut e: *mut ExpDesc) {
    if (*e).k as u32 == VCALL as i32 as u32 {
        (*e).k = VNONRELOC;
        (*e).u.info = (*((*(*fs).f).code).offset((*e).u.info as isize) >> 0 + 7 as i32
            & !(!(0 as Instruction) << 8 as i32) << 0) as i32;
    } else if (*e).k as u32 == VVARARG as i32 as u32 {
        *((*(*fs).f).code).offset((*e).u.info as isize) = *((*(*fs).f).code)
            .offset((*e).u.info as isize)
            & !(!(!(0 as Instruction) << 8 as i32)
                << 0 + 7 as i32 + 8 as i32 + 1 as i32 + 8 as i32)
            | (2 as i32 as Instruction) << 0 + 7 as i32 + 8 as i32 + 1 as i32 + 8 as i32
                & !(!(0 as Instruction) << 8 as i32)
                    << 0 + 7 as i32 + 8 as i32 + 1 as i32 + 8 as i32;
        (*e).k = VRELOC;
    }
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaK_dischargevars(mut fs: *mut FuncState, mut e: *mut ExpDesc) {
    match (*e).k as u32 {
        11 => {
            const2exp(const2val(fs, e), e);
        }
        9 => {
            let mut temp: i32 = (*e).u.var.ridx as i32;
            (*e).u.info = temp;
            (*e).k = VNONRELOC;
        }
        10 => {
            (*e).u.info = luaK_codeABCk(fs, OP_GETUPVAL, 0, (*e).u.info, 0, 0);
            (*e).k = VRELOC;
        }
        13 => {
            (*e).u.info = luaK_codeABCk(
                fs,
                OP_GETTABUP,
                0,
                (*e).u.ind.t as i32,
                (*e).u.ind.idx as i32,
                0,
            );
            (*e).k = VRELOC;
        }
        14 => {
            freereg(fs, (*e).u.ind.t as i32);
            (*e).u.info = luaK_codeABCk(
                fs,
                OP_GETI,
                0,
                (*e).u.ind.t as i32,
                (*e).u.ind.idx as i32,
                0,
            );
            (*e).k = VRELOC;
        }
        15 => {
            freereg(fs, (*e).u.ind.t as i32);
            (*e).u.info = luaK_codeABCk(
                fs,
                OP_GETFIELD,
                0,
                (*e).u.ind.t as i32,
                (*e).u.ind.idx as i32,
                0,
            );
            (*e).k = VRELOC;
        }
        12 => {
            freeregs(fs, (*e).u.ind.t as i32, (*e).u.ind.idx as i32);
            (*e).u.info = luaK_codeABCk(
                fs,
                OP_GETTABLE,
                0,
                (*e).u.ind.t as i32,
                (*e).u.ind.idx as i32,
                0,
            );
            (*e).k = VRELOC;
        }
        19 | 18 => {
            luaK_setoneret(fs, e);
        }
        _ => {}
    };
}
unsafe extern "C-unwind" fn discharge2reg(
    mut fs: *mut FuncState,
    mut e: *mut ExpDesc,
    mut reg: i32,
) {
    luaK_dischargevars(fs, e);
    let mut current_block_14: u64;
    match (*e).k as u32 {
        1 => {
            luaK_nil(fs, reg, 1 as i32);
            current_block_14 = 13242334135786603907;
        }
        3 => {
            luaK_codeABCk(fs, OP_LOADFALSE, reg, 0, 0, 0);
            current_block_14 = 13242334135786603907;
        }
        2 => {
            luaK_codeABCk(fs, OP_LOADTRUE, reg, 0, 0, 0);
            current_block_14 = 13242334135786603907;
        }
        7 => {
            str2K(fs, e);
            current_block_14 = 6937071982253665452;
        }
        4 => {
            current_block_14 = 6937071982253665452;
        }
        5 => {
            luaK_float(fs, reg, (*e).u.nval);
            current_block_14 = 13242334135786603907;
        }
        6 => {
            luaK_int(fs, reg, (*e).u.ival);
            current_block_14 = 13242334135786603907;
        }
        17 => {
            let mut pc: *mut Instruction =
                &mut *((*(*fs).f).code).offset((*e).u.info as isize) as *mut Instruction;
            *pc = *pc & !(!(!(0 as Instruction) << 8 as i32) << 0 + 7 as i32)
                | (reg as Instruction) << 0 + 7 as i32
                    & !(!(0 as Instruction) << 8 as i32) << 0 + 7 as i32;
            current_block_14 = 13242334135786603907;
        }
        8 => {
            if reg != (*e).u.info {
                luaK_codeABCk(fs, OP_MOVE, reg, (*e).u.info, 0, 0);
            }
            current_block_14 = 13242334135786603907;
        }
        _ => return,
    }
    match current_block_14 {
        6937071982253665452 => {
            luaK_codek(fs, reg, (*e).u.info);
        }
        _ => {}
    }
    (*e).u.info = reg;
    (*e).k = VNONRELOC;
}
unsafe extern "C-unwind" fn discharge2anyreg(mut fs: *mut FuncState, mut e: *mut ExpDesc) {
    if (*e).k as u32 != VNONRELOC as i32 as u32 {
        luaK_reserveregs(fs, 1 as i32);
        discharge2reg(fs, e, (*fs).freereg as i32 - 1 as i32);
    }
}
unsafe extern "C-unwind" fn code_loadbool(
    mut fs: *mut FuncState,
    mut A: i32,
    mut op: OpCode,
) -> i32 {
    luaK_getlabel(fs);
    return luaK_codeABCk(fs, op, A, 0, 0, 0);
}
unsafe extern "C-unwind" fn need_value(mut fs: *mut FuncState, mut list: i32) -> i32 {
    while list != -(1 as i32) {
        let mut i: Instruction = *getjumpcontrol(fs, list);
        if (i >> 0 & !(!(0 as Instruction) << 7 as i32) << 0) as OpCode as u32
            != OP_TESTSET as i32 as u32
        {
            return 1 as i32;
        }
        list = getjump(fs, list);
    }
    return 0;
}
unsafe extern "C-unwind" fn exp2reg(mut fs: *mut FuncState, mut e: *mut ExpDesc, mut reg: i32) {
    discharge2reg(fs, e, reg);
    if (*e).k as u32 == VJMP as i32 as u32 {
        luaK_concat(fs, &mut (*e).t, (*e).u.info);
    }
    if (*e).t != (*e).f {
        let mut final_0: i32 = 0;
        let mut p_f: i32 = -(1 as i32);
        let mut p_t: i32 = -(1 as i32);
        if need_value(fs, (*e).t) != 0 || need_value(fs, (*e).f) != 0 {
            let mut fj: i32 = if (*e).k as u32 == VJMP as i32 as u32 {
                -(1 as i32)
            } else {
                luaK_jump(fs)
            };
            p_f = code_loadbool(fs, reg, OP_LFALSESKIP);
            p_t = code_loadbool(fs, reg, OP_LOADTRUE);
            luaK_patchtohere(fs, fj);
        }
        final_0 = luaK_getlabel(fs);
        patchlistaux(fs, (*e).f, final_0, reg, p_f);
        patchlistaux(fs, (*e).t, final_0, reg, p_t);
    }
    (*e).t = -(1 as i32);
    (*e).f = (*e).t;
    (*e).u.info = reg;
    (*e).k = VNONRELOC;
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaK_exp2nextreg(mut fs: *mut FuncState, mut e: *mut ExpDesc) {
    luaK_dischargevars(fs, e);
    freeexp(fs, e);
    luaK_reserveregs(fs, 1 as i32);
    exp2reg(fs, e, (*fs).freereg as i32 - 1 as i32);
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaK_exp2anyreg(
    mut fs: *mut FuncState,
    mut e: *mut ExpDesc,
) -> i32 {
    luaK_dischargevars(fs, e);
    if (*e).k as u32 == VNONRELOC as i32 as u32 {
        if !((*e).t != (*e).f) {
            return (*e).u.info;
        }
        if (*e).u.info >= luaY_nvarstack(fs) {
            exp2reg(fs, e, (*e).u.info);
            return (*e).u.info;
        }
    }
    luaK_exp2nextreg(fs, e);
    return (*e).u.info;
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaK_exp2anyregup(mut fs: *mut FuncState, mut e: *mut ExpDesc) {
    if (*e).k as u32 != VUPVAL as i32 as u32 || (*e).t != (*e).f {
        luaK_exp2anyreg(fs, e);
    }
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaK_exp2val(mut fs: *mut FuncState, mut e: *mut ExpDesc) {
    if (*e).t != (*e).f {
        luaK_exp2anyreg(fs, e);
    } else {
        luaK_dischargevars(fs, e);
    };
}
unsafe extern "C-unwind" fn luaK_exp2K(mut fs: *mut FuncState, mut e: *mut ExpDesc) -> i32 {
    if !((*e).t != (*e).f) {
        let mut info: i32 = 0;
        match (*e).k as u32 {
            2 => {
                info = boolT(fs);
            }
            3 => {
                info = boolF(fs);
            }
            1 => {
                info = nilK(fs);
            }
            6 => {
                info = luaK_intK(fs, (*e).u.ival);
            }
            5 => {
                info = luaK_numberK(fs, (*e).u.nval);
            }
            7 => {
                info = stringK(fs, (*e).u.strval);
            }
            4 => {
                info = (*e).u.info;
            }
            _ => return 0,
        }
        if info <= ((1 as i32) << 8 as i32) - 1 as i32 {
            (*e).k = VK;
            (*e).u.info = info;
            return 1 as i32;
        }
    }
    return 0;
}
unsafe extern "C-unwind" fn exp2RK(mut fs: *mut FuncState, mut e: *mut ExpDesc) -> i32 {
    if luaK_exp2K(fs, e) != 0 {
        return 1 as i32;
    } else {
        luaK_exp2anyreg(fs, e);
        return 0;
    };
}
unsafe extern "C-unwind" fn codeABRK(
    mut fs: *mut FuncState,
    mut o: OpCode,
    mut a: i32,
    mut b: i32,
    mut ec: *mut ExpDesc,
) {
    let mut k: i32 = exp2RK(fs, ec);
    luaK_codeABCk(fs, o, a, b, (*ec).u.info, k);
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaK_storevar(
    mut fs: *mut FuncState,
    mut var: *mut ExpDesc,
    mut ex: *mut ExpDesc,
) {
    match (*var).k as u32 {
        9 => {
            freeexp(fs, ex);
            exp2reg(fs, ex, (*var).u.var.ridx as i32);
            return;
        }
        10 => {
            let mut e: i32 = luaK_exp2anyreg(fs, ex);
            luaK_codeABCk(fs, OP_SETUPVAL, e, (*var).u.info, 0, 0);
        }
        13 => {
            codeABRK(
                fs,
                OP_SETTABUP,
                (*var).u.ind.t as i32,
                (*var).u.ind.idx as i32,
                ex,
            );
        }
        14 => {
            codeABRK(
                fs,
                OP_SETI,
                (*var).u.ind.t as i32,
                (*var).u.ind.idx as i32,
                ex,
            );
        }
        15 => {
            codeABRK(
                fs,
                OP_SETFIELD,
                (*var).u.ind.t as i32,
                (*var).u.ind.idx as i32,
                ex,
            );
        }
        12 => {
            codeABRK(
                fs,
                OP_SETTABLE,
                (*var).u.ind.t as i32,
                (*var).u.ind.idx as i32,
                ex,
            );
        }
        _ => {}
    }
    freeexp(fs, ex);
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaK_self(
    mut fs: *mut FuncState,
    mut e: *mut ExpDesc,
    mut key: *mut ExpDesc,
) {
    let mut ereg: i32 = 0;
    luaK_exp2anyreg(fs, e);
    ereg = (*e).u.info;
    freeexp(fs, e);
    (*e).u.info = (*fs).freereg as i32;
    (*e).k = VNONRELOC;
    luaK_reserveregs(fs, 2 as i32);
    codeABRK(fs, OP_SELF, (*e).u.info, ereg, key);
    freeexp(fs, key);
}
unsafe extern "C-unwind" fn negatecondition(mut fs: *mut FuncState, mut e: *mut ExpDesc) {
    let mut pc: *mut Instruction = getjumpcontrol(fs, (*e).u.info);
    *pc = *pc & !(!(!(0 as Instruction) << 1 as i32) << 0 + 7 as i32 + 8 as i32)
        | (((*pc >> 0 + 7 as i32 + 8 as i32 & !(!(0 as Instruction) << 1 as i32) << 0) as i32
            ^ 1 as i32) as Instruction)
            << 0 + 7 as i32 + 8 as i32
            & !(!(0 as Instruction) << 1 as i32) << 0 + 7 as i32 + 8 as i32;
}
unsafe extern "C-unwind" fn jumponcond(
    mut fs: *mut FuncState,
    mut e: *mut ExpDesc,
    mut cond_0: i32,
) -> i32 {
    if (*e).k as u32 == VRELOC as i32 as u32 {
        let mut ie: Instruction = *((*(*fs).f).code).offset((*e).u.info as isize);
        if (ie >> 0 & !(!(0 as Instruction) << 7 as i32) << 0) as OpCode as u32
            == OP_NOT as i32 as u32
        {
            removelastinstruction(fs);
            return condjump(
                fs,
                OP_TEST,
                (ie >> 0 + 7 as i32 + 8 as i32 + 1 as i32 & !(!(0 as Instruction) << 8 as i32) << 0)
                    as i32,
                0,
                0,
                (cond_0 == 0) as i32,
            );
        }
    }
    discharge2anyreg(fs, e);
    freeexp(fs, e);
    return condjump(
        fs,
        OP_TESTSET,
        ((1 as i32) << 8 as i32) - 1 as i32,
        (*e).u.info,
        0,
        cond_0,
    );
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaK_goiftrue(mut fs: *mut FuncState, mut e: *mut ExpDesc) {
    let mut pc: i32 = 0;
    luaK_dischargevars(fs, e);
    match (*e).k as u32 {
        16 => {
            negatecondition(fs, e);
            pc = (*e).u.info;
        }
        4 | 5 | 6 | 7 | 2 => {
            pc = -(1 as i32);
        }
        _ => {
            pc = jumponcond(fs, e, 0);
        }
    }
    luaK_concat(fs, &mut (*e).f, pc);
    luaK_patchtohere(fs, (*e).t);
    (*e).t = -(1 as i32);
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaK_goiffalse(mut fs: *mut FuncState, mut e: *mut ExpDesc) {
    let mut pc: i32 = 0;
    luaK_dischargevars(fs, e);
    match (*e).k as u32 {
        16 => {
            pc = (*e).u.info;
        }
        1 | 3 => {
            pc = -(1 as i32);
        }
        _ => {
            pc = jumponcond(fs, e, 1 as i32);
        }
    }
    luaK_concat(fs, &mut (*e).t, pc);
    luaK_patchtohere(fs, (*e).f);
    (*e).f = -(1 as i32);
}
unsafe extern "C-unwind" fn codenot(mut fs: *mut FuncState, mut e: *mut ExpDesc) {
    match (*e).k as u32 {
        1 | 3 => {
            (*e).k = VTRUE;
        }
        4 | 5 | 6 | 7 | 2 => {
            (*e).k = VFALSE;
        }
        16 => {
            negatecondition(fs, e);
        }
        17 | 8 => {
            discharge2anyreg(fs, e);
            freeexp(fs, e);
            (*e).u.info = luaK_codeABCk(fs, OP_NOT, 0, (*e).u.info, 0, 0);
            (*e).k = VRELOC;
        }
        _ => {}
    }
    let mut temp: i32 = (*e).f;
    (*e).f = (*e).t;
    (*e).t = temp;
    removevalues(fs, (*e).f);
    removevalues(fs, (*e).t);
}
unsafe extern "C-unwind" fn isKstr(mut fs: *mut FuncState, mut e: *mut ExpDesc) -> i32 {
    return ((*e).k as u32 == VK as i32 as u32
        && !((*e).t != (*e).f)
        && (*e).u.info <= ((1 as i32) << 8 as i32) - 1 as i32
        && (*((*(*fs).f).k).offset((*e).u.info as isize)).tt_ as i32
            == 4 as i32 | (0) << 4 as i32 | (1 as i32) << 6 as i32) as i32;
}
unsafe extern "C-unwind" fn isKint(mut e: *mut ExpDesc) -> i32 {
    return ((*e).k as u32 == VKINT as i32 as u32 && !((*e).t != (*e).f)) as i32;
}
unsafe extern "C-unwind" fn isCint(mut e: *mut ExpDesc) -> i32 {
    return (isKint(e) != 0
        && (*e).u.ival as lua_Unsigned <= (((1 as i32) << 8 as i32) - 1 as i32) as lua_Unsigned)
        as i32;
}
unsafe extern "C-unwind" fn isSCint(mut e: *mut ExpDesc) -> i32 {
    return (isKint(e) != 0 && fitsC((*e).u.ival) != 0) as i32;
}
unsafe extern "C-unwind" fn isSCnumber(
    mut e: *mut ExpDesc,
    mut pi: *mut i32,
    mut isfloat: *mut i32,
) -> i32 {
    let mut i: lua_Integer = 0;
    if (*e).k as u32 == VKINT as i32 as u32 {
        i = (*e).u.ival;
    } else if (*e).k as u32 == VKFLT as i32 as u32
        && luaV_flttointeger::<F2Ieq>((*e).u.nval, &mut i) != 0
    {
        *isfloat = 1 as i32;
    } else {
        return 0;
    }
    if !((*e).t != (*e).f) && fitsC(i) != 0 {
        *pi = i as i32 + (((1 as i32) << 8 as i32) - 1 as i32 >> 1 as i32);
        return 1 as i32;
    } else {
        return 0;
    };
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaK_indexed(
    mut fs: *mut FuncState,
    mut t: *mut ExpDesc,
    mut k: *mut ExpDesc,
) {
    if (*k).k as u32 == VKSTR as i32 as u32 {
        str2K(fs, k);
    }
    if (*t).k as u32 == VUPVAL as i32 as u32 && isKstr(fs, k) == 0 {
        luaK_exp2anyreg(fs, t);
    }
    if (*t).k as u32 == VUPVAL as i32 as u32 {
        let mut temp: i32 = (*t).u.info;
        (*t).u.ind.t = temp as lu_byte;
        (*t).u.ind.idx = (*k).u.info as i16;
        (*t).k = VINDEXUP;
    } else {
        (*t).u.ind.t = (if (*t).k as u32 == VLOCAL as i32 as u32 {
            (*t).u.var.ridx as i32
        } else {
            (*t).u.info
        }) as lu_byte;
        if isKstr(fs, k) != 0 {
            (*t).u.ind.idx = (*k).u.info as i16;
            (*t).k = VINDEXSTR;
        } else if isCint(k) != 0 {
            (*t).u.ind.idx = (*k).u.ival as i32 as i16;
            (*t).k = VINDEXI;
        } else {
            (*t).u.ind.idx = luaK_exp2anyreg(fs, k) as i16;
            (*t).k = VINDEXED;
        }
    };
}
unsafe extern "C-unwind" fn validop(mut op: i32, mut v1: *mut TValue, mut v2: *mut TValue) -> i32 {
    match op {
        7 | 8 | 9 | 10 | 11 | 13 => {
            let mut i: lua_Integer = 0;
            return (luaV_tointegerns::<F2Ieq>(v1, &mut i) != 0
                && luaV_tointegerns::<F2Ieq>(v2, &mut i) != 0) as i32;
        }
        5 | 6 | 3 => {
            return ((if (*v2).tt_ as i32 == 3 as i32 | (0) << 4 as i32 {
                (*v2).value_.i as lua_Number
            } else {
                (*v2).value_.n
            }) != 0 as lua_Number) as i32;
        }
        _ => return 1 as i32,
    };
}
unsafe extern "C-unwind" fn constfolding(
    mut fs: *mut FuncState,
    mut op: i32,
    mut e1: *mut ExpDesc,
    mut e2: *const ExpDesc,
) -> i32 {
    let mut v1: TValue = TValue {
        value_: Value {
            gc: 0 as *mut GCObject,
        },
        tt_: 0,
    };
    let mut v2: TValue = TValue {
        value_: Value {
            gc: 0 as *mut GCObject,
        },
        tt_: 0,
    };
    let mut res: TValue = TValue {
        value_: Value {
            gc: 0 as *mut GCObject,
        },
        tt_: 0,
    };
    if tonumeral(e1, &mut v1) == 0
        || tonumeral(e2, &mut v2) == 0
        || validop(op, &mut v1, &mut v2) == 0
    {
        return 0;
    }
    luaO_rawarith((*(*fs).ls).L, op, &mut v1, &mut v2, &mut res);
    if res.tt_ as i32 == 3 as i32 | (0) << 4 as i32 {
        (*e1).k = VKINT;
        (*e1).u.ival = res.value_.i;
    } else {
        let mut n: lua_Number = res.value_.n;
        if !(n == n) || n == 0 as lua_Number {
            return 0;
        }
        (*e1).k = VKFLT;
        (*e1).u.nval = n;
    }
    return 1 as i32;
}
#[inline]
unsafe extern "C-unwind" fn binopr2op(
    mut opr: BinOpr,
    mut baser: BinOpr,
    mut base: OpCode,
) -> OpCode {
    return (opr as i32 - baser as i32 + base as i32) as OpCode;
}
#[inline]
unsafe extern "C-unwind" fn unopr2op(mut opr: UnOpr) -> OpCode {
    return (opr as i32 - OPR_MINUS as i32 + OP_UNM as i32) as OpCode;
}
#[inline]
unsafe extern "C-unwind" fn binopr2TM(mut opr: BinOpr) -> TMS {
    return (opr as i32 - OPR_ADD as i32 + TM_ADD as i32) as TMS;
}
unsafe extern "C-unwind" fn codeunexpval(
    mut fs: *mut FuncState,
    mut op: OpCode,
    mut e: *mut ExpDesc,
    mut line: i32,
) {
    let mut r: i32 = luaK_exp2anyreg(fs, e);
    freeexp(fs, e);
    (*e).u.info = luaK_codeABCk(fs, op, 0, r, 0, 0);
    (*e).k = VRELOC;
    luaK_fixline(fs, line);
}
unsafe extern "C-unwind" fn finishbinexpval(
    mut fs: *mut FuncState,
    mut e1: *mut ExpDesc,
    mut e2: *mut ExpDesc,
    mut op: OpCode,
    mut v2: i32,
    mut flip: i32,
    mut line: i32,
    mut mmop: OpCode,
    mut event: TMS,
) {
    let mut v1: i32 = luaK_exp2anyreg(fs, e1);
    let mut pc: i32 = luaK_codeABCk(fs, op, 0, v1, v2, 0);
    freeexps(fs, e1, e2);
    (*e1).u.info = pc;
    (*e1).k = VRELOC;
    luaK_fixline(fs, line);
    luaK_codeABCk(fs, mmop, v1, v2, event as i32, flip);
    luaK_fixline(fs, line);
}
unsafe extern "C-unwind" fn codebinexpval(
    mut fs: *mut FuncState,
    mut opr: BinOpr,
    mut e1: *mut ExpDesc,
    mut e2: *mut ExpDesc,
    mut line: i32,
) {
    let mut op: OpCode = binopr2op(opr, OPR_ADD, OP_ADD);
    let mut v2: i32 = luaK_exp2anyreg(fs, e2);
    finishbinexpval(fs, e1, e2, op, v2, 0, line, OP_MMBIN, binopr2TM(opr));
}
unsafe extern "C-unwind" fn codebini(
    mut fs: *mut FuncState,
    mut op: OpCode,
    mut e1: *mut ExpDesc,
    mut e2: *mut ExpDesc,
    mut flip: i32,
    mut line: i32,
    mut event: TMS,
) {
    let mut v2: i32 = (*e2).u.ival as i32 + (((1 as i32) << 8 as i32) - 1 as i32 >> 1 as i32);
    finishbinexpval(fs, e1, e2, op, v2, flip, line, OP_MMBINI, event);
}
unsafe extern "C-unwind" fn codebinK(
    mut fs: *mut FuncState,
    mut opr: BinOpr,
    mut e1: *mut ExpDesc,
    mut e2: *mut ExpDesc,
    mut flip: i32,
    mut line: i32,
) {
    let mut event: TMS = binopr2TM(opr);
    let mut v2: i32 = (*e2).u.info;
    let mut op: OpCode = binopr2op(opr, OPR_ADD, OP_ADDK);
    finishbinexpval(fs, e1, e2, op, v2, flip, line, OP_MMBINK, event);
}
unsafe extern "C-unwind" fn finishbinexpneg(
    mut fs: *mut FuncState,
    mut e1: *mut ExpDesc,
    mut e2: *mut ExpDesc,
    mut op: OpCode,
    mut line: i32,
    mut event: TMS,
) -> i32 {
    if isKint(e2) == 0 {
        return 0;
    } else {
        let mut i2: lua_Integer = (*e2).u.ival;
        if !(fitsC(i2) != 0 && fitsC(-i2) != 0) {
            return 0;
        } else {
            let mut v2: i32 = i2 as i32;
            finishbinexpval(
                fs,
                e1,
                e2,
                op,
                -v2 + (((1 as i32) << 8 as i32) - 1 as i32 >> 1 as i32),
                0,
                line,
                OP_MMBINI,
                event,
            );
            *((*(*fs).f).code).offset(((*fs).pc - 1 as i32) as isize) = *((*(*fs).f).code)
                .offset(((*fs).pc - 1 as i32) as isize)
                & !(!(!(0 as Instruction) << 8 as i32) << 0 + 7 as i32 + 8 as i32 + 1 as i32)
                | ((v2 + (((1 as i32) << 8 as i32) - 1 as i32 >> 1 as i32)) as Instruction)
                    << 0 + 7 as i32 + 8 as i32 + 1 as i32
                    & !(!(0 as Instruction) << 8 as i32) << 0 + 7 as i32 + 8 as i32 + 1 as i32;
            return 1 as i32;
        }
    };
}
unsafe extern "C-unwind" fn swapexps(mut e1: *mut ExpDesc, mut e2: *mut ExpDesc) {
    let mut temp: ExpDesc = *e1;
    *e1 = *e2;
    *e2 = temp;
}
unsafe extern "C-unwind" fn codebinNoK(
    mut fs: *mut FuncState,
    mut opr: BinOpr,
    mut e1: *mut ExpDesc,
    mut e2: *mut ExpDesc,
    mut flip: i32,
    mut line: i32,
) {
    if flip != 0 {
        swapexps(e1, e2);
    }
    codebinexpval(fs, opr, e1, e2, line);
}
unsafe extern "C-unwind" fn codearith(
    mut fs: *mut FuncState,
    mut opr: BinOpr,
    mut e1: *mut ExpDesc,
    mut e2: *mut ExpDesc,
    mut flip: i32,
    mut line: i32,
) {
    if tonumeral(e2, 0 as *mut TValue) != 0 && luaK_exp2K(fs, e2) != 0 {
        codebinK(fs, opr, e1, e2, flip, line);
    } else {
        codebinNoK(fs, opr, e1, e2, flip, line);
    };
}
unsafe extern "C-unwind" fn codecommutative(
    mut fs: *mut FuncState,
    mut op: BinOpr,
    mut e1: *mut ExpDesc,
    mut e2: *mut ExpDesc,
    mut line: i32,
) {
    let mut flip: i32 = 0;
    if tonumeral(e1, 0 as *mut TValue) != 0 {
        swapexps(e1, e2);
        flip = 1 as i32;
    }
    if op as u32 == OPR_ADD as i32 as u32 && isSCint(e2) != 0 {
        codebini(fs, OP_ADDI, e1, e2, flip, line, TM_ADD);
    } else {
        codearith(fs, op, e1, e2, flip, line);
    };
}
unsafe extern "C-unwind" fn codebitwise(
    mut fs: *mut FuncState,
    mut opr: BinOpr,
    mut e1: *mut ExpDesc,
    mut e2: *mut ExpDesc,
    mut line: i32,
) {
    let mut flip: i32 = 0;
    if (*e1).k as u32 == VKINT as i32 as u32 {
        swapexps(e1, e2);
        flip = 1 as i32;
    }
    if (*e2).k as u32 == VKINT as i32 as u32 && luaK_exp2K(fs, e2) != 0 {
        codebinK(fs, opr, e1, e2, flip, line);
    } else {
        codebinNoK(fs, opr, e1, e2, flip, line);
    };
}
unsafe extern "C-unwind" fn codeorder(
    mut fs: *mut FuncState,
    mut opr: BinOpr,
    mut e1: *mut ExpDesc,
    mut e2: *mut ExpDesc,
) {
    let mut r1: i32 = 0;
    let mut r2: i32 = 0;
    let mut im: i32 = 0;
    let mut isfloat: i32 = 0;
    let mut op: OpCode = OP_MOVE;
    if isSCnumber(e2, &mut im, &mut isfloat) != 0 {
        r1 = luaK_exp2anyreg(fs, e1);
        r2 = im;
        op = binopr2op(opr, OPR_LT, OP_LTI);
    } else if isSCnumber(e1, &mut im, &mut isfloat) != 0 {
        r1 = luaK_exp2anyreg(fs, e2);
        r2 = im;
        op = binopr2op(opr, OPR_LT, OP_GTI);
    } else {
        r1 = luaK_exp2anyreg(fs, e1);
        r2 = luaK_exp2anyreg(fs, e2);
        op = binopr2op(opr, OPR_LT, OP_LT);
    }
    freeexps(fs, e1, e2);
    (*e1).u.info = condjump(fs, op, r1, r2, isfloat, 1 as i32);
    (*e1).k = VJMP;
}
unsafe extern "C-unwind" fn codeeq(
    mut fs: *mut FuncState,
    mut opr: BinOpr,
    mut e1: *mut ExpDesc,
    mut e2: *mut ExpDesc,
) {
    let mut r1: i32 = 0;
    let mut r2: i32 = 0;
    let mut im: i32 = 0;
    let mut isfloat: i32 = 0;
    let mut op: OpCode = OP_MOVE;
    if (*e1).k as u32 != VNONRELOC as i32 as u32 {
        swapexps(e1, e2);
    }
    r1 = luaK_exp2anyreg(fs, e1);
    if isSCnumber(e2, &mut im, &mut isfloat) != 0 {
        op = OP_EQI;
        r2 = im;
    } else if exp2RK(fs, e2) != 0 {
        op = OP_EQK;
        r2 = (*e2).u.info;
    } else {
        op = OP_EQ;
        r2 = luaK_exp2anyreg(fs, e2);
    }
    freeexps(fs, e1, e2);
    (*e1).u.info = condjump(
        fs,
        op,
        r1,
        r2,
        isfloat,
        (opr as u32 == OPR_EQ as i32 as u32) as i32,
    );
    (*e1).k = VJMP;
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaK_prefix(
    mut fs: *mut FuncState,
    mut opr: UnOpr,
    mut e: *mut ExpDesc,
    mut line: i32,
) {
    static mut ef: ExpDesc = {
        let mut init = ExpDesc {
            k: VKINT,
            u: ExpVariant {
                ival: 0 as lua_Integer,
            },
            t: -(1 as i32),
            f: -(1 as i32),
        };
        init
    };
    luaK_dischargevars(fs, e);
    let mut current_block_3: u64;
    match opr as u32 {
        0 | 1 => {
            if constfolding(fs, (opr as u32).wrapping_add(12) as i32, e, &raw const ef) != 0 {
                current_block_3 = 7815301370352969686;
            } else {
                current_block_3 = 8039936322597116006;
            }
        }
        3 => {
            current_block_3 = 8039936322597116006;
        }
        2 => {
            codenot(fs, e);
            current_block_3 = 7815301370352969686;
        }
        _ => {
            current_block_3 = 7815301370352969686;
        }
    }
    match current_block_3 {
        8039936322597116006 => {
            codeunexpval(fs, unopr2op(opr), e, line);
        }
        _ => {}
    };
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaK_infix(
    mut fs: *mut FuncState,
    mut op: BinOpr,
    mut v: *mut ExpDesc,
) {
    luaK_dischargevars(fs, v);
    match op as u32 {
        19 => {
            luaK_goiftrue(fs, v);
        }
        20 => {
            luaK_goiffalse(fs, v);
        }
        12 => {
            luaK_exp2nextreg(fs, v);
        }
        0 | 1 | 2 | 5 | 6 | 3 | 4 | 7 | 8 | 9 | 10 | 11 => {
            if tonumeral(v, 0 as *mut TValue) == 0 {
                luaK_exp2anyreg(fs, v);
            }
        }
        13 | 16 => {
            if tonumeral(v, 0 as *mut TValue) == 0 {
                exp2RK(fs, v);
            }
        }
        14 | 15 | 17 | 18 => {
            let mut dummy: i32 = 0;
            let mut dummy2: i32 = 0;
            if isSCnumber(v, &mut dummy, &mut dummy2) == 0 {
                luaK_exp2anyreg(fs, v);
            }
        }
        _ => {}
    };
}
unsafe extern "C-unwind" fn codeconcat(
    mut fs: *mut FuncState,
    mut e1: *mut ExpDesc,
    mut e2: *mut ExpDesc,
    mut line: i32,
) {
    let mut ie2: *mut Instruction = previousinstruction(fs);
    if (*ie2 >> 0 & !(!(0 as Instruction) << 7 as i32) << 0) as OpCode as u32
        == OP_CONCAT as i32 as u32
    {
        let mut n: i32 = (*ie2 >> 0 + 7 as i32 + 8 as i32 + 1 as i32
            & !(!(0 as Instruction) << 8 as i32) << 0) as i32;
        freeexp(fs, e2);
        *ie2 = *ie2 & !(!(!(0 as Instruction) << 8 as i32) << 0 + 7 as i32)
            | ((*e1).u.info as Instruction) << 0 + 7 as i32
                & !(!(0 as Instruction) << 8 as i32) << 0 + 7 as i32;
        *ie2 = *ie2 & !(!(!(0 as Instruction) << 8 as i32) << 0 + 7 as i32 + 8 as i32 + 1 as i32)
            | ((n + 1 as i32) as Instruction) << 0 + 7 as i32 + 8 as i32 + 1 as i32
                & !(!(0 as Instruction) << 8 as i32) << 0 + 7 as i32 + 8 as i32 + 1 as i32;
    } else {
        luaK_codeABCk(fs, OP_CONCAT, (*e1).u.info, 2 as i32, 0, 0);
        freeexp(fs, e2);
        luaK_fixline(fs, line);
    };
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaK_posfix(
    mut fs: *mut FuncState,
    mut opr: BinOpr,
    mut e1: *mut ExpDesc,
    mut e2: *mut ExpDesc,
    mut line: i32,
) {
    luaK_dischargevars(fs, e2);
    if opr as u32 <= OPR_SHR as i32 as u32
        && constfolding(fs, (opr as u32).wrapping_add(0 as u32) as i32, e1, e2) != 0
    {
        return;
    }
    let mut current_block_30: u64;
    match opr as u32 {
        19 => {
            luaK_concat(fs, &mut (*e2).f, (*e1).f);
            *e1 = *e2;
            current_block_30 = 8180496224585318153;
        }
        20 => {
            luaK_concat(fs, &mut (*e2).t, (*e1).t);
            *e1 = *e2;
            current_block_30 = 8180496224585318153;
        }
        12 => {
            luaK_exp2nextreg(fs, e2);
            codeconcat(fs, e1, e2, line);
            current_block_30 = 8180496224585318153;
        }
        0 | 2 => {
            codecommutative(fs, opr, e1, e2, line);
            current_block_30 = 8180496224585318153;
        }
        1 => {
            if finishbinexpneg(fs, e1, e2, OP_ADDI, line, TM_SUB) != 0 {
                current_block_30 = 8180496224585318153;
            } else {
                current_block_30 = 12599329904712511516;
            }
        }
        5 | 6 | 3 | 4 => {
            current_block_30 = 12599329904712511516;
        }
        7 | 8 | 9 => {
            codebitwise(fs, opr, e1, e2, line);
            current_block_30 = 8180496224585318153;
        }
        10 => {
            if isSCint(e1) != 0 {
                swapexps(e1, e2);
                codebini(fs, OP_SHLI, e1, e2, 1 as i32, line, TM_SHL);
            } else if !(finishbinexpneg(fs, e1, e2, OP_SHRI, line, TM_SHL) != 0) {
                codebinexpval(fs, opr, e1, e2, line);
            }
            current_block_30 = 8180496224585318153;
        }
        11 => {
            if isSCint(e2) != 0 {
                codebini(fs, OP_SHRI, e1, e2, 0, line, TM_SHR);
            } else {
                codebinexpval(fs, opr, e1, e2, line);
            }
            current_block_30 = 8180496224585318153;
        }
        13 | 16 => {
            codeeq(fs, opr, e1, e2);
            current_block_30 = 8180496224585318153;
        }
        17 | 18 => {
            swapexps(e1, e2);
            opr = (opr as u32)
                .wrapping_sub(OPR_GT as i32 as u32)
                .wrapping_add(OPR_LT as i32 as u32) as BinOpr;
            current_block_30 = 1118134448028020070;
        }
        14 | 15 => {
            current_block_30 = 1118134448028020070;
        }
        _ => {
            current_block_30 = 8180496224585318153;
        }
    }
    match current_block_30 {
        12599329904712511516 => {
            codearith(fs, opr, e1, e2, 0, line);
        }
        1118134448028020070 => {
            codeorder(fs, opr, e1, e2);
        }
        _ => {}
    };
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaK_fixline(mut fs: *mut FuncState, mut line: i32) {
    removelastlineinfo(fs);
    savelineinfo(fs, (*fs).f, line);
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaK_settablesize(
    mut fs: *mut FuncState,
    mut pc: i32,
    mut ra: i32,
    mut asize: i32,
    mut hsize: i32,
) {
    let mut inst: *mut Instruction =
        &mut *((*(*fs).f).code).offset(pc as isize) as *mut Instruction;
    let mut rb: i32 = if hsize != 0 {
        luaO_ceillog2(hsize as u32) + 1 as i32
    } else {
        0
    };
    let mut extra: i32 = asize / (((1 as i32) << 8 as i32) - 1 as i32 + 1 as i32);
    let mut rc: i32 = asize % (((1 as i32) << 8 as i32) - 1 as i32 + 1 as i32);
    let mut k: i32 = (extra > 0) as i32;
    *inst = (OP_NEWTABLE as i32 as Instruction) << 0
        | (ra as Instruction) << 0 + 7 as i32
        | (rb as Instruction) << 0 + 7 as i32 + 8 as i32 + 1 as i32
        | (rc as Instruction) << 0 + 7 as i32 + 8 as i32 + 1 as i32 + 8 as i32
        | (k as Instruction) << 0 + 7 as i32 + 8 as i32;
    *inst.offset(1) =
        (OP_EXTRAARG as i32 as Instruction) << 0 | (extra as Instruction) << 0 + 7 as i32;
}

/// Emit a SETLIST instruction.
/// `base` is the register that keeps table;
/// `nelems` is #table plus those to be stored now;
/// `tostore` is number of values (in registers `base + 1`,...) to add to
/// table (or LUA_MULTIRET to add up to stack top).
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaK_setlist(
    mut fs: *mut FuncState,
    mut base: i32,
    mut nelems: i32,
    mut tostore: i32,
) {
    if tostore == LUA_MULTRET {
        tostore = 0;
    }
    if nelems <= MAXARG_C as i32 {
        luaK_codeABCk(fs, OP_SETLIST, base, tostore, nelems, 0);
    } else {
        let mut extra: i32 = nelems / (MAXARG_C as i32 + 1);
        nelems %= MAXARG_C as i32 + 1;
        luaK_codeABCk(fs, OP_SETLIST, base, tostore, nelems, 1);
        codeextraarg(fs, extra);
    }
    // free registers with list values
    (*fs).freereg = (base + 1 as i32) as lu_byte;
}

/// Return the final target of a jump (skipping jumps to jumps).
unsafe extern "C-unwind" fn finaltarget(mut code: *mut Instruction, mut i: i32) -> i32 {
    let mut count: i32 = 0;
    count = 0;
    while count < 100 {
        let mut pc: Instruction = *code.offset(i as isize);
        if get_opcode(pc) != OP_JMP {
            break;
        }
        i += getarg_sj(pc) + 1;
        count += 1;
    }
    return i;
}

/// Do a final pass over the code of a function, doing small peephole
/// optimizations and adjustments.
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaK_finish(mut fs: *mut FuncState) {
    let mut i: i32 = 0;
    let mut p: *mut Proto = (*fs).f;
    i = 0;
    while i < (*fs).pc {
        let mut pc: *mut Instruction = &mut *((*p).code).offset(i as isize) as *mut Instruction;
        let mut current_block_7: u64;
        match get_opcode(*pc) {
            OP_RETURN0 | OP_RETURN1 => {
                if !((*fs).needclose as i32 != 0 || (*p).is_vararg as i32 != 0) {
                    current_block_7 = 12599329904712511516;
                } else {
                    *pc = *pc & !(!(!(0 as Instruction) << 7 as i32) << 0)
                        | (OP_RETURN as i32 as Instruction) << 0
                            & !(!(0 as Instruction) << 7 as i32) << 0;
                    current_block_7 = 11006700562992250127;
                }
            }
            OP_RETURN | OP_TAILCALL => {
                current_block_7 = 11006700562992250127;
            }
            OP_JMP => {
                let mut target: i32 = finaltarget((*p).code, i);
                fixjump(fs, i, target);
                current_block_7 = 12599329904712511516;
            }
            _ => {
                current_block_7 = 12599329904712511516;
            }
        }
        match current_block_7 {
            11006700562992250127 => {
                if (*fs).needclose != 0 {
                    *pc = *pc & !(!(!(0 as Instruction) << 1 as i32) << 0 + 7 as i32 + 8 as i32)
                        | (1 as i32 as Instruction) << 0 + 7 as i32 + 8 as i32
                            & !(!(0 as Instruction) << 1 as i32) << 0 + 7 as i32 + 8 as i32;
                }
                if (*p).is_vararg != 0 {
                    *pc = *pc
                        & !(!(!(0 as Instruction) << 8 as i32)
                            << 0 + 7 as i32 + 8 as i32 + 1 as i32 + 8 as i32)
                        | (((*p).numparams as i32 + 1 as i32) as Instruction)
                            << 0 + 7 as i32 + 8 as i32 + 1 as i32 + 8 as i32
                            & !(!(0 as Instruction) << 8 as i32)
                                << 0 + 7 as i32 + 8 as i32 + 1 as i32 + 8 as i32;
                }
            }
            _ => {}
        }
        i += 1;
    }
}

#[cfg(feature = "jit")]
pub unsafe fn luaK_build_loop_counters(L: *mut lua_State, proto: *mut Proto) {
    // Iterate through instructions backwards, finding targets of backwards jumps.
    // First do an initial count of all backwards jumps (including for loops).

    let code = std::slice::from_raw_parts((*proto).code, (*proto).sizecode as usize);

    let g = utils::GlobalState::new_unchecked((*L).l_G);

    let Some(mut loop_points_guard) = utils::AllocGuard::<[bool]>::alloc_slice(g, (*proto).sizecode as usize) else {
        luaD_throw(L, 4);
    };

    let loop_points = loop_points_guard.as_ptr().as_mut();

    loop_points.fill(false);
    loop_points[0] = true; // Hot entry

    for (pc, i) in code.iter().copied().enumerate().rev() {
        match get_opcode(i) {
            OP_JMP => {
                // TODO: Check for backwards jump
                let target = getarg_sj(i) + 1;

                if target.is_negative() {
                    let target_pc = (pc as u32).wrapping_add_signed(target);

                    *loop_points.get_unchecked_mut(target_pc as usize) = true;
                }
            }
            OP_CALL => {
                // Trace after call
                *loop_points.get_unchecked_mut(pc + 1) = true;
            }
            OP_FORLOOP | OP_TFORLOOP => {
                let target = getarg_bx(i) - 1;
                let target_pc = (pc as u32).wrapping_sub(target);
                *loop_points.get_unchecked_mut(target_pc as usize) = true;

                // TODO: Remove this trace point once loops are traceable
                *loop_points.get_unchecked_mut(pc + 1) = true;
            }
            _ => continue,
        }
    }

    let loop_count = loop_points.iter().filter(|is_loop| **is_loop).count();

    let Some(loop_counters) = g.alloc_slice::<LoopCounter>(loop_count) else {
        drop(loop_points_guard);
        luaD_throw(L, 4);
    };

    let mut loop_counter = loop_counters.as_ptr().cast::<LoopCounter>();

    loop_points.iter().enumerate().filter(|(_, is_loop)| **is_loop).for_each(|(pc, _)| {
        loop_counter.write(LoopCounter { pc: pc as u32, count: 0, trace: None });
        loop_counter = loop_counter.add(1);
    });

    (*proto).loop_cnts = loop_counters.as_ptr().cast();
    (*proto).size_loop_cnts = loop_count as u32;
}
