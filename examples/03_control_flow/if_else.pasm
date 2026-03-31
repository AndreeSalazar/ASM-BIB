# ASM-BIB — If/Else via compare + conditional jump
# if (rax > 10) → resultado = 1, else → resultado = 0

@arch('x86_64')
@format('elf')

@section('.text')
@export
def _start():
    mov(rax, 15)

    cmp(rax, 10)
    jle('es_menor_igual')

    # rax > 10
    mov(rdi, 1)
    jmp('exit')

    @label('es_menor_igual')
    mov(rdi, 0)

    @label('exit')
    mov(rax, 60)
    syscall()
