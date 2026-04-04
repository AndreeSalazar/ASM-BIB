# Reporte Completo: ASM-BIB vs ml64.exe + link.exe

## BLOQUE A: Lo que falta para reemplazar ml64.exe 100%

### A1. Gaps en Codificación Binaria (encoder.rs)

| Gap | Ejemplo | Estado | Impacto |
|-----|---------|--------|---------|
| **MOV mem, imm** | `mov [rsp+8], 0x42` | ❌ Falta | MSVC emite esto constantemente para inicializar locales |
| **MOV mem, imm (64-bit)** | `mov QWORD PTR [rbp-8], 0` | ❌ Falta | C7 /0 con size prefix |
| **SUB/ADD/CMP reg, mem** | `cmp rax, [rbx+8]` | ❌ Falta | Comparación con memoria directa |
| **SUB/ADD/CMP mem, reg** | `add [rsp+0x20], rcx` | ❌ Falta | Store aritmético |
| **SUB/ADD/CMP mem, imm** | `sub DWORD PTR [rsp+4], 1` | ❌ Falta | Aritmética en pila |
| **AND/OR mem, reg/imm** | `and [mask], rax` | ❌ Falta | Lógica en memoria |
| **IMUL reg, reg, imm** | `imul rax, rbx, 12` | ❌ Falta | Multiplicación 3-operand (6B/69) |
| **LEA reg, [rip+label]** offset fix | Labels locales correctos | ⚠️ Parcial | RIP-offsets entre pass1/pass2 pueden variar |
| **TEST reg, imm** | `test eax, 0xFF` | ❌ Falta | F7 /0 — Pattern C++ muy frecuente |
| **CMP reg, mem** | `cmp rax, [rbx]` | ❌ Falta | 3B /r con mem |
| **MOV reg8/16 ↔ mem** | `mov al, [rbx]` | ⚠️ Parcial | 8-bit loads/stores en memoria |
| **MOVAPS mem, xmm** | `movaps [rsp+0x10], xmm0` | ❌ Falta | SSE store (0F 29) — dirección inversa |
| **MOVUPS mem, xmm** | Idem stores | ❌ Falta | SSE store |

> [!IMPORTANT]
> **Los 3 gaps más críticos son:**
> 1. `MOV mem, imm` — sin esto, no se puede inicializar variables locales en la pila
> 2. `SUB/ADD/CMP reg, mem` y `mem, reg` — sin esto, no se puede operar contra la pila
> 3. `IMUL reg, reg, imm` — multiplicación de 3 operandos es muy frecuente en MSVC

### A2. Directivas MASM que ml64.exe soporta y ASM-BIB NO tiene

| Directiva | Propósito | Estado ASM-BIB |
|-----------|-----------|----------------|
| `PROC FRAME` | Función con SEH info | ❌ Parser no entiende `FRAME` |
| `.ALLOCSTACK n` | Genera UNWIND_CODE para sub rsp | ❌ No existe |
| `.PUSHREG reg` | UNWIND_CODE para push reg | ❌ No existe |
| `.SAVEXMM128 reg, off` | UNWIND_CODE para XMM saves | ❌ No existe |
| `.SETFRAME reg, off` | Define frame pointer | ❌ No existe |
| `.ENDPROLOG` | Marca fin de prólogo | ❌ No existe |
| `MACRO / ENDM` | Macros del usuario | ❌ No existe — Solo built-ins |
| `PROTO` | Prototipos de función | ❌ No existe |
| `INVOKE` (MASM nativo) | Call con ABI automático | ⚠️ Parcial — Solo como macro, no MASM |
| `TYPEDEF` | Tipos de usuario | ❌ No existe |
| `UNION / ENDS` | Uniones de datos | ❌ No existe |
| `OPTION CASEMAP:NONE` | Case sensitivity | ❌ No existe |
| `OPTION PROLOGUE:NONE` | Custom prolog | ❌ No existe |
| `ASSUME` | Segment assumption | ❌ No existe |
| `COMM` | Common variables | ❌ No existe |
| `.IF / .ELSE / .ENDIF` | MASM high-level flow | ❌ No existe (tenemos @pasm equivalente) |
| `.REPEAT / .UNTIL` | MASM loop | ❌ No existe |
| `LOCAL` | Variables locales auto | ⚠️ Parcial — parser lo reconoce pero no genera stack |
| `BYTE/WORD/DWORD/QWORD PTR` | Size override en parser | ❌ No existe — Necesario para dis-ambiguación |
| `OFFSET label` | Dirección de label | ❌ No existe |
| `SIZEOF / LENGTHOF` | Tamaño de tipos | ❌ No existe |

