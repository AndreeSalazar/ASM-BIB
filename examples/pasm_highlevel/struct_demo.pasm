# Struct Demo using ASM-BIB v0.3
# MASM Vector Math

@arch('x86_64')
@format('win64')

@struct
class Vector4:
    x = float32(1.0)
    y = float32(2.0)
    z = float32(3.0)
    w = float32(0.0)

@section('.data')
    @align(16)
    vecA = Vector4(1.0, 0.0, 0.0, 0.0)
    
    @align(16)
    vecB = Vector4(0.0, 1.0, 0.0, 0.0)
    
    fmt = string("Dot Product is %f\n")

@section('.text')
@export
def main():
    prologue(40)
    
    # Load aligned vectors
    vmovaps(xmm0, oword(vecA))
    vmovaps(xmm1, oword(vecB))
    
    # Execute Dot Product
    dot4(xmm0, xmm1)
    
    # Convert and format print output (not strictly standard for XMMs but it tests dot4 generation)
    # The actual printf with float might require 8 bytes or passing XMM to RDX for MSVCRT, 
    # but the ASM-BIB macro validates the x86 syntax expansion of SIMD natively.
    
    epilogue()
