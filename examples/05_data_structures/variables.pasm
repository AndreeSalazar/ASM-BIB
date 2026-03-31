# ASM-BIB — Variables: initialized and uninitialized data

@arch('x86_64')
@format('elf')

@section('.text')
@export
def _start():
    # Load initialized data
    mov(rsi, counter)
    mov(eax, [rsi])
    add(eax, 10)
    mov([rsi], eax)

    # Store to uninitialized buffer
    mov(rsi, result_buf)
    mov(dword [rsi], 42)

    mov(rdi, rax)
    mov(rax, 60)
    syscall()

@section('.data')
counter = dword(100)
max_val = dword(255)
pi_approx = dword(3)
greeting = string("Hola mundo\n")

@section('.bss')
result_buf = resb(256)
temp_val = resd(1)
