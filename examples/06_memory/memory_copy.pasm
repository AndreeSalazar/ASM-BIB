# ASM-BIB — Memory Copy (memcpy via rep movsb)

@arch('x86_64')
@format('elf')

@section('.text')
@export
def _start():
    # Copy src → dst, 13 bytes
    lea(rsi, src_data)
    lea(rdi, dst_buf)
    mov(rcx, 13)
    cld()
    rep movsb()

    # Verify: read first byte of dst
    mov(rsi, dst_buf)
    movzx(eax, byte [rsi])

    mov(rdi, rax)
    mov(rax, 60)
    syscall()

@section('.data')
src_data = string("Hello World!\n")

@section('.bss')
dst_buf = resb(64)
