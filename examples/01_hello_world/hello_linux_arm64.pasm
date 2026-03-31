# ASM-BIB — Hello World Linux ARM64 (syscall write)
# ARM64 ABI: x0-x7 params, x8 = syscall number

@arch('arm64')
@format('elf')

@section('.text')
@export
def _start():
    # sys_write(1, msg, 20)
    mov(x0, 1)
    adr(x1, msg)
    mov(x2, 20)
    mov(x8, 64)
    svc(0)

    # sys_exit(0)
    mov(x0, 0)
    mov(x8, 93)
    svc(0)

@section('.data')
msg = string("Hello from ASM-BIB!\n")
