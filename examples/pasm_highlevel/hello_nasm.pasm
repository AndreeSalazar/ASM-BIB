# PASM Hello World — NASM target (Windows x64)
# Same Python-like syntax, outputs NASM Intel assembly

@arch('x86_64')
@format('win64')

@section('.text')
@export
def main():
    print("Hello, World!\n")
    exit(0)
