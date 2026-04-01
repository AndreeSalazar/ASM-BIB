@arch('x86_32')
@format('pe32')

@section('.data')
msg = string("Hello 32-bit!")

@section('.text')
@export
def main():
    push(0)
    lea(eax, msg)
    push(eax)
    push(0)
    call(ExitProcess)
