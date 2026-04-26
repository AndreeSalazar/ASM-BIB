# Doble Misión: ASM-BIB MASM Completo + ADead-BIB Runtime 2.0 Killer Determinista

## Parte A — ASM-BIB: Completar el Árbol MASM

### Diagnóstico Actual (post-audit)

El audit report dice ~65% para reemplazar ml64.exe. Después de revisar el código actual, la situación ha **mejorado significativamente** desde el audit. Muchas cosas que el audit marcó como faltantes **ya están implementadas**:

| Lo que el audit decía que faltaba | Estado real en código |
|---|---|
| Control Registers CR0-CR4 | ✅ Ya en `register.rs` L65 |
| Debug Registers DR0-DR7 | ✅ Ya en `register.rs` L67 |
| SPL/BPL/SIL/DIL | ✅ Ya en `register.rs` L60 |
| R8B-R15B | ✅ Ya en `register.rs` L61 |
| AH/BH/CH/DH encoding en SIB | ✅ Ya corregido en `sib.rs` L49-52 |
| Segment registers en SIB | ✅ Ya en `sib.rs` L76-81 |
| Ring 0 instructions (LGDT/LIDT/LTR/etc) | ✅ Ya en `encoder.rs` L1306-1434 |
| MOV CRn/DRn | ✅ Ya en `encoder.rs` L173-205 |
| IN/OUT | ✅ Ya en `encoder.rs` L1399-1434 |
| ENTER | ✅ Ya en `encoder.rs` L1437-1443 |
| Memory fences (MFENCE/LFENCE/SFENCE) | ✅ Ya en `encoder.rs` L1393-1396 |
| LOCK prefix | ✅ Ya en `encoder.rs` L1396 |
| SHUFPS/SHUFPD | ✅ Ya en `encoder.rs` L1620-1639 |
| CMPPS/CMPSS/CMPPD/CMPSD | ✅ Ya en `encoder.rs` L1641-1665 |
| PMULUDQ | ✅ Ya en `encoder.rs` L1568-1577 |
| PUNPCK* | ✅ Ya en `encoder.rs` L1579-1601 |
| AVX 3-operand family completa | ✅ Ya en `encoder.rs` L1667-1818 |
| FMA instructions | ✅ Ya en `encoder.rs` L1820-1852 |

### Lo que REALMENTE falta para MASM completo

> [!IMPORTANT]
> El estado real es **~82%** para reemplazar ml64.exe, no 65%. Los gaps restantes son:

#### 1. Parser — Gaps Críticos
- **`@defmacro` expansion**: Almacena macros pero NO las expande. Los user macros son dead code.
- **`@rodata` section**: No mapeado → `.const` para MASM.
- **Error reporting con source location**: No hay source mapping (línea/columna). Debug imposible.
- **`@ring(0/1/2/3)` decorator**: No existe — necesario para validación de privilegio.
- **`@segment(name, attrs)` directive**: No existe — custom segments.
- **`@bits(16/32/64)` directive**: No existe — necesario para bootloaders (transición real→protected→long).

#### 2. NASM Emitter — Paridad con MASM
- Sin `LOCAL` support
- Sin `PROC FRAME` / SEH directives
- Sin struct emission
- Sin INCLUDELIB equivalente
- Sin size disambiguation para memory ops

#### 3. Encoder — Gaps menores
- **PREFETCH\* instructions** (cache hints): `PREFETCHT0`, `PREFETCHT1`, `PREFETCHT2`, `PREFETCHNTA`
- **16-bit R8W-R15W registers**: Faltan en el enum `Register`
- **`MOVSXD` (63 /r)**: Sign-extend DWORD→QWORD (diferente de MOVSX)
- **CMOVcc mem operand**: Solo soporta reg,reg — falta reg,mem
- **SETcc mem operand**: Solo soporta reg — falta mem
- **Flat binary output** (`--flat`/`--bin`): Para bootloaders sin PE headers

#### 4. Stdlib/Macros — Dead code
- `prologue()`, `epilogue()`, `linux_syscall()` en `macros/stdlib.rs` existen pero **nunca se llaman**.
- Pipeline no invoca stdlib. Son dead code.

#### 5. Tests — Cobertura insuficiente
- Solo 7 fixture tests + 4 ML64 comparison tests
- **No hay tests de encoder** (codificación binaria)
- **No hay tests de linker** (PE generation)
- **No hay tests de Ring 0** instructions

#### 6. AUDIT_REPORT.md — Desactualizado
- El audit report sigue diciendo 65%/40% cuando el código ya supera eso
- Necesita actualización completa

### Cambios Propuestos para ASM-BIB