### A3. COFF Object Deficiencias

| Componente | ml64 .obj | ASM-BIB .obj | Gap |
|------------|-----------|--------------|-----|
| **Aux Symbol Records** | Sí (section aux, function aux) | ❌ No | Linkers avanzados esperan aux records |
| **COMDAT sections** | `IMAGE_SCN_LNK_COMDAT` + selection type | ❌ No | Template C++ instantiation |
| **Debug Info (.debug$S, .debug$T)** | CodeView/PDB | ❌ No | Sin debugging |
| **Line Number Info** | Via CV | ❌ No | Sin source mapping |
| **Section alignment > 16** | Arbitrario | ⚠️ Solo 16 | DX12 GPUs necesitan 256 |
| **UNWIND_CODE array** | Array detallado en .xdata | ❌ Minimal (4 bytes dummy) | SEH no funciona en producción |
| **Timestamp** | Epoch | Siempre 0 | Menor |
| **Section groups** | Groupings | ❌ No | Import stubs |

---

## BLOQUE B: Lo que necesita un Linker Propio (link-bib.exe)

> [!WARNING]
> Construir un linker completo equivale a construir un **mini sistema operativo loader**. Es el componente más complejo de toda la toolchain después del propio compilador C++.

### B1. Estructura del PE Executable que el linker genera

```
┌─────────────────────────────────┐
│ DOS Stub (64 bytes mínimo)      │  ← "MZ" + offset a PE header
├─────────────────────────────────┤
│ PE Signature ("PE\0\0")         │  ← 4 bytes
├─────────────────────────────────┤
│ COFF File Header (20 bytes)     │  ← Machine, section count, etc.
├─────────────────────────────────┤
│ Optional Header (240 bytes x64) │  ← Entry point, image base, sizes
│   ├─ Standard Fields            │    AddressOfEntryPoint, BaseOfCode
│   ├─ Windows-Specific Fields    │    ImageBase, SectionAlignment, ...
│   └─ Data Directories (16×8)   │    Import, Export, Resource, Reloc, ...
├─────────────────────────────────┤
│ Section Table (N × 40 bytes)    │  ← .text, .rdata, .data, .pdata...
├─────────────────────────────────┤
│ .text (code)                    │  ← Merged code from all .obj
│ .rdata (read-only data)         │  ← Merged .rdata + import tables
│ .data (writable data)           │  ← Merged .data
│ .pdata (exception info)         │  ← Merged .pdata
│ .reloc (base relocations)       │  ← Para DLLs y ASLR
└─────────────────────────────────┘
```

### B2. Las 9 Tareas del Linker

| # | Tarea | Complejidad | Descripción |
|---|-------|-------------|-------------|
| **1** | **Parsear .obj archivos** | Media | Leer COFF headers, sections, symbols, relocations, string table |
| **2** | **Parsear .lib archivos** | Alta | Leer archive member headers, extraer .obj members para import libs |
| **3** | **Resolver símbolos globales** | Media | Tabla hash global: cada símbolo undefined busca en todos los .obj/.lib |
| **4** | **Layout de secciones** | Media | Merge secciones con mismo nombre, alinear, calcular RVAs |
| **5** | **Aplicar relocations** | Alta | Para cada relocation record, patchear bytes en la sección con la dirección final |
| **6** | **Generar Import Table** | **Muy Alta** | Import Directory Table + Import Lookup Table (ILT) + Import Address Table (IAT) + Hint/Name Table |
| **7** | **Generar Export Table** | Alta | Export Directory + Address Table + Name Pointer Table + Ordinal Table |
| **8** | **Generar .reloc** | Media | Base Relocation Table para DLLs (tipo 3 = HIGHLOW, tipo 10 = DIR64) |
| **9** | **Escribir PE final** | Media | DOS stub + PE header + Optional header + section headers + section data |

### B3. Import Table — El corazón del linker Windows

Cuando tu programa llama `ExitProcess`, el linker debe generar esto en `.rdata`:

