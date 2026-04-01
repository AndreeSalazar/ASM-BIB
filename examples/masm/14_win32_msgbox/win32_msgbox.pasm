# MASM — Win32 MessageBox (x86-32)
@arch('x86_32')
@format('pe32')

@section('.data')
msg_text = string("Hello from ASM-BIB MASM (32-bit)!")
msg_title = string("ASM-BIB")

@section('.text')
@export
def main():
    # MessageBoxA(0, text, title, MB_OK=0)
    push(0)
    lea(eax, msg_title)
    push(eax)
    lea(eax, msg_text)
    push(eax)
    push(0)
    call(MessageBoxA)
    
    # ExitProcess(0)
    push(0)
    call(ExitProcess)
