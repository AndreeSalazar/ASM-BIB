# ASM-BIB — Multiple Parameters (System V AMD64 ABI)
# fn compute(a, b, c, d) = (a + b) * (c - d)
# Params: rdi, rsi, rdx, rcx → result in rax

@arch('x86_64')
@format('elf')

@section('.text')
@export
def _start():
    mov(rdi, 10)
    mov(rsi, 5)
    mov(rdx, 8)
    mov(rcx, 2)
    call(compute)

    mov(rdi, rax)
    mov(rax, 60)
    syscall()

# (a + b) * (c - d) = (10+5) * (8-2) = 15 * 6 = 90
def compute():
    push(rbp)
    mov(rbp, rsp)

    mov(rax, rdi)
    add(rax, rsi)

    mov(r10, rdx)
    sub(r10, rcx)

    imul(rax, r10)

    pop(rbp)
    ret()
