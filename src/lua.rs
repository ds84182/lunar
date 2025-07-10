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
unsafe extern "C" {
    static mut stdin: *mut FILE;
    static mut stdout: *mut FILE;
    static mut stderr: *mut FILE;
}

use lunar::*;

use libc::{FILE, fprintf, fflush, fwrite, strlen, strchr, strcmp, getenv, fputs, fgets};

unsafe fn signal(signum: i32, handler: Option<unsafe extern "C" fn(i32) -> ()>) -> Option<unsafe extern "C" fn(i32) -> ()> {
    std::mem::transmute(libc::signal(signum, std::mem::transmute(handler)))
}

static mut globalL: *mut lua_State = 0 as *const lua_State as *mut lua_State;
static mut progname: *const std::ffi::c_char = b"lua\0" as *const u8
    as *const std::ffi::c_char;
unsafe extern "C-unwind" fn lstop(mut L: *mut lua_State, mut ar: *mut lua_Debug) {
    lua_sethook(L, None, 0 as i32, 0 as i32);
    luaL_error(L, b"interrupted!\0" as *const u8 as *const std::ffi::c_char);
}
unsafe extern "C" fn laction(mut i: i32) {
    let mut flag: i32 = (1 as i32) << 0 as i32
        | (1 as i32) << 1 as i32
        | (1 as i32) << 2 as i32
        | (1 as i32) << 3 as i32;
    signal(i, None);
    lua_sethook(
        globalL,
        Some(lstop as unsafe extern "C-unwind" fn(*mut lua_State, *mut lua_Debug) -> ()),
        flag,
        1 as i32,
    );
}
unsafe extern "C" fn print_usage(mut badoption: *const std::ffi::c_char) {
    fprintf(stderr, b"%s: \0" as *const u8 as *const std::ffi::c_char, progname);
    fflush(stderr);
    if *badoption.offset(1 as i32 as isize) as i32 == 'e' as i32
        || *badoption.offset(1 as i32 as isize) as i32
            == 'l' as i32
    {
        fprintf(
            stderr,
            b"'%s' needs argument\n\0" as *const u8 as *const std::ffi::c_char,
            badoption,
        );
        fflush(stderr);
    } else {
        fprintf(
            stderr,
            b"unrecognized option '%s'\n\0" as *const u8 as *const std::ffi::c_char,
            badoption,
        );
        fflush(stderr);
    }
    fprintf(
        stderr,
        b"usage: %s [options] [script [args]]\nAvailable options are:\n  -e stat   execute string 'stat'\n  -i        enter interactive mode after executing 'script'\n  -l mod    require library 'mod' into global 'mod'\n  -l g=mod  require library 'mod' into global 'g'\n  -v        show version information\n  -E        ignore environment variables\n  -W        turn warnings on\n  --        stop handling options\n  -         stop handling options and execute stdin\n\0"
            as *const u8 as *const std::ffi::c_char,
        progname,
    );
    fflush(stderr);
}
unsafe extern "C" fn l_message(
    mut pname: *const std::ffi::c_char,
    mut msg: *const std::ffi::c_char,
) {
    if !pname.is_null() {
        fprintf(stderr, b"%s: \0" as *const u8 as *const std::ffi::c_char, pname);
        fflush(stderr);
    }
    fprintf(stderr, b"%s\n\0" as *const u8 as *const std::ffi::c_char, msg);
    fflush(stderr);
}
unsafe extern "C" fn report(
    mut L: *mut lua_State,
    mut status: i32,
) -> i32 {
    if status != 0 as i32 {
        let mut msg: *const std::ffi::c_char = lua_tolstring(
            L,
            -(1 as i32),
            0 as *mut size_t,
        );
        if msg.is_null() {
            msg = b"(error message not a string)\0" as *const u8
                as *const std::ffi::c_char;
        }
        l_message(progname, msg);
        lua_settop(L, -(1 as i32) - 1 as i32);
    }
    return status;
}
unsafe extern "C-unwind" fn msghandler(mut L: *mut lua_State) -> i32 {
    let mut msg: *const std::ffi::c_char = lua_tolstring(
        L,
        1 as i32,
        0 as *mut size_t,
    );
    if msg.is_null() {
        if luaL_callmeta(
            L,
            1 as i32,
            b"__tostring\0" as *const u8 as *const std::ffi::c_char,
        ) != 0 && lua_type(L, -(1 as i32)) == 4 as i32
        {
            return 1 as i32
        } else {
            msg = lua_pushfstring(
                L,
                b"(error object is a %s value)\0" as *const u8
                    as *const std::ffi::c_char,
                lua_typename(L, lua_type(L, 1 as i32)),
            );
        }
    }
    luaL_traceback(L, L, msg, 1 as i32);
    return 1 as i32;
}
unsafe extern "C" fn docall(
    mut L: *mut lua_State,
    mut narg: i32,
    mut nres: i32,
) -> i32 {
    let mut status: i32 = 0;
    let mut base: i32 = lua_gettop(L) - narg;
    lua_pushcclosure(
        L,
        Some(msghandler as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
        0 as i32,
    );
    lua_rotate(L, base, 1 as i32);
    globalL = L;
    signal(
        2 as i32,
        Some(laction as unsafe extern "C" fn(i32) -> ()),
    );
    status = lua_pcallk(L, narg, nres, base, 0 as i32 as lua_KContext, None);
    signal(2 as i32, None);
    lua_rotate(L, base, -(1 as i32));
    lua_settop(L, -(1 as i32) - 1 as i32);
    return status;
}
unsafe extern "C" fn print_version() {
    fwrite(
        b"Lua 5.4.8  Copyright (C) 1994-2025 Lua.org, PUC-Rio\0" as *const u8
            as *const std::ffi::c_char as *const std::ffi::c_void,
        ::core::mem::size_of::<std::ffi::c_char>(),
        strlen(
            b"Lua 5.4.8  Copyright (C) 1994-2025 Lua.org, PUC-Rio\0" as *const u8
                as *const std::ffi::c_char,
        ),
        stdout,
    );
    fwrite(
        b"\n\0" as *const u8 as *const std::ffi::c_char as *const std::ffi::c_void,
        ::core::mem::size_of::<std::ffi::c_char>(),
        1,
        stdout,
    );
    fflush(stdout);
}
unsafe extern "C" fn createargtable(
    mut L: *mut lua_State,
    mut argv: *mut *mut std::ffi::c_char,
    mut argc: i32,
    mut script: i32,
) {
    let mut i: i32 = 0;
    let mut narg: i32 = 0;
    narg = argc - (script + 1 as i32);
    lua_createtable(L, narg, script + 1 as i32);
    i = 0 as i32;
    while i < argc {
        lua_pushstring(L, *argv.offset(i as isize));
        lua_rawseti(L, -(2 as i32), (i - script) as lua_Integer);
        i += 1;
        i;
    }
    lua_setglobal(L, b"arg\0" as *const u8 as *const std::ffi::c_char);
}
unsafe extern "C" fn dochunk(
    mut L: *mut lua_State,
    mut status: i32,
) -> i32 {
    if status == 0 as i32 {
        status = docall(L, 0 as i32, 0 as i32);
    }
    return report(L, status);
}
unsafe extern "C" fn dofile(
    mut L: *mut lua_State,
    mut name: *const std::ffi::c_char,
) -> i32 {
    return dochunk(L, luaL_loadfilex(L, name, 0 as *const std::ffi::c_char));
}
unsafe extern "C" fn dostring(
    mut L: *mut lua_State,
    mut s: *const std::ffi::c_char,
    mut name: *const std::ffi::c_char,
) -> i32 {
    return dochunk(
        L,
        luaL_loadbufferx(L, s, strlen(s), name, 0 as *const std::ffi::c_char),
    );
}
unsafe extern "C" fn dolibrary(
    mut L: *mut lua_State,
    mut globname: *mut std::ffi::c_char,
) -> i32 {
    let mut status: i32 = 0;
    let mut suffix: *mut std::ffi::c_char = 0 as *mut std::ffi::c_char;
    let mut modname: *mut std::ffi::c_char = strchr(globname, '=' as i32);
    if modname.is_null() {
        modname = globname;
        suffix = strchr(
            modname,
            *(b"-\0" as *const u8 as *const std::ffi::c_char) as i32,
        );
    } else {
        *modname = '\0' as i32 as std::ffi::c_char;
        modname = modname.offset(1);
        modname;
    }
    lua_getglobal(L, b"require\0" as *const u8 as *const std::ffi::c_char);
    lua_pushstring(L, modname);
    status = docall(L, 1 as i32, 1 as i32);
    if status == 0 as i32 {
        if !suffix.is_null() {
            *suffix = '\0' as i32 as std::ffi::c_char;
        }
        lua_setglobal(L, globname);
    }
    return report(L, status);
}
unsafe extern "C" fn pushargs(mut L: *mut lua_State) -> i32 {
    let mut i: i32 = 0;
    let mut n: i32 = 0;
    if lua_getglobal(L, b"arg\0" as *const u8 as *const std::ffi::c_char)
        != 5 as i32
    {
        luaL_error(L, b"'arg' is not a table\0" as *const u8 as *const std::ffi::c_char);
    }
    n = luaL_len(L, -(1 as i32)) as i32;
    luaL_checkstack(
        L,
        n + 3 as i32,
        b"too many arguments to script\0" as *const u8 as *const std::ffi::c_char,
    );
    i = 1 as i32;
    while i <= n {
        lua_rawgeti(L, -i, i as lua_Integer);
        i += 1;
        i;
    }
    lua_rotate(L, -i, -(1 as i32));
    lua_settop(L, -(1 as i32) - 1 as i32);
    return n;
}
unsafe extern "C" fn handle_script(
    mut L: *mut lua_State,
    mut argv: *mut *mut std::ffi::c_char,
) -> i32 {
    let mut status: i32 = 0;
    let mut fname: *const std::ffi::c_char = *argv.offset(0 as i32 as isize);
    if strcmp(fname, b"-\0" as *const u8 as *const std::ffi::c_char)
        == 0 as i32
        && strcmp(
            *argv.offset(-(1 as i32) as isize),
            b"--\0" as *const u8 as *const std::ffi::c_char,
        ) != 0 as i32
    {
        fname = 0 as *const std::ffi::c_char;
    }
    status = luaL_loadfilex(L, fname, 0 as *const std::ffi::c_char);
    if status == 0 as i32 {
        let mut n: i32 = pushargs(L);
        status = docall(L, n, -(1 as i32));
    }
    return report(L, status);
}
unsafe extern "C" fn collectargs(
    mut argv: *mut *mut std::ffi::c_char,
    mut first: *mut i32,
) -> i32 {
    let mut args: i32 = 0 as i32;
    let mut i: i32 = 0;
    if !(*argv.offset(0 as i32 as isize)).is_null() {
        if *(*argv.offset(0 as i32 as isize))
            .offset(0 as i32 as isize) != 0
        {
            progname = *argv.offset(0 as i32 as isize);
        }
    } else {
        *first = -(1 as i32);
        return 0 as i32;
    }
    i = 1 as i32;
    while !(*argv.offset(i as isize)).is_null() {
        *first = i;
        if *(*argv.offset(i as isize)).offset(0 as i32 as isize)
            as i32 != '-' as i32
        {
            return args;
        }
        let mut current_block_31: u64;
        match *(*argv.offset(i as isize)).offset(1 as i32 as isize)
            as i32
        {
            45 => {
                if *(*argv.offset(i as isize)).offset(2 as i32 as isize)
                    as i32 != '\0' as i32
                {
                    return 1 as i32;
                }
                *first = i + 1 as i32;
                return args;
            }
            0 => return args,
            69 => {
                if *(*argv.offset(i as isize)).offset(2 as i32 as isize)
                    as i32 != '\0' as i32
                {
                    return 1 as i32;
                }
                args |= 16 as i32;
                current_block_31 = 4761528863920922185;
            }
            87 => {
                if *(*argv.offset(i as isize)).offset(2 as i32 as isize)
                    as i32 != '\0' as i32
                {
                    return 1 as i32;
                }
                current_block_31 = 4761528863920922185;
            }
            105 => {
                args |= 2 as i32;
                current_block_31 = 8087486506634012100;
            }
            118 => {
                current_block_31 = 8087486506634012100;
            }
            101 => {
                args |= 8 as i32;
                current_block_31 = 9495257101768435433;
            }
            108 => {
                current_block_31 = 9495257101768435433;
            }
            _ => return 1 as i32,
        }
        match current_block_31 {
            8087486506634012100 => {
                if *(*argv.offset(i as isize)).offset(2 as i32 as isize)
                    as i32 != '\0' as i32
                {
                    return 1 as i32;
                }
                args |= 4 as i32;
            }
            9495257101768435433 => {
                if *(*argv.offset(i as isize)).offset(2 as i32 as isize)
                    as i32 == '\0' as i32
                {
                    i += 1;
                    i;
                    if (*argv.offset(i as isize)).is_null()
                        || *(*argv.offset(i as isize))
                            .offset(0 as i32 as isize) as i32
                            == '-' as i32
                    {
                        return 1 as i32;
                    }
                }
            }
            _ => {}
        }
        i += 1;
        i;
    }
    *first = 0 as i32;
    return args;
}
unsafe extern "C" fn runargs(
    mut L: *mut lua_State,
    mut argv: *mut *mut std::ffi::c_char,
    mut n: i32,
) -> i32 {
    let mut i: i32 = 0;
    i = 1 as i32;
    while i < n {
        let mut option: i32 = *(*argv.offset(i as isize))
            .offset(1 as i32 as isize) as i32;
        match option {
            101 | 108 => {
                let mut status: i32 = 0;
                let mut extra: *mut std::ffi::c_char = (*argv.offset(i as isize))
                    .offset(2 as i32 as isize);
                if *extra as i32 == '\0' as i32 {
                    i += 1;
                    extra = *argv.offset(i as isize);
                }
                status = if option == 'e' as i32 {
                    dostring(
                        L,
                        extra,
                        b"=(command line)\0" as *const u8 as *const std::ffi::c_char,
                    )
                } else {
                    dolibrary(L, extra)
                };
                if status != 0 as i32 {
                    return 0 as i32;
                }
            }
            87 => {
                lua_warning(
                    L,
                    b"@on\0" as *const u8 as *const std::ffi::c_char,
                    0 as i32,
                );
            }
            _ => {}
        }
        i += 1;
        i;
    }
    return 1 as i32;
}
unsafe extern "C" fn handle_luainit(mut L: *mut lua_State) -> i32 {
    let mut name: *const std::ffi::c_char = b"=LUA_INIT_5_4\0" as *const u8
        as *const std::ffi::c_char;
    let mut init: *const std::ffi::c_char = getenv(
        name.offset(1 as i32 as isize),
    );
    if init.is_null() {
        name = b"=LUA_INIT\0" as *const u8 as *const std::ffi::c_char;
        init = getenv(name.offset(1 as i32 as isize));
    }
    if init.is_null() {
        return 0 as i32
    } else if *init.offset(0 as i32 as isize) as i32
        == '@' as i32
    {
        return dofile(L, init.offset(1 as i32 as isize))
    } else {
        return dostring(L, init, name)
    };
}
unsafe extern "C" fn get_prompt(
    mut L: *mut lua_State,
    mut firstline: i32,
) -> *const std::ffi::c_char {
    if lua_getglobal(
        L,
        (if firstline != 0 {
            b"_PROMPT\0" as *const u8 as *const std::ffi::c_char
        } else {
            b"_PROMPT2\0" as *const u8 as *const std::ffi::c_char
        }),
    ) == 0 as i32
    {
        return if firstline != 0 {
            b"> \0" as *const u8 as *const std::ffi::c_char
        } else {
            b">> \0" as *const u8 as *const std::ffi::c_char
        }
    } else {
        let mut p: *const std::ffi::c_char = luaL_tolstring(
            L,
            -(1 as i32),
            0 as *mut size_t,
        );
        lua_rotate(L, -(2 as i32), -(1 as i32));
        lua_settop(L, -(1 as i32) - 1 as i32);
        return p;
    };
}
unsafe extern "C" fn incomplete(
    mut L: *mut lua_State,
    mut status: i32,
) -> i32 {
    if status == 3 as i32 {
        let mut lmsg: size_t = 0;
        let mut msg: *const std::ffi::c_char = lua_tolstring(
            L,
            -(1 as i32),
            &mut lmsg,
        );
        if lmsg
            >= (size_of::<[std::ffi::c_char; 6]>())
                .wrapping_div(
                    size_of::<std::ffi::c_char>(),
                )
                .wrapping_sub(1)
            && strcmp(
                msg
                    .offset(lmsg as isize)
                    .offset(
                        -((size_of::<[std::ffi::c_char; 6]>()
                            as std::ffi::c_ulong)
                            .wrapping_div(
                                size_of::<std::ffi::c_char>()
                                    as std::ffi::c_ulong,
                            )
                            .wrapping_sub(1 as i32 as std::ffi::c_ulong)
                            as isize),
                    ),
                b"<eof>\0" as *const u8 as *const std::ffi::c_char,
            ) == 0 as i32
        {
            return 1 as i32;
        }
    }
    return 0 as i32;
}
unsafe extern "C" fn pushline(
    mut L: *mut lua_State,
    mut firstline: i32,
) -> i32 {
    let mut buffer: [std::ffi::c_char; 512] = [0; 512];
    let mut b: *mut std::ffi::c_char = buffer.as_mut_ptr();
    let mut l: size_t = 0;
    let mut prmt: *const std::ffi::c_char = get_prompt(L, firstline);
    fputs(prmt, stdout);
    fflush(stdout);
    let mut readstatus: i32 = (fgets(b, 512 as i32, stdin)
        != 0 as *mut std::ffi::c_void as *mut std::ffi::c_char) as i32;
    lua_settop(L, -(1 as i32) - 1 as i32);
    if readstatus == 0 {
        return 0 as i32;
    }
    l = strlen(b);
    if l > 0 as i32 as size_t
        && *b.offset(l.wrapping_sub(1 as i32 as size_t) as isize)
            as i32 == '\n' as i32
    {
        l = l.wrapping_sub(1);
        *b.offset(l as isize) = '\0' as i32 as std::ffi::c_char;
    }
    if firstline != 0
        && *b.offset(0 as i32 as isize) as i32 == '=' as i32
    {
        lua_pushfstring(
            L,
            b"return %s\0" as *const u8 as *const std::ffi::c_char,
            b.offset(1 as i32 as isize),
        );
    } else {
        lua_pushlstring(L, b, l);
    }
    return 1 as i32;
}
unsafe extern "C" fn addreturn(mut L: *mut lua_State) -> i32 {
    let mut line: *const std::ffi::c_char = lua_tolstring(
        L,
        -(1 as i32),
        0 as *mut size_t,
    );
    let mut retline: *const std::ffi::c_char = lua_pushfstring(
        L,
        b"return %s;\0" as *const u8 as *const std::ffi::c_char,
        line,
    );
    let mut status: i32 = luaL_loadbufferx(
        L,
        retline,
        strlen(retline),
        b"=stdin\0" as *const u8 as *const std::ffi::c_char,
        0 as *const std::ffi::c_char,
    );
    if status == 0 as i32 {
        lua_rotate(L, -(2 as i32), -(1 as i32));
        lua_settop(L, -(1 as i32) - 1 as i32);
        *line.offset(0 as i32 as isize) as i32 != '\0' as i32;
    } else {
        lua_settop(L, -(2 as i32) - 1 as i32);
    }
    return status;
}
unsafe extern "C" fn multiline(mut L: *mut lua_State) -> i32 {
    loop {
        let mut len: size_t = 0;
        let mut line: *const std::ffi::c_char = lua_tolstring(
            L,
            1 as i32,
            &mut len,
        );
        let mut status: i32 = luaL_loadbufferx(
            L,
            line,
            len,
            b"=stdin\0" as *const u8 as *const std::ffi::c_char,
            0 as *const std::ffi::c_char,
        );
        if incomplete(L, status) == 0 || pushline(L, 0 as i32) == 0 {
            return status;
        }
        lua_rotate(L, -(2 as i32), -(1 as i32));
        lua_settop(L, -(1 as i32) - 1 as i32);
        lua_pushstring(L, b"\n\0" as *const u8 as *const std::ffi::c_char);
        lua_rotate(L, -(2 as i32), 1 as i32);
        lua_concat(L, 3 as i32);
    };
}
unsafe extern "C" fn loadline(mut L: *mut lua_State) -> i32 {
    let mut status: i32 = 0;
    lua_settop(L, 0 as i32);
    if pushline(L, 1 as i32) == 0 {
        return -(1 as i32);
    }
    status = addreturn(L);
    if status != 0 as i32 {
        status = multiline(L);
    }
    lua_rotate(L, 1 as i32, -(1 as i32));
    lua_settop(L, -(1 as i32) - 1 as i32);
    return status;
}
unsafe extern "C" fn l_print(mut L: *mut lua_State) {
    let mut n: i32 = lua_gettop(L);
    if n > 0 as i32 {
        luaL_checkstack(
            L,
            20 as i32,
            b"too many results to print\0" as *const u8 as *const std::ffi::c_char,
        );
        lua_getglobal(L, b"print\0" as *const u8 as *const std::ffi::c_char);
        lua_rotate(L, 1 as i32, 1 as i32);
        if lua_pcallk(
            L,
            n,
            0 as i32,
            0 as i32,
            0 as i32 as lua_KContext,
            None,
        ) != 0 as i32
        {
            l_message(
                progname,
                lua_pushfstring(
                    L,
                    b"error calling 'print' (%s)\0" as *const u8
                        as *const std::ffi::c_char,
                    lua_tolstring(L, -(1 as i32), 0 as *mut size_t),
                ),
            );
        }
    }
}
unsafe extern "C" fn doREPL(mut L: *mut lua_State) {
    let mut status: i32 = 0;
    let mut oldprogname: *const std::ffi::c_char = progname;
    progname = 0 as *const std::ffi::c_char;
    loop {
        status = loadline(L);
        if !(status != -(1 as i32)) {
            break;
        }
        if status == 0 as i32 {
            status = docall(L, 0 as i32, -(1 as i32));
        }
        if status == 0 as i32 {
            l_print(L);
        } else {
            report(L, status);
        }
    }
    lua_settop(L, 0 as i32);
    fwrite(
        b"\n\0" as *const u8 as *const std::ffi::c_char as *const std::ffi::c_void,
        size_of::<std::ffi::c_char>(),
        1,
        stdout,
    );
    fflush(stdout);
    progname = oldprogname;
}
unsafe extern "C-unwind" fn pmain(mut L: *mut lua_State) -> i32 {
    let mut argc: i32 = lua_tointegerx(
        L,
        1 as i32,
        0 as *mut i32,
    ) as i32;
    let mut argv: *mut *mut std::ffi::c_char = lua_touserdata(L, 2 as i32)
        as *mut *mut std::ffi::c_char;
    let mut script: i32 = 0;
    let mut args: i32 = collectargs(argv, &mut script);
    let mut optlim: i32 = if script > 0 as i32 {
        script
    } else {
        argc
    };
    luaL_checkversion_(
        L,
        504 as i32 as lua_Number,
        (size_of::<lua_Integer>())
            .wrapping_mul(16)
            .wrapping_add(size_of::<lua_Number>()),
    );
    if args == 1 as i32 {
        print_usage(*argv.offset(script as isize));
        return 0 as i32;
    }
    if args & 4 as i32 != 0 {
        print_version();
    }
    if args & 16 as i32 != 0 {
        lua_pushboolean(L, 1 as i32);
        lua_setfield(
            L,
            -(1000000 as i32) - 1000 as i32,
            b"LUA_NOENV\0" as *const u8 as *const std::ffi::c_char,
        );
    }
    luaL_openlibs(L);
    createargtable(L, argv, argc, script);
    lua_gc(L, 1 as i32);
    lua_gc(L, 10 as i32, 0 as i32, 0 as i32);
    if args & 16 as i32 == 0 {
        if handle_luainit(L) != 0 as i32 {
            return 0 as i32;
        }
    }
    if runargs(L, argv, optlim) == 0 {
        return 0 as i32;
    }
    if script > 0 as i32 {
        if handle_script(L, argv.offset(script as isize)) != 0 as i32 {
            return 0 as i32;
        }
    }
    if args & 2 as i32 != 0 {
        doREPL(L);
    } else if script < 1 as i32
        && args & (8 as i32 | 4 as i32) == 0
    {
        print_version();
        doREPL(L);
    }
    lua_pushboolean(L, 1 as i32);
    return 1 as i32;
}
unsafe fn main_0(
    mut argc: i32,
    mut argv: *mut *mut std::ffi::c_char,
) -> i32 {
    let mut status: i32 = 0;
    let mut result: i32 = 0;
    let mut L: *mut lua_State = luaL_newstate();
    if L.is_null() {
        l_message(
            *argv.offset(0 as i32 as isize),
            b"cannot create state: not enough memory\0" as *const u8
                as *const std::ffi::c_char,
        );
        return 1 as i32;
    }
    lua_gc(L, 0 as i32);
    lua_pushcclosure(
        L,
        Some(pmain as unsafe extern "C-unwind" fn(*mut lua_State) -> i32),
        0 as i32,
    );
    lua_pushinteger(L, argc as lua_Integer);
    lua_pushlightuserdata(L, argv as *mut std::ffi::c_void);
    status = lua_pcallk(
        L,
        2 as i32,
        1 as i32,
        0 as i32,
        0 as i32 as lua_KContext,
        None,
    );
    result = lua_toboolean(L, -(1 as i32));
    report(L, status);
    lua_close(L);
    return if result != 0 && status == 0 as i32 {
        0 as i32
    } else {
        1 as i32
    };
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
                (args.len() - 1) as i32,
                args.as_mut_ptr() as *mut *mut std::ffi::c_char,
            ) as i32,
        )
    }
}