#### [MODIFY] [register.rs](file:///c:/Users/andre/OneDrive/Documentos/ASM-BIB/src/ir/register.rs)
- Agregar R8W-R15W (16-bit extended registers)
- Agregar `Tbyte` (80-bit) size para x87 FPU support futuro

#### [MODIFY] [instruction.rs](file:///c:/Users/andre/OneDrive/Documentos/ASM-BIB/src/ir/instruction.rs)
- Agregar `Movsxd` opcode
- Agregar `Prefetcht0`, `Prefetcht1`, `Prefetcht2`, `Prefetchnta` opcodes

#### [MODIFY] [encoder.rs](file:///c:/Users/andre/OneDrive/Documentos/ASM-BIB/src/targets/x86_64/encoder.rs)
- Implementar MOVSXD encoding (63 /r con REX.W)
- Implementar PREFETCH* encodings (0F 18 /0-3)
- Agregar CMOVcc reg,mem forms
- Agregar SETcc mem forms

#### [MODIFY] [sib.rs](file:///c:/Users/andre/OneDrive/Documentos/ASM-BIB/src/targets/x86_64/sib.rs)
- Agregar R8W-R15W mappings

#### [MODIFY] [AUDIT_REPORT.md](file:///c:/Users/andre/OneDrive/Documentos/ASM-BIB/AUDIT_REPORT.md)
- Actualización completa del estado real: 65% → 85%+
- Marcar como ✅ todo lo que ya fue implementado
- Actualizar plan de mejoras

---

## Parte B — ADead-BIB Runtime 2.0 Killer: Arquitectura y Monolito Determinista

### Contexto

El runtime actual en `02_core/adeb-core/src/runtime/` tiene:
- `cpu_detect.rs` — Detección de features CPU
- `dispatcher.rs` — Auto-dispatch CPU/GPU
- `gpu_dispatcher.rs` — GPU dispatch
- `gpu_misuse_detector.rs` — Detector de mal uso GPU

**Falta por completo** la infraestructura de runtime determinista para:
1. Memoria continua
2. Ejecución matemática determinista
3. Scheduler determinista
4. Stack management
5. Error como valor
6. Lifetime determinista
7. ABI calling convention

### Arquitectura Propuesta: **2 Responsabilidades Monolíticas**

```
02_core/adeb-core/src/runtime/killer_v2/
├── mod.rs                          ← Raíz del Runtime 2.0 Killer
├── arquitectura/                   ← RESPONSABILIDAD 1: Infraestructura
│   ├── mod.rs
│   ├── memoria_continua.rs         ← Arena allocator + bump allocator determinista
│   ├── stack_management.rs         ← Stack frames, shadow space, red zone
│   ├── lifetime_determinista.rs    ← Ownership sin GC, drop determinista
│   └── abi_calling_convention.rs   ← Win64/SysV ABI completo con validación
│
└── monolito/                       ← RESPONSABILIDAD 2: Ejecución
    ├── mod.rs
    ├── ejecucion_matematica.rs     ← IEEE 754 strict, sin NaN silencioso
    ├── scheduler_determinista.rs   ← Round-robin determinista, no preemptive
    └── error_como_valor.rs         ← Result<T, E> nativo, cero excepciones
```

### Diseño Ultra-Determinista

> [!IMPORTANT]
> Cada módulo sigue la regla: **ZERO sorpresas en runtime**. Todo comportamiento es predecible en compile-time.

#### Responsabilidad 1: Arquitectura (Infraestructura)

| Módulo | Filosofía | Determinismo |
|--------|-----------|-------------|
| `memoria_continua.rs` | Arena allocator con bump pointer. Todas las allocations son O(1). Free = reset del arena entero. Cero fragmentación. | Tiempo de allocation constante, sin syscalls en hot path |
| `stack_management.rs` | Stack frames con shadow space Win64 (32 bytes), red zone detection, stack canary, overflow guard pages. | Tamaño de stack calculado en compile-time. Stack overflow = abort, nunca undefined behavior |
| `lifetime_determinista.rs` | Ownership tracking con borrow checker simplificado. Drop orden = orden de declaración inverso. Cero GC. | Drop timing 100% predecible. No hay pause-the-world |
| `abi_calling_convention.rs` | Win64 ABI completo: RCX/RDX/R8/R9 para ints, XMM0-3 para floats, shadow space 32 bytes, stack alignment 16. SysV: RDI/RSI/RDX/RCX/R8/R9 | ABI violations detectadas en compile-time |

#### Responsabilidad 2: Monolito (Ejecución)

