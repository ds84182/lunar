use crate::*;

unsafe extern "C-unwind" fn error_expected(mut ls: *mut LexState, mut token: i32) -> ! {
    luaX_syntaxerror(
        ls,
        luaO_pushfstring((*ls).L, c"%s expected".as_ptr(), luaX_token2str(ls, token)),
    );
}
unsafe extern "C-unwind" fn errorlimit(
    mut fs: *mut FuncState,
    mut limit: i32,
    mut what: *const std::ffi::c_char,
) -> ! {
    let mut L: *mut lua_State = (*(*fs).ls).L;
    let mut msg: *const std::ffi::c_char = 0 as *const std::ffi::c_char;
    let mut line: i32 = (*(*fs).f).linedefined;
    let mut where_0: *const std::ffi::c_char = if line == 0 {
        c"main function".as_ptr()
    } else {
        luaO_pushfstring(L, c"function at line %d".as_ptr(), line)
    };
    msg = luaO_pushfstring(
        L,
        c"too many %s (limit is %d) in %s".as_ptr(),
        what,
        limit,
        where_0,
    );
    luaX_syntaxerror((*fs).ls, msg);
}
unsafe extern "C-unwind" fn checklimit(
    mut fs: *mut FuncState,
    mut v: i32,
    mut l: i32,
    mut what: *const std::ffi::c_char,
) {
    if v > l {
        errorlimit(fs, l, what);
    }
}
unsafe extern "C-unwind" fn testnext(mut ls: *mut LexState, mut c: i32) -> i32 {
    if (*ls).t.token == c {
        luaX_next(ls);
        return 1 as i32;
    } else {
        return 0;
    };
}
unsafe extern "C-unwind" fn check(mut ls: *mut LexState, mut c: i32) {
    if (*ls).t.token != c {
        error_expected(ls, c);
    }
}
unsafe extern "C-unwind" fn checknext(mut ls: *mut LexState, mut c: i32) {
    check(ls, c);
    luaX_next(ls);
}
unsafe extern "C-unwind" fn check_match(
    mut ls: *mut LexState,
    mut what: i32,
    mut who: i32,
    mut where_0: i32,
) {
    if ((testnext(ls, what) == 0) as i32 != 0) as i32 as std::ffi::c_long != 0 {
        if where_0 == (*ls).linenumber {
            error_expected(ls, what);
        } else {
            luaX_syntaxerror(
                ls,
                luaO_pushfstring(
                    (*ls).L,
                    c"%s expected (to close %s at line %d)".as_ptr(),
                    luaX_token2str(ls, what),
                    luaX_token2str(ls, who),
                    where_0,
                ),
            );
        }
    }
}
unsafe extern "C-unwind" fn str_checkname(mut ls: *mut LexState) -> *mut TString {
    let mut ts: *mut TString = 0 as *mut TString;
    check(ls, TK_NAME as i32);
    ts = (*ls).t.seminfo.ts;
    luaX_next(ls);
    return ts;
}
unsafe extern "C-unwind" fn init_exp(mut e: *mut ExpDesc, mut k: ExpKind, mut i: i32) {
    (*e).t = -(1 as i32);
    (*e).f = (*e).t;
    (*e).k = k;
    (*e).u.info = i;
}
unsafe extern "C-unwind" fn codestring(mut e: *mut ExpDesc, mut s: *mut TString) {
    (*e).t = -(1 as i32);
    (*e).f = (*e).t;
    (*e).k = VKSTR;
    (*e).u.strval = s;
}
unsafe extern "C-unwind" fn codename(mut ls: *mut LexState, mut e: *mut ExpDesc) {
    codestring(e, str_checkname(ls));
}
unsafe extern "C-unwind" fn registerlocalvar(
    mut ls: *mut LexState,
    mut fs: *mut FuncState,
    mut varname: *mut TString,
) -> i32 {
    let mut f: *mut Proto = (*fs).f;
    let mut oldsize: i32 = (*f).sizelocvars;
    (*f).locvars = luaM_growaux_(
        (*ls).L,
        (*f).locvars as *mut c_void,
        (*fs).ndebugvars as i32,
        &mut (*f).sizelocvars,
        size_of::<LocVar>() as usize as i32,
        (if 32767 as i32 as size_t <= (!(0 as size_t)).wrapping_div(size_of::<LocVar>() as usize) {
            32767
        } else {
            (!(0 as size_t)).wrapping_div(size_of::<LocVar>() as usize) as u32
        }) as i32,
        c"local variables".as_ptr(),
    ) as *mut LocVar;
    while oldsize < (*f).sizelocvars {
        let fresh95 = oldsize;
        oldsize = oldsize + 1;
        let ref mut fresh96 = (*((*f).locvars).offset(fresh95 as isize)).varname;
        *fresh96 = 0 as *mut TString;
    }
    let ref mut fresh97 = (*((*f).locvars).offset((*fs).ndebugvars as isize)).varname;
    *fresh97 = varname;
    (*((*f).locvars).offset((*fs).ndebugvars as isize)).startpc = (*fs).pc;
    if (*f).marked as i32 & (1 as i32) << 5 as i32 != 0
        && (*varname).marked as i32 & ((1 as i32) << 3 as i32 | (1 as i32) << 4 as i32) != 0
    {
        luaC_barrier_(
            (*ls).L,
            &mut (*(f as *mut GCUnion)).gc,
            &mut (*(varname as *mut GCUnion)).gc,
        );
    } else {
    };
    let fresh98 = (*fs).ndebugvars;
    (*fs).ndebugvars = (*fs).ndebugvars + 1;
    return fresh98 as i32;
}
unsafe extern "C-unwind" fn new_localvar(mut ls: *mut LexState, mut name: *mut TString) -> i32 {
    let mut L: *mut lua_State = (*ls).L;
    let mut fs: *mut FuncState = (*ls).fs;
    let mut dyd: *mut Dyndata = (*ls).dyd;
    let mut var: *mut Vardesc = 0 as *mut Vardesc;
    checklimit(
        fs,
        (*dyd).actvar.n + 1 as i32 - (*fs).firstlocal,
        200,
        c"local variables".as_ptr(),
    );
    (*dyd).actvar.arr = luaM_growaux_(
        L,
        (*dyd).actvar.arr as *mut c_void,
        (*dyd).actvar.n + 1 as i32,
        &mut (*dyd).actvar.size,
        size_of::<Vardesc>() as usize as i32,
        (if (32767 as i32 * 2 as i32 + 1 as i32) as size_t
            <= (!(0 as size_t)).wrapping_div(size_of::<Vardesc>() as usize)
        {
            (32767 as i32 * 2 as i32 + 1 as i32) as u32
        } else {
            (!(0 as size_t)).wrapping_div(size_of::<Vardesc>() as usize) as u32
        }) as i32,
        c"local variables".as_ptr(),
    ) as *mut Vardesc;
    let fresh99 = (*dyd).actvar.n;
    (*dyd).actvar.n = (*dyd).actvar.n + 1;
    var = &mut *((*dyd).actvar.arr).offset(fresh99 as isize) as *mut Vardesc;
    (*var).vd.kind = 0 as lu_byte;
    (*var).vd.name = name;
    return (*dyd).actvar.n - 1 as i32 - (*fs).firstlocal;
}
unsafe extern "C-unwind" fn getlocalvardesc(mut fs: *mut FuncState, mut vidx: i32) -> *mut Vardesc {
    return &mut *((*(*(*fs).ls).dyd).actvar.arr).offset(((*fs).firstlocal + vidx) as isize)
        as *mut Vardesc;
}
unsafe extern "C-unwind" fn reglevel(mut fs: *mut FuncState, mut nvar: i32) -> i32 {
    loop {
        let fresh100 = nvar;
        nvar = nvar - 1;
        if !(fresh100 > 0) {
            break;
        }
        let mut vd: *mut Vardesc = getlocalvardesc(fs, nvar);
        if (*vd).vd.kind as i32 != 3 as i32 {
            return (*vd).vd.ridx as i32 + 1 as i32;
        }
    }
    return 0;
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaY_nvarstack(mut fs: *mut FuncState) -> i32 {
    return reglevel(fs, (*fs).nactvar as i32);
}
unsafe extern "C-unwind" fn localdebuginfo(mut fs: *mut FuncState, mut vidx: i32) -> *mut LocVar {
    let mut vd: *mut Vardesc = getlocalvardesc(fs, vidx);
    if (*vd).vd.kind as i32 == 3 as i32 {
        return 0 as *mut LocVar;
    } else {
        let mut idx: i32 = (*vd).vd.pidx as i32;
        return &mut *((*(*fs).f).locvars).offset(idx as isize) as *mut LocVar;
    };
}
unsafe extern "C-unwind" fn init_var(mut fs: *mut FuncState, mut e: *mut ExpDesc, mut vidx: i32) {
    (*e).t = -(1 as i32);
    (*e).f = (*e).t;
    (*e).k = VLOCAL;
    (*e).u.var.vidx = vidx as u16;
    (*e).u.var.ridx = (*getlocalvardesc(fs, vidx)).vd.ridx;
}
unsafe extern "C-unwind" fn check_readonly(mut ls: *mut LexState, mut e: *mut ExpDesc) {
    let mut fs: *mut FuncState = (*ls).fs;
    let mut varname: *mut TString = 0 as *mut TString;
    match (*e).k as u32 {
        11 => {
            varname = (*((*(*ls).dyd).actvar.arr).offset((*e).u.info as isize))
                .vd
                .name;
        }
        9 => {
            let mut vardesc: *mut Vardesc = getlocalvardesc(fs, (*e).u.var.vidx as i32);
            if (*vardesc).vd.kind as i32 != 0 {
                varname = (*vardesc).vd.name;
            }
        }
        10 => {
            let mut up: *mut Upvaldesc =
                &mut *((*(*fs).f).upvalues).offset((*e).u.info as isize) as *mut Upvaldesc;
            if (*up).kind as i32 != 0 {
                varname = (*up).name;
            }
        }
        _ => return,
    }
    if !varname.is_null() {
        let mut msg: *const std::ffi::c_char = luaO_pushfstring(
            (*ls).L,
            c"attempt to assign to const variable '%s'".as_ptr(),
            ((*varname).contents).as_mut_ptr(),
        );
        luaK_semerror(ls, msg);
    }
}
unsafe extern "C-unwind" fn adjustlocalvars(mut ls: *mut LexState, mut nvars: i32) {
    let mut fs: *mut FuncState = (*ls).fs;
    let mut reglevel_0: i32 = luaY_nvarstack(fs);
    let mut i: i32 = 0;
    i = 0;
    while i < nvars {
        let fresh101 = (*fs).nactvar;
        (*fs).nactvar = ((*fs).nactvar).wrapping_add(1);
        let mut vidx: i32 = fresh101 as i32;
        let mut var: *mut Vardesc = getlocalvardesc(fs, vidx);
        let fresh102 = reglevel_0;
        reglevel_0 = reglevel_0 + 1;
        (*var).vd.ridx = fresh102 as lu_byte;
        (*var).vd.pidx = registerlocalvar(ls, fs, (*var).vd.name) as i16;
        i += 1;
        i;
    }
}
unsafe extern "C-unwind" fn removevars(mut fs: *mut FuncState, mut tolevel: i32) {
    (*(*(*fs).ls).dyd).actvar.n -= (*fs).nactvar as i32 - tolevel;
    while (*fs).nactvar as i32 > tolevel {
        (*fs).nactvar = ((*fs).nactvar).wrapping_sub(1);
        let mut var: *mut LocVar = localdebuginfo(fs, (*fs).nactvar as i32);
        if !var.is_null() {
            (*var).endpc = (*fs).pc;
        }
    }
}
unsafe extern "C-unwind" fn searchupvalue(mut fs: *mut FuncState, mut name: *mut TString) -> i32 {
    let mut i: i32 = 0;
    let mut up: *mut Upvaldesc = (*(*fs).f).upvalues;
    i = 0;
    while i < (*fs).nups as i32 {
        if (*up.offset(i as isize)).name == name {
            return i;
        }
        i += 1;
        i;
    }
    return -(1 as i32);
}
unsafe extern "C-unwind" fn allocupvalue(mut fs: *mut FuncState) -> *mut Upvaldesc {
    let mut f: *mut Proto = (*fs).f;
    let mut oldsize: i32 = (*f).sizeupvalues;
    checklimit(
        fs,
        (*fs).nups as i32 + 1 as i32,
        255 as i32,
        c"upvalues".as_ptr(),
    );
    (*f).upvalues = luaM_growaux_(
        (*(*fs).ls).L,
        (*f).upvalues as *mut c_void,
        (*fs).nups as i32,
        &mut (*f).sizeupvalues,
        size_of::<Upvaldesc>() as usize as i32,
        (if 255 as i32 as size_t <= (!(0 as size_t)).wrapping_div(size_of::<Upvaldesc>() as usize) {
            255
        } else {
            (!(0 as size_t)).wrapping_div(size_of::<Upvaldesc>() as usize) as u32
        }) as i32,
        c"upvalues".as_ptr(),
    ) as *mut Upvaldesc;
    while oldsize < (*f).sizeupvalues {
        let fresh103 = oldsize;
        oldsize = oldsize + 1;
        let ref mut fresh104 = (*((*f).upvalues).offset(fresh103 as isize)).name;
        *fresh104 = 0 as *mut TString;
    }
    let fresh105 = (*fs).nups;
    (*fs).nups = ((*fs).nups).wrapping_add(1);
    return &mut *((*f).upvalues).offset(fresh105 as isize) as *mut Upvaldesc;
}
unsafe extern "C-unwind" fn newupvalue(
    mut fs: *mut FuncState,
    mut name: *mut TString,
    mut v: *mut ExpDesc,
) -> i32 {
    let mut up: *mut Upvaldesc = allocupvalue(fs);
    let mut prev: *mut FuncState = (*fs).prev;
    if (*v).k as u32 == VLOCAL as i32 as u32 {
        (*up).instack = 1 as i32 as lu_byte;
        (*up).idx = (*v).u.var.ridx;
        (*up).kind = (*getlocalvardesc(prev, (*v).u.var.vidx as i32)).vd.kind;
    } else {
        (*up).instack = 0 as lu_byte;
        (*up).idx = (*v).u.info as lu_byte;
        (*up).kind = (*((*(*prev).f).upvalues).offset((*v).u.info as isize)).kind;
    }
    (*up).name = name;
    if (*(*fs).f).marked as i32 & (1 as i32) << 5 as i32 != 0
        && (*name).marked as i32 & ((1 as i32) << 3 as i32 | (1 as i32) << 4 as i32) != 0
    {
        luaC_barrier_(
            (*(*fs).ls).L,
            &mut (*((*fs).f as *mut GCUnion)).gc,
            &mut (*(name as *mut GCUnion)).gc,
        );
    } else {
    };
    return (*fs).nups as i32 - 1 as i32;
}
unsafe extern "C-unwind" fn searchvar(
    mut fs: *mut FuncState,
    mut n: *mut TString,
    mut var: *mut ExpDesc,
) -> i32 {
    let mut i: i32 = 0;
    i = (*fs).nactvar as i32 - 1 as i32;
    while i >= 0 {
        let mut vd: *mut Vardesc = getlocalvardesc(fs, i);
        if n == (*vd).vd.name {
            if (*vd).vd.kind as i32 == 3 as i32 {
                init_exp(var, VCONST, (*fs).firstlocal + i);
            } else {
                init_var(fs, var, i);
            }
            return (*var).k as i32;
        }
        i -= 1;
        i;
    }
    return -(1 as i32);
}
unsafe extern "C-unwind" fn markupval(mut fs: *mut FuncState, mut level: i32) {
    let mut bl: *mut BlockCnt = (*fs).bl;
    while (*bl).nactvar as i32 > level {
        bl = (*bl).previous;
    }
    (*bl).upval = 1 as i32 as lu_byte;
    (*fs).needclose = 1 as i32 as lu_byte;
}
unsafe extern "C-unwind" fn marktobeclosed(mut fs: *mut FuncState) {
    let mut bl: *mut BlockCnt = (*fs).bl;
    (*bl).upval = 1 as i32 as lu_byte;
    (*bl).insidetbc = 1 as i32 as lu_byte;
    (*fs).needclose = 1 as i32 as lu_byte;
}
unsafe extern "C-unwind" fn singlevaraux(
    mut fs: *mut FuncState,
    mut n: *mut TString,
    mut var: *mut ExpDesc,
    mut base: i32,
) {
    if fs.is_null() {
        init_exp(var, VVOID, 0);
    } else {
        let mut v: i32 = searchvar(fs, n, var);
        if v >= 0 {
            if v == VLOCAL as i32 && base == 0 {
                markupval(fs, (*var).u.var.vidx as i32);
            }
        } else {
            let mut idx: i32 = searchupvalue(fs, n);
            if idx < 0 {
                singlevaraux((*fs).prev, n, var, 0);
                if (*var).k as u32 == VLOCAL as i32 as u32
                    || (*var).k as u32 == VUPVAL as i32 as u32
                {
                    idx = newupvalue(fs, n, var);
                } else {
                    return;
                }
            }
            init_exp(var, VUPVAL, idx);
        }
    };
}
unsafe extern "C-unwind" fn singlevar(mut ls: *mut LexState, mut var: *mut ExpDesc) {
    let mut varname: *mut TString = str_checkname(ls);
    let mut fs: *mut FuncState = (*ls).fs;
    singlevaraux(fs, varname, var, 1 as i32);
    if (*var).k as u32 == VVOID as i32 as u32 {
        let mut key: ExpDesc = ExpDesc {
            k: VVOID,
            u: ExpVariant { ival: 0 },
            t: 0,
            f: 0,
        };
        singlevaraux(fs, (*ls).envn, var, 1 as i32);
        luaK_exp2anyregup(fs, var);
        codestring(&mut key, varname);
        luaK_indexed(fs, var, &mut key);
    }
}
unsafe extern "C-unwind" fn adjust_assign(
    mut ls: *mut LexState,
    mut nvars: i32,
    mut nexps: i32,
    mut e: *mut ExpDesc,
) {
    let mut fs: *mut FuncState = (*ls).fs;
    let mut needed: i32 = nvars - nexps;
    if (*e).k as u32 == VCALL as i32 as u32 || (*e).k as u32 == VVARARG as i32 as u32 {
        let mut extra: i32 = needed + 1 as i32;
        if extra < 0 {
            extra = 0;
        }
        luaK_setreturns(fs, e, extra);
    } else {
        if (*e).k as u32 != VVOID as i32 as u32 {
            luaK_exp2nextreg(fs, e);
        }
        if needed > 0 {
            luaK_nil(fs, (*fs).freereg as i32, needed);
        }
    }
    if needed > 0 {
        luaK_reserveregs(fs, needed);
    } else {
        (*fs).freereg = ((*fs).freereg as i32 + needed) as lu_byte;
    };
}
unsafe extern "C-unwind" fn jumpscopeerror(mut ls: *mut LexState, mut gt: *mut Labeldesc) -> ! {
    let mut varname: *const std::ffi::c_char =
        ((*(*getlocalvardesc((*ls).fs, (*gt).nactvar as i32)).vd.name).contents).as_mut_ptr();
    let mut msg: *const std::ffi::c_char =
        c"<goto %s> at line %d jumps into the scope of local '%s'".as_ptr();
    msg = luaO_pushfstring(
        (*ls).L,
        msg,
        ((*(*gt).name).contents).as_mut_ptr(),
        (*gt).line,
        varname,
    );
    luaK_semerror(ls, msg);
}
unsafe extern "C-unwind" fn solvegoto(
    mut ls: *mut LexState,
    mut g: i32,
    mut label: *mut Labeldesc,
) {
    let mut i: i32 = 0;
    let mut gl: *mut Labellist = &mut (*(*ls).dyd).gt;
    let mut gt: *mut Labeldesc = &mut *((*gl).arr).offset(g as isize) as *mut Labeldesc;
    if ((((*gt).nactvar as i32) < (*label).nactvar as i32) as i32 != 0) as i32 as std::ffi::c_long
        != 0
    {
        jumpscopeerror(ls, gt);
    }
    luaK_patchlist((*ls).fs, (*gt).pc, (*label).pc);
    i = g;
    while i < (*gl).n - 1 as i32 {
        *((*gl).arr).offset(i as isize) = *((*gl).arr).offset((i + 1 as i32) as isize);
        i += 1;
        i;
    }
    (*gl).n -= 1;
    (*gl).n;
}
unsafe extern "C-unwind" fn findlabel(
    mut ls: *mut LexState,
    mut name: *mut TString,
) -> *mut Labeldesc {
    let mut i: i32 = 0;
    let mut dyd: *mut Dyndata = (*ls).dyd;
    i = (*(*ls).fs).firstlabel;
    while i < (*dyd).label.n {
        let mut lb: *mut Labeldesc = &mut *((*dyd).label.arr).offset(i as isize) as *mut Labeldesc;
        if (*lb).name == name {
            return lb;
        }
        i += 1;
        i;
    }
    return 0 as *mut Labeldesc;
}
unsafe extern "C-unwind" fn newlabelentry(
    mut ls: *mut LexState,
    mut l: *mut Labellist,
    mut name: *mut TString,
    mut line: i32,
    mut pc: i32,
) -> i32 {
    let mut n: i32 = (*l).n;
    (*l).arr = luaM_growaux_(
        (*ls).L,
        (*l).arr as *mut c_void,
        n,
        &mut (*l).size,
        size_of::<Labeldesc>() as usize as i32,
        (if 32767 as i32 as size_t <= (!(0 as size_t)).wrapping_div(size_of::<Labeldesc>() as usize)
        {
            32767
        } else {
            (!(0 as size_t)).wrapping_div(size_of::<Labeldesc>() as usize) as u32
        }) as i32,
        c"labels/gotos".as_ptr(),
    ) as *mut Labeldesc;
    let ref mut fresh106 = (*((*l).arr).offset(n as isize)).name;
    *fresh106 = name;
    (*((*l).arr).offset(n as isize)).line = line;
    (*((*l).arr).offset(n as isize)).nactvar = (*(*ls).fs).nactvar;
    (*((*l).arr).offset(n as isize)).close = 0 as lu_byte;
    (*((*l).arr).offset(n as isize)).pc = pc;
    (*l).n = n + 1 as i32;
    return n;
}
unsafe extern "C-unwind" fn newgotoentry(
    mut ls: *mut LexState,
    mut name: *mut TString,
    mut line: i32,
    mut pc: i32,
) -> i32 {
    return newlabelentry(ls, &mut (*(*ls).dyd).gt, name, line, pc);
}
unsafe extern "C-unwind" fn solvegotos(mut ls: *mut LexState, mut lb: *mut Labeldesc) -> i32 {
    let mut gl: *mut Labellist = &mut (*(*ls).dyd).gt;
    let mut i: i32 = (*(*(*ls).fs).bl).firstgoto;
    let mut needsclose: i32 = 0;
    while i < (*gl).n {
        if (*((*gl).arr).offset(i as isize)).name == (*lb).name {
            needsclose |= (*((*gl).arr).offset(i as isize)).close as i32;
            solvegoto(ls, i, lb);
        } else {
            i += 1;
            i;
        }
    }
    return needsclose;
}
unsafe extern "C-unwind" fn createlabel(
    mut ls: *mut LexState,
    mut name: *mut TString,
    mut line: i32,
    mut last: i32,
) -> i32 {
    let mut fs: *mut FuncState = (*ls).fs;
    let mut ll: *mut Labellist = &mut (*(*ls).dyd).label;
    let mut l: i32 = newlabelentry(ls, ll, name, line, luaK_getlabel(fs));
    if last != 0 {
        (*((*ll).arr).offset(l as isize)).nactvar = (*(*fs).bl).nactvar;
    }
    if solvegotos(ls, &mut *((*ll).arr).offset(l as isize)) != 0 {
        luaK_codeABCk(fs, OP_CLOSE, luaY_nvarstack(fs), 0, 0, 0);
        return 1 as i32;
    }
    return 0;
}
unsafe extern "C-unwind" fn movegotosout(mut fs: *mut FuncState, mut bl: *mut BlockCnt) {
    let mut i: i32 = 0;
    let mut gl: *mut Labellist = &mut (*(*(*fs).ls).dyd).gt;
    i = (*bl).firstgoto;
    while i < (*gl).n {
        let mut gt: *mut Labeldesc = &mut *((*gl).arr).offset(i as isize) as *mut Labeldesc;
        if reglevel(fs, (*gt).nactvar as i32) > reglevel(fs, (*bl).nactvar as i32) {
            (*gt).close = ((*gt).close as i32 | (*bl).upval as i32) as lu_byte;
        }
        (*gt).nactvar = (*bl).nactvar;
        i += 1;
        i;
    }
}
unsafe extern "C-unwind" fn enterblock(
    mut fs: *mut FuncState,
    mut bl: *mut BlockCnt,
    mut isloop: lu_byte,
) {
    (*bl).isloop = isloop;
    (*bl).nactvar = (*fs).nactvar;
    (*bl).firstlabel = (*(*(*fs).ls).dyd).label.n;
    (*bl).firstgoto = (*(*(*fs).ls).dyd).gt.n;
    (*bl).upval = 0 as lu_byte;
    (*bl).insidetbc =
        (!((*fs).bl).is_null() && (*(*fs).bl).insidetbc as i32 != 0) as i32 as lu_byte;
    (*bl).previous = (*fs).bl;
    (*fs).bl = bl;
}
unsafe extern "C-unwind" fn undefgoto(mut ls: *mut LexState, mut gt: *mut Labeldesc) -> ! {
    let mut msg: *const std::ffi::c_char = 0 as *const std::ffi::c_char;
    if (*gt).name
        == luaS_newlstr(
            (*ls).L,
            c"break".as_ptr(),
            (size_of::<[std::ffi::c_char; 6]>() as usize)
                .wrapping_div(size_of::<std::ffi::c_char>() as usize)
                .wrapping_sub(1),
        )
    {
        msg = c"break outside loop at line %d".as_ptr();
        msg = luaO_pushfstring((*ls).L, msg, (*gt).line);
    } else {
        msg = c"no visible label '%s' for <goto> at line %d".as_ptr();
        msg = luaO_pushfstring(
            (*ls).L,
            msg,
            ((*(*gt).name).contents).as_mut_ptr(),
            (*gt).line,
        );
    }
    luaK_semerror(ls, msg);
}
unsafe extern "C-unwind" fn leaveblock(mut fs: *mut FuncState) {
    let mut bl: *mut BlockCnt = (*fs).bl;
    let mut ls: *mut LexState = (*fs).ls;
    let mut hasclose: i32 = 0;
    let mut stklevel: i32 = reglevel(fs, (*bl).nactvar as i32);
    removevars(fs, (*bl).nactvar as i32);
    if (*bl).isloop != 0 {
        hasclose = createlabel(
            ls,
            luaS_newlstr(
                (*ls).L,
                c"break".as_ptr(),
                (size_of::<[std::ffi::c_char; 6]>() as usize)
                    .wrapping_div(size_of::<std::ffi::c_char>() as usize)
                    .wrapping_sub(1),
            ),
            0,
            0,
        );
    }
    if hasclose == 0 && !((*bl).previous).is_null() && (*bl).upval as i32 != 0 {
        luaK_codeABCk(fs, OP_CLOSE, stklevel, 0, 0, 0);
    }
    (*fs).freereg = stklevel as lu_byte;
    (*(*ls).dyd).label.n = (*bl).firstlabel;
    (*fs).bl = (*bl).previous;
    if !((*bl).previous).is_null() {
        movegotosout(fs, bl);
    } else if (*bl).firstgoto < (*(*ls).dyd).gt.n {
        undefgoto(
            ls,
            &mut *((*(*ls).dyd).gt.arr).offset((*bl).firstgoto as isize),
        );
    }
}
unsafe extern "C-unwind" fn addprototype(mut ls: *mut LexState) -> *mut Proto {
    let mut clp: *mut Proto = 0 as *mut Proto;
    let mut L: *mut lua_State = (*ls).L;
    let mut fs: *mut FuncState = (*ls).fs;
    let mut f: *mut Proto = (*fs).f;
    if (*fs).np >= (*f).sizep {
        let mut oldsize: i32 = (*f).sizep;
        (*f).p = luaM_growaux_(
            L,
            (*f).p as *mut c_void,
            (*fs).np,
            &mut (*f).sizep,
            size_of::<*mut Proto>() as usize as i32,
            (if (((1 as i32) << 8 as i32 + 8 as i32 + 1 as i32) - 1 as i32) as size_t
                <= (!(0 as size_t)).wrapping_div(size_of::<*mut Proto>() as usize)
            {
                (((1 as i32) << 8 as i32 + 8 as i32 + 1 as i32) - 1 as i32) as u32
            } else {
                (!(0 as size_t)).wrapping_div(size_of::<*mut Proto>() as usize) as u32
            }) as i32,
            c"functions".as_ptr(),
        ) as *mut *mut Proto;
        while oldsize < (*f).sizep {
            let fresh107 = oldsize;
            oldsize = oldsize + 1;
            let ref mut fresh108 = *((*f).p).offset(fresh107 as isize);
            *fresh108 = 0 as *mut Proto;
        }
    }
    clp = luaF_newproto(L);
    let fresh109 = (*fs).np;
    (*fs).np = (*fs).np + 1;
    let ref mut fresh110 = *((*f).p).offset(fresh109 as isize);
    *fresh110 = clp;
    if (*f).marked as i32 & (1 as i32) << 5 as i32 != 0
        && (*clp).marked as i32 & ((1 as i32) << 3 as i32 | (1 as i32) << 4 as i32) != 0
    {
        luaC_barrier_(
            L,
            &mut (*(f as *mut GCUnion)).gc,
            &mut (*(clp as *mut GCUnion)).gc,
        );
    } else {
    };
    return clp;
}
unsafe extern "C-unwind" fn codeclosure(mut ls: *mut LexState, mut v: *mut ExpDesc) {
    let mut fs: *mut FuncState = (*(*ls).fs).prev;
    init_exp(
        v,
        VRELOC,
        luaK_codeABx(fs, OP_CLOSURE, 0, ((*fs).np - 1 as i32) as u32),
    );
    luaK_exp2nextreg(fs, v);
}
unsafe extern "C-unwind" fn open_func(
    mut ls: *mut LexState,
    mut fs: *mut FuncState,
    mut bl: *mut BlockCnt,
) {
    let mut f: *mut Proto = (*fs).f;
    (*fs).prev = (*ls).fs;
    (*fs).ls = ls;
    (*ls).fs = fs;
    (*fs).pc = 0;
    (*fs).previousline = (*f).linedefined;
    (*fs).iwthabs = 0 as lu_byte;
    (*fs).lasttarget = 0;
    (*fs).freereg = 0 as lu_byte;
    (*fs).nk = 0;
    (*fs).nabslineinfo = 0;
    (*fs).np = 0;
    (*fs).nups = 0 as lu_byte;
    (*fs).ndebugvars = 0 as i16;
    (*fs).nactvar = 0 as lu_byte;
    (*fs).needclose = 0 as lu_byte;
    (*fs).firstlocal = (*(*ls).dyd).actvar.n;
    (*fs).firstlabel = (*(*ls).dyd).label.n;
    (*fs).bl = 0 as *mut BlockCnt;
    (*f).source = (*ls).source;
    if (*f).marked as i32 & (1 as i32) << 5 as i32 != 0
        && (*(*f).source).marked as i32 & ((1 as i32) << 3 as i32 | (1 as i32) << 4 as i32) != 0
    {
        luaC_barrier_(
            (*ls).L,
            &mut (*(f as *mut GCUnion)).gc,
            &mut (*((*f).source as *mut GCUnion)).gc,
        );
    } else {
    };
    (*f).maxstacksize = 2 as i32 as lu_byte;
    enterblock(fs, bl, 0 as lu_byte);
}
unsafe extern "C-unwind" fn close_func(mut ls: *mut LexState) {
    let mut L: *mut lua_State = (*ls).L;
    let mut fs: *mut FuncState = (*ls).fs;
    let mut f: *mut Proto = (*fs).f;
    luaK_ret(fs, luaY_nvarstack(fs), 0);
    leaveblock(fs);
    luaK_finish(fs);
    (*f).code = luaM_shrinkvector_(
        L,
        (*f).code as *mut c_void,
        &mut (*f).sizecode,
        (*fs).pc,
        size_of::<Instruction>() as usize as i32,
    ) as *mut Instruction;
    (*f).lineinfo = luaM_shrinkvector_(
        L,
        (*f).lineinfo as *mut c_void,
        &mut (*f).sizelineinfo,
        (*fs).pc,
        size_of::<ls_byte>() as usize as i32,
    ) as *mut ls_byte;
    (*f).abslineinfo = luaM_shrinkvector_(
        L,
        (*f).abslineinfo as *mut c_void,
        &mut (*f).sizeabslineinfo,
        (*fs).nabslineinfo,
        size_of::<AbsLineInfo>() as usize as i32,
    ) as *mut AbsLineInfo;
    (*f).k = luaM_shrinkvector_(
        L,
        (*f).k as *mut c_void,
        &mut (*f).sizek,
        (*fs).nk,
        size_of::<TValue>() as usize as i32,
    ) as *mut TValue;
    (*f).p = luaM_shrinkvector_(
        L,
        (*f).p as *mut c_void,
        &mut (*f).sizep,
        (*fs).np,
        size_of::<*mut Proto>() as usize as i32,
    ) as *mut *mut Proto;
    (*f).locvars = luaM_shrinkvector_(
        L,
        (*f).locvars as *mut c_void,
        &mut (*f).sizelocvars,
        (*fs).ndebugvars as i32,
        size_of::<LocVar>() as usize as i32,
    ) as *mut LocVar;
    (*f).upvalues = luaM_shrinkvector_(
        L,
        (*f).upvalues as *mut c_void,
        &mut (*f).sizeupvalues,
        (*fs).nups as i32,
        size_of::<Upvaldesc>() as usize as i32,
    ) as *mut Upvaldesc;
    luaK_build_loop_counters(L, f);
    (*ls).fs = (*fs).prev;
    if (*(*L).l_G).GCdebt > 0 as l_mem {
        luaC_step(L);
    }
}
unsafe extern "C-unwind" fn block_follow(mut ls: *mut LexState, mut withuntil: i32) -> i32 {
    match (*ls).t.token {
        259 | 260 | 261 | 288 => return 1 as i32,
        276 => return withuntil,
        _ => return 0,
    };
}
unsafe extern "C-unwind" fn statlist(mut ls: *mut LexState) {
    while block_follow(ls, 1 as i32) == 0 {
        if (*ls).t.token == TK_RETURN as i32 {
            statement(ls);
            return;
        }
        statement(ls);
    }
}
unsafe extern "C-unwind" fn fieldsel(mut ls: *mut LexState, mut v: *mut ExpDesc) {
    let mut fs: *mut FuncState = (*ls).fs;
    let mut key: ExpDesc = ExpDesc {
        k: VVOID,
        u: ExpVariant { ival: 0 },
        t: 0,
        f: 0,
    };
    luaK_exp2anyregup(fs, v);
    luaX_next(ls);
    codename(ls, &mut key);
    luaK_indexed(fs, v, &mut key);
}
unsafe extern "C-unwind" fn yindex(mut ls: *mut LexState, mut v: *mut ExpDesc) {
    luaX_next(ls);
    expr(ls, v);
    luaK_exp2val((*ls).fs, v);
    checknext(ls, ']' as i32);
}
unsafe extern "C-unwind" fn recfield(mut ls: *mut LexState, mut cc: *mut ConsControl) {
    let mut fs: *mut FuncState = (*ls).fs;
    let mut reg: i32 = (*(*ls).fs).freereg as i32;
    let mut tab: ExpDesc = ExpDesc {
        k: VVOID,
        u: ExpVariant { ival: 0 },
        t: 0,
        f: 0,
    };
    let mut key: ExpDesc = ExpDesc {
        k: VVOID,
        u: ExpVariant { ival: 0 },
        t: 0,
        f: 0,
    };
    let mut val: ExpDesc = ExpDesc {
        k: VVOID,
        u: ExpVariant { ival: 0 },
        t: 0,
        f: 0,
    };
    if (*ls).t.token == TK_NAME as i32 {
        checklimit(
            fs,
            (*cc).nh,
            2147483647 as i32,
            c"items in a constructor".as_ptr(),
        );
        codename(ls, &mut key);
    } else {
        yindex(ls, &mut key);
    }
    (*cc).nh += 1;
    (*cc).nh;
    checknext(ls, '=' as i32);
    tab = *(*cc).t;
    luaK_indexed(fs, &mut tab, &mut key);
    expr(ls, &mut val);
    luaK_storevar(fs, &mut tab, &mut val);
    (*fs).freereg = reg as lu_byte;
}
unsafe extern "C-unwind" fn closelistfield(mut fs: *mut FuncState, mut cc: *mut ConsControl) {
    if (*cc).v.k as u32 == VVOID as i32 as u32 {
        return;
    }
    luaK_exp2nextreg(fs, &mut (*cc).v);
    (*cc).v.k = VVOID;
    if (*cc).tostore == 50 {
        luaK_setlist(fs, (*(*cc).t).u.info, (*cc).na, (*cc).tostore);
        (*cc).na += (*cc).tostore;
        (*cc).tostore = 0;
    }
}
unsafe extern "C-unwind" fn lastlistfield(mut fs: *mut FuncState, mut cc: *mut ConsControl) {
    if (*cc).tostore == 0 {
        return;
    }
    if (*cc).v.k as u32 == VCALL as i32 as u32 || (*cc).v.k as u32 == VVARARG as i32 as u32 {
        luaK_setreturns(fs, &mut (*cc).v, -(1 as i32));
        luaK_setlist(fs, (*(*cc).t).u.info, (*cc).na, -(1 as i32));
        (*cc).na -= 1;
        (*cc).na;
    } else {
        if (*cc).v.k as u32 != VVOID as i32 as u32 {
            luaK_exp2nextreg(fs, &mut (*cc).v);
        }
        luaK_setlist(fs, (*(*cc).t).u.info, (*cc).na, (*cc).tostore);
    }
    (*cc).na += (*cc).tostore;
}
unsafe extern "C-unwind" fn listfield(mut ls: *mut LexState, mut cc: *mut ConsControl) {
    expr(ls, &mut (*cc).v);
    (*cc).tostore += 1;
    (*cc).tostore;
}
unsafe extern "C-unwind" fn field(mut ls: *mut LexState, mut cc: *mut ConsControl) {
    match (*ls).t.token {
        291 => {
            if luaX_lookahead(ls) != '=' as i32 {
                listfield(ls, cc);
            } else {
                recfield(ls, cc);
            }
        }
        91 => {
            recfield(ls, cc);
        }
        _ => {
            listfield(ls, cc);
        }
    };
}
unsafe extern "C-unwind" fn constructor(mut ls: *mut LexState, mut t: *mut ExpDesc) {
    let mut fs: *mut FuncState = (*ls).fs;
    let mut line: i32 = (*ls).linenumber;
    let mut pc: i32 = luaK_codeABCk(fs, OP_NEWTABLE, 0, 0, 0, 0);
    let mut cc: ConsControl = ConsControl {
        v: ExpDesc {
            k: VVOID,
            u: ExpVariant { ival: 0 },
            t: 0,
            f: 0,
        },
        t: 0 as *mut ExpDesc,
        nh: 0,
        na: 0,
        tostore: 0,
    };
    luaK_code(fs, 0 as Instruction);
    cc.tostore = 0;
    cc.nh = cc.tostore;
    cc.na = cc.nh;
    cc.t = t;
    init_exp(t, VNONRELOC, (*fs).freereg as i32);
    luaK_reserveregs(fs, 1 as i32);
    init_exp(&mut cc.v, VVOID, 0);
    checknext(ls, '{' as i32);
    while !((*ls).t.token == '}' as i32) {
        closelistfield(fs, &mut cc);
        field(ls, &mut cc);
        if !(testnext(ls, ',' as i32) != 0 || testnext(ls, ';' as i32) != 0) {
            break;
        }
    }
    check_match(ls, '}' as i32, '{' as i32, line);
    lastlistfield(fs, &mut cc);
    luaK_settablesize(fs, pc, (*t).u.info, cc.na, cc.nh);
}
unsafe extern "C-unwind" fn setvararg(mut fs: *mut FuncState, mut nparams: i32) {
    (*(*fs).f).is_vararg = 1 as i32 as lu_byte;
    luaK_codeABCk(fs, OP_VARARGPREP, nparams, 0, 0, 0);
}
unsafe extern "C-unwind" fn parlist(mut ls: *mut LexState) {
    let mut fs: *mut FuncState = (*ls).fs;
    let mut f: *mut Proto = (*fs).f;
    let mut nparams: i32 = 0;
    let mut isvararg: i32 = 0;
    if (*ls).t.token != ')' as i32 {
        loop {
            match (*ls).t.token {
                291 => {
                    new_localvar(ls, str_checkname(ls));
                    nparams += 1;
                    nparams;
                }
                280 => {
                    luaX_next(ls);
                    isvararg = 1 as i32;
                }
                _ => {
                    luaX_syntaxerror(ls, c"<name> or '...' expected".as_ptr());
                }
            }
            if !(isvararg == 0 && testnext(ls, ',' as i32) != 0) {
                break;
            }
        }
    }
    adjustlocalvars(ls, nparams);
    (*f).numparams = (*fs).nactvar;
    if isvararg != 0 {
        setvararg(fs, (*f).numparams as i32);
    }
    luaK_reserveregs(fs, (*fs).nactvar as i32);
}
unsafe extern "C-unwind" fn body(
    mut ls: *mut LexState,
    mut e: *mut ExpDesc,
    mut ismethod: i32,
    mut line: i32,
) {
    let mut new_fs: FuncState = FuncState {
        f: 0 as *mut Proto,
        prev: 0 as *mut FuncState,
        ls: 0 as *mut LexState,
        bl: 0 as *mut BlockCnt,
        pc: 0,
        lasttarget: 0,
        previousline: 0,
        nk: 0,
        np: 0,
        nabslineinfo: 0,
        firstlocal: 0,
        firstlabel: 0,
        ndebugvars: 0,
        nactvar: 0,
        nups: 0,
        freereg: 0,
        iwthabs: 0,
        needclose: 0,
    };
    let mut bl: BlockCnt = BlockCnt {
        previous: 0 as *mut BlockCnt,
        firstlabel: 0,
        firstgoto: 0,
        nactvar: 0,
        upval: 0,
        isloop: 0,
        insidetbc: 0,
    };
    new_fs.f = addprototype(ls);
    (*new_fs.f).linedefined = line;
    open_func(ls, &mut new_fs, &mut bl);
    checknext(ls, '(' as i32);
    if ismethod != 0 {
        new_localvar(
            ls,
            luaX_newstring(
                ls,
                c"self".as_ptr(),
                (size_of::<[std::ffi::c_char; 5]>() as usize)
                    .wrapping_div(size_of::<std::ffi::c_char>() as usize)
                    .wrapping_sub(1),
            ),
        );
        adjustlocalvars(ls, 1 as i32);
    }
    parlist(ls);
    checknext(ls, ')' as i32);
    statlist(ls);
    (*new_fs.f).lastlinedefined = (*ls).linenumber;
    check_match(ls, TK_END as i32, TK_FUNCTION as i32, line);
    codeclosure(ls, e);
    close_func(ls);
}
unsafe extern "C-unwind" fn explist(mut ls: *mut LexState, mut v: *mut ExpDesc) -> i32 {
    let mut n: i32 = 1 as i32;
    expr(ls, v);
    while testnext(ls, ',' as i32) != 0 {
        luaK_exp2nextreg((*ls).fs, v);
        expr(ls, v);
        n += 1;
        n;
    }
    return n;
}
unsafe extern "C-unwind" fn funcargs(mut ls: *mut LexState, mut f: *mut ExpDesc) {
    let mut fs: *mut FuncState = (*ls).fs;
    let mut args: ExpDesc = ExpDesc {
        k: VVOID,
        u: ExpVariant { ival: 0 },
        t: 0,
        f: 0,
    };
    let mut base: i32 = 0;
    let mut nparams: i32 = 0;
    let mut line: i32 = (*ls).linenumber;
    match (*ls).t.token {
        40 => {
            luaX_next(ls);
            if (*ls).t.token == ')' as i32 {
                args.k = VVOID;
            } else {
                explist(ls, &mut args);
                if args.k as u32 == VCALL as i32 as u32 || args.k as u32 == VVARARG as i32 as u32 {
                    luaK_setreturns(fs, &mut args, -(1 as i32));
                }
            }
            check_match(ls, ')' as i32, '(' as i32, line);
        }
        123 => {
            constructor(ls, &mut args);
        }
        292 => {
            codestring(&mut args, (*ls).t.seminfo.ts);
            luaX_next(ls);
        }
        _ => {
            luaX_syntaxerror(ls, c"function arguments expected".as_ptr());
        }
    }
    base = (*f).u.info;
    if args.k as u32 == VCALL as i32 as u32 || args.k as u32 == VVARARG as i32 as u32 {
        nparams = -(1 as i32);
    } else {
        if args.k as u32 != VVOID as i32 as u32 {
            luaK_exp2nextreg(fs, &mut args);
        }
        nparams = (*fs).freereg as i32 - (base + 1 as i32);
    }
    init_exp(
        f,
        VCALL,
        luaK_codeABCk(fs, OP_CALL, base, nparams + 1 as i32, 2 as i32, 0),
    );
    luaK_fixline(fs, line);
    (*fs).freereg = (base + 1 as i32) as lu_byte;
}
unsafe extern "C-unwind" fn primaryexp(mut ls: *mut LexState, mut v: *mut ExpDesc) {
    match (*ls).t.token {
        40 => {
            let mut line: i32 = (*ls).linenumber;
            luaX_next(ls);
            expr(ls, v);
            check_match(ls, ')' as i32, '(' as i32, line);
            luaK_dischargevars((*ls).fs, v);
            return;
        }
        291 => {
            singlevar(ls, v);
            return;
        }
        _ => {
            luaX_syntaxerror(ls, c"unexpected symbol".as_ptr());
        }
    };
}
unsafe extern "C-unwind" fn suffixedexp(mut ls: *mut LexState, mut v: *mut ExpDesc) {
    let mut fs: *mut FuncState = (*ls).fs;
    primaryexp(ls, v);
    loop {
        match (*ls).t.token {
            46 => {
                fieldsel(ls, v);
            }
            91 => {
                let mut key: ExpDesc = ExpDesc {
                    k: VVOID,
                    u: ExpVariant { ival: 0 },
                    t: 0,
                    f: 0,
                };
                luaK_exp2anyregup(fs, v);
                yindex(ls, &mut key);
                luaK_indexed(fs, v, &mut key);
            }
            58 => {
                let mut key_0: ExpDesc = ExpDesc {
                    k: VVOID,
                    u: ExpVariant { ival: 0 },
                    t: 0,
                    f: 0,
                };
                luaX_next(ls);
                codename(ls, &mut key_0);
                luaK_self(fs, v, &mut key_0);
                funcargs(ls, v);
            }
            40 | 292 | 123 => {
                luaK_exp2nextreg(fs, v);
                funcargs(ls, v);
            }
            _ => return,
        }
    }
}
unsafe extern "C-unwind" fn simpleexp(mut ls: *mut LexState, mut v: *mut ExpDesc) {
    match (*ls).t.token {
        289 => {
            init_exp(v, VKFLT, 0);
            (*v).u.nval = (*ls).t.seminfo.r;
        }
        290 => {
            init_exp(v, VKINT, 0);
            (*v).u.ival = (*ls).t.seminfo.i;
        }
        292 => {
            codestring(v, (*ls).t.seminfo.ts);
        }
        269 => {
            init_exp(v, VNIL, 0);
        }
        275 => {
            init_exp(v, VTRUE, 0);
        }
        262 => {
            init_exp(v, VFALSE, 0);
        }
        280 => {
            let mut fs: *mut FuncState = (*ls).fs;
            if (*(*fs).f).is_vararg == 0 {
                luaX_syntaxerror(ls, c"cannot use '...' outside a vararg function".as_ptr());
            }
            init_exp(v, VVARARG, luaK_codeABCk(fs, OP_VARARG, 0, 0, 1 as i32, 0));
        }
        123 => {
            constructor(ls, v);
            return;
        }
        264 => {
            luaX_next(ls);
            body(ls, v, 0, (*ls).linenumber);
            return;
        }
        _ => {
            suffixedexp(ls, v);
            return;
        }
    }
    luaX_next(ls);
}
unsafe extern "C-unwind" fn getunopr(mut op: i32) -> UnOpr {
    match op {
        270 => return OPR_NOT,
        45 => return OPR_MINUS,
        126 => return OPR_BNOT,
        35 => return OPR_LEN,
        _ => return OPR_NOUNOPR,
    };
}
unsafe extern "C-unwind" fn getbinopr(mut op: i32) -> BinOpr {
    match op {
        43 => return OPR_ADD,
        45 => return OPR_SUB,
        42 => return OPR_MUL,
        37 => return OPR_MOD,
        94 => return OPR_POW,
        47 => return OPR_DIV,
        278 => return OPR_IDIV,
        38 => return OPR_BAND,
        124 => return OPR_BOR,
        126 => return OPR_BXOR,
        285 => return OPR_SHL,
        286 => return OPR_SHR,
        279 => return OPR_CONCAT,
        284 => return OPR_NE,
        281 => return OPR_EQ,
        60 => return OPR_LT,
        283 => return OPR_LE,
        62 => return OPR_GT,
        282 => return OPR_GE,
        256 => return OPR_AND,
        271 => return OPR_OR,
        _ => return OPR_NOBINOPR,
    };
}
static mut priority: [C2RustUnnamed_14; 21] = [
    {
        let mut init = C2RustUnnamed_14 {
            left: 10 as lu_byte,
            right: 10 as lu_byte,
        };
        init
    },
    {
        let mut init = C2RustUnnamed_14 {
            left: 10 as lu_byte,
            right: 10 as lu_byte,
        };
        init
    },
    {
        let mut init = C2RustUnnamed_14 {
            left: 11 as i32 as lu_byte,
            right: 11 as i32 as lu_byte,
        };
        init
    },
    {
        let mut init = C2RustUnnamed_14 {
            left: 11 as i32 as lu_byte,
            right: 11 as i32 as lu_byte,
        };
        init
    },
    {
        let mut init = C2RustUnnamed_14 {
            left: 14 as i32 as lu_byte,
            right: 13 as i32 as lu_byte,
        };
        init
    },
    {
        let mut init = C2RustUnnamed_14 {
            left: 11 as i32 as lu_byte,
            right: 11 as i32 as lu_byte,
        };
        init
    },
    {
        let mut init = C2RustUnnamed_14 {
            left: 11 as i32 as lu_byte,
            right: 11 as i32 as lu_byte,
        };
        init
    },
    {
        let mut init = C2RustUnnamed_14 {
            left: 6 as i32 as lu_byte,
            right: 6 as i32 as lu_byte,
        };
        init
    },
    {
        let mut init = C2RustUnnamed_14 {
            left: 4 as i32 as lu_byte,
            right: 4 as i32 as lu_byte,
        };
        init
    },
    {
        let mut init = C2RustUnnamed_14 {
            left: 5 as i32 as lu_byte,
            right: 5 as i32 as lu_byte,
        };
        init
    },
    {
        let mut init = C2RustUnnamed_14 {
            left: 7 as i32 as lu_byte,
            right: 7 as i32 as lu_byte,
        };
        init
    },
    {
        let mut init = C2RustUnnamed_14 {
            left: 7 as i32 as lu_byte,
            right: 7 as i32 as lu_byte,
        };
        init
    },
    {
        let mut init = C2RustUnnamed_14 {
            left: 9 as i32 as lu_byte,
            right: 8 as i32 as lu_byte,
        };
        init
    },
    {
        let mut init = C2RustUnnamed_14 {
            left: 3 as i32 as lu_byte,
            right: 3 as i32 as lu_byte,
        };
        init
    },
    {
        let mut init = C2RustUnnamed_14 {
            left: 3 as i32 as lu_byte,
            right: 3 as i32 as lu_byte,
        };
        init
    },
    {
        let mut init = C2RustUnnamed_14 {
            left: 3 as i32 as lu_byte,
            right: 3 as i32 as lu_byte,
        };
        init
    },
    {
        let mut init = C2RustUnnamed_14 {
            left: 3 as i32 as lu_byte,
            right: 3 as i32 as lu_byte,
        };
        init
    },
    {
        let mut init = C2RustUnnamed_14 {
            left: 3 as i32 as lu_byte,
            right: 3 as i32 as lu_byte,
        };
        init
    },
    {
        let mut init = C2RustUnnamed_14 {
            left: 3 as i32 as lu_byte,
            right: 3 as i32 as lu_byte,
        };
        init
    },
    {
        let mut init = C2RustUnnamed_14 {
            left: 2 as i32 as lu_byte,
            right: 2 as i32 as lu_byte,
        };
        init
    },
    {
        let mut init = C2RustUnnamed_14 {
            left: 1 as i32 as lu_byte,
            right: 1 as i32 as lu_byte,
        };
        init
    },
];
unsafe extern "C-unwind" fn subexpr(
    mut ls: *mut LexState,
    mut v: *mut ExpDesc,
    mut limit: i32,
) -> BinOpr {
    let mut op: BinOpr = OPR_ADD;
    let mut uop: UnOpr = OPR_MINUS;
    luaE_incCstack((*ls).L);
    uop = getunopr((*ls).t.token);
    if uop as u32 != OPR_NOUNOPR as i32 as u32 {
        let mut line: i32 = (*ls).linenumber;
        luaX_next(ls);
        subexpr(ls, v, 12 as i32);
        luaK_prefix((*ls).fs, uop, v, line);
    } else {
        simpleexp(ls, v);
    }
    op = getbinopr((*ls).t.token);
    while op as u32 != OPR_NOBINOPR as i32 as u32 && priority[op as usize].left as i32 > limit {
        let mut v2: ExpDesc = ExpDesc {
            k: VVOID,
            u: ExpVariant { ival: 0 },
            t: 0,
            f: 0,
        };
        let mut nextop: BinOpr = OPR_ADD;
        let mut line_0: i32 = (*ls).linenumber;
        luaX_next(ls);
        luaK_infix((*ls).fs, op, v);
        nextop = subexpr(ls, &mut v2, priority[op as usize].right as i32);
        luaK_posfix((*ls).fs, op, v, &mut v2, line_0);
        op = nextop;
    }
    (*(*ls).L).nCcalls = ((*(*ls).L).nCcalls).wrapping_sub(1);
    (*(*ls).L).nCcalls;
    return op;
}
unsafe extern "C-unwind" fn expr(mut ls: *mut LexState, mut v: *mut ExpDesc) {
    subexpr(ls, v, 0);
}
unsafe extern "C-unwind" fn block(mut ls: *mut LexState) {
    let mut fs: *mut FuncState = (*ls).fs;
    let mut bl: BlockCnt = BlockCnt {
        previous: 0 as *mut BlockCnt,
        firstlabel: 0,
        firstgoto: 0,
        nactvar: 0,
        upval: 0,
        isloop: 0,
        insidetbc: 0,
    };
    enterblock(fs, &mut bl, 0 as lu_byte);
    statlist(ls);
    leaveblock(fs);
}
unsafe extern "C-unwind" fn check_conflict(
    mut ls: *mut LexState,
    mut lh: *mut LHS_assign,
    mut v: *mut ExpDesc,
) {
    let mut fs: *mut FuncState = (*ls).fs;
    let mut extra: i32 = (*fs).freereg as i32;
    let mut conflict: i32 = 0;
    while !lh.is_null() {
        if VINDEXED as i32 as u32 <= (*lh).v.k as u32 && (*lh).v.k as u32 <= VINDEXSTR as i32 as u32
        {
            if (*lh).v.k as u32 == VINDEXUP as i32 as u32 {
                if (*v).k as u32 == VUPVAL as i32 as u32 && (*lh).v.u.ind.t as i32 == (*v).u.info {
                    conflict = 1 as i32;
                    (*lh).v.k = VINDEXSTR;
                    (*lh).v.u.ind.t = extra as lu_byte;
                }
            } else {
                if (*v).k as u32 == VLOCAL as i32 as u32
                    && (*lh).v.u.ind.t as i32 == (*v).u.var.ridx as i32
                {
                    conflict = 1 as i32;
                    (*lh).v.u.ind.t = extra as lu_byte;
                }
                if (*lh).v.k as u32 == VINDEXED as i32 as u32
                    && (*v).k as u32 == VLOCAL as i32 as u32
                    && (*lh).v.u.ind.idx as i32 == (*v).u.var.ridx as i32
                {
                    conflict = 1 as i32;
                    (*lh).v.u.ind.idx = extra as i16;
                }
            }
        }
        lh = (*lh).prev;
    }
    if conflict != 0 {
        if (*v).k as u32 == VLOCAL as i32 as u32 {
            luaK_codeABCk(fs, OP_MOVE, extra, (*v).u.var.ridx as i32, 0, 0);
        } else {
            luaK_codeABCk(fs, OP_GETUPVAL, extra, (*v).u.info, 0, 0);
        }
        luaK_reserveregs(fs, 1 as i32);
    }
}
unsafe extern "C-unwind" fn restassign(
    mut ls: *mut LexState,
    mut lh: *mut LHS_assign,
    mut nvars: i32,
) {
    let mut e: ExpDesc = ExpDesc {
        k: VVOID,
        u: ExpVariant { ival: 0 },
        t: 0,
        f: 0,
    };
    if !(VLOCAL as i32 as u32 <= (*lh).v.k as u32 && (*lh).v.k as u32 <= VINDEXSTR as i32 as u32) {
        luaX_syntaxerror(ls, c"syntax error".as_ptr());
    }
    check_readonly(ls, &mut (*lh).v);
    if testnext(ls, ',' as i32) != 0 {
        let mut nv: LHS_assign = LHS_assign {
            prev: 0 as *mut LHS_assign,
            v: ExpDesc {
                k: VVOID,
                u: ExpVariant { ival: 0 },
                t: 0,
                f: 0,
            },
        };
        nv.prev = lh;
        suffixedexp(ls, &mut nv.v);
        if !(VINDEXED as i32 as u32 <= nv.v.k as u32 && nv.v.k as u32 <= VINDEXSTR as i32 as u32) {
            check_conflict(ls, lh, &mut nv.v);
        }
        luaE_incCstack((*ls).L);
        restassign(ls, &mut nv, nvars + 1 as i32);
        (*(*ls).L).nCcalls = ((*(*ls).L).nCcalls).wrapping_sub(1);
        (*(*ls).L).nCcalls;
    } else {
        let mut nexps: i32 = 0;
        checknext(ls, '=' as i32);
        nexps = explist(ls, &mut e);
        if nexps != nvars {
            adjust_assign(ls, nvars, nexps, &mut e);
        } else {
            luaK_setoneret((*ls).fs, &mut e);
            luaK_storevar((*ls).fs, &mut (*lh).v, &mut e);
            return;
        }
    }
    init_exp(&mut e, VNONRELOC, (*(*ls).fs).freereg as i32 - 1 as i32);
    luaK_storevar((*ls).fs, &mut (*lh).v, &mut e);
}
unsafe extern "C-unwind" fn cond(mut ls: *mut LexState) -> i32 {
    let mut v: ExpDesc = ExpDesc {
        k: VVOID,
        u: ExpVariant { ival: 0 },
        t: 0,
        f: 0,
    };
    expr(ls, &mut v);
    if v.k as u32 == VNIL as i32 as u32 {
        v.k = VFALSE;
    }
    luaK_goiftrue((*ls).fs, &mut v);
    return v.f;
}
unsafe extern "C-unwind" fn gotostat(mut ls: *mut LexState) {
    let mut fs: *mut FuncState = (*ls).fs;
    let mut line: i32 = (*ls).linenumber;
    let mut name: *mut TString = str_checkname(ls);
    let mut lb: *mut Labeldesc = findlabel(ls, name);
    if lb.is_null() {
        newgotoentry(ls, name, line, luaK_jump(fs));
    } else {
        let mut lblevel: i32 = reglevel(fs, (*lb).nactvar as i32);
        if luaY_nvarstack(fs) > lblevel {
            luaK_codeABCk(fs, OP_CLOSE, lblevel, 0, 0, 0);
        }
        luaK_patchlist(fs, luaK_jump(fs), (*lb).pc);
    };
}
unsafe extern "C-unwind" fn breakstat(mut ls: *mut LexState) {
    let mut line: i32 = (*ls).linenumber;
    luaX_next(ls);
    newgotoentry(
        ls,
        luaS_newlstr(
            (*ls).L,
            c"break".as_ptr(),
            (size_of::<[std::ffi::c_char; 6]>() as usize)
                .wrapping_div(size_of::<std::ffi::c_char>() as usize)
                .wrapping_sub(1),
        ),
        line,
        luaK_jump((*ls).fs),
    );
}
unsafe extern "C-unwind" fn checkrepeated(mut ls: *mut LexState, mut name: *mut TString) {
    let mut lb: *mut Labeldesc = findlabel(ls, name);
    if ((lb != 0 as *mut c_void as *mut Labeldesc) as i32 != 0) as i32 as std::ffi::c_long != 0 {
        let mut msg: *const std::ffi::c_char = c"label '%s' already defined on line %d".as_ptr();
        msg = luaO_pushfstring((*ls).L, msg, ((*name).contents).as_mut_ptr(), (*lb).line);
        luaK_semerror(ls, msg);
    }
}
unsafe extern "C-unwind" fn labelstat(
    mut ls: *mut LexState,
    mut name: *mut TString,
    mut line: i32,
) {
    checknext(ls, TK_DBCOLON as i32);
    while (*ls).t.token == ';' as i32 || (*ls).t.token == TK_DBCOLON as i32 {
        statement(ls);
    }
    checkrepeated(ls, name);
    createlabel(ls, name, line, block_follow(ls, 0));
}
unsafe extern "C-unwind" fn whilestat(mut ls: *mut LexState, mut line: i32) {
    let mut fs: *mut FuncState = (*ls).fs;
    let mut whileinit: i32 = 0;
    let mut condexit: i32 = 0;
    let mut bl: BlockCnt = BlockCnt {
        previous: 0 as *mut BlockCnt,
        firstlabel: 0,
        firstgoto: 0,
        nactvar: 0,
        upval: 0,
        isloop: 0,
        insidetbc: 0,
    };
    luaX_next(ls);
    whileinit = luaK_getlabel(fs);
    condexit = cond(ls);
    enterblock(fs, &mut bl, 1 as i32 as lu_byte);
    checknext(ls, TK_DO as i32);
    block(ls);
    luaK_patchlist(fs, luaK_jump(fs), whileinit);
    check_match(ls, TK_END as i32, TK_WHILE as i32, line);
    leaveblock(fs);
    luaK_patchtohere(fs, condexit);
}
unsafe extern "C-unwind" fn repeatstat(mut ls: *mut LexState, mut line: i32) {
    let mut condexit: i32 = 0;
    let mut fs: *mut FuncState = (*ls).fs;
    let mut repeat_init: i32 = luaK_getlabel(fs);
    let mut bl1: BlockCnt = BlockCnt {
        previous: 0 as *mut BlockCnt,
        firstlabel: 0,
        firstgoto: 0,
        nactvar: 0,
        upval: 0,
        isloop: 0,
        insidetbc: 0,
    };
    let mut bl2: BlockCnt = BlockCnt {
        previous: 0 as *mut BlockCnt,
        firstlabel: 0,
        firstgoto: 0,
        nactvar: 0,
        upval: 0,
        isloop: 0,
        insidetbc: 0,
    };
    enterblock(fs, &mut bl1, 1 as i32 as lu_byte);
    enterblock(fs, &mut bl2, 0 as lu_byte);
    luaX_next(ls);
    statlist(ls);
    check_match(ls, TK_UNTIL as i32, TK_REPEAT as i32, line);
    condexit = cond(ls);
    leaveblock(fs);
    if bl2.upval != 0 {
        let mut exit_0: i32 = luaK_jump(fs);
        luaK_patchtohere(fs, condexit);
        luaK_codeABCk(fs, OP_CLOSE, reglevel(fs, bl2.nactvar as i32), 0, 0, 0);
        condexit = luaK_jump(fs);
        luaK_patchtohere(fs, exit_0);
    }
    luaK_patchlist(fs, condexit, repeat_init);
    leaveblock(fs);
}
unsafe extern "C-unwind" fn exp1(mut ls: *mut LexState) {
    let mut e: ExpDesc = ExpDesc {
        k: VVOID,
        u: ExpVariant { ival: 0 },
        t: 0,
        f: 0,
    };
    expr(ls, &mut e);
    luaK_exp2nextreg((*ls).fs, &mut e);
}
unsafe extern "C-unwind" fn fixforjump(
    mut fs: *mut FuncState,
    mut pc: i32,
    mut dest: i32,
    mut back: i32,
) {
    let mut jmp: *mut Instruction = &mut *((*(*fs).f).code).offset(pc as isize) as *mut Instruction;
    let mut offset: i32 = dest - (pc + 1 as i32);
    if back != 0 {
        offset = -offset;
    }
    if ((offset > ((1 as i32) << 8 as i32 + 8 as i32 + 1 as i32) - 1 as i32) as i32 != 0) as i32
        as std::ffi::c_long
        != 0
    {
        luaX_syntaxerror((*fs).ls, c"control structure too long".as_ptr());
    }
    *jmp = *jmp
        & !(!(!(0 as Instruction) << 8 as i32 + 8 as i32 + 1 as i32) << 0 + 7 as i32 + 8 as i32)
        | (offset as Instruction) << 0 + 7 as i32 + 8 as i32
            & !(!(0 as Instruction) << 8 as i32 + 8 as i32 + 1 as i32) << 0 + 7 as i32 + 8 as i32;
}
unsafe extern "C-unwind" fn forbody(
    mut ls: *mut LexState,
    mut base: i32,
    mut line: i32,
    mut nvars: i32,
    mut isgen: i32,
) {
    static mut forprep_0: [OpCode; 2] = [OP_FORPREP, OP_TFORPREP];
    static mut forloop: [OpCode; 2] = [OP_FORLOOP, OP_TFORLOOP];
    let mut bl: BlockCnt = BlockCnt {
        previous: 0 as *mut BlockCnt,
        firstlabel: 0,
        firstgoto: 0,
        nactvar: 0,
        upval: 0,
        isloop: 0,
        insidetbc: 0,
    };
    let mut fs: *mut FuncState = (*ls).fs;
    let mut prep: i32 = 0;
    let mut endfor: i32 = 0;
    checknext(ls, TK_DO as i32);
    prep = luaK_codeABx(fs, forprep_0[isgen as usize], base, 0 as u32);
    enterblock(fs, &mut bl, 0 as lu_byte);
    adjustlocalvars(ls, nvars);
    luaK_reserveregs(fs, nvars);
    block(ls);
    leaveblock(fs);
    fixforjump(fs, prep, luaK_getlabel(fs), 0);
    if isgen != 0 {
        luaK_codeABCk(fs, OP_TFORCALL, base, 0, nvars, 0);
        luaK_fixline(fs, line);
    }
    endfor = luaK_codeABx(fs, forloop[isgen as usize], base, 0 as u32);
    fixforjump(fs, endfor, prep + 1 as i32, 1 as i32);
    luaK_fixline(fs, line);
}
unsafe extern "C-unwind" fn fornum(
    mut ls: *mut LexState,
    mut varname: *mut TString,
    mut line: i32,
) {
    let mut fs: *mut FuncState = (*ls).fs;
    let mut base: i32 = (*fs).freereg as i32;
    new_localvar(
        ls,
        luaX_newstring(
            ls,
            c"(for state)".as_ptr(),
            (size_of::<[std::ffi::c_char; 12]>() as usize)
                .wrapping_div(size_of::<std::ffi::c_char>() as usize)
                .wrapping_sub(1),
        ),
    );
    new_localvar(
        ls,
        luaX_newstring(
            ls,
            c"(for state)".as_ptr(),
            (size_of::<[std::ffi::c_char; 12]>() as usize)
                .wrapping_div(size_of::<std::ffi::c_char>() as usize)
                .wrapping_sub(1),
        ),
    );
    new_localvar(
        ls,
        luaX_newstring(
            ls,
            c"(for state)".as_ptr(),
            (size_of::<[std::ffi::c_char; 12]>() as usize)
                .wrapping_div(size_of::<std::ffi::c_char>() as usize)
                .wrapping_sub(1),
        ),
    );
    new_localvar(ls, varname);
    checknext(ls, '=' as i32);
    exp1(ls);
    checknext(ls, ',' as i32);
    exp1(ls);
    if testnext(ls, ',' as i32) != 0 {
        exp1(ls);
    } else {
        luaK_int(fs, (*fs).freereg as i32, 1 as i32 as lua_Integer);
        luaK_reserveregs(fs, 1 as i32);
    }
    adjustlocalvars(ls, 3 as i32);
    forbody(ls, base, line, 1 as i32, 0);
}
unsafe extern "C-unwind" fn forlist(mut ls: *mut LexState, mut indexname: *mut TString) {
    let mut fs: *mut FuncState = (*ls).fs;
    let mut e: ExpDesc = ExpDesc {
        k: VVOID,
        u: ExpVariant { ival: 0 },
        t: 0,
        f: 0,
    };
    let mut nvars: i32 = 5 as i32;
    let mut line: i32 = 0;
    let mut base: i32 = (*fs).freereg as i32;
    new_localvar(
        ls,
        luaX_newstring(
            ls,
            c"(for state)".as_ptr(),
            (size_of::<[std::ffi::c_char; 12]>() as usize)
                .wrapping_div(size_of::<std::ffi::c_char>() as usize)
                .wrapping_sub(1),
        ),
    );
    new_localvar(
        ls,
        luaX_newstring(
            ls,
            c"(for state)".as_ptr(),
            (size_of::<[std::ffi::c_char; 12]>() as usize)
                .wrapping_div(size_of::<std::ffi::c_char>() as usize)
                .wrapping_sub(1),
        ),
    );
    new_localvar(
        ls,
        luaX_newstring(
            ls,
            c"(for state)".as_ptr(),
            (size_of::<[std::ffi::c_char; 12]>() as usize)
                .wrapping_div(size_of::<std::ffi::c_char>() as usize)
                .wrapping_sub(1),
        ),
    );
    new_localvar(
        ls,
        luaX_newstring(
            ls,
            c"(for state)".as_ptr(),
            (size_of::<[std::ffi::c_char; 12]>() as usize)
                .wrapping_div(size_of::<std::ffi::c_char>() as usize)
                .wrapping_sub(1),
        ),
    );
    new_localvar(ls, indexname);
    while testnext(ls, ',' as i32) != 0 {
        new_localvar(ls, str_checkname(ls));
        nvars += 1;
        nvars;
    }
    checknext(ls, TK_IN as i32);
    line = (*ls).linenumber;
    adjust_assign(ls, 4 as i32, explist(ls, &mut e), &mut e);
    adjustlocalvars(ls, 4 as i32);
    marktobeclosed(fs);
    luaK_checkstack(fs, 3 as i32);
    forbody(ls, base, line, nvars - 4 as i32, 1 as i32);
}
unsafe extern "C-unwind" fn forstat(mut ls: *mut LexState, mut line: i32) {
    let mut fs: *mut FuncState = (*ls).fs;
    let mut varname: *mut TString = 0 as *mut TString;
    let mut bl: BlockCnt = BlockCnt {
        previous: 0 as *mut BlockCnt,
        firstlabel: 0,
        firstgoto: 0,
        nactvar: 0,
        upval: 0,
        isloop: 0,
        insidetbc: 0,
    };
    enterblock(fs, &mut bl, 1 as i32 as lu_byte);
    luaX_next(ls);
    varname = str_checkname(ls);
    match (*ls).t.token {
        61 => {
            fornum(ls, varname, line);
        }
        44 | 267 => {
            forlist(ls, varname);
        }
        _ => {
            luaX_syntaxerror(ls, c"'=' or 'in' expected".as_ptr());
        }
    }
    check_match(ls, TK_END as i32, TK_FOR as i32, line);
    leaveblock(fs);
}
unsafe extern "C-unwind" fn test_then_block(mut ls: *mut LexState, mut escapelist: *mut i32) {
    let mut bl: BlockCnt = BlockCnt {
        previous: 0 as *mut BlockCnt,
        firstlabel: 0,
        firstgoto: 0,
        nactvar: 0,
        upval: 0,
        isloop: 0,
        insidetbc: 0,
    };
    let mut fs: *mut FuncState = (*ls).fs;
    let mut v: ExpDesc = ExpDesc {
        k: VVOID,
        u: ExpVariant { ival: 0 },
        t: 0,
        f: 0,
    };
    let mut jf: i32 = 0;
    luaX_next(ls);
    expr(ls, &mut v);
    checknext(ls, TK_THEN as i32);
    if (*ls).t.token == TK_BREAK as i32 {
        let mut line: i32 = (*ls).linenumber;
        luaK_goiffalse((*ls).fs, &mut v);
        luaX_next(ls);
        enterblock(fs, &mut bl, 0 as lu_byte);
        newgotoentry(
            ls,
            luaS_newlstr(
                (*ls).L,
                c"break".as_ptr(),
                (size_of::<[std::ffi::c_char; 6]>() as usize)
                    .wrapping_div(size_of::<std::ffi::c_char>() as usize)
                    .wrapping_sub(1),
            ),
            line,
            v.t,
        );
        while testnext(ls, ';' as i32) != 0 {}
        if block_follow(ls, 0) != 0 {
            leaveblock(fs);
            return;
        } else {
            jf = luaK_jump(fs);
        }
    } else {
        luaK_goiftrue((*ls).fs, &mut v);
        enterblock(fs, &mut bl, 0 as lu_byte);
        jf = v.f;
    }
    statlist(ls);
    leaveblock(fs);
    if (*ls).t.token == TK_ELSE as i32 || (*ls).t.token == TK_ELSEIF as i32 {
        luaK_concat(fs, escapelist, luaK_jump(fs));
    }
    luaK_patchtohere(fs, jf);
}
unsafe extern "C-unwind" fn ifstat(mut ls: *mut LexState, mut line: i32) {
    let mut fs: *mut FuncState = (*ls).fs;
    let mut escapelist: i32 = -(1 as i32);
    test_then_block(ls, &mut escapelist);
    while (*ls).t.token == TK_ELSEIF as i32 {
        test_then_block(ls, &mut escapelist);
    }
    if testnext(ls, TK_ELSE as i32) != 0 {
        block(ls);
    }
    check_match(ls, TK_END as i32, TK_IF as i32, line);
    luaK_patchtohere(fs, escapelist);
}
unsafe extern "C-unwind" fn localfunc(mut ls: *mut LexState) {
    let mut b: ExpDesc = ExpDesc {
        k: VVOID,
        u: ExpVariant { ival: 0 },
        t: 0,
        f: 0,
    };
    let mut fs: *mut FuncState = (*ls).fs;
    let mut fvar: i32 = (*fs).nactvar as i32;
    new_localvar(ls, str_checkname(ls));
    adjustlocalvars(ls, 1 as i32);
    body(ls, &mut b, 0, (*ls).linenumber);
    (*localdebuginfo(fs, fvar)).startpc = (*fs).pc;
}
unsafe extern "C-unwind" fn getlocalattribute(mut ls: *mut LexState) -> i32 {
    if testnext(ls, '<' as i32) != 0 {
        let mut attr: *const std::ffi::c_char = ((*str_checkname(ls)).contents).as_mut_ptr();
        checknext(ls, '>' as i32);
        if strcmp(attr, c"const".as_ptr()) == 0 {
            return 1 as i32;
        } else if strcmp(attr, c"close".as_ptr()) == 0 {
            return 2 as i32;
        } else {
            luaK_semerror(
                ls,
                luaO_pushfstring((*ls).L, c"unknown attribute '%s'".as_ptr(), attr),
            );
        }
    }
    return 0;
}
unsafe extern "C-unwind" fn checktoclose(mut fs: *mut FuncState, mut level: i32) {
    if level != -(1 as i32) {
        marktobeclosed(fs);
        luaK_codeABCk(fs, OP_TBC, reglevel(fs, level), 0, 0, 0);
    }
}
unsafe extern "C-unwind" fn localstat(mut ls: *mut LexState) {
    let mut fs: *mut FuncState = (*ls).fs;
    let mut toclose: i32 = -(1 as i32);
    let mut var: *mut Vardesc = 0 as *mut Vardesc;
    let mut vidx: i32 = 0;
    let mut kind: i32 = 0;
    let mut nvars: i32 = 0;
    let mut nexps: i32 = 0;
    let mut e: ExpDesc = ExpDesc {
        k: VVOID,
        u: ExpVariant { ival: 0 },
        t: 0,
        f: 0,
    };
    loop {
        vidx = new_localvar(ls, str_checkname(ls));
        kind = getlocalattribute(ls);
        (*getlocalvardesc(fs, vidx)).vd.kind = kind as lu_byte;
        if kind == 2 as i32 {
            if toclose != -(1 as i32) {
                luaK_semerror(
                    ls,
                    c"multiple to-be-closed variables in local list".as_ptr(),
                );
            }
            toclose = (*fs).nactvar as i32 + nvars;
        }
        nvars += 1;
        nvars;
        if !(testnext(ls, ',' as i32) != 0) {
            break;
        }
    }
    if testnext(ls, '=' as i32) != 0 {
        nexps = explist(ls, &mut e);
    } else {
        e.k = VVOID;
        nexps = 0;
    }
    var = getlocalvardesc(fs, vidx);
    if nvars == nexps
        && (*var).vd.kind as i32 == 1 as i32
        && luaK_exp2const(fs, &mut e, &mut (*var).k) != 0
    {
        (*var).vd.kind = 3 as i32 as lu_byte;
        adjustlocalvars(ls, nvars - 1 as i32);
        (*fs).nactvar = ((*fs).nactvar).wrapping_add(1);
        (*fs).nactvar;
    } else {
        adjust_assign(ls, nvars, nexps, &mut e);
        adjustlocalvars(ls, nvars);
    }
    checktoclose(fs, toclose);
}
unsafe extern "C-unwind" fn funcname(mut ls: *mut LexState, mut v: *mut ExpDesc) -> i32 {
    let mut ismethod: i32 = 0;
    singlevar(ls, v);
    while (*ls).t.token == '.' as i32 {
        fieldsel(ls, v);
    }
    if (*ls).t.token == ':' as i32 {
        ismethod = 1 as i32;
        fieldsel(ls, v);
    }
    return ismethod;
}
unsafe extern "C-unwind" fn funcstat(mut ls: *mut LexState, mut line: i32) {
    let mut ismethod: i32 = 0;
    let mut v: ExpDesc = ExpDesc {
        k: VVOID,
        u: ExpVariant { ival: 0 },
        t: 0,
        f: 0,
    };
    let mut b: ExpDesc = ExpDesc {
        k: VVOID,
        u: ExpVariant { ival: 0 },
        t: 0,
        f: 0,
    };
    luaX_next(ls);
    ismethod = funcname(ls, &mut v);
    body(ls, &mut b, ismethod, line);
    check_readonly(ls, &mut v);
    luaK_storevar((*ls).fs, &mut v, &mut b);
    luaK_fixline((*ls).fs, line);
}
unsafe extern "C-unwind" fn exprstat(mut ls: *mut LexState) {
    let mut fs: *mut FuncState = (*ls).fs;
    let mut v: LHS_assign = LHS_assign {
        prev: 0 as *mut LHS_assign,
        v: ExpDesc {
            k: VVOID,
            u: ExpVariant { ival: 0 },
            t: 0,
            f: 0,
        },
    };
    suffixedexp(ls, &mut v.v);
    if (*ls).t.token == '=' as i32 || (*ls).t.token == ',' as i32 {
        v.prev = 0 as *mut LHS_assign;
        restassign(ls, &mut v, 1 as i32);
    } else {
        let mut inst: *mut Instruction = 0 as *mut Instruction;
        if !(v.v.k as u32 == VCALL as i32 as u32) {
            luaX_syntaxerror(ls, c"syntax error".as_ptr());
        }
        inst = &mut *((*(*fs).f).code).offset(v.v.u.info as isize) as *mut Instruction;
        *inst = *inst
            & !(!(!(0 as Instruction) << 8 as i32)
                << 0 + 7 as i32 + 8 as i32 + 1 as i32 + 8 as i32)
            | (1 as i32 as Instruction) << 0 + 7 as i32 + 8 as i32 + 1 as i32 + 8 as i32
                & !(!(0 as Instruction) << 8 as i32)
                    << 0 + 7 as i32 + 8 as i32 + 1 as i32 + 8 as i32;
    };
}
unsafe extern "C-unwind" fn retstat(mut ls: *mut LexState) {
    let mut fs: *mut FuncState = (*ls).fs;
    let mut e: ExpDesc = ExpDesc {
        k: VVOID,
        u: ExpVariant { ival: 0 },
        t: 0,
        f: 0,
    };
    let mut nret: i32 = 0;
    let mut first: i32 = luaY_nvarstack(fs);
    if block_follow(ls, 1 as i32) != 0 || (*ls).t.token == ';' as i32 {
        nret = 0;
    } else {
        nret = explist(ls, &mut e);
        if e.k as u32 == VCALL as i32 as u32 || e.k as u32 == VVARARG as i32 as u32 {
            luaK_setreturns(fs, &mut e, -(1 as i32));
            if e.k as u32 == VCALL as i32 as u32 && nret == 1 as i32 && (*(*fs).bl).insidetbc == 0 {
                *((*(*fs).f).code).offset(e.u.info as isize) = *((*(*fs).f).code)
                    .offset(e.u.info as isize)
                    & !(!(!(0 as Instruction) << 7 as i32) << 0)
                    | (OP_TAILCALL as i32 as Instruction) << 0
                        & !(!(0 as Instruction) << 7 as i32) << 0;
            }
            nret = -(1 as i32);
        } else if nret == 1 as i32 {
            first = luaK_exp2anyreg(fs, &mut e);
        } else {
            luaK_exp2nextreg(fs, &mut e);
        }
    }
    luaK_ret(fs, first, nret);
    testnext(ls, ';' as i32);
}
unsafe extern "C-unwind" fn statement(mut ls: *mut LexState) {
    let mut line: i32 = (*ls).linenumber;
    luaE_incCstack((*ls).L);
    match (*ls).t.token {
        59 => {
            luaX_next(ls);
        }
        266 => {
            ifstat(ls, line);
        }
        277 => {
            whilestat(ls, line);
        }
        258 => {
            luaX_next(ls);
            block(ls);
            check_match(ls, TK_END as i32, TK_DO as i32, line);
        }
        263 => {
            forstat(ls, line);
        }
        272 => {
            repeatstat(ls, line);
        }
        264 => {
            funcstat(ls, line);
        }
        268 => {
            luaX_next(ls);
            if testnext(ls, TK_FUNCTION as i32) != 0 {
                localfunc(ls);
            } else {
                localstat(ls);
            }
        }
        287 => {
            luaX_next(ls);
            labelstat(ls, str_checkname(ls), line);
        }
        273 => {
            luaX_next(ls);
            retstat(ls);
        }
        257 => {
            breakstat(ls);
        }
        265 => {
            luaX_next(ls);
            gotostat(ls);
        }
        _ => {
            exprstat(ls);
        }
    }
    (*(*ls).fs).freereg = luaY_nvarstack((*ls).fs) as lu_byte;
    (*(*ls).L).nCcalls = ((*(*ls).L).nCcalls).wrapping_sub(1);
    (*(*ls).L).nCcalls;
}
unsafe extern "C-unwind" fn mainfunc(mut ls: *mut LexState, mut fs: *mut FuncState) {
    let mut bl: BlockCnt = BlockCnt {
        previous: 0 as *mut BlockCnt,
        firstlabel: 0,
        firstgoto: 0,
        nactvar: 0,
        upval: 0,
        isloop: 0,
        insidetbc: 0,
    };
    let mut env: *mut Upvaldesc = 0 as *mut Upvaldesc;
    open_func(ls, fs, &mut bl);
    setvararg(fs, 0);
    env = allocupvalue(fs);
    (*env).instack = 1 as i32 as lu_byte;
    (*env).idx = 0 as lu_byte;
    (*env).kind = 0 as lu_byte;
    (*env).name = (*ls).envn;
    if (*(*fs).f).marked as i32 & (1 as i32) << 5 as i32 != 0
        && (*(*env).name).marked as i32 & ((1 as i32) << 3 as i32 | (1 as i32) << 4 as i32) != 0
    {
        luaC_barrier_(
            (*ls).L,
            &mut (*((*fs).f as *mut GCUnion)).gc,
            &mut (*((*env).name as *mut GCUnion)).gc,
        );
    } else {
    };
    luaX_next(ls);
    statlist(ls);
    check(ls, TK_EOS as i32);
    close_func(ls);
}
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaY_parser(
    mut L: *mut lua_State,
    mut z: *mut ZIO,
    mut buff: *mut Mbuffer,
    mut dyd: *mut Dyndata,
    mut name: *const std::ffi::c_char,
    mut firstchar: i32,
) -> *mut LClosure {
    let mut lexstate: LexState = LexState {
        current: 0,
        linenumber: 0,
        lastline: 0,
        t: Token {
            token: 0,
            seminfo: SemInfo { r: 0. },
        },
        lookahead: Token {
            token: 0,
            seminfo: SemInfo { r: 0. },
        },
        fs: 0 as *mut FuncState,
        L: 0 as *mut lua_State,
        z: 0 as *mut ZIO,
        buff: 0 as *mut Mbuffer,
        h: 0 as *mut Table,
        dyd: 0 as *mut Dyndata,
        source: 0 as *mut TString,
        envn: 0 as *mut TString,
    };
    let mut funcstate: FuncState = FuncState {
        f: 0 as *mut Proto,
        prev: 0 as *mut FuncState,
        ls: 0 as *mut LexState,
        bl: 0 as *mut BlockCnt,
        pc: 0,
        lasttarget: 0,
        previousline: 0,
        nk: 0,
        np: 0,
        nabslineinfo: 0,
        firstlocal: 0,
        firstlabel: 0,
        ndebugvars: 0,
        nactvar: 0,
        nups: 0,
        freereg: 0,
        iwthabs: 0,
        needclose: 0,
    };
    let mut cl: *mut LClosure = luaF_newLclosure(L, 1 as i32);
    let mut io: *mut TValue = &mut (*(*L).top.p).val;
    let mut x_: *mut LClosure = cl;
    (*io).value_.gc = &mut (*(x_ as *mut GCUnion)).gc;
    (*io).tt_ = (6 as i32 | (0) << 4 as i32 | (1 as i32) << 6 as i32) as lu_byte;
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
    luaD_inctop(L);
    lexstate.h = luaH_new(L);
    let mut io_0: *mut TValue = &mut (*(*L).top.p).val;
    let mut x__0: *mut Table = lexstate.h;
    (*io_0).value_.gc = &mut (*(x__0 as *mut GCUnion)).gc;
    (*io_0).tt_ = (5 as i32 | (0) << 4 as i32 | (1 as i32) << 6 as i32) as lu_byte;
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
    luaD_inctop(L);
    (*cl).p = luaF_newproto(L);
    funcstate.f = (*cl).p;
    if (*cl).marked as i32 & (1 as i32) << 5 as i32 != 0
        && (*(*cl).p).marked as i32 & ((1 as i32) << 3 as i32 | (1 as i32) << 4 as i32) != 0
    {
        luaC_barrier_(
            L,
            &mut (*(cl as *mut GCUnion)).gc,
            &mut (*((*cl).p as *mut GCUnion)).gc,
        );
    } else {
    };
    (*funcstate.f).source = luaS_new(L, name);
    if (*funcstate.f).marked as i32 & (1 as i32) << 5 as i32 != 0
        && (*(*funcstate.f).source).marked as i32
            & ((1 as i32) << 3 as i32 | (1 as i32) << 4 as i32)
            != 0
    {
        luaC_barrier_(
            L,
            &mut (*(funcstate.f as *mut GCUnion)).gc,
            &mut (*((*funcstate.f).source as *mut GCUnion)).gc,
        );
    } else {
    };
    lexstate.buff = buff;
    lexstate.dyd = dyd;
    (*dyd).label.n = 0;
    (*dyd).gt.n = (*dyd).label.n;
    (*dyd).actvar.n = (*dyd).gt.n;
    luaX_setinput(L, &mut lexstate, z, (*funcstate.f).source, firstchar);
    mainfunc(&mut lexstate, &mut funcstate);
    (*L).top.p = ((*L).top.p).offset(-1);
    (*L).top.p;
    return cl;
}
