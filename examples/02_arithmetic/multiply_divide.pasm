# ASM-BIB — Multiply & Divide
# mul, imul, div, idiv examples

@arch('x86_64')
@format('elf')

@section('.text')
@export
def _start():
    # Multiply: 7 * 6 = 42
    mov(rax, 7)
    mov(rbx, 6)
    imul(rax, rbx)
    mov(r12, rax)

    # Divide: 42 / 7 = 6
    mov(rax, 42)
    xor(rdx, rdx)
    mov(rbx, 7)
    div(rbx)
    # rax = cociente (6), rdx = residuo (0)

    # Exit con cociente
    mov(rdi, rax)
    mov(rax, 60)
    syscall()
