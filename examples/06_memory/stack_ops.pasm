# ASM-BIB — Stack Operations: push, pop, stack frame

@arch('x86_64')
@format('elf')

@section('.text')
@export
def _start():
    # Save registers via stack
    push(rbx)
    push(r12)
    push(r13)

    mov(rbx, 100)
    mov(r12, 200)
    mov(r13, 300)

    # Use stack-allocated local variable
    sub(rsp, 16)
    mov(qword [rsp], 42)
    mov(qword [rsp + 8], 99)

    # Read back
    mov(rax, [rsp])
    add(rax, [rsp + 8])

    add(rsp, 16)

    # Restore
    pop(r13)
    pop(r12)
    pop(rbx)

    mov(rdi, rax)
    mov(rax, 60)
    syscall()