```
Import Directory Table:
  ┌─ ILT RVA ─┬─ TimeDateStamp ─┬─ ForwarderChain ─┬─ Name RVA ─┬─ IAT RVA ─┐
  │ 0x00003000 │    0x00000000    │    0x00000000     │ 0x00003080 │ 0x00003040│
  └────────────┴──────────────────┴───────────────────┴────────────┴───────────┘
  ┌─ 0 ────────┬── 0 ─────────────┬── 0 ──────────────┬── 0 ───────┬── 0 ─────┐ (null terminator)
  └────────────┴──────────────────┴───────────────────┴────────────┴───────────┘

Import Lookup Table (ILT) @ 0x3000:
  [0x0000000000003090]  ← Hint/Name entry for "ExitProcess"
  [0x0000000000000000]  ← null terminator

Import Address Table (IAT) @ 0x3040: (copia exacta de ILT, pero patcheada por el loader)
  [0x0000000000003090]  ← Will become the actual address at runtime
  [0x0000000000000000]

Hint/Name Table @ 0x3090:
  [0x0000]  ← Hint (ordinal hint)
  "ExitProcess\0"

DLL Name @ 0x3080:
  "kernel32.dll\0"
```

El `call ExitProcess` en tu código se convierte en `call [IAT_entry]` — un `FF 15` indirecto.

---

## BLOQUE C: Roadmap Priorizado

### Fase 7: Encoding Gaps Críticos (encoder.rs)
> Cerrar los 3 gaps más críticos que rompen la compilación de C++ real.

- `MOV mem, imm` (C7 /0 + SIB) 
- `SUB/ADD/CMP/AND/OR reg, mem` y `mem, reg` y `mem, imm`
- `IMUL reg, reg, imm` (3 operandos: 6B/69)
- `TEST reg, imm` (F7 /0)
- SSE stores: `MOVAPS mem, xmm` / `MOVUPS mem, xmm`

### Fase 8: COFF Production Quality  
> Hacer que nuestros .obj sean indistinguibles de los de ml64.

- UNWIND_CODE array real en .xdata (no dummy)
- Aux symbol records (section definition, function begin/end)
- Section alignment variable (4, 8, 16, 64, 256, 4096)
- COMDAT section support

### Fase 9: Parser MASM Directives ✅ COMPLETADA
> Directivas MASM para Ring 1/2/3 — generación MASM + COFF interno desde Rust.

#### 9A. Size Disambiguation — `BYTE/WORD/DWORD/QWORD PTR` ✅
- Parser: `dword(expr)`, `word(expr)`, `byte(expr)`, `qword(expr)` pseudo-calls
  → emiten `Operand::Memory { size: Some(N) }` con override explícito
- MASM Emitter: `maybe_size_prefix()` emite `DWORD PTR [rsp+8]` etc.
- Encoder: `size.unwrap_or(8)` en `MOV mem,imm` / `ADD/SUB/CMP mem,imm` respeta size

#### 9B. `OFFSET label` ✅
- Parser: `offset(label)` pseudo-call → emite `Operand::Label` (LEA context)
- MASM Emitter: labels usados como operando de datos reciben `OFFSET` via `is_memory_context`

#### 9C. `LOCAL var:TYPE` con Stack Frame Automático ✅
- Parser: `@local(name, type)` → calcula offset alineado, inserta en `current_locals`
- Acceso a local por nombre → `Operand::Memory { base: RBP, disp: -offset }`
- Auto-prologue: si hay `@local` sin `prologue()`, genera `push rbp / mov rbp,rsp / sub rsp,N`
- MASM Emitter: emite `LOCAL name:TYPE` antes del cuerpo de instrucciones

#### 9D. `PROC FRAME` + SEH Unwind Directives ✅
- `@frame` decorator → `has_frame: true` → MASM emite `name PROC FRAME`
- `@pushreg(reg)` → `.PUSHREG reg`
- `@allocstack(n)` → `.ALLOCSTACK n`
- `@savereg(reg, off)` → `.SAVEREG reg, offset`
- `@savexmm128(reg, off)` → `.SAVEXMM128 reg, offset`
- `@setframe(reg, off)` → `.SETFRAME reg, offset`
- `@endprolog` → `.ENDPROLOG`
- IR: `SehDirective` enum con variantes para cada tipo
- COFF: `.xdata` genera `UNWIND_INFO` real con `UNWIND_CODEs` (PUSH_NONVOL, SET_FPREG, ALLOC_SMALL/LARGE)

