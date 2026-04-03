# ASM-BIB DirectX 12 COM & MSVC Linker Native Test

@arch('x86_64')
@format('win64')
@includelib('kernel32.lib')
@includelib('user32.lib')

@struct
class ID3D12DeviceVtbl:
    QueryInterface = qword(0)
    AddRef = qword(8)
    Release = qword(16)
    GetNodeCount = qword(24)
    CreateCommandQueue = qword(32)

@section('.text')

@export
def main():
    prologue(40)
    
    # 1. Indirect Call Test (VTable COM Simulation)
    # R10 will be our fake "this" pointer
    # R11 will be our fake "vtable"
    mov(r10, 0x1000)
    mov(r11, 0x2000)
    
    # Normally: mov rcx, [r10]; mov rax, [rcx]; call qword ptr [rax + 32]
    # For now let's just assemble the instruction to see if SIB encodes it natively
    
    # Direct AST memory operand for: call qword ptr [r11 + 32]
    # Which represents calling CreateCommandQueue inside ID3D12Device!
    call(qword([r11 + 32]))
    
    xor(ecx, ecx)
    call(ExitProcess)
