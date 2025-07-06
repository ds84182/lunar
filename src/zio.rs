use crate::*;

impl ZIO {
    pub fn new(state: *mut lua_State, reader: lua_Reader, data: *mut c_void) -> ZIO {
        ZIO {
            L: state,
            reader,
            data,
            n: 0,
            p: ptr::null(),
        }
    }

    pub unsafe fn fill(&mut self) -> i32 {
        let mut size = 0;
        let state = self.L;
        let buff = self.reader.unwrap()(state, self.data, &mut size);
        if buff.is_null() || size == 0 {
            return -1;
        }
        // discount char being returned
        self.n = size - 1;
        let c = *buff;
        self.p = buff.add(1);
        c as i32
    }

    pub unsafe fn read(&mut self, mut b: &mut [u8]) -> usize {
        while !b.is_empty() {
            let m;
            // no bytes in buffer?
            if self.n == 0 {
                // try to read more
                if self.fill() == -1 {
                    // no more input; return number of missing bytes
                    return b.len()
                } else {
                    // luaZ_fill consumed first byte; put it back
                    self.n += 1;
                    self.p = self.p.offset(-1);
                }
            }
            // min. between n and z->n
            m = if b.len() <= self.n { b.len() } else { self.n };
            ptr::copy_nonoverlapping(self.p, b.as_mut_ptr().cast(), m);
            self.n -= m;
            self.p = self.p.add(m);
            b = b.get_unchecked_mut(m..);
        }
        0
    }

    pub unsafe fn read_byte(&mut self) -> i32 {
        if self.n > 0 {
            self.n -= 1;
            let p = self.p;
            self.p = p.add(1);
            *p as i32
        } else {
            self.fill()
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe fn luaZ_fill(mut z: *mut ZIO) -> i32 {
    let mut size: size_t = 0;
    let mut L: *mut lua_State = (*z).L;
    let mut buff: *const std::ffi::c_char = 0 as *const std::ffi::c_char;
    buff = ((*z).reader).expect("non-null function pointer")(L, (*z).data, &mut size);
    if buff.is_null() || size == 0 as size_t {
        return -(1 as i32);
    }
    (*z).n = size.wrapping_sub(1 as i32 as size_t);
    (*z).p = buff;
    let fresh0 = (*z).p;
    (*z).p = ((*z).p).offset(1);
    return *fresh0 as std::ffi::c_uchar as i32;
}

#[unsafe(no_mangle)]
pub unsafe fn luaZ_init(
    mut L: *mut lua_State,
    mut z: *mut ZIO,
    mut reader: lua_Reader,
    mut data: *mut c_void,
) {
    (*z).L = L;
    (*z).reader = reader;
    (*z).data = data;
    (*z).n = 0 as size_t;
    (*z).p = 0 as *const std::ffi::c_char;
}

#[unsafe(no_mangle)]
pub unsafe fn luaZ_read(mut z: *mut ZIO, mut b: *mut c_void, mut n: size_t) -> size_t {
    while n != 0 {
        let mut m: size_t = 0;
        if (*z).n == 0 as size_t {
            if luaZ_fill(z) == -(1 as i32) {
                return n;
            } else {
                (*z).n = ((*z).n).wrapping_add(1);
                (*z).n;
                (*z).p = ((*z).p).offset(-1);
                (*z).p;
            }
        }
        m = if n <= (*z).n { n } else { (*z).n };
        memcpy(b, (*z).p as *const c_void, m);
        (*z).n = ((*z).n).wrapping_sub(m);
        (*z).p = ((*z).p).offset(m as isize);
        b = (b as *mut std::ffi::c_char).offset(m as isize) as *mut c_void;
        n = n.wrapping_sub(m);
    }
    return 0 as size_t;
}
