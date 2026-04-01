# MASM — Win64 API Direct Calls
# Uses shadow space + Win64 ABI

@arch('x86_64')
@format('win64')

@section('.data')
msg = string("Hello from Win64 API!\n")
title = string("ASM-BIB MASM")
bytes_written = dword(0)
stdout_handle = qword(0)

@section('.text')
@export
def main():
    push(rbp)
    mov(rbp, rsp)
    sub(rsp, 64)
    
    # GetStdHandle(STD_OUTPUT_HANDLE = -11)
    mov(rcx, -11)
    call(GetStdHandle)
    mov([stdout_handle], rax)
    
    # WriteConsoleA(handle, buf, len, &written, 0)
    mov(rcx, rax)
    lea(rdx, msg)
    mov(r8, 22)
    lea(r9, bytes_written)
    mov(rax, 0)
    push(rax)
    sub(rsp, 32)
    call(WriteConsoleA)
    add(rsp, 40)
    
    # ExitProcess(0)
    xor(ecx, ecx)
    call(ExitProcess)
