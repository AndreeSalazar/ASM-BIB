# MASM — Constants and Equates
@arch('x86_64')
@format('win64')

@section('.data')
buffer = resb(4096)
result = qword(0)

@section('.text')
@export
def main():
    # Use constants in code
    mov(rax, 4096)
    mov(rbx, 0x1000)
    
    # Page-aligned allocation pattern
    add(rax, 4095)
    and(rax, -4096)
    
    mov([result], rax)
    
    xor(ecx, ecx)
    call(ExitProcess)
