# System Patterns

**Last Updated:** 2026-03-17

Patterns and constraints that apply across the card-worder codebase.

---

## Memory allocation (embedded platform)

**Constraint:** The target platform has limited stack space. We cannot allocate much memory on the stack.

**Pattern:** Prefer **static allocation** almost always.

- Use statically allocated buffers, heapless containers, and `const`/`static` where possible.
- Avoid large stack-allocated structs, big local arrays, and deep recursion.
- When size is bounded and known at compile time, use fixed-capacity types (e.g. `heapless::Vec`, `heapless::String`, `[u8; N]`) instead of heap or unbounded stack allocation.

This applies to all Rust code in the project targeting the ESP32/Cardputer.
