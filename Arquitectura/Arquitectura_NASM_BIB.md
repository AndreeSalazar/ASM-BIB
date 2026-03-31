# NASM-BIB — Arquitectura Global de ASM 💀🦈
> Eddi Andreé Salazar Matos | Lima, Perú 🇵🇪 | Marzo 2026 | Techne v1.0
> Objetivo: Abstraer TODOS los ASM globales con sintaxis Python-like

---

## 1. Todos los ASM y sus Compiladores — Mapa Global

### x86 / x86-64 (el más importante)
| Assembler | Sintaxis | OS | Dueño | Relevancia |
|---|---|---|---|---|
| **NASM** | Intel | Win/Linux/Mac | Open Source BSD | 🔴 Alta |
| **MASM** | Intel | Solo Windows | Microsoft | 🔴 Alta |
| **GAS** | AT&T (`movl %eax, %ebx`) | Linux/Mac | GNU | 🔴 Alta |
| **FASM** | Intel ultra-minimal | Win/Linux | Tomasz Grysztar | 🟡 Media |
| **YASM** | Intel/AT&T | Win/Linux | Open Source | 🟡 Media |
| **TASM** | Intel | DOS/Windows | Borland (legacy) | 🟢 Baja |
| **JWASM** | Intel (MASM-compat) | Win/Linux | Open Source | 🟢 Baja |

### ARM (móviles, Raspberry Pi, Apple Silicon)
| Assembler | Sintaxis | OS | Dueño | Relevancia |
|---|---|---|---|---|
| **armasm** | ARM nativo | Win/Linux | ARM Ltd | 🔴 Alta |
| **GAS ARM** | AT&T ARM | Linux | GNU | 🔴 Alta |
| **LLVM MC** | ARM LLVM | Win/Linux/Mac | LLVM | 🟡 Media |
| **Keil** | ARM | Embedded | ARM/Keil | 🟡 Media |

### RISC-V (el futuro open hardware)
| Assembler | Sintaxis | OS | Dueño | Relevancia |
|---|---|---|---|---|
| **GAS RISC-V** | AT&T RISC-V | Linux | GNU | 🔴 Alta |
| **LLVM MC** | RISC-V | Win/Linux | LLVM | 🟡 Media |
| **Spike** | RISC-V | Linux | UC Berkeley | 🟢 Baja |

### MIPS (routers, embedded, PlayStation)
| Assembler | Sintaxis | OS | Dueño | Relevancia |
|---|---|---|---|---|
| **GAS MIPS** | AT&T MIPS | Linux | GNU | 🟡 Media |
| **LLVM MC** | MIPS | Linux | LLVM | 🟢 Baja |

### PowerPC (consolas, servidores IBM)
| Assembler | Sintaxis | OS | Dueño | Relevancia |
|---|---|---|---|---|
| **GAS PPC** | AT&T PPC | Linux | GNU | 🟡 Media |
| **LLVM MC** | PPC | Linux | LLVM | 🟢 Baja |

### Otros relevantes
| Assembler | Arquitectura | Relevancia |
|---|---|---|
| **avr-as** | AVR (Arduino) | 🟡 Media |
| **sdas** | Z80 (retro, embebido) | 🟢 Baja |
| **ca65** | 6502 (retro, NES) | 🟢 Baja |
| **spasm** | SPIR-V (GPU) | 🟡 Media |

---

## 2. Las Diferencias Clave que NASM-BIB debe abstraer

### Intel vs AT&T — el problema principal
```
# La misma instrucción en dos mundos:

Intel (NASM/MASM/FASM):
  mov rax, rbx        ← destino primero, fuente después

AT&T (GAS):
  movq %rbx, %rax    ← fuente primero, destino después
                      ← % en registros
                      ← sufijo de tamaño (q=64, l=32, w=16, b=8)

NASM-BIB Python-like (tuyo):
  mov(rax, rbx)       ← mismo para todos, sin confusión
```

### Diferencias de secciones
```
NASM:    section .text / section .data / section .bss
MASM:    .code / .data / .data?
GAS:     .text / .data / .bss
FASM:    section '.text' code / section '.data' data

NASM-BIB:
  @text   → genera el correcto para cada target
  @data   → ídem
  @bss    → ídem
```

