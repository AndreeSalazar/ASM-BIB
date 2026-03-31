# ASM-BIB — Arrays: define, access, modify

@arch('x86_64')
@format('elf')

@section('.text')
@export
def _start():
    # Read array[2] (third element = 30)
    mov(rsi, my_array)
    mov(eax, [rsi + 8])

    # Write array[0] = 99
    mov(rsi, my_array)
    mov(dword [rsi], 99)

    # Sum first 3 elements
    xor(rax, rax)
    add(eax, [rsi])
    add(eax, [rsi + 4])
    add(eax, [rsi + 8])

    mov(rdi, rax)
    mov(rax, 60)
    syscall()

@section('.data')
my_array = dword(10, 20, 30, 40, 50)
