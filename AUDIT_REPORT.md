# ASM-BIB — Auditoría Completa v3.0

> Fecha: 2026-04-26 | Actualizado: Antigravity AI

---

## 1. RESUMEN EJECUTIVO

**Estado general: ~85% completo para reemplazar ml64.exe, ~75% para OS propio Ring 0-3**

| Componente | Estado | Nota |
|-----------|--------|------|
| Lexer | ✅ 95% | Robusto, soporta Python+C syntax |
| Parser | ✅ 90% | ~1730 líneas, @directives, structs, control flow, SEH |
| IR | ✅ 95% | Completo: opcodes, operands, sections, structs, enums, R8W-R15W, MOVSXD, PREFETCH* |
| MASM Emitter | ✅ 95% | Genera MASM válido con PROC FRAME, LOCAL, SEH, TBYTE PTR |
| NASM Emitter | ⚠️ 70% | Funcional pero sin SEH, LOCAL, structs |
| x86_64 Encoder | ✅ 90% | Ring 0-3 completo, AVX/FMA, MOVSXD, PREFETCH*, CMOVcc/SETcc mem |
| COFF Generator | ✅ 85% | .pdata/.xdata SEH, aux symbols |
| PE Linker | ✅ 80% | Import/Export tables, .reloc, DLL support |
| Stdlib/Macros | ❌ 30% | Solo prologue/epilogue/syscall — no se usa en pipeline |
| Tests | ⚠️ 60% | 7 fixture tests + 4 ML64 tests, sin tests de encoder/linker |

---

## 2. LO QUE ESTÁ COMPLETO Y FUNCIONA

### 2A. Pipeline Principal
- `.pasm` → Lexer → Parser → IR → MASM/NASM `.asm` **funciona**
- `--build --masm` → ml64.exe → link.exe → `.exe` **funciona**
- `--native` → COFF `.obj` interno **funciona**
- `--link` → PE `.exe`/`.dll` directo **funciona (básico)**

### 2B. Instrucciones Codificadas en encoder.rs (~1940 líneas)
- **Movimiento**: MOV (reg↔reg, reg↔imm, reg↔mem, mem↔imm, label↔reg/imm), LEA, PUSH, POP, MOVZX, MOVSX, MOVSXD, XCHG, BSWAP, ENTER
- **Aritmética**: ADD, SUB, CMP, XOR, AND, OR, ADC, SBB (todas con reg/imm/mem combinaciones), INC, DEC, NEG, NOT, MUL, DIV, IDIV, IMUL (1/2/3 operandos)
- **Shifts**: SHL, SHR, SAR, ROL, ROR, RCL, RCR (imm/CL/1)
- **Flow**: JMP, JE/JNE + 14 conditional jumps, CALL (label/reg/mem), RET, LEAVE, LOOP/LOOPE/LOOPNE, JCXZ/JECXZ/JRCXZ
- **SETcc**: 12 variantes (reg + mem)
- **CMOVcc**: 12 variantes (reg,reg + reg,mem)
- **Bit**: BT/BTS/BTR/BTC, BSF/BSR, POPCNT/LZCNT/TZCNT
- **Atomic**: XADD, CMPXCHG, CMPXCHG8B, CMPXCHG16B, LOCK prefix
- **String**: REP MOVSB/W/D/Q, REP STOSB/W/D/Q, REPE CMPS, REPNE SCAS, LODS, CLD, STD
- **SSE**: MOVAPS/MOVUPS, ADD/SUB/MUL/DIV/MIN/MAX/SQRT/PS, MOVSS scalar, stores, SHUFPS, CMPPS, UNPCKLPS, UNPCKHPS
- **SSE2**: MOVAPD/MOVUPD packed double, MOVSD scalar double, integer ops, PSHUFD/PSHUFHW/PSHUFLW, PSHUFB, PMULUDQ, PUNPCK*, SHUFPD, CMPPD/CMPSD2
- **AVX**: Toda la familia VEX 3-operand (packed float/double/integer), MOV 2-operand, VBROADCAST, VPERM2F128, VINSERTF128, VEXTRACTF128, VZEROALL, VZEROUPPER
- **FMA**: VFMADD132/213/231 PS/SS/PD/SD (12 variantes completas)
- **Conversiones**: CVTSI2SS/SD, CVTSS2SI/SD, CVTSD2SS/SI, CVTTSS2SI, CVTTSD2SI
- **Cache**: PREFETCHT0, PREFETCHT1, PREFETCHT2, PREFETCHNTA
- **Fences**: MFENCE, LFENCE, SFENCE
- **Ring 0**: MOV CRn/DRn, LGDT/SGDT/LIDT/SIDT, LTR/STR/LLDT/SLDT, LMSW/SMSW, INVLPG, SWAPGS, WBINVD/INVD, CLTS, RDMSR/WRMSR, IN/OUT
- **Sistema**: SYSCALL, INT, HLT, CLI, STI, NOP, CPUID, IRETQ, RDTSC, RDTSCP
- **Misc**: CQO, CDQ, CBW, CWD, CWDE, LAHF, SAHF, XLAT, PUSHF, POPF

