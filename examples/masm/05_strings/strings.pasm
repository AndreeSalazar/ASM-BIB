# MASM — String Operations
@arch('x86_64')
@format('win64')

@section('.data')
source = string("Hello, MASM World!")
dest = resb(64)
search_char = byte(77)

@section('.text')
@export
def main():
    # Copy string (rep movsb)
    lea(rsi, source)
    lea(rdi, dest)
    mov(rcx, 18)
    rep movsb()
    
    # Search for 'M' in source (scasb)
    lea(rdi, source)
    mov(rcx, 18)
    mov(al, 77)
    scasb()
    
    xor(ecx, ecx)
    call(ExitProcess)
