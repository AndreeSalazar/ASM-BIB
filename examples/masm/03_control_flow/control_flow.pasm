# MASM — Control Flow (branches + loops)
@arch('x86_64')
@format('win64')

@section('.data')
counter = qword(0)

@section('.text')
@export
def main():
    # Conditional branch
    mov(rax, 42)
    cmp(rax, 42)
    je(equal_label)
    jmp(done)
    
    @label('equal_label')
    mov(rbx, 1)
    
    # Loop: count 0 to 9
    xor(rcx, rcx)
    @label('loop_start')
    cmp(rcx, 10)
    jge(loop_done)
    inc(rcx)
    jmp(loop_start)
    
    @label('loop_done')
    mov([counter], rcx)
    
    @label('done')
    xor(ecx, ecx)
    call(ExitProcess)
