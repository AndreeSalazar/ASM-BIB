@arch('x86_64')
@format('win64')

@section('.data')
pi = float64(3.14)
vals = float32(1.0, 2.0)

@section('.text')
@export
def main():
    movaps(xmm0, [pi])
    xor(ecx, ecx)
    call(ExitProcess)
