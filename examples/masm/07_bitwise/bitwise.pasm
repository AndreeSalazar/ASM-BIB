# MASM — Bitwise Operations
@arch('x86_64')
@format('win64')

@section('.data')
result = qword(0)

@section('.text')
@export
def main():
    # AND — mask bits
    mov(rax, 0xFF0F)
    and(rax, 0x0F0F)
    
    # OR — set bits
    mov(rbx, 0x00F0)
    or(rbx, 0x0F00)
    
    # XOR — toggle bits
    mov(rcx, 0xAAAA)
    xor(rcx, 0x5555)
    
    # NOT — invert all bits
    mov(rdx, 0xFF00)
    not(rdx)
    
    # SHL — shift left (multiply by 2^n)
    mov(rax, 1)
    shl(rax, 4)
    
    # SHR — shift right (unsigned divide by 2^n)
    mov(rax, 256)
    shr(rax, 3)
    
    # SAR — arithmetic shift right (preserves sign)
    mov(rax, -128)
    sar(rax, 2)
    
    # ROL / ROR — rotate
    mov(rax, 1)
    rol(rax, 3)
    ror(rax, 1)
    
    mov([result], rax)
    
    xor(ecx, ecx)
    call(ExitProcess)
