# ASM-BIB — Auditoría Completa v2.0

> Fecha: 2026-04-05 | Autor: Cascade AI Audit

---

## 1. RESUMEN EJECUTIVO

**Estado general: ~65% completo para reemplazar ml64.exe, ~40% para OS propio Ring 0-3**

| Componente | Estado | Nota |
|-----------|--------|------|
| Lexer | ✅ 95% | Robusto, soporta Python+C syntax |
| Parser | ✅ 90% | ~1730 líneas, @directives, structs, control flow, SEH |
| IR | ✅ 90% | Completo: opcodes, operands, sections, structs, enums |
| MASM Emitter | ✅ 95% | Genera MASM válido con PROC FRAME, LOCAL, SEH |
| NASM Emitter | ⚠️ 70% | Funcional pero sin SEH, LOCAL, structs |
| x86_64 Encoder | ⚠️ 75% | Falta Ring 0 instructions, AVX completo |
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

### 2B. Instrucciones Codificadas en encoder.rs (~1257 líneas)
- **Movimiento**: MOV (reg↔reg, reg↔imm, reg↔mem, mem↔imm, label↔reg/imm), LEA, PUSH, POP, MOVZX, MOVSX, XCHG, BSWAP
- **Aritmética**: ADD, SUB, CMP, XOR, AND, OR, ADC, SBB (todas con reg/imm/mem combinaciones), INC, DEC, NEG, NOT, MUL, DIV, IDIV, IMUL (1/2/3 operandos)
- **Shifts**: SHL, SHR, SAR, ROL, ROR, RCL, RCR (imm/CL/1)
- **Flow**: JMP, JE/JNE + 12 conditional jumps, CALL (label/reg/mem), RET, LEAVE, LOOP/LOOPE/LOOPNE
- **SETcc**: 12 variantes
- **CMOVcc**: 12 variantes
- **Bit**: BT/BTS/BTR/BTC, BSF/BSR, POPCNT/LZCNT/TZCNT
- **Atomic**: XADD, CMPXCHG
- **String**: REP MOVSB/W/D/Q, REP STOSB/W/D/Q, REPE CMPS, REPNE SCAS, LODS
- **SSE**: MOVAPS/MOVUPS, ADD/SUB/MUL/DIV/MIN/MAX/SQRT/PS, MOVSS scalar, stores
- **SSE2**: MOVAPD/MOVUPD packed double, MOVSD scalar double, integer ops (PADD/PSUB/PMULL/PAND/POR/PXOR), MOVDQA/MOVDQU + stores
- **AVX**: VADDPS (solo ejemplo), VZEROALL, VZEROUPPER
- **Conversiones**: CVTSI2SS/SD, CVTSS2SI/SD, CVTSD2SS/SI, CVTTSS2SI, CVTTSD2SI
- **Sistema**: SYSCALL, INT, HLT, CLI, STI, NOP, CPUID, IRETQ, RDTSC, RDTSCP
- **Misc**: CQO, CDQ, CBW, CWD, CWDE, LAHF, SAHF, XLAT, PUSHF, POPF

### 2C. COFF Object File
- Headers, sections, symbols, relocations, string table ✅
- .drectve section para INCLUDELIB/EXPORT ✅
- .pdata/.xdata con UNWIND_INFO real (PUSH_NONVOL + SET_FPREG) ✅
- Aux symbols para sections y funciones ✅

### 2D. PE Linker Interno
- DOS stub + PE signature ✅
- COFF + Optional Header (PE32+) ✅
- Section layout con alignment ✅
- Import Table completa (IDT + ILT + IAT + Hint/Name) ✅
- Export Table para DLLs ✅
- Base Relocations (.reloc) ✅
- Built-in imports: kernel32 (48+), user32 (34+), msvcrt (45+), ucrt (14+) ✅

---

## 3. GAPS CRÍTICOS PARA MASM PROPIO

### 3A. Encoder — Instrucciones Faltantes para Ring 3 Completo

