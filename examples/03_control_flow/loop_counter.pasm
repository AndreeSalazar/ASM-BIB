# ASM-BIB — Loop with counter
# Count from 0 to 9, accumulate sum = 0+1+2+...+9 = 45

@arch('x86_64')
@format('elf')

@section('.text')
@export
def _start():
    xor(rax, rax)
    xor(rcx, rcx)

    @label('loop')
    cmp(rcx, 10)
    jge('done')

    add(rax, rcx)
    inc(rcx)
    jmp('loop')

    @label('done')
    mov(rdi, rax)
    mov(rax, 60)
    syscall()