#### 9E. User-Defined `MACRO / ENDM` ✅
- `@defmacro(name)` ... `@endmacro` → almacena body tokenizado en `user_macros`
- Expansión automática al invocar `name()` — busca en `try_expand_builtin`
- MASM Emitter: macros se expanden inline (no se emite MACRO/ENDM, se expande)

#### 9F. Encoder Gaps Cerrados (Ring 3 completo) ✅
- `AND/OR mem, reg` — ej: `and [rsp+0x20], rax` (0x21/0x09)
- `AND/OR mem, imm` — ej: `or DWORD PTR [rsp+4], 0xFF` (0x81)
- `AND/OR reg, label` — ej: `and rax, [global_mask]` (RIP-relative)
- Previo: `MOV mem,imm`, `SUB/ADD/CMP mem,{reg,imm}`, `IMUL 3-op`, `TEST reg,imm`, SSE stores

#### Uso PASM (Ring 3 — modo usuario Windows):
```python
@format("win64")
@section(".text")

@export
@frame
def MyFunction():
    @pushreg(rbp)
    push(rbp)
    @allocstack(0x40)
    sub(rsp, 0x40)
    @setframe(rbp, 0)
    mov(rbp, rsp)
    @endprolog
    
    @local(counter, dword)
    @local(buffer, qword)
    
    mov(dword([rbp - 4]), 0)    # counter = 0
    invoke(MessageBoxA, 0, "Hello", "ASM-BIB", 0)
    
    leave()
    ret()
```

#### MASM Generado:
```asm
; Generated by ASM-BIB -- MASM dialect
option casemap:none

INCLUDELIB kernel32.lib
INCLUDELIB user32.lib

EXTERNDEF ExitProcess:PROC
EXTERNDEF MessageBoxA:PROC

.code
PUBLIC MyFunction
MyFunction PROC FRAME
    LOCAL counter:DWORD, buffer:QWORD
    .PUSHREG rbp
    push rbp
    .ALLOCSTACK 40h
    sub rsp, 40h
    .SETFRAME rbp, 0
    mov rbp, rsp
    .ENDPROLOG
    mov DWORD PTR [rbp - 4], 0
    ; ... invoke expansion ...
    leave
    ret
MyFunction ENDP
END
```

### Fase 10: Linker Core — `src/linker/` ✅ COMPLETADA
> Linker interno que reemplaza link.exe para generar .exe/.dll directamente.

#### 10A. COFF Reader (`coff_reader.rs`) ✅
- Parsea COFF .obj completos: headers, secciones, símbolos, relocations, string table
- Soporta nombres largos (string table offset) para secciones y símbolos
- Todos los tipos de relocación AMD64: REL32, ADDR32NB, ADDR64, REL32_1..5

#### 10B. Resolución de Símbolos ✅
- Tabla hash global: cada símbolo se resuelve contra locales → imports → IAT
- Soporte `__imp_` prefix para imports indirectos
- Auto-detección de librería por nombre de función (kernel32, user32, msvcrt)
- Símbolos no resueltos se skipean silenciosamente (permisivo)

#### 10C. Layout de Secciones ✅
- SectionAlignment = 0x1000 (4KB páginas), FileAlignment = 0x200 (512 bytes)
- RVA y file offset calculados automáticamente
- Merge de secciones IR por tipo (.text, .rdata, .data, .bss)

#### 10D. Aplicar Relocations ✅ (`relocator.rs`)
- `IMAGE_REL_AMD64_REL32` — RIP-relative (call, jmp, lea, mov [rip+disp])
- `IMAGE_REL_AMD64_ADDR32NB` — RVA sin base (pdata/xdata)
- `IMAGE_REL_AMD64_ADDR64` — 64-bit absoluto + genera base relocation
- Addend preservation (existing bytes sumados al target)

### Fase 11: Import Table Generator ✅ COMPLETADA

#### 11A. Parser de .lib Archives (`import_lib.rs`) ✅
- Lee formato COFF archive (`!<arch>\n` signature)
- Parsea Short Import Headers (sig1=0, sig2=0xFFFF)
- Extrae: DLL name + function name + ordinal hint

#### 11B. Built-in Import Definitions ✅
- `kernel32.dll` — 48+ funciones (ExitProcess, VirtualAlloc, CreateThread, etc.)
- `user32.dll` — 34+ funciones (MessageBoxA/W, CreateWindowEx, etc.)
- `msvcrt.dll` — 45+ funciones (printf, malloc, memcpy, strlen, etc.)
- `ucrt` — 14+ funciones (_initterm, _exit, etc.)
- No necesita .lib externos para programas estándar

