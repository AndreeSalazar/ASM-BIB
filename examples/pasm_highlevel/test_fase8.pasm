# ASM-BIB Fase 8 Test — COFF Quality + DLL + API calls
# Tests: Real UNWIND_INFO, section alignment, DLL imports work correctly

@arch('x86_64')
@format('win64')
@includelib('kernel32.lib')
@includelib('user32.lib')

@section('.data')
msg_title = db("ASM-BIB Test")
msg_text = db("Fase 7+8 OK! COFF Production Quality.")

@section('.text')

@export
def main():
    prologue(48)
    
    # Call MessageBoxA(0, text, title, 0)
    xor(ecx, ecx)           # hWnd = NULL
    lea(rdx, msg_text)       # lpText
    lea(r8, msg_title)       # lpCaption
    xor(r9d, r9d)            # uType = MB_OK (0)
    call(MessageBoxA)
    
    # ExitProcess(0)
    xor(ecx, ecx)
    call(ExitProcess)
