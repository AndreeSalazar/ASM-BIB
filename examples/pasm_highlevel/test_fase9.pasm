@section('.rdata')
my_message = string("Hola FASE 9\n")

@section('.text')
@export
def main():
    # Test FASE 9: LOCAL variables (automatically sets up prologue/epilogue)
    local("loc1", "DWORD")
    local("loc2", "QWORD")
    
    # Test FASE 9: OFFSET pseudo-directive
    mov(rcx, offset(my_message))
    
    # Test FASE 9: Size Disambiguation memory writes
    mov(dword(loc1), 42)
    mov(qword(loc2), 9999)
    # Read them back
    mov(eax, dword(loc1))
    
    # Exit cleanly!
    # Because 'local' generated `push rbp; mov rbp, rsp; sub rsp, 16`
    # We must properly `leave` and `ret`.
    leave()
    ret()
