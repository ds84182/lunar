This is Lua 5.4.7, released on 13 Jun 2024.

But it actually isn't.

# Lunar

Lunar is Lua 5.4.7, transpiled to Rust using [c2rust](https://github.com/immunant/c2rust).

## Why?

Lunar uses Rust's panic machinery instead of setjmp + longjmp, which unwinds the stack for RAII languages like Rust and C++.

There will be more features added in the future, like a tracing JIT.

## Roadmap

Not in order.

- [ ] VM Interpreter Optimizations? Tailcall-based? Dynasm?
- [ ] Tracing JIT? (behind a feature flag)
- [ ] ???
- [ ] Syntax Extensions? Static Typing?
- [ ] Prolog-esque Backtracing (behind a feature flag)