| Instrucción | Uso | Prioridad |
|------------|-----|-----------|
| `ENTER imm16, imm8` | Stack frame setup | Media |
| `MOVSX reg, WORD PTR [mem]` con size override | Loads con sign-extend 16→32/64 | Alta |
| `SHUFPS/SHUFPD/PSHUFD` con imm8 | SSE shuffle | Media |
| `CMPPS/CMPSS/CMPPD/CMPSD` con imm8 | SSE comparison predicates | Media |
| `PMULUDQ` encoding | SSE2 multiply | Baja |
| `PUNPCK*` encodings | SSE2 unpack | Baja |
| AVX 3-operand (todos menos VADDPS) | Toda la familia AVX | Alta |
| `LOCK` prefix | Atomic operations | Alta |
| `PREFETCH*` | Cache hints | Baja |
| `MFENCE/LFENCE/SFENCE` | Memory barriers | Media |

### 3B. Encoder — Instrucciones Faltantes para Ring 0/1/2 (OS Propio)

| Instrucción | Opcode | Uso en OS |
|------------|--------|-----------|
| **`MOV CRn, reg`** | `0F 22 /r` | Configurar paging (CR3), protected mode (CR0) |
| **`MOV reg, CRn`** | `0F 20 /r` | Leer control registers |
| **`MOV DRn, reg`** | `0F 23 /r` | Hardware breakpoints |
| **`MOV reg, DRn`** | `0F 21 /r` | Leer debug registers |
| **`LGDT [mem]`** | `0F 01 /2` | Cargar Global Descriptor Table |
| **`SGDT [mem]`** | `0F 01 /0` | Guardar GDT |
| **`LIDT [mem]`** | `0F 01 /3` | Cargar Interrupt Descriptor Table |
| **`SIDT [mem]`** | `0F 01 /1` | Guardar IDT |
| **`LTR reg`** | `0F 00 /3` | Cargar Task Register |
| **`STR reg`** | `0F 00 /1` | Guardar Task Register |
| **`LLDT reg`** | `0F 00 /2` | Cargar LDT |
| **`SLDT reg`** | `0F 00 /0` | Guardar LDT |
| **`LMSW reg`** | `0F 01 /6` | Load Machine Status Word |
| **`SMSW reg`** | `0F 01 /4` | Store Machine Status Word |
| **`INVLPG [mem]`** | `0F 01 /7` | Invalidar TLB entry |
| **`RDMSR`** | `0F 32` | Leer Model-Specific Register |
| **`WRMSR`** | `0F 30` | Escribir MSR |
| **`IN al/ax/eax, imm8`** | `E4/E5/E5` | Leer puerto I/O |
| **`IN al/ax/eax, dx`** | `EC/ED/ED` | Leer puerto I/O (dinámico) |
| **`OUT imm8, al/ax/eax`** | `E6/E7/E7` | Escribir puerto I/O |
| **`OUT dx, al/ax/eax`** | `EE/EF/EF` | Escribir puerto I/O (dinámico) |
| **`SWAPGS`** | `0F 01 F8` | Swap GS base (Ring 0→3 transition) |
| **`WBINVD`** | `0F 09` | Write-back + invalidate cache |
| **`INVD`** | `0F 08` | Invalidate cache (sin write-back) |
| **`CLTS`** | `0F 06` | Clear Task-Switched flag en CR0 |
| **`HLT`** | `F4` | ✅ Ya existe |
| **`IRETQ`** | `48 CF` | ✅ Ya existe |

### 3C. IR — Registros Faltantes

El `Register` enum NO tiene:
- **Control Registers**: CR0, CR2, CR3, CR4 (esenciales para paging/protected mode)
- **Debug Registers**: DR0-DR3, DR6, DR7 (hardware breakpoints)
- **8-bit high**: R8B-R15B (SPL, BPL, SIL, DIL en modo 64-bit)
- **Segment registers en encoder**: CS/DS/ES/FS/GS/SS están en IR pero NO en sib.rs

### 3D. Parser — Funcionalidades Incompletas

| Feature | Estado | Impacto |
|---------|--------|---------|
| `@defmacro` expansion | ⚠️ Almacena pero no expande correctamente | User macros no funcionan |
| `@rodata` section | ❌ No mapeado | `.rodata` → `.const` |
| Error reporting con línea/columna | ❌ No hay source mapping | Debugging imposible |
| `@ring(0/1/2/3)` decorator | ❌ No existe | Necesario para OS |
| `@segment(name, attrs)` | ❌ No existe | Custom segments con atributos |
| `@bits(16/32/64)` | ❌ No existe | Cambiar modo en runtime (boot → protected → long) |