### 2C. IR Registers Completo
- **64-bit**: RAX-R15
- **32-bit**: EAX-R15D
- **16-bit**: AX-BP + R8W-R15W (✅ completo)
- **8-bit**: AL-DH + SPL/BPL/SIL/DIL + R8B-R15B
- **Segment**: CS/DS/ES/FS/GS/SS
- **Control**: CR0, CR2, CR3, CR4
- **Debug**: DR0-DR3, DR6, DR7
- **SSE/AVX**: XMM0-15, YMM0-15, ZMM0-31

### 2D. COFF Object File
- Headers, sections, symbols, relocations, string table ✅
- .drectve section para INCLUDELIB/EXPORT ✅
- .pdata/.xdata con UNWIND_INFO real (PUSH_NONVOL + SET_FPREG) ✅
- Aux symbols para sections y funciones ✅

### 2E. PE Linker Interno
- DOS stub + PE signature ✅
- COFF + Optional Header (PE32+) ✅
- Section layout con alignment ✅
- Import Table completa (IDT + ILT + IAT + Hint/Name) ✅
- Export Table para DLLs ✅
- Base Relocations (.reloc) ✅
- Built-in imports: kernel32 (48+), user32 (34+), msvcrt (45+), ucrt (14+) ✅

---

## 3. GAPS RESTANTES PARA MASM 100%

### 3A. Parser — Funcionalidades Pendientes

| Feature | Estado | Impacto |
|---------|--------|---------|
| `@defmacro` expansion | ⚠️ Almacena pero no expande | User macros no funcionan |
| `@rodata` section | ❌ No mapeado | `.rodata` → `.const` |
| Error reporting con línea/columna | ❌ No hay source mapping | Debugging difícil |
| `@ring(0/1/2/3)` decorator | ❌ No existe | Validación de privilegio |
| `@segment(name, attrs)` | ❌ No existe | Custom segments |
| `@bits(16/32/64)` | ❌ No existe | Transición boot → long mode |

### 3B. NASM Emitter — Paridad con MASM

- Sin `LOCAL` support
- Sin `PROC FRAME` / SEH directives
- Sin struct emission
- Sin INCLUDELIB equivalente
- Sin size disambiguation para memory ops

### 3C. Flat Binary Output

- `--flat` / `--bin` output mode (sin PE headers) para bootloaders
- `@org(0x7C00)` para boot sector
- `@bits(16)` para real mode boot code

### 3D. Stdlib/Macros — Dead Code

- `prologue()`, `epilogue()`, `linux_syscall()` existen pero nunca se llaman
- Pipeline no invoca stdlib

### 3E. Tests — Cobertura

- Falta tests de encoder (codificación binaria correcta)
- Falta tests de linker (PE generation)
- Falta tests de Ring 0 instructions

---

## 4. ARCHIVOS POR TAMAÑO (Complejidad)

| Archivo | Líneas | Rol |
|---------|--------|-----|
| `encoder.rs` | 1940 | Codificador binario x86_64 (más completo) |
| `parser.rs` | 1730+ | Parser principal |
| `pe_writer.rs` | 1016 | Linker PE interno |
| `masm.rs` | 768 | Emitter MASM |
| `instruction.rs` | 680+ | IR opcodes + operands |
| `coff.rs` | 590 | Generador COFF .obj |
| `lexer.rs` | 545 | Tokenizer |
| `main.rs` | 552 | CLI + pipeline |
| `register.rs` | 240+ | IR registers (completo) |
| `nasm.rs` | 274 | Emitter NASM |
| `section.rs` | 212 | IR sections/functions/structs |
| `sib.rs` | 200+ | ModR/M + SIB encoding (completo) |
| `relocator.rs` | ~250 | Relocations |
| `import_lib.rs` | ~200 | Import lib parser |
| `coff_reader.rs` | ~190 | COFF reader |
| `vex.rs` | 52 | VEX prefix builder |

---
