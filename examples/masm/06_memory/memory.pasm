# MASM — Memory Addressing Modes
@arch('x86_64')
@format('win64')

@section('.data')
array = dword(10, 20, 30, 40, 50, 60, 70, 80)
buffer = resb(256)
value = qword(0)

@section('.text')
@export
def main():
    # Direct addressing
    mov(eax, [array])
    
    # Register indirect
    lea(rbx, array)
    mov(eax, [rbx])
    
    # Base + displacement
    mov(eax, [rbx + 4])
    
    # Base + index * scale
    xor(rcx, rcx)
    mov(rcx, 2)
    mov(eax, [rbx + rcx * 4])
    
    # Store to memory
    mov(rax, 0xDEADBEEF)
    mov([value], rax)
    
    # Fill buffer
    lea(rdi, buffer)
    mov(al, 0)
    mov(rcx, 256)
    rep stosb()
    
    xor(ecx, ecx)
    call(ExitProcess)
