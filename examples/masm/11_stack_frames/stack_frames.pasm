# MASM — Stack Frame Management (Win64 ABI)
@arch('x86_64')
@format('win64')

@section('.text')

def callee():
    # Function with local space
    push(rbp)
    mov(rbp, rsp)
    sub(rsp, 48)
    
    # Use local space [rbp - 8] .. [rbp - 48]
    mov([rbp - 8], rcx)
    mov([rbp - 16], rdx)
    
    # Compute something
    mov(rax, [rbp - 8])
    add(rax, [rbp - 16])
    
    leave()
    ret()

@export
def main():
    push(rbp)
    mov(rbp, rsp)
    sub(rsp, 32)
    
    # Call with Win64 shadow space
    mov(rcx, 10)
    mov(rdx, 20)
    call(callee)
    
    # rax now = 30
    leave()
    xor(ecx, ecx)
    call(ExitProcess)
