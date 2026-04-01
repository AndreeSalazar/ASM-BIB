# MASM — Floating Point (SSE scalar)
@arch('x86_64')
@format('win64')

@section('.data')
pi = float64(3.14159265358979)
radius = float64(5.0)
area = float64(0.0)
two = float64(2.0)

@section('.text')
@export
def main():
    push(rbp)
    mov(rbp, rsp)
    sub(rsp, 32)
    
    # area = pi * radius * radius
    movaps(xmm0, [pi])
    movaps(xmm1, [radius])
    mulps(xmm0, xmm1)
    mulps(xmm0, xmm1)
    movaps([area], xmm0)
    
    leave()
    xor(ecx, ecx)
    call(ExitProcess)
