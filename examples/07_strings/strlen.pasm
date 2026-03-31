# ASM-BIB — String Length (strlen)
# Count characters until null terminator

@arch('x86_64')
@format('elf')

@section('.text')
@export
def _start():
    mov(rdi, my_string)
    call(strlen)

    mov(rdi, rax)
    mov(rax, 60)
    syscall()

# fn strlen(str: rdi) -> rax (length)
def strlen():
    xor(rax, rax)

    @label('scan')
    cmp(byte [rdi + rax], 0)
    je('str_done')
    inc(rax)
    jmp('scan')

    @label('str_done')
    ret()

@section('.data')
my_string = string("ASM-BIB Rocks!")
