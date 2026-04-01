# MASM — Win64 Console Hello World
# Uses GetStdHandle + WriteFile + ExitProcess

@arch('x86_64')
@format('win64')

@section('.data')
msg = string("Hello from MASM via ASM-BIB!\n")
bytes_written = dword(0)

@section('.text')
@export
def main():
    # Get stdout handle
    mov(rcx, -11)
    call(GetStdHandle)
    mov(rbx, rax)
    
    # WriteFile(handle, msg, len, &bytes_written, 0)
    mov(rcx, rbx)
    lea(rdx, msg)
    mov(r8, 29)
    lea(r9, bytes_written)
    push(0)
    call(WriteFile)
    
    # ExitProcess(0)
    xor(ecx, ecx)
    call(ExitProcess)
