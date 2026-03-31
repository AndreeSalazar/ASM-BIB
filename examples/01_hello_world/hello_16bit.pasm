# ASM-BIB — Hello World DOS/BIOS 16-bit (int 10h teletype)
# Real mode, BIOS interrupt for character output

@arch('x86_16')
@format('flat')
@org(0x7C00)

@section('.text')
def _start():
    cli()
    xor(ax, ax)
    mov(ds, ax)
    mov(es, ax)
    sti()

    # Print each char via BIOS int 0x10 (AH=0x0E teletype)
    mov(si, msg)

    @label('print_loop')
    mov(al, [si])
    cmp(al, 0)
    je('done')
    mov(ah, 0x0E)
    int(0x10)
    inc(si)
    jmp('print_loop')

    @label('done')
    hlt()

@section('.data')
msg = string("Hello ASM-BIB 16bit!")
