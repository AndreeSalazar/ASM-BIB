# MASM — Procedures with parameters and locals
@arch('x86_64')
@format('win64')

@section('.text')

def add_numbers():
    # rcx = a, rdx = b (Win64 calling convention)
    mov(rax, rcx)
    add(rax, rdx)
    ret()

def factorial():
    # rcx = n, returns n! in rax
    push(rbp)
    mov(rbp, rsp)
    sub(rsp, 32)
    
    cmp(rcx, 1)
    jle(base_case)
    
    # Recursive: n * factorial(n-1)
    push(rcx)
    dec(rcx)
    call(factorial)
    pop(rcx)
    imul(rax, rcx)
    jmp(fact_done)
    
    @label('base_case')
    mov(rax, 1)
    
    @label('fact_done')
    leave()
    ret()

@export
def main():
    # add_numbers(30, 12) → rax = 42
    mov(rcx, 30)
    mov(rdx, 12)
    call(add_numbers)
    
    # factorial(5) → rax = 120
    mov(rcx, 5)
    call(factorial)
    
    xor(ecx, ecx)
    call(ExitProcess)
