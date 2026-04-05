# ASM-BIB v2.0 — Assembler Independiente MASM/NASM

> Eddi Andreé Salazar Matos | Lima, Peru
>
> x86-64 Assembler con sintaxis Python-like. Reemplazo completo de ml64.exe.

![Banner](https://img.shields.io/badge/ASM--BIB-v2.0-red?style=for-the-badge&logo=rust)

---

## Arquitectura Dual: MASM + NASM

| Target   | Ring Level | Uso                                      |
|----------|-----------|------------------------------------------|
| **MASM** | Ring 1-3  | Drivers (Ring 1-2), Userland (Ring 3)    |
| **NASM** | Ring 0    | Kernel, bootloaders, bare-metal          |

- **MASM path**: `.pasm` -> IR -> Encoder -> COFF `.obj` (reemplaza `ml64.exe`)
- **NASM path**: `.pasm` -> IR -> NASM `.asm` (para Ring 0 con `nasm`)
- **Bridge**: `.obj` exporta metadata inteligente para **ADead-BIB** (compilador C/C++ -> machine code)

---

## Pipeline v2.0

```text
.pasm source
    |
    v
  Lexer -> Parser -> IR (Intermediate Representation)
                        |
          +-------------+-------------+
          |             |             |
          v             v             v
    MASM Emitter   NASM Emitter   COFF Encoder
    (.asm Ring1-3)  (.asm Ring0)   (.obj directo)
                                      |
                    +-----------------+
                    |                 |
                    v                 v
              Internal PE       ADead-BIB Bridge
              Linker (.exe)     (C/C++ -> .exe)
```

---

## Instrucciones Soportadas (Encoder Binario)

### GPR Completo (Ring 1-3 MASM Standard)
- **Movement**: MOV, MOVZX, MOVSX, LEA, XCHG, PUSH, POP, ENTER, LEAVE
- **Arithmetic**: ADD, SUB, MUL, IMUL (1/2/3-op), DIV, IDIV, INC, DEC, NEG, ADC, SBB
- **Logic**: AND, OR, XOR, NOT, SHL, SHR, SAR, ROL, ROR, RCL, RCR
- **Compare**: CMP, TEST
- **Jumps**: JMP, JE/JNE, JL/JLE/JG/JGE, JB/JBE/JA/JAE, JS/JNS, JO/JNO, JP/JNP, JCXZ/JECXZ/JRCXZ
- **Loop**: LOOP, LOOPE, LOOPNE
- **Call**: CALL (label/reg/mem), RET, RET imm16, LEAVE
- **SETcc**: SETE/SETNE/SETL/SETLE/SETG/SETGE/SETB/SETBE/SETA/SETAE/SETS/SETNS
- **CMOVcc**: CMOVE/CMOVNE/CMOVL/CMOVLE/CMOVG/CMOVGE/CMOVB/CMOVBE/CMOVA/CMOVAE
- **String**: REP MOVSB/W/D/Q, REP STOSB/W/D/Q, REPE CMPSB/W/D, REPNE SCASB/W/D, LODS/SCAS/CMPS
- **Bit**: BT/BTS/BTR/BTC, BSF/BSR, POPCNT/LZCNT/TZCNT, BSWAP
- **Atomic**: XADD, CMPXCHG, CMPXCHG8B, CMPXCHG16B
- **System**: NOP, CPUID, RDTSC, RDTSCP, INT, SYSCALL, HLT, CLD, STD
- **Misc**: CQO, CDQ, CBW, CWD, CWDE, LAHF, SAHF, XLAT, PUSHF, POPF

### SSE/SSE2 Completo
- **Packed float**: MOVAPS/MOVUPS, ADD/SUB/MUL/DIV/MIN/MAX/SQRT/RSQRT/RCPPS, XORPS/ANDPS/ORPS
- **Scalar float**: MOVSS, ADD/SUB/MUL/DIV/SQRT/MIN/MAXSS, COMISS/UCOMISS
- **Packed double**: MOVAPD/MOVUPD, ADD/SUB/MUL/DIV/MIN/MAX/SQRTPD, XORPD/ANDPD/ORPD
- **Scalar double**: MOVSD, ADD/SUB/MUL/DIV/SQRTSD, COMISD/UCOMISD
- **Integer**: MOVDQA/MOVDQU, PADD/PSUB (B/W/D/Q), PMULLW/PMULLD/PMULUDQ, PAND/POR/PXOR
- **Compare**: PCMPEQ/PCMPGT (B/W/D), CMPPS/CMPSS/CMPPD/CMPSD (con imm8)
- **Shift**: PSLL/PSRL/PSRA (W/D/Q) — por imm8 y por registro XMM
- **Shuffle**: PSHUFD/PSHUFHW/PSHUFLW/PSHUFB, SHUFPS/SHUFPD, UNPCKLPS/UNPCKHPS
- **Unpack**: PUNPCKL/PUNPCKH (BW/WD/DQ)
- **Convert**: CVTSI2SS/SD, CVTSS2SI/SD, CVTTSS2SI/SD, CVTSS2SD, CVTSD2SS
- **Data move**: MOVD, MOVQ

### AVX/AVX2 Completo (VEX 3-operand)
- **Packed float**: VADDPS, VSUBPS, VMULPS, VDIVPS, VXORPS, VANDPS, VORPS, VMINPS, VMAXPS, VSQRTPS
- **Scalar float**: VADDSS, VSUBSS, VMULSS, VDIVSS, VSQRTSS
- **Packed double**: VADDPD, VSUBPD, VMULPD, VDIVPD, VXORPD
- **Scalar double**: VADDSD, VSUBSD, VMULSD, VDIVSD, VSQRTSD
- **Integer**: VPADD/VPSUB (B/W/D/Q), VPMULLW/VPMULLD, VPAND/VPOR/VPXOR
- **MOV**: VMOVAPS/UPS/APD/UPD/SS/SD/DQA/DQU (load + store)
- **Special**: VBROADCASTSS/SD, VPERM2F128, VINSERTF128, VEXTRACTF128, VCMPPS, VSHUFPS
- **Dot product**: VDPPS, VDPPD
- **FMA**: VFMADD132/213/231 (PS/SS/PD/SD) — 12 instrucciones

### Ring 0 (Solo via NASM)
- **Control Regs**: MOV CR0-CR4, MOV DR0-DR7
- **GDT/IDT**: LGDT, SGDT, LIDT, SIDT
- **Task/LDT**: LTR, STR, LLDT, SLDT
- **MSW**: LMSW, SMSW
- **System**: INVLPG, SWAPGS, WBINVD, INVD, CLTS, RDMSR, WRMSR
- **I/O**: IN, OUT (imm8 y DX)
- **Fences**: MFENCE, LFENCE, SFENCE

### Registros
- **GPR 64**: RAX-R15
- **GPR 32**: EAX-R15D
- **GPR 16**: AX-R15W
- **GPR 8**: AL/CL/DL/BL, AH/BH/CH/DH, SPL/BPL/SIL/DIL, R8B-R15B
- **Segment**: CS/DS/ES/FS/GS/SS
- **SSE**: XMM0-XMM15
- **AVX**: YMM0-YMM15
- **Control**: CR0/CR2/CR3/CR4
- **Debug**: DR0-DR3/DR6/DR7

---

## COFF .obj Export (Bridge para ADead-BIB)

El encoder genera `.obj` COFF compatibles con MSVC que incluyen:
- **Symbol Table**: Todos los simbolos exportados/externos con tipos correctos
- **Relocations**: REL32 para llamadas, ADDR32NB para datos
- **.pdata/.xdata**: SEH unwind info (PROC FRAME)
- **.drectve**: INCLUDELIB/EXPORT autolink directives
- **Section alignment**: Configurable por seccion
- **Aux symbols**: Function/Section auxiliary records

ADead-BIB puede consumir estos `.obj` directamente para:
1. Resolver simbolos C/C++ contra funciones ASM
2. Linkear mixed C++ + ASM en un solo PE
3. Usar las tablas SEH de ASM-BIB sin reconstruir

---

## Sintaxis Python-like

```python
@arch('x86_64')
@format('win64')

@section('.data')
    msg = string("Hello from ASM-BIB!\n")

@section('.text')
@export
def main():
    prologue(32)
    print(msg)
    epilogue()
```

### Control Flow
- `@if(reg, ==, val) / @else / @endif`
- `@loop(reg, n) / @endloop`
- `@while(reg, <, val) / @endwhile`
- `@switch(reg) / @case(x) / @default / @endswitch`

### Calling Conventions
- `@stdcall` — Win32 (Ring 2-3)
- `@fastcall` — Win64 standard (Ring 1-3)
- `@naked` — Sin prologue/epilogue
- `@frame` — PROC FRAME con SEH unwind

### Structs
```python
@struct
class Float3:
    x = float32(1.0)
    y = float32(0.0)
    z = float32(0.0)
```

---

## Build

```bash
# Emitir MASM (.asm Ring 1-3)
cargo run -- hello.pasm --masm

# Emitir NASM (.asm Ring 0)
cargo run -- kernel.pasm --nasm

# Generar .obj COFF directo (reemplaza ml64.exe)
cargo run -- hello.pasm --native --obj

# Build completo (.exe con linker interno)
cargo run -- hello.pasm --native --link

# Build con ml64.exe externo
cargo run -- hello.pasm --build --masm
```

---

## Roadmap

- [x] **v1.0** — Lexer + Parser + IR + MASM/NASM emitters
- [x] **v1.5** — COFF encoder (reemplaza ml64.exe), SEH/.pdata/.xdata
- [x] **v2.0** — Internal PE linker, Import/Export tables, complete Ring 1-3 instruction set
- [ ] **v2.1** — ADead-BIB bridge protocol (metadata enrichment en .obj)
- [ ] **v3.0** — Flat binary output (`--bin`) para bootloaders, `@org`, `@bits(16)`
