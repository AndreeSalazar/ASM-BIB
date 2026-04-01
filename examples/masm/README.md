# MASM Examples — ASM-BIB 💀🦈

Ejemplos completos de MASM generados por ASM-BIB.

## Índice

| # | Ejemplo | Descripción |
|---|---------|-------------|
| 01 | hello_console | Win64 console output (WriteFile + ExitProcess) |
| 02 | arithmetic | Operaciones aritméticas: add, sub, imul, idiv, inc, dec, neg |
| 03 | control_flow | Branches + loops: cmp, je, jne, jl, jg, jmp |
| 04 | procedures | PROC con parámetros, LOCAL, call convention Win64 |
| 05 | strings | Operaciones de string: rep movsb, scasb |
| 06 | memory | Modos de direccionamiento: directo, indirecto, base+idx*scale |
| 07 | bitwise | Operaciones bit a bit: and, or, xor, not, shl, shr, sar, rol, ror |
| 08 | structs | Struct definitions + instancias |
| 09 | sse_avx | SSE (movaps, addps, mulps) + AVX (vmovaps, vaddps) |
| 10 | win64_api | Win64 API: GetStdHandle, WriteConsoleA, ExitProcess |
| 11 | stack_frames | Stack frame manual + shadow space Win64 |
| 12 | floating_point | Floating point SSE scalar (REAL4 / REAL8) |
| 13 | macros_equ | Constantes (EQU) + equates |
| 14 | win32_msgbox | Win32 MessageBoxA (x86-32) |

## Uso

```bash
# Generar MASM desde .pasm
asm-bib examples/masm/01_hello_console/hello_console.pasm --masm -o hello.asm

# Ensamblar con ML64 (Win64)
ml64 hello.asm /link /subsystem:console kernel32.lib

# Ensamblar con ML (Win32)
ml /c /coff example.asm
link /subsystem:console example.obj kernel32.lib user32.lib
```

## Convenciones MASM emitidas

- `.686p` + `.model flat, stdcall` para Win32
- ML64 mode implícito para Win64
- `BYTE PTR`, `WORD PTR`, `DWORD PTR`, `QWORD PTR` en operandos de memoria
- Hexadecimal con sufijo `h` (ej: `0DEADh`)
- Strings: `"text", 0Ah, 0` (escape explícito)
- `PROC / ENDP` con parámetros tipados
- `LOCAL` para variables locales
- `PUBLIC` para funciones exportadas
- `EXTERNDEF` para símbolos externos
- `INCLUDELIB` para librerías
- Structs: `STRUCT / ENDS`
- Constantes: `EQU`
- `END main` con entry point
