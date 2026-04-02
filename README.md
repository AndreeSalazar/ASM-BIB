# ASM-BIB 💀🦈

**x86 ASM abstraction — Escribe .pasm, exporta a NASM o MASM.**

> Python-like syntax → NASM / MASM → tu compilador ASM-BIB principal

## Pipeline

```
.pasm source → Lexer → Parser → Program IR → Emitter → .asm (NASM | MASM)
```

## Un mismo código → NASM o MASM

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

## Exportar

```bash
asm-bib hello.pasm --nasm -o hello.asm     # NASM Intel
asm-bib hello.pasm --masm -o hello.asm     # MASM Microsoft
```

## Arquitecturas x86

| Arch | Registros | Status |
|------|-----------|--------|
| x86-64 | rax..r15, xmm0..15, ymm, zmm | ✅ Completo |
| x86-32 | eax..ebp, r8d..r15d | ✅ Completo |
| x86-16 | ax..bp (bootloader) | ✅ Completo |

## Emitters

| Emitter | Directivas | Instrucciones | Data | Structs | PTR Size | Status |
|---------|-----------|---------------|------|---------|----------|--------|
| **NASM** | bits, org, section, global, extern | ✅ All x86 | db/dw/dd/dq/resX | via comment | N/A | ✅ Completo |
| **MASM** | .686p/.model/option, .code/.data/.const/.data? | ✅ All x86 | BYTE/WORD/DWORD/QWORD/REAL4/REAL8 | STRUCT/ENDS | BYTE PTR..ZMMWORD PTR | ✅ Canon completo |

## MASM — Canon completo ✅

- **Procesador**: `.8086` / `.686p` / ML64 implícito según `@arch`
- **Modelo de memoria**: `.model tiny/small/flat` según `@format`
- **Secciones**: `.code` / `.data` / `.data?` / `.const` / `SEGMENT`
- **Procedimientos**: `PROC` con parámetros tipados + `ENDP`
- **Variables locales**: `LOCAL var:TYPE`
- **Calificadores de tamaño**: `BYTE PTR` → `ZMMWORD PTR`
- **Structs**: `STRUCT` / `ENDS` con campos tipados
- **Enums**: Serie de `EQU` con prefijo `EnumName_Variant`
- **Constantes**: `name EQU value`
- **Hexadecimal**: Sufijo `h` con prefijo `0` si empieza en letra
- **Strings**: Escape explícito `"text", 0Ah, 0Dh, 0`
- **Extern**: `EXTERNDEF name:PROC` / `INCLUDELIB lib.lib`
- **AUTO INCLUDELIB**: Detecta `call(ExitProcess)` → `INCLUDELIB kernel32.lib`
- **Entry point**: `END main` / `END _start`

## NASM — Completo ✅

- **Bits**: `bits 16/32/64` según `@arch`
- **Origin**: `org 0x7C00` para bootloaders
- **Secciones**: `section .text` / `.data` / `.bss` / `.rodata`
- **Global/Extern**: `global main` + `extern printf`
- **Hexadecimal**: Formato `0x` estándar
- **Strings**: Escape con bytes: `"text", 10, 0`
- **Labels locales**: `.label:` dentro de funciones

## Ejemplos MASM

14 ejemplos completos en `examples/masm/`:

| # | Ejemplo | Target |
|---|---------|--------|
| 01 | hello_console | Win64 WriteFile + ExitProcess |
| 02 | arithmetic | add, sub, imul, idiv, inc, dec, neg |
| 03 | control_flow | cmp, je, jne, jl, jg, jmp, loops |
| 04 | procedures | PROC, params, LOCAL, factorial recursivo |
| 05 | strings | rep movsb, scasb, string data |
| 06 | memory | BYTE/DWORD PTR, base+idx*scale, arrays |
| 07 | bitwise | and, or, xor, not, shl, shr, sar, rol, ror |
| 08 | structs | STRUCT instances, field access |
| 09 | sse_avx | movaps, addps, mulps, vmovaps, vaddps |
| 10 | win64_api | GetStdHandle, WriteConsoleA, shadow space |
| 11 | stack_frames | Manual stack frame + Win64 ABI |
| 12 | floating_point | SSE scalar, REAL4/REAL8 |
| 13 | macros_equ | Constants (EQU), page alignment |
| 14 | win32_msgbox | MessageBoxA (x86-32) |

## Estructura del proyecto

| Componente | Archivo | Función |
|-----------|---------|---------|
| Lexer | `src/frontend/lexer.rs` | Tokeniza .pasm (Python+C hybrid) |
| Parser | `src/frontend/parser.rs` | Tokens → Program IR |
| AST | `src/frontend/ast.rs` | Nodos AST del lenguaje .pasm |
| IR | `src/ir/` | Instruction, Register, Section, DataDef, StructDef, EnumDef |
| Emitters | `src/emitters/` | NASM, MASM |
| Targets | `src/targets/x86_64/` | x86 validation, registers, instructions |
| Macros | `src/macros/stdlib.rs` | prologue/epilogue/syscall generators |

## Build

```bash
cargo build --release
```

## Eddi Andreé Salazar Matos — Lima, Perú 🇵🇪 — Techne v2.0
