@arch('x86_64')
@format('win64')

@section('.data')
msg = string("Hello!\n")

@section('.text')
@export
def main():
    lea(rcx, msg)
    call(printf)
    xor(ecx, ecx)
    call(ExitProcess)
