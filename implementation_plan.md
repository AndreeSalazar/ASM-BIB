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

### Fase 9: Parser MASM Directives  
> Agregar las directivas que ml64.exe acepta.

- `BYTE/WORD/DWORD/QWORD PTR` size disambiguation
- `OFFSET label`
- `LOCAL var:TYPE` con generación de stack frame
- `PROC FRAME / .ALLOCSTACK / .PUSHREG / .ENDPROLOG`
- User-defined `MACRO / ENDM`

### Fase 10: Linker Core (Mini link-bib.exe) — Módulo `src/linker/`
> El objetivo final: un linker propio para no depender de link.exe.

- Parsear COFF .obj (reutilizar nuestras definiciones)
- Resolver símbolos globales + undefined
- Merge de secciones por nombre
- Layout con SectionAlignment/FileAlignment
- Aplicar relocations (IMAGE_REL_AMD64_REL32, ADDR32NB, ADDR64)

### Fase 11: Import Table Generator
> El componente más crítico para que el .exe funcione sin link.exe.

- Parsear .lib import archives
- Generar Import Directory, ILT, IAT, Hint/Name Tables
- Convertir `call ExitProcess` → `call [__imp_ExitProcess]` con IAT thunk

### Fase 12: PE Writer
> Escribir el ejecutable final completo.

- DOS MZ stub
- PE\0\0 signature + COFF header
- Optional Header (entry point, image base 0x140000000, stack/heap reserves)
- Data Directories (import, exception, reloc)
- Section data with proper FileAlignment padding

### Fase 13: Base Relocations + Export Table  
> Para DLLs completos.

- .reloc section (base relocation table)
- Export table para funciones marcadas @export
- Generar .lib de importación automáticamente

---

## Open Questions

> [!IMPORTANT]
> **1.** ¿Quieres que priorice las **Fases 7-8** (hacer el COFF perfecto) primero, o saltar directo a **Fase 10** (linker propio)?
> 
> **2.** ¿El linker propio debería reemplazar `link.exe` completamente, o solo para casos simples (1-2 .obj + kernel32.lib)?
>
> **3.** ¿Necesitas soporte completo de `.lib` (archives con múltiples .obj) o solo import libraries de Windows?
