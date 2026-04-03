# ASM-BIB Fase 7 Test — All critical encoding gaps
# Tests: MOV mem/imm, ADD/SUB/CMP reg/mem, IMUL 3-op, TEST reg/imm, SSE stores

@arch('x86_64')
@format('win64')
@includelib('kernel32.lib')

@section('.text')

@export
def main():
    prologue(128)
    
    # === Test 1: MOV mem, imm ===
    mov(qword([rsp + 32]), 0)
    mov(qword([rsp + 40]), 42)
    mov(qword([rsp + 48]), 0xFF)
    
    # === Test 2: ADD/SUB/CMP reg, mem ===
    mov(qword([rsp + 56]), 100)
    mov(rax, 50)
    add(rax, qword([rsp + 56]))
    # rax should be 150

    # === Test 3: ADD/SUB mem, reg ===
    mov(rcx, 10)
    add(qword([rsp + 56]), rcx)
    # [rsp+56] = 110

    # === Test 4: SUB/CMP mem, imm ===
    sub(qword([rsp + 56]), 10)
    # [rsp+56] = 100

    # === Test 5: CMP reg, mem ===
    mov(rax, 100)
    cmp(rax, qword([rsp + 56]))  
    # Should set ZF (equal)
    
    # === Test 6: IMUL 3-operand ===
    mov(rbx, 7)
    imul(rax, rbx, 6)
    # rax = 42
    
    # === Test 7: IMUL reg, imm (2-op shorthand) ===
    mov(rcx, 10)
    imul(rcx, rcx, 5)
    # rcx = 50
    
    # === Test 8: TEST reg, imm ===
    mov(rax, 0xFF)
    test(rax, 0x80)
    # Should NOT set ZF (0xFF & 0x80 = 0x80 != 0)
    
    test(eax, 0xFF)
    # Test with 32-bit
    
    # === Test 9: TEST al, imm8 ===
    mov(al, 0x42)
    test(al, 0x02)
    
    # === Test 10: INC/DEC ===
    mov(rcx, 99)
    inc(rcx)
    # rcx = 100
    dec(rcx)
    # rcx = 99
    
    # === Test 11: NEG/NOT ===
    mov(rdx, 42)
    neg(rdx)
    # rdx = -42
    not(rdx)
    # rdx = 41
    
    # === Test 12: SHL/SHR/SAR ===
    mov(rax, 1)
    shl(rax, 4)
    # rax = 16
    shr(rax, 2)
    # rax = 4
    
    # === Test 13: AND/OR reg, imm ===
    mov(rax, 0xFF)
    and(rax, 0x0F)
    # rax = 0x0F
    or(rax, 0xF0)
    # rax = 0xFF
    
    # === Test 14: XCHG ===
    mov(rax, 1)
    mov(rbx, 2)
    xchg(rax, rbx)
    # rax=2, rbx=1
    
    # === Test 15: NOP / CQO / CDQ ===
    nop()
    mov(rax, -1)
    cqo()
    # rdx = -1 (sign extended)
    
    # === Exit with code 0 (success) ===
    xor(ecx, ecx)
    call(ExitProcess)
