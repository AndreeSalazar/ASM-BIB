# ASM-BIB — Hello World Linux x86-64 (syscall write)
# Syntax: Python+C hybrid → compiles to ANY dialect

@arch('x86_64')
@format('elf')

@section('.text')
@export
def _start():
    # sys_write(1, msg, 20)
    mov(rax, 1)
    mov(rdi, 1)
    lea(rsi, msg)
    mov(rdx, 20)
    syscall()

    # sys_exit(0)
    mov(rax, 60)
    xor(rdi, rdi)
    syscall()

@section('.data')
msg = string("Hello from ASM-BIB!\n")
