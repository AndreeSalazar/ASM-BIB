# ASM-BIB — Sum Array of integers
# Suma todos los elementos de un array

@arch('x86_64')
@format('elf')

@section('.text')
@export
def _start():
    xor(rax, rax)
    xor(rcx, rcx)
    mov(rsi, numbers)

    @label('sum_loop')
    cmp(rcx, 5)
    jge('done')
    add(eax, [rsi + rcx * 4])
    inc(rcx)
    jmp('sum_loop')

    @label('done')
    mov(rdi, rax)
    mov(rax, 60)
    syscall()

@section('.data')
numbers = dword(10, 20, 30, 40, 50)
