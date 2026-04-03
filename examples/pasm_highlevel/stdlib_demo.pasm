# PASM stdlib demo — all builtins in action
# Compiles to MASM → ml64 → .exe

@arch('x86_64')
@format('win64')

@section('.data')
buf = buffer(256)

@section('.text')
@export
def main():
    # I/O
    print("=== ASM-BIB stdlib demo ===\n")
    printf("%s %d + %d = %d\n", "Result:", 10, 20, 30)

    # Math: min/max
    mov(rax, 42)
    mov(rbx, 17)
    min(rax, rbx)
    printf("min(42,17) = %lld\n", rax)

    # Control flow: @if/@else/@endif
    mov(rax, 100)
    @if(rax, "==", 100)
        print("rax is 100!\n")
    @else
        print("rax is NOT 100\n")
    @endif

    # Loop: @loop/@endloop
    mov(r12, 0)
    @loop(rcx, 3)
        push(rcx)
        add(r12, 1)
        printf("loop iteration %lld\n", r12)
        pop(rcx)
    @endloop

    print("=== done ===\n")
    exit(0)
