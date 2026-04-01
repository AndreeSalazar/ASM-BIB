@arch('x86_64')
@format('win64')

@section('.data')
value = qword(42)
buffer = resb(256)

@section('.text')
@export
def main():
    mov(rax, [value])
    mov([value], rbx)
    lea(rdi, buffer)
    xor(ecx, ecx)
    call(ExitProcess)