### Diferencias de directivas
```
# Definir bytes:
NASM:  db 0x41, 0x42    dw 0x1234    dd 0xDEADBEEF
MASM:  BYTE 0x41        WORD 0x1234  DWORD 0xDEAD
GAS:   .byte 0x41       .word 0x1234 .long 0xDEAD
FASM:  db 0x41          dw 0x1234    dd 0xDEAD

NASM-BIB:
  byte(0x41)   word(0x1234)   dword(0xDEAD)   qword(0xDEADBEEF)
```

### Diferencias de macros
```
NASM:   %macro nombre params ... %endmacro
MASM:   nombre MACRO params ... ENDM
GAS:    .macro nombre params ... .endm
FASM:   macro nombre params { ... }

NASM-BIB:
  @macro
  def nombre(params):
      ...  # Python puro
```

---

## 3. Arquitectura NASM-BIB — Capas

```
Tu código Python-like
        ↓
┌─────────────────────────────────────────┐
│  NASM-BIB Frontend                      │
│  Lexer → Parser → ASM-IR               │
│  (sintaxis Python, keywords ASM)        │
├─────────────────────────────────────────┤
│  ASM-IR (Instruction Representation)   │
│  Instrucción normalizada               │
│  { op: Mov, dst: RAX, src: RBX,        │
│    size: Q64, arch: X86_64 }           │
├─────────────────────────────────────────┤
│  Target Selector                        │
│  x86_64 / ARM64 / RISC-V / MIPS / PPC  │
├───────────┬─────────┬───────────────────┤
│ Emitter   │ Emitter │ Emitter           │
│ NASM .asm │ GAS .s  │ MASM .asm         │
│ FASM .asm │ armasm  │ Binario directo   │
└───────────┴─────────┴───────────────────┘
        ↓               ↓              ↓
   .asm exportado   .s exportado   PE/ELF directo
   (para NASM)      (para GAS)     (sin intermediario)
```

---

## 4. Sintaxis Python-like — Diseño Completo

### Estructura básica
```python
# NASM-BIB — sintaxis Python-like

@arch('x86_64')           # target architecture
@format('pe')             # pe / elf / flat / nasm / gas / masm

@section('.text')
@export                   # función exportable
def main():
    # Prólogo estándar
    push(rbp)
    mov(rbp, rsp)
    sub(rsp, 32)          # shadow space Windows

    # Llamar printf
    lea(rcx, msg)
    call(printf)

    # Epílogo
    xor(eax, eax)
    leave()
    ret()

@section('.data')
msg = string("Hello from NASM-BIB!\n")
```

### Registros disponibles por arquitectura
```python
# x86-64 — todos reconocidos automáticamente
# 64-bit: rax rbx rcx rdx rsi rdi rsp rbp r8..r15
# 32-bit: eax ebx ecx edx esi edi esp ebp r8d..r15d
# 16-bit: ax bx cx dx si di sp bp
#  8-bit: al ah bl bh cl ch dl dh
# SSE:    xmm0..xmm15
# AVX:    ymm0..ymm15
# AVX-512:zmm0..zmm31

# ARM64
# x0..x30, sp, xzr (zero register)
# w0..w30 (32-bit versions)
# v0..v31 (SIMD/FP)

# RISC-V
# x0(zero) x1(ra) x2(sp) x3(gp) ... x31
# f0..f31 (floating point)
```

### Control de flujo
```python
@section('.text')
def ejemplo():
    mov(rax, 0)

    # Loop
    @label('loop_start')
    inc(rax)
    cmp(rax, 10)
    jl('loop_start')      # jump if less

    # If/else via jumps
    cmp(rbx, 0)
    je('es_cero')
    mov(rcx, 1)
    jmp('fin')

    @label('es_cero')
    mov(rcx, 0)

    @label('fin')
    ret()
```

### Macros Python-like
```python
# Definir macro
@macro
def prologue(stack_size=32):
    push(rbp)
    mov(rbp, rsp)
    sub(rsp, stack_size)

@macro
def epilogue():
    leave()
    ret()

# Usar macro
@section('.text')
def mi_funcion():
    prologue(64)
    # ... código ...
    epilogue()
```

### Datos
```python
@section('.data')
# Tipos básicos
num_byte  = byte(0xFF)
num_word  = word(0x1234)
num_dword = dword(0xDEADBEEF)
num_qword = qword(0x123456789ABCDEF0)
mensaje   = string("Hola\n")        # null-terminated
mensaje_w = wstring("Hola\n")       # wide string UTF-16

@section('.bss')
# Variables sin inicializar
buffer    = resb(256)   # reserva 256 bytes
contador  = resd(1)     # reserva 1 dword
```

