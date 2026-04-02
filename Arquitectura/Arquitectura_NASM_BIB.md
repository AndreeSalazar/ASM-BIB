# ASM-BIB — Arquitectura v2.0 💀🦈
> Eddi Andreé Salazar Matos | Lima, Perú 🇵🇪 | Techne v2.0
> Objetivo: Abstraer x86 ASM con sintaxis Python-like → NASM o MASM

---

## 1. Scope — Solo x86, Solo NASM + MASM

| Assembler | Sintaxis | OS | Status |
|---|---|---|---|
| **NASM** | Intel | Win/Linux/Mac | ✅ Target principal |
| **MASM** | Intel | Solo Windows | ✅ Canon completo |

### Arquitecturas soportadas
| Arch | Registros | Status |
|------|-----------|--------|
| x86-64 | rax..r15, xmm0..15, ymm0..15, zmm0..31 | ✅ Completo |
| x86-32 | eax..ebp, r8d..r15d | ✅ Completo |
| x86-16 | ax..bp (bootloader) | ✅ Completo |

---

## 2. Pipeline

```
.pasm source → Lexer → Parser → Program IR → Emitter → .asm (NASM | MASM)
                                                ↓
                                         nasm/ml64 → .obj → linker → .exe
```

---

## 3. Diferencias NASM vs MASM que ASM-BIB abstrae

### Secciones
```
NASM:    section .text / section .data / section .bss
MASM:    .code / .data / .data?

ASM-BIB:
  @section('.text')   → genera el correcto para cada target
  @section('.data')   → ídem
  @section('.bss')    → ídem
```

### Data
```
NASM:  db 0x41    dw 0x1234    dd 0xDEADBEEF
MASM:  BYTE 41h   WORD 1234h   DWORD 0DEADBEEFh

ASM-BIB:
  byte(0x41)   word(0x1234)   dword(0xDEADBEEF)
```

### Exportar/Importar
```
NASM:  global main / extern printf
MASM:  PUBLIC main / EXTERNDEF printf:PROC / INCLUDELIB msvcrt.lib

ASM-BIB:
  @export          → genera global/PUBLIC automáticamente
  call(printf)     → MASM auto-genera EXTERNDEF + INCLUDELIB
```

---

## 4. Sintaxis Python-like

```python
@arch('x86_64')
@format('pe')

@section('.text')
@export
def main():
    push(rbp)
    mov(rbp, rsp)
    sub(rsp, 32)
    lea(rcx, msg)
    call(printf)
    xor(eax, eax)
    leave()
    ret()

@section('.data')
msg = string("Hello from ASM-BIB!\n")
```

---

## 5. Instrucciones x86

```
Movimiento:     mov, movzx, movsx, lea, xchg, push, pop
Aritmética:     add, sub, mul, imul, div, idiv, inc, dec, neg
Lógica:         and, or, xor, not, shl, shr, sar, rol, ror
Comparación:    cmp, test
Saltos:         jmp, je, jne, jl, jle, jg, jge, jb, jbe, ja, jae
Llamadas:       call, ret, leave
Strings:        rep movsb, rep stosb, scasb
Sistema:        syscall, int, hlt, cli, sti, nop, cpuid, iretq
SSE:            movaps, movups, addps, mulps, xorps
AVX:            vmovaps, vaddps, vmulps
```

---

## 6. Estructura del Proyecto

```
ASM-BIB/
├── src/
│   ├── main.rs                   ← CLI: asm-bib archivo.pasm
│   ├── frontend/
│   │   ├── lexer.rs              ← tokeniza sintaxis Python-like
│   │   ├── parser.rs             ← parse → Program IR
│   │   └── ast.rs                ← nodos AST
│   ├── ir/
│   │   ├── instruction.rs        ← Opcode + Operand + Instruction
│   │   ├── register.rs           ← Registros x86 + Arch + Size
│   │   └── section.rs            ← Section, Function, DataDef, Program
│   ├── emitters/
│   │   ├── mod.rs                ← OutputFormat (Nasm | Masm)
│   │   ├── nasm.rs               ← exporta sintaxis NASM Intel
│   │   └── masm.rs               ← exporta sintaxis MASM Microsoft
│   ├── targets/
│   │   └── x86_64/
│   │       ├── mod.rs            ← X86_64Encoder
│   │       ├── instructions.rs   ← lista de instrucciones válidas
│   │       └── registers.rs      ← lista de registros válidos
│   └── macros/
│       └── stdlib.rs             ← prologue/epilogue/syscall
├── examples/
│   └── masm/                     ← 14 ejemplos MASM completos
├── tests/
│   ├── masm_fixtures.rs          ← tests de fixture MASM
│   └── fixtures/                 ← .pasm + .expected.asm
├── Cargo.toml
└── README.md
```

---

## 7. CLI

```bash
asm-bib hello.pasm --nasm -o hello.asm     # NASM Intel
asm-bib hello.pasm --masm -o hello.asm     # MASM Microsoft
asm-bib hello.pasm --step                  # debug pipeline
```

---

## 8. Frontend futuro — Python + C → .pasm → ASM

El frontend Python+C puede reconstruir el .pasm:
1. Escribes en Python+C (alto nivel)
2. Se genera .pasm automáticamente
3. ASM-BIB compila .pasm → .asm (NASM o MASM)
4. nasm/ml64 ensambla → .obj
5. Linker → .exe / .dll

---

> *"x86 ASM sin dolor. NASM + MASM. Un solo lenguaje. 💀🦈"*
