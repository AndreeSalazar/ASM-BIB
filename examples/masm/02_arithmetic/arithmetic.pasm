# MASM — Integer Arithmetic
@arch('x86_64')
@format('win64')

@section('.data')
result = qword(0)

@section('.text')
@export
def main():
    # Addition
    mov(rax, 100)
    add(rax, 200)
    
    # Subtraction
    mov(rbx, 500)
    sub(rbx, 150)
    
    # Multiplication (signed)
    mov(rax, 25)
    imul(rax, 4)
    
    # Division (signed)
    mov(rax, 100)
    xor(rdx, rdx)
    mov(rcx, 7)
    idiv(rcx)
    
    # Increment / Decrement
    inc(rax)
    dec(rbx)
    
    # Negate
    neg(rcx)
    
    # Store result
    mov([result], rax)
    
    xor(ecx, ecx)
    call(ExitProcess)
