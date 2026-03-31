# ASM-BIB — Switch/Case via jump table
# Simulates switch(n) { case 0: ..., case 1: ..., case 2: ... }

@arch('x86_64')
@format('elf')

@section('.text')
@export
def _start():
    mov(rax, 1)

    cmp(rax, 0)
    je('case_0')
    cmp(rax, 1)
    je('case_1')
    cmp(rax, 2)
    je('case_2')
    jmp('default')

    @label('case_0')
    mov(rdi, 10)
    jmp('exit')

    @label('case_1')
    mov(rdi, 20)
    jmp('exit')

    @label('case_2')
    mov(rdi, 30)
    jmp('exit')

    @label('default')
    mov(rdi, 0)

    @label('exit')
    mov(rax, 60)
    syscall()
