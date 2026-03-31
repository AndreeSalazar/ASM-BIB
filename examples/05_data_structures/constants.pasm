# ASM-BIB — Constants and multiple data types

@arch('x86_64')
@format('elf')

@section('.text')
@export
def _start():
    # Use byte constant
    mov(rsi, char_A)
    movzx(eax, byte [rsi])

    # Use word constant
    mov(rsi, port_num)
    movzx(eax, word [rsi])

    # Use qword constant
    mov(rsi, big_number)
    mov(rax, [rsi])

    mov(rdi, rax)
    mov(rax, 60)
    syscall()

@section('.data')
char_A = byte(0x41)
char_B = byte(0x42)
port_num = word(0x3F8)
screen_w = dword(1920)
screen_h = dword(1080)
big_number = qword(0x123456789ABCDEF0)
message = string("Constants demo\n")