| Módulo | Filosofía | Determinismo |
|--------|-----------|-------------|
| `ejecucion_matematica.rs` | IEEE 754 strict mode. NaN = error explícito, nunca silencioso. Division by zero = error, nunca infinity. Overflow = error, nunca wrap. | Resultado matemático idéntico en toda plataforma x86-64 |
| `scheduler_determinista.rs` | Cooperative scheduling. Yield points explícitos. Round-robin con quantum fijo. No preemption = no data races. | Orden de ejecución 100% reproducible |
| `error_como_valor.rs` | Result<T, KillerError> para todo. Cero excepciones. Cero panics. Unwind = abort. Error propagation con ? operator nativo. | Flujo de error siempre visible en el type system |

### Archivos a Crear

#### [NEW] [mod.rs](file:///c:/Users/andre/OneDrive/Documentos/ADead-BIB/C_Compiler/02_core/adeb-core/src/runtime/killer_v2/mod.rs)
Root module del Runtime 2.0 Killer.

#### [NEW] [arquitectura/mod.rs](file:///c:/Users/andre/OneDrive/Documentos/ADead-BIB/C_Compiler/02_core/adeb-core/src/runtime/killer_v2/arquitectura/mod.rs)
Re-exports de los 4 módulos de infraestructura.

#### [NEW] [arquitectura/memoria_continua.rs](file:///c:/Users/andre/OneDrive/Documentos/ADead-BIB/C_Compiler/02_core/adeb-core/src/runtime/killer_v2/arquitectura/memoria_continua.rs)
Arena allocator determinista con bump pointer.

#### [NEW] [arquitectura/stack_management.rs](file:///c:/Users/andre/OneDrive/Documentos/ADead-BIB/C_Compiler/02_core/adeb-core/src/runtime/killer_v2/arquitectura/stack_management.rs)
Stack frame management determinista.

#### [NEW] [arquitectura/lifetime_determinista.rs](file:///c:/Users/andre/OneDrive/Documentos/ADead-BIB/C_Compiler/02_core/adeb-core/src/runtime/killer_v2/arquitectura/lifetime_determinista.rs)
Ownership y lifetime tracking sin GC.

#### [NEW] [arquitectura/abi_calling_convention.rs](file:///c:/Users/andre/OneDrive/Documentos/ADead-BIB/C_Compiler/02_core/adeb-core/src/runtime/killer_v2/arquitectura/abi_calling_convention.rs)
Win64/SysV ABI completo con validación compile-time.

#### [NEW] [monolito/mod.rs](file:///c:/Users/andre/OneDrive/Documentos/ADead-BIB/C_Compiler/02_core/adeb-core/src/runtime/killer_v2/monolito/mod.rs)
Re-exports de los 3 módulos de ejecución.

#### [NEW] [monolito/ejecucion_matematica.rs](file:///c:/Users/andre/OneDrive/Documentos/ADead-BIB/C_Compiler/02_core/adeb-core/src/runtime/killer_v2/monolito/ejecucion_matematica.rs)
Ejecución matemática IEEE 754 strict determinista.

#### [NEW] [monolito/scheduler_determinista.rs](file:///c:/Users/andre/OneDrive/Documentos/ADead-BIB/C_Compiler/02_core/adeb-core/src/runtime/killer_v2/monolito/scheduler_determinista.rs)
Scheduler cooperative determinista.

#### [NEW] [monolito/error_como_valor.rs](file:///c:/Users/andre/OneDrive/Documentos/ADead-BIB/C_Compiler/02_core/adeb-core/src/runtime/killer_v2/monolito/error_como_valor.rs)
Error handling como valores, cero excepciones.

#### [MODIFY] [mod.rs](file:///c:/Users/andre/OneDrive/Documentos/ADead-BIB/C_Compiler/02_core/adeb-core/src/runtime/mod.rs)
Agregar `pub mod killer_v2;` para integrar al runtime existente.

---

## Open Questions

> [!WARNING]
> **¿Flat binary output?** — Para el `--flat`/`--bin` mode en ASM-BIB (bootloaders), ¿quieres que lo implemente ahora o es para una fase futura?

> [!IMPORTANT]
> **¿NASM paridad?** — El NASM emitter tiene gaps significativos. ¿Quieres que lo equipare con MASM en esta iteración, o lo dejamos como secundario?

> [!IMPORTANT]
> **¿Macro expansion?** — El `@defmacro` en el parser almacena pero no expande. ¿Es prioridad arreglar el macro system, o es secundario?

---

## Verification Plan

### Automated Tests
1. `cargo build` en ASM-BIB para verificar que los cambios al encoder/IR compilan
2. `cargo test` en ASM-BIB para verificar tests existentes no rompen
3. `cargo build` en el workspace ADead-BIB para verificar que los nuevos módulos compilan
4. `cargo test` en adeb-core para verificar los unit tests del Runtime 2.0 Killer

### Manual Verification
- Revisión visual de la estructura de carpetas del killer_v2
- Validar que el AUDIT_REPORT.md refleja el estado real del código
