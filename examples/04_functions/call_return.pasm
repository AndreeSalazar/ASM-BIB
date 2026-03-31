# ASM-BIB — Functions: call/ret with stack frame
# Demonstrates prologue/epilogue and calling convention

@arch('x86_64')
@format('elf')

@section('.text')
@export
def _start():
    mov(rdi, 25)
    mov(rsi, 17)
    call(add_numbers)

    mov(rdi, rax)
    mov(rax, 60)
    syscall()

# fn add_numbers(a: rdi, b: rsi) -> rax
def add_numbers():
    push(rbp)
    mov(rbp, rsp)

    mov(rax, rdi)
    add(rax, rsi)

    pop(rbp)
    ret()

