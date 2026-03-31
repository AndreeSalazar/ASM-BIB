# ASM-BIB — Nested Loops
# Outer loop 0..3, inner loop 0..3 → total iterations = 16

@arch('x86_64')
@format('elf')

@section('.text')
@export
def _start():
    xor(r12, r12)
    xor(rcx, rcx)

    @label('outer')
    cmp(rcx, 4)
    jge('done')
    xor(rdx, rdx)

    @label('inner')
    cmp(rdx, 4)
    jge('next_outer')
    inc(r12)
    inc(rdx)
    jmp('inner')

    @label('next_outer')
    inc(rcx)
    jmp('outer')

    @label('done')
    mov(rdi, r12)
    mov(rax, 60)
    syscall()
