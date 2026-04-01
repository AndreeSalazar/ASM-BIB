@arch('x86_64')
@format('win64')

@section('.data')
src = string("Test")
dst = resb(16)

@section('.text')
@export
def main():
    lea(rsi, src)
    lea(rdi, dst)
    mov(rcx, 5)
    rep movsb()
    xor(ecx, ecx)
    call(ExitProcess)
