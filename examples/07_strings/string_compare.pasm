# ASM-BIB — String Compare (strcmp)
# Compare two strings byte by byte

@arch('x86_64')
@format('elf')

@section('.text')
@export
def _start():
    mov(rdi, str_a)
    mov(rsi, str_b)
    call(strcmp)

    # rax = 0 if equal, nonzero if different
    mov(rdi, rax)
    mov(rax, 60)
    syscall()

# fn strcmp(a: rdi, b: rsi) -> rax (0=equal)
def strcmp():
    @label('cmp_loop')
    movzx(eax, byte [rdi])
    movzx(ecx, byte [rsi])
    cmp(al, cl)
    jne('not_equal')
    cmp(al, 0)
    je('equal')
    inc(rdi)
    inc(rsi)
    jmp('cmp_loop')

    @label('equal')
    xor(eax, eax)
    ret()

    @label('not_equal')
    sub(eax, ecx)
    ret()

@section('.data')
str_a = string("hello")
str_b = string("hello")
