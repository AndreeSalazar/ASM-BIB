# ASM-BIB — String to Uppercase
# Convert lowercase a-z → A-Z in place

@arch('x86_64')
@format('elf')

@section('.text')
@export
def _start():
    mov(rdi, text)
    call(to_upper)

    # Write result to stdout
    mov(rax, 1)
    mov(rdi, 1)
    lea(rsi, text)
    mov(rdx, 14)
    syscall()

    mov(rax, 60)
    xor(rdi, rdi)
    syscall()

# fn to_upper(str: rdi) — in-place
def to_upper():
    @label('upper_loop')
    movzx(eax, byte [rdi])
    cmp(al, 0)
    je('upper_done')

    # Check if lowercase: 'a' (0x61) <= al <= 'z' (0x7A)
    cmp(al, 0x61)
    jb('skip')
    cmp(al, 0x7A)
    ja('skip')

    # Convert: subtract 0x20
    sub(al, 0x20)
    mov([rdi], al)

    @label('skip')
    inc(rdi)
    jmp('upper_loop')

    @label('upper_done')
    ret()

@section('.data')
text = string("hello world!\n")