### 3E. NASM Emitter — Gaps vs MASM

- Sin `LOCAL` support
- Sin `PROC FRAME` / SEH directives  
- Sin struct emission (solo comentario)
- Sin INCLUDELIB equivalente
- Sin size disambiguation para memory ops

---

## 4. BUGS Y PROBLEMAS ENCONTRADOS

### Bug 1: SSE2 Packed Double Store — Doble Prefix 0x66
```
@encoder.rs:1126-1127
```
Cuando se hace `MOVAPD [mem], xmm`, el código emite `0x66` dos veces (una del match arm exterior en L1110, otra en L1126). Esto genera código inválido.

### Bug 2: `encode_reg` no maneja AH/BH/CH/DH
```
@sib.rs:53
```
`AH`, `BH`, `CH`, `DH` caen en el wildcard `_ => { ri.is_wide = false; 0 }` — deberían tener val 4,7,5,6 respectivamente con un flag `is_high_byte`.

### Bug 3: `Movsd` (string) vs `Movsd2` (SSE2) — Ambiguity
El parser necesita distinguir entre `movsd` string y `movsd` SSE2 scalar double. Actualmente el lexer no puede disambiguar esto; `Movsd` siempre es string op.

### Bug 4: Stdlib macros nunca se usan
`prologue()`, `epilogue()`, `linux_syscall()` en `macros/stdlib.rs` están declarados pero **nunca llamados** desde el parser ni el emitter. Son dead code.

### Bug 5: 96 warnings de compilación
Muchas constantes, structs y funciones definidas pero nunca usadas — especialmente en `coff_reader.rs`, `import_lib.rs`, `relocator.rs`. Indica código preparado pero no integrado.

---

## 5. PLAN DE MEJORAS PARA OS RING 0-3

### Fase 14: Ring 0/1/2 Instructions (PRIORITARIO)

1. Agregar registros CR0-CR4, DR0-DR7 al IR + encoder
2. Codificar instrucciones privilegiadas: MOV CRn/DRn, LGDT/LIDT/LTR, RDMSR/WRMSR, IN/OUT, INVLPG, SWAPGS
3. Agregar `@ring(n)` decorator al parser para validación de privilegio
4. Agregar LOCK prefix support

### Fase 15: Encoder Completeness

1. Completar AVX 3-operand family (VEX encoding ya existe en vex.rs)
2. Agregar MFENCE/LFENCE/SFENCE
3. Agregar ENTER instruction
4. Fix SSE2 double prefix bug
5. Fix AH/BH/CH/DH encoding
6. Agregar SPL/BPL/SIL/DIL

### Fase 16: Flat Binary Output (para bootloaders)

1. `--flat` / `--bin` output mode (sin PE headers)
2. `@org(0x7C00)` para boot sector
3. `@bits(16)` para real mode boot code
4. Soporte para transición 16→32→64 bit en un mismo archivo

### Fase 17: NASM Convergencia

1. Equiparar NASM emitter con MASM features
2. Shared encoder backend
3. Unified test suite

---

## 6. ARCHIVOS POR TAMAÑO (Complejidad)

| Archivo | Líneas | Rol |
|---------|--------|-----|
| `parser.rs` | 1730 | Parser principal |
| `encoder.rs` | 1257 | Codificador binario x86_64 |
| `pe_writer.rs` | 1016 | Linker PE interno |
| `masm.rs` | 767 | Emitter MASM |
| `instruction.rs` | 618 | IR opcodes + operands |
| `coff.rs` | 590 | Generador COFF .obj |
| `lexer.rs` | 545 | Tokenizer |
| `main.rs` | 552 | CLI + pipeline |
| `nasm.rs` | 274 | Emitter NASM |
| `section.rs` | 212 | IR sections/functions/structs |
| `register.rs` | 186 | IR registers |
| `relocator.rs` | ~250 | Relocations |
| `import_lib.rs` | ~200 | Import lib parser |
| `coff_reader.rs` | ~190 | COFF reader |
| `sib.rs` | 156 | ModR/M + SIB encoding |

---
