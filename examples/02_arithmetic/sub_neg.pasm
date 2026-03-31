# ASM-BIB — Subtraction and Negation

@arch('x86_64')
@format('elf')

@section('.text')
@export
def _start():
    # sub: 100 - 58 = 42
    mov(rax, 100)
    sub(rax, 58)
    mov(r12, rax)

    # neg: negar un valor
    mov(rbx, 42)
    neg(rbx)
    # rbx = -42

    # inc/dec
    mov(rcx, 10)
    inc(rcx)
    dec(rcx)

    # Exit con resultado de sub
    mov(rdi, r12)
    mov(rax, 60)
    syscall()
