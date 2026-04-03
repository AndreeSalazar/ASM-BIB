# PASM Hello World — MASM target (Windows x64)
# Just write print() like Python, get real MASM!

@arch('x86_64')
@format('win64')

@section('.text')
@export
def main():
    print("Hello, World!\n")
    exit(0)
