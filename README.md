# ASM-BIB 💀🦈

**Universal ASM abstraction — Escribe una vez, exporta a TODOS los dialectos.**

> Python-like syntax → NASM / GAS / MASM / FASM / Flat binary

## Un mismo código → todos los targets

```python
@arch('x86_64')
@format('elf')

@section('.text')
@export
def main():
    push(rbp)
    mov(rbp, rsp)
    lea(rcx, msg)
    call(printf)
    xor(eax, eax)
    ret()

@section('.data')
msg = string("Hello from ASM-BIB!\n")
```

## Exportar a cualquier dialecto

```bash
asm-bib hello.pasm --nasm -o hello.asm     # NASM Intel
asm-bib hello.pasm --gas  -o hello.s       # GAS AT&T
asm-bib hello.pasm --masm -o hello.asm     # MASM Microsoft
asm-bib hello.pasm --fasm -o hello.asm     # FASM
asm-bib boot.pasm  --flat -o boot.bin      # Flat binary
```

## Arquitecturas soportadas

| Arch | Registros | Status |
|------|-----------|--------|
| x86-64 | rax..r15, xmm0..15, ymm, zmm | ✅ Completo |
| x86-32 | eax..ebp, r8d..r15d | ✅ Completo |
| x86-16 | ax..bp (bootloader) | ✅ Completo |
| ARM64 | x0..x30, w0..w30, v0..v31 | ✅ Base |
| RISC-V | x0..x31, f0..f31 | ✅ Base |
| MIPS | $0..$31 | ✅ Base |

## Emitters — Estado de avance

| Emitter | Directivas | Instrucciones | Data | Structs | PTR Size | Extras | Status |
|---------|-----------|---------------|------|---------|----------|--------|--------|
| **NASM** | bits, org, section, global | ✅ All x86 | db/dw/dd/dq/resX | via comment | N/A | hex 0x format | ✅ Completo |
| **GAS** | .text/.data/.bss, .globl | ✅ All x86 (AT&T suffix) | .byte/.word/.long/.quad/.asciz | via labels | N/A | AT&T reversed operands | ✅ Completo |
| **MASM** | .686p/.model/option, .code/.data/.const/.data? | ✅ All x86 | BYTE/WORD/DWORD/QWORD/REAL4/REAL8 | STRUCT/ENDS | BYTE PTR..ZMMWORD PTR | PROC+params, LOCAL, EQU, EXTERNDEF, INCLUDELIB, END | ✅ **Canon completo** |
| **FASM** | format, section attrs, public | ✅ All x86 | db/dw/dd/dq/rb/rw/rd/rq | via comment | N/A | format directive | ✅ Completo |
| **Flat** | BITS, ORG, global | ✅ All x86 | db/dw/dd/dq/resX | via comment | N/A | bootloader binary | ✅ Completo |

## MASM — Canon completo ✅

El emitter MASM ahora cubre **todo el estándar**:

- **Procesador**: `.8086` / `.686p` / ML64 implícito según `@arch`
- **Modelo de memoria**: `.model tiny/small/flat` según `@format`
- **Secciones**: `.code` / `.data` / `.data?` / `.const` / `SEGMENT`
- **Procedimientos**: `PROC` con parámetros tipados + `ENDP`
- **Variables locales**: `LOCAL var:TYPE`
- **Calificadores de tamaño**: `BYTE PTR`, `WORD PTR`, `DWORD PTR`, `QWORD PTR`, `XMMWORD PTR`, `YMMWORD PTR`, `ZMMWORD PTR`
- **Structs**: `STRUCT` / `ENDS` con campos tipados
- **Enums**: Serie de `EQU` con prefijo `EnumName_Variant`
- **Constantes**: `name EQU value`
- **Hexadecimal**: Sufijo `h` con prefijo `0` si empieza en letra (canon MASM)
- **Strings**: Escape explícito `"text", 0Ah, 0Dh, 0`
- **Extern**: `EXTERNDEF name:PROC` / `INCLUDELIB lib.lib`
- **PUBLIC**: Funciones exportadas
- **Alineación**: `ALIGN n`
- **Entry point**: `END main` / `END _start`

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

## IR — Representación intermedia

```
.pasm source → Lexer → Parser → Program IR → Emitter → .asm output
```

| Componente | Archivo | Función |
|-----------|---------|---------|
| Lexer | `src/frontend/lexer.rs` | Tokeniza .pasm (Python+C hybrid) |
| Parser | `src/frontend/parser.rs` | Tokens → Program IR |
| IR | `src/ir/` | Instruction, Register, Section, DataDef, StructDef, EnumDef |
| Emitters | `src/emitters/` | NASM, GAS, MASM, FASM, Flat |
| Targets | `src/targets/` | x86_64, ARM64, RISC-V, MIPS validation |
| Macros | `src/macros/stdlib.rs` | prologue/epilogue/syscall generators |

## Build

```bash
cargo build --release
```

## Eddi Andreé Salazar Matos — Lima, Perú 🇵🇪 — Techne v1.0
