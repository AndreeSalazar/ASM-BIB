# ASM-BIB — LEA and Address Modes
# Demonstrates all x86-64 addressing modes

@arch('x86_64')
@format('elf')

@section('.text')
@export
def _start():
    mov(rsi, table)

    # Direct: [rsi]
    mov(eax, [rsi])

    # Displacement: [rsi + 8]
    mov(ebx, [rsi + 8])

    # Index + scale: [rsi + rcx*4]
    mov(rcx, 2)
    mov(edx, [rsi + rcx * 4])

    # LEA for arithmetic: rax = rbx + rcx*2 + 10
    lea(rax, [rbx + rcx * 2 + 10])

    mov(rdi, rax)
    mov(rax, 60)
    syscall()

@section('.data')
table = dword(10, 20, 30, 40, 50)
