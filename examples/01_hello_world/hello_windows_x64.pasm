# ASM-BIB — Hello World Windows x86-64 (printf via CRT)
# Uses Windows x64 calling convention: rcx, rdx, r8, r9

@arch('x86_64')
@format('pe')

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