---

## 5. Instrucciones — Cobertura por Arquitectura

### x86-64 — Categorías a implementar
```
Movimiento:     mov, movzx, movsx, lea, xchg, push, pop
Aritmética:     add, sub, mul, imul, div, idiv, inc, dec, neg
Lógica:         and, or, xor, not, shl, shr, sar, rol, ror
Comparación:    cmp, test
Saltos:         jmp, je, jne, jl, jle, jg, jge, jb, jbe, ja, jae
Llamadas:       call, ret, leave
Strings:        rep movsb, rep stosb, scasb
Sistema:        syscall, int, hlt, cli, sti, nop, cpuid
SSE:            movaps, movups, addps, mulps, xorps...
AVX:            vmovaps, vaddps, vmulps... (VEX prefix)
AVX-512:        zmm operations (futuro)
```

### ARM64 — Categorías
```
Movimiento:     mov, ldr, str, ldp, stp, adr, adrp
Aritmética:     add, sub, mul, madd, msub, sdiv, udiv
Lógica:         and, orr, eor, mvn, lsl, lsr, asr, ror
Comparación:    cmp, cmn, tst
Saltos:         b, bl, br, blr, ret, cbz, cbnz, tbz, tbnz
Condición:      b.eq, b.ne, b.lt, b.gt, b.le, b.ge
Sistema:        svc, mrs, msr, nop
SIMD:           fmov, fadd, fmul, fdiv, fcmp...
```

### RISC-V — Categorías
```
Base (RV64I):   lui, auipc, jal, jalr
                lb, lh, lw, ld, sb, sh, sw, sd
                add, sub, xor, or, and, sll, srl, sra
                beq, bne, blt, bge, bltu, bgeu
                addi, xori, ori, andi, slli, srli, srai
Sistema:        ecall, ebreak, fence
Extensiones:    M(mul/div), A(atomic), F(float), D(double), C(compressed)
```

---

## 6. Exportadores — Output por Target

```
NASM-BIB exporta a:

┌──────────────┬────────────────────────────────────────────┐
│ Target       │ Output                                     │
├──────────────┼────────────────────────────────────────────┤
│ --nasm       │ archivo.asm  (sintaxis NASM Intel)         │
│ --gas        │ archivo.s    (sintaxis GAS AT&T)           │
│ --masm       │ archivo.asm  (sintaxis MASM Microsoft)     │
│ --fasm       │ archivo.asm  (sintaxis FASM)               │
│ --armasm     │ archivo.s    (sintaxis ARM official)       │
│ --pe         │ archivo.exe/.dll (PE directo — sin pasar   │
│              │              por NASM externo)             │
│ --elf        │ archivo.elf/.so  (ELF directo)             │
│ --flat       │ archivo.bin  (binario plano — bootloader)  │
│ --po         │ archivo.po   (FastOS format)               │
└──────────────┴────────────────────────────────────────────┘
```

---

## 7. Integración con ADead-BIB

```
# Flujo combinado C/C++ + ASM

kernel.c   → ADead-BIB  → kernel.o   (IR)
boot.pasm  → NASM-BIB   → boot.o     (IR)
isr.pasm   → NASM-BIB   → isr.o      (IR)
           ↓
      LinkDead-BIB
           ↓
      FastOS.bin / kernel.po

# .pasm = Python ASM — extensión tuya
```

### Casos de uso en FastOS
```python
# boot.pasm — Stage 1 bootloader
@arch('x86_16')           # ← 16-bit real mode
@format('flat')
@org(0x7C00)

@section('.text')
def start():
    cli()
    xor(ax, ax)
    mov(ds, ax)
    mov(es, ax)
    mov(ss, ax)
    mov(sp, 0x7C00)
    sti()
    call('load_stage2')
    jmp(0x8000)           # saltar a stage 2

# Firma de boot sector
@section('.boot_sig')
boot_sig = word(0xAA55)
```

```python
# isr.pasm — Interrupt Service Routines
@arch('x86_64')
@format('elf')

@section('.text')
@naked                    # sin prólogo/epílogo automático
def isr_timer():
    push(rax)
    push(rbx)
    # ... handler ...
    pop(rbx)
    pop(rax)
    iretq()               # interrupt return 64-bit
```

---

## 8. Estructura del Proyecto NASM-BIB

