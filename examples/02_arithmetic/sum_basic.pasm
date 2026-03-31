# ASM-BIB — Basic Sum: a + b = c
# Returns sum in eax (exit code)

@arch('x86_64')
@format('elf')

@section('.text')
@export
def _start():
    # a = 25, b = 17, resultado = a + b
    mov(rax, 25)
    mov(rbx, 17)
    add(rax, rbx)

    # exit con resultado (echo $?)
    mov(rdi, rax)
    mov(rax, 60)
    syscall()
