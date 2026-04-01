# MASM — SSE and AVX Operations
@arch('x86_64')
@format('win64')

@section('.data')
vec_a = float32(1.0, 2.0, 3.0, 4.0)
vec_b = float32(5.0, 6.0, 7.0, 8.0)
vec_result = float32(0.0, 0.0, 0.0, 0.0)

@section('.text')
@export
def main():
    # SSE: load + add + store
    movaps(xmm0, [vec_a])
    movaps(xmm1, [vec_b])
    addps(xmm0, xmm1)
    movaps([vec_result], xmm0)
    
    # SSE: multiply
    movaps(xmm2, [vec_a])
    mulps(xmm2, [vec_b])
    
    # SSE: zero a register
    xorps(xmm3, xmm3)
    
    # AVX: 256-bit add (if available)
    vmovaps(ymm0, [vec_a])
    vaddps(ymm0, ymm0, [vec_b])
    
    xor(ecx, ecx)
    call(ExitProcess)
