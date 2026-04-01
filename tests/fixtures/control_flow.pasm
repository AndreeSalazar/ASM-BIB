@arch('x86_64')
@format('win64')

@section('.text')
@export
def main():
    mov(rax, 10)
    cmp(rax, 10)
    je(equal)
    jmp(done)
    @label('equal')
    mov(rbx, 1)
    @label('done')
    xor(ecx, ecx)
    call(ExitProcess)
