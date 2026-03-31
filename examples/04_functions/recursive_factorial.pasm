# ASM-BIB — Recursive Factorial
# factorial(5) = 120

@arch('x86_64')
@format('elf')

@section('.text')
@export
def _start():
    mov(rdi, 5)
    call(factorial)
    mov(rdi, rax)
    mov(rax, 60)
    syscall()

# fn factorial(n: rdi) -> rax
def factorial():
    push(rbp)
    mov(rbp, rsp)

    cmp(rdi, 1)
    jle('base_case')

    # recursive: n * factorial(n-1)
    push(rdi)
    dec(rdi)
    call(factorial)
    pop(rdi)
    imul(rax, rdi)
    jmp('fact_end')

    @label('base_case')
    mov(rax, 1)

    @label('fact_end')
    pop(rbp)
    ret()