```
NASM-BIB/                         ← proyecto Rust
├── src/
│   ├── main.rs                   ← CLI: nasm-bib archivo.pasm
│   ├── frontend/
│   │   ├── lexer.rs              ← tokeniza sintaxis Python-like
│   │   ├── parser.rs             ← parse → ASM-IR
│   │   └── ast.rs                ← nodos ASM-IR
│   ├── ir/
│   │   ├── instruction.rs        ← Instrucción normalizada
│   │   ├── register.rs           ← Registros por arquitectura
│   │   └── section.rs            ← Secciones
│   ├── targets/
│   │   ├── x86_64/
│   │   │   ├── encoder.rs        ← bytes x86-64
│   │   │   ├── registers.rs      ← rax, rbx, xmm0...
│   │   │   └── instructions.rs   ← encoding de cada instrucción
│   │   ├── arm64/
│   │   │   ├── encoder.rs
│   │   │   └── registers.rs
│   │   ├── riscv/
│   │   │   └── encoder.rs
│   │   └── mips/
│   │       └── encoder.rs
│   ├── emitters/
│   │   ├── nasm.rs               ← exporta sintaxis NASM
│   │   ├── gas.rs                ← exporta sintaxis GAS/AT&T
│   │   ├── masm.rs               ← exporta sintaxis MASM
│   │   ├── fasm.rs               ← exporta sintaxis FASM
│   │   ├── pe.rs                 ← PE directo (comparte con ADead-BIB)
│   │   ├── elf.rs                ← ELF directo
│   │   └── flat.rs               ← binario plano
│   └── macros/
│       └── stdlib.rs             ← macros built-in (prologue, epilogue...)
├── examples/
│   ├── hello_x86.pasm
│   ├── hello_arm.pasm
│   ├── bootloader.pasm
│   └── isr.pasm
├── Cargo.toml
└── README.md
```

---

## 9. CLI NASM-BIB

```bash
# Compilar directo a binario
nasm-bib boot.pasm --flat --arch x86_16 --org 0x7C00 -o boot.bin
nasm-bib kernel.pasm --elf --arch x86_64 -o kernel.o
nasm-bib shader.pasm --spirv -o shader.spv

# Exportar a ASM clásico
nasm-bib mi_func.pasm --nasm -o mi_func.asm   # para NASM
nasm-bib mi_func.pasm --gas  -o mi_func.s     # para GAS
nasm-bib mi_func.pasm --masm -o mi_func.asm   # para MASM

# Multi-arch desde mismo source
nasm-bib mi_func.pasm --arch x86_64 --pe  -o mi_func_win.dll
nasm-bib mi_func.pasm --arch arm64  --elf -o mi_func_arm.so

# Step mode (hereda de ADead-BIB)
nasm-bib --step mi_func.pasm

# Integrado con ADead-BIB
adB link kernel.o boot.o isr.o -o FastOS.bin
```

---

## 10. Roadmap NASM-BIB

| Fase | Qué | Prioridad |
|---|---|---|
| **v1.0** | x86-64 completo + PE/ELF directo + sintaxis Python-like base | 🔴 |
| **v1.1** | Exportadores NASM + GAS + MASM + FASM | 🔴 |
| **v1.2** | ARM64 encoder + exportador armasm | 🟡 |
| **v1.3** | RISC-V encoder | 🟡 |
| **v2.0** | Integración LinkDead-BIB → FastOS pipeline completo | 🔴 |
| **v2.1** | MIPS + PPC + AVR | 🟢 |
| **v3.0** | SPIR-V target (GPU ASM) | 🟢 |

---

## 11. Impacto Final

```
Sin NASM-BIB:
  → aprender sintaxis Intel vs AT&T
  → diferente tool por cada OS/arch
  → imposible para 99% de developers

Con NASM-BIB:
  → sintaxis Python que todos entienden
  → mismo código → x86, ARM, RISC-V, MIPS
  → exporta a NASM/GAS/MASM/FASM si quieren
  → FastOS lo usa como ASM nativo
  → ADead-BIB + NASM-BIB = stack completo

ADead-BIB stack final:
  C/C++/Python/JS → ADead-BIB  ┐
  Python ASM      → NASM-BIB   ├→ LinkDead-BIB → FastOS
  GPU shaders     → SPIR-V     ┘
```

> *"ASM sin dolor. Todos los targets. Un solo lenguaje. 💀🦈"*
