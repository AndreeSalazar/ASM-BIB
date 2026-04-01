# MASM — Struct Definitions
@arch('x86_64')
@format('win64')

@section('.data')
# Point struct instance
point_x = dword(100)
point_y = dword(200)

# Rectangle
rect_left = dword(0)
rect_top = dword(0)
rect_right = dword(640)
rect_bottom = dword(480)

@section('.text')
@export
def main():
    # Load struct fields
    mov(eax, [point_x])
    mov(ebx, [point_y])
    
    # Compute distance squared: dx*dx + dy*dy
    mov(ecx, [rect_right])
    sub(ecx, [rect_left])
    imul(ecx, ecx)
    
    mov(edx, [rect_bottom])
    sub(edx, [rect_top])
    imul(edx, edx)
    
    add(ecx, edx)
    
    xor(ecx, ecx)
    call(ExitProcess)
