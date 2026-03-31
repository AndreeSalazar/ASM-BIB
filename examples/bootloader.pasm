# NASM-BIB — Stage 1 bootloader

@arch('x86_16')
@format('flat')
@org(0x7C00)

@section('.text')
def start():
    cli()
    xor(ax, ax)
    mov(ds, ax)
    mov(es, ax)
    mov(ss, ax)
    mov(sp, 0x7C00)
    sti()
    hlt()

@section('.boot_sig')
boot_sig = word(0xAA55)