#### 11C. Import Table Generation ✅
- Import Directory Table (IDT): 20 bytes por DLL + null terminator
- Import Lookup Table (ILT): 8-byte entries con RVA a Hint/Name
- Import Address Table (IAT): copia idéntica del ILT (patcheada por el loader)
- Hint/Name Table: ordinal hint (2 bytes) + null-terminated name
- DLL name strings
- IAT map: symbol_name → IAT entry RVA (para resolver `call [__imp_X]`)

### Fase 12: PE Writer ✅ COMPLETADA (`pe_writer.rs`)

#### 12A. DOS Stub ✅
- 64-byte MZ header con `e_lfanew` apuntando al PE signature

#### 12B. PE Headers ✅
- `PE\0\0` signature (4 bytes)
- COFF Header: Machine=AMD64, NumberOfSections, SizeOfOptionalHeader=240
- Characteristics: EXECUTABLE_IMAGE | LARGE_ADDRESS_AWARE (+DLL si aplica)

#### 12C. Optional Header (PE32+, 240 bytes) ✅
- Magic = 0x020B (PE32+)
- AddressOfEntryPoint — auto-detecta main/_start/WinMain
- ImageBase = 0x140000000 (configurable)
- SectionAlignment = 0x1000, FileAlignment = 0x200
- SizeOfImage, SizeOfHeaders (aligned)
- Stack/Heap reserve/commit (configurable)
- Subsystem: 3=CONSOLE, 2=WINDOWS (auto-detectado)
- DllCharacteristics: DYNAMIC_BASE | NX_COMPAT

#### 12D. Data Directories (16 × 8 bytes) ✅
- [0] Export Table → .edata (DLLs)
- [1] Import Table → IDT en .rdata
- [5] Base Relocation → .reloc (DLLs)
- [12] IAT → IAT en .rdata

#### 12E. Section Headers + Data ✅
- .text — código ejecutable
- .rdata — datos read-only + import tables
- .data — datos read-write
- .edata — export table (DLLs)
- .reloc — base relocations (DLLs)
- Padding a FileAlignment entre secciones

### Fase 13: Base Relocations + Export Table ✅ COMPLETADA

#### 13A. Base Relocations (.reloc) ✅
- `BaseRelocationBuilder` genera bloques de 4KB páginas
- Cada entrada: `(type:4 bits) | (offset:12 bits)`
- Tipos: IMAGE_REL_BASED_DIR64 (10), HIGHLOW (3), ABSOLUTE (0)
- Automático: toda ADDR64 relocation genera base reloc entry

#### 13B. Export Table (.edata) ✅
- `ExportTableBuilder` genera tabla completa
- Export Directory Table (40 bytes): Name, NumberOfFunctions, etc.
- Address Table: RVAs de funciones exportadas
- Name Pointer Table: RVAs a strings de nombres
- Ordinal Table: mapeo nombre→ordinal
- Soporta funciones marcadas `@export` en PASM

#### 13C. CLI Integration ✅
- `--link` flag: .pasm → .exe directamente (sin ml64/link.exe)
- `--dll` + `--link`: genera .dll con exports + relocations
- Auto-detección de entry point y subsystem
- INCLUDELIB directives se pasan como extra_libs

---

## Uso Completo del Pipeline

```bash
# Pipeline clásico (ml64 + link.exe)
asm-bib hello.pasm --masm --build

# Pipeline interno COFF (ml64 reemplazado, link.exe aún necesario)
asm-bib hello.pasm --native --build

# Pipeline 100% interno (sin dependencias externas!)
asm-bib hello.pasm --link

# DLL con exports
asm-bib mylib.pasm --link --dll
```

## Arquitectura Final

```
hello.pasm
    │
    ├── Lexer → Tokens
    ├── Parser → IR (Program)
    ├── Emitter → MASM/NASM (.asm)     [--masm/--nasm]
    ├── Encoder → COFF .obj             [--native]
    └── Linker → PE .exe/.dll           [--link]  ← ¡NUEVO!
         ├── coff_reader.rs  (Phase 10)
         ├── import_lib.rs   (Phase 11)
         ├── pe_writer.rs    (Phase 12)
         └── relocator.rs    (Phase 13)
```
