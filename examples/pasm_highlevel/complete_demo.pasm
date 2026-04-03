# ASM-BIB v0.3 Complete Demo
# Showcases @switch, @struct, new macros (pow, scanf), and calling conventions

@arch('x86_64')
@format('win64')

@struct
class Point3D:
    x = float32(0.0)
    y = float32(0.0)
    z = float32(0.0)

@section('.data')
    menu_str = string("1. Math test (pow)\n2. Math test (sqrt)\n3. Struct test\nSelect: ")
    fmt_int = string("%d")
    result_str = string("Result: %d\n")
    float_res = string("Result: %f\n")
    invalid_str = string("Invalid option!\n")
    
    @align(16)
    pts = Point3D(1.0, 2.0, 3.0)

@section('.bss')
    choice = resd(1)

@section('.text')

@public
@stdcall
def DoMath():
    prologue(32)
    # pow(base=2, exp=10) -> 1024
    pow(2, 10)
    printf(result_str, rax)
    epilogue()

@public
@fastcall
def DoSqrt():
    prologue(32)
    # sqrt(9.0) using SSE
    mov(rax, 9)
    cvtsi2ss(xmm0, rax)
    sqrt(xmm0)
    cvtss2si(rax, xmm0)
    printf(result_str, rax)
    epilogue()

@export
def main():
    prologue(40)
    
    # 1. Print Menu
    print(menu_str)
    
    # 2. Get Input
    scanf(fmt_int, choice)
    
    # 3. Handle selection with Switch
    mov(eax, dword(choice))
    
    @switch(eax)
        @case(1)
            call(DoMath)
            @break
            
        @case(2)
            call(DoSqrt)
            @break
            
        @case(3)
            # Struct accessing via registers
            lea(rcx, pts)
            movss(xmm0, dword(rcx))       # pts.x
            addss(xmm0, dword(rcx + 4))   # pts.x + pts.y
            cvtss2si(rax, xmm0)
            printf(result_str, rax)
            @break
            
        @default
            print(invalid_str)
            @break
            
    @endswitch
    
    exit(0)
