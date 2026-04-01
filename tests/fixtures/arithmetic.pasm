@arch('x86_64')
@format('win64')

@section('.data')
result = qword(0)

@section('.text')
@export
def main():
    mov(rax, 100)
    add(rax, 200)
    mov(rbx, 500)
    sub(rbx, 150)
    imul(rax, 4)
    xor(ecx, ecx)
    call(ExitProcess)
