# ============================================================
# ASM-BIB Standard Library — Ring 1-3 MASM Utilities
# ============================================================
# Complete set of everyday C/C++ utility functions implemented
# in pure x86-64 assembly for MSVC ABI (Win64 fastcall).
#
# Export as .obj for ADead-BIB bridge consumption:
#   asm-bib stdlib_ring3.pasm --native --obj -o stdlib_ring3.obj
#
# Categories:
#   1. String:   asm_strlen, asm_strcpy, asm_strcmp, asm_strcat,
#                asm_memcpy, asm_memset, asm_memcmp, asm_strchr
#   2. Math:     asm_abs, asm_min, asm_max, asm_clamp, asm_swap
#   3. Bit:      asm_popcount, asm_bsr64, asm_bsf64, asm_bswap32, asm_bswap64
#   4. Utility:  asm_is_aligned, asm_align_up, asm_noop
#
# ABI: Win64 fastcall
#   args: RCX, RDX, R8, R9 (int/ptr) | XMM0-XMM3 (float)
#   return: RAX (int/ptr) | XMM0 (float)
# ============================================================

@arch('x86_64')
@format('win64')

@section('.text')

# ── asm_strlen(const char* str) → u64 length ──────────────
@export
def asm_strlen():
    xor(rax, rax)
    @label(strlen_loop)
    cmp(byte [rcx + rax], 0)
    je(strlen_done)
    inc(rax)
    jmp(strlen_loop)
    @label(strlen_done)
    ret()

# ── asm_strcpy(char* dst, const char* src) → char* dst ────
@export
def asm_strcpy():
    mov(rax, rcx)
    @label(strcpy_loop)
    mov(r8b, byte [rdx])
    mov(byte [rcx], r8b)
    test(r8b, r8b)
    jz(strcpy_done)
    inc(rcx)
    inc(rdx)
    jmp(strcpy_loop)
    @label(strcpy_done)
    ret()

# ── asm_strcmp(const char* a, const char* b) → i32 ────────
@export
def asm_strcmp():
    @label(strcmp_loop)
    movzx(eax, byte [rcx])
    movzx(r8d, byte [rdx])
    sub(eax, r8d)
    jne(strcmp_done)
    cmp(byte [rcx], 0)
    je(strcmp_done)
    inc(rcx)
    inc(rdx)
    jmp(strcmp_loop)
    @label(strcmp_done)
    ret()

# ── asm_strcat(char* dst, const char* src) → char* dst ────
@export
def asm_strcat():
    mov(rax, rcx)
    @label(strcat_find)
    cmp(byte [rcx], 0)
    je(strcat_copy)
    inc(rcx)
    jmp(strcat_find)
    @label(strcat_copy)
    mov(r8b, byte [rdx])
    mov(byte [rcx], r8b)
    test(r8b, r8b)
    jz(strcat_done)
    inc(rcx)
    inc(rdx)
    jmp(strcat_copy)
    @label(strcat_done)
    ret()

# ── asm_strchr(const char* s, int c) → char* or NULL ─────
@export
def asm_strchr():
    @label(strchr_loop)
    movzx(eax, byte [rcx])
    cmp(al, dl)
    je(strchr_found)
    test(al, al)
    jz(strchr_notfound)
    inc(rcx)
    jmp(strchr_loop)
    @label(strchr_found)
    mov(rax, rcx)
    ret()
    @label(strchr_notfound)
    xor(rax, rax)
    ret()

# ── asm_memcpy(void* dst, const void* src, u64 n) → void* dst
@export
def asm_memcpy():
    mov(rax, rcx)
    mov(rcx, r8)
    push(rdi)
    push(rsi)
    mov(rdi, rax)
    mov(rsi, rdx)
    rep movsb()
    pop(rsi)
    pop(rdi)
    ret()

# ── asm_memset(void* dst, int val, u64 n) → void* dst ────
@export
def asm_memset():
    mov(r9, rcx)
    mov(rcx, r8)
    push(rdi)
    mov(rdi, r9)
    mov(eax, edx)
    rep stosb()
    pop(rdi)
    mov(rax, r9)
    ret()

# ── asm_memcmp(const void* a, const void* b, u64 n) → i32
@export
def asm_memcmp():
    test(r8, r8)
    jz(memcmp_eq)
    @label(memcmp_loop)
    movzx(eax, byte [rcx])
    movzx(r9d, byte [rdx])
    sub(eax, r9d)
    jne(memcmp_done)
    inc(rcx)
    inc(rdx)
    dec(r8)
    jnz(memcmp_loop)
    @label(memcmp_eq)
    xor(eax, eax)
    @label(memcmp_done)
    ret()

# ── asm_abs(i64 x) → u64 ─────────────────────────────────
@export
def asm_abs():
    mov(rax, rcx)
    cqo()
    xor(rax, rdx)
    sub(rax, rdx)
    ret()

# ── asm_min(i64 a, i64 b) → i64 ──────────────────────────
@export
def asm_min():
    mov(rax, rcx)
    cmp(rcx, rdx)
    cmovg(rax, rdx)
    ret()

# ── asm_max(i64 a, i64 b) → i64 ──────────────────────────
@export
def asm_max():
    mov(rax, rcx)
    cmp(rcx, rdx)
    cmovl(rax, rdx)
    ret()

# ── asm_clamp(i64 val, i64 lo, i64 hi) → i64 ────────────
@export
def asm_clamp():
    mov(rax, rcx)
    cmp(rax, rdx)
    cmovl(rax, rdx)
    cmp(rax, r8)
    cmovg(rax, r8)
    ret()

# ── asm_swap(i64* a, i64* b) → void ──────────────────────
@export
def asm_swap():
    mov(rax, qword [rcx])
    mov(r8, qword [rdx])
    mov(qword [rcx], r8)
    mov(qword [rdx], rax)
    ret()

# ── asm_popcount(u64 x) → u32 ────────────────────────────
@export
def asm_popcount():
    popcnt(rax, rcx)
    ret()

# ── asm_bsr64(u64 x) → i32 ──────────────────────────────
@export
def asm_bsr64():
    test(rcx, rcx)
    jz(bsr_zero)
    bsr(rax, rcx)
    ret()
    @label(bsr_zero)
    mov(eax, -1)
    ret()

# ── asm_bsf64(u64 x) → i32 ──────────────────────────────
@export
def asm_bsf64():
    test(rcx, rcx)
    jz(bsf_zero)
    bsf(rax, rcx)
    ret()
    @label(bsf_zero)
    mov(eax, -1)
    ret()

# ── asm_bswap32(u32 x) → u32 ────────────────────────────
@export
def asm_bswap32():
    mov(eax, ecx)
    bswap(eax)
    ret()

# ── asm_bswap64(u64 x) → u64 ────────────────────────────
@export
def asm_bswap64():
    mov(rax, rcx)
    bswap(rax)
    ret()

# ── asm_is_aligned(u64 addr, u64 alignment) → bool ───────
@export
def asm_is_aligned():
    mov(rax, rdx)
    dec(rax)
    test(rcx, rax)
    setz(al)
    movzx(eax, al)
    ret()

# ── asm_align_up(u64 val, u64 alignment) → u64 ──────────
@export
def asm_align_up():
    mov(rax, rdx)
    dec(rax)
    add(rcx, rax)
    not(rax)
    and(rax, rcx)
    ret()

# ── asm_noop() → void ────────────────────────────────────
@export
def asm_noop():
    nop()
    ret()
