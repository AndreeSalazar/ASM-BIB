# NASM-BIB — Hello World x86_64

@arch('x86_64')
@format('elf')

@section('.text')
@export
def main():
    push(rbp)
    mov(rbp, rsp)
    sub(rsp, 32)

    lea(rcx, msg)
    call(printf)

    xor(eax, eax)
    leave()
    ret()

@section('.data')
msg = string("Hello from ASM-BIB!\n")
