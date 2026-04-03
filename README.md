# ASM-BIB — Arquitectura v0.3 💀🦈

> Eddi Andreé Salazar Matos | Lima, Perú 🇵🇪
>
> Objetivo: Escribir x86 ASM con sintaxis Python-like y sin dolor. MASM Nativo Completo.

![Banner](https://img.shields.io/badge/ASM--BIB-v0.3-red?style=for-the-badge&logo=rust)

---

## 🗺️ Roadmap de Desarrollo

1. **`v0.3` (Actual)** — Stdlib completa (switch/case, structs, calling conventions, scanf, pow, macros listos). Canon MASM 100% abstraído.
2. **`v0.4` - `games/ stdlib`** — Expansión de la librería estándar para juegos: Funciones math, físicas, y abstracción intensiva de SIMD (SSE/AVX/AVX2).
3. **`v1.0` - MASM Assembler Propio** — Reemplazar la dependencia de `ml64.exe`! ASM-BIB emitirá los `.obj` directamente, siendo un orquestador independiente (Rust construyendo Rust/ASM).

---

## 1. Pipeline v0.3

```text
.pasm source → Lexer → Parser → IR (Intermediate Rep) → Emitter → .asm (MASM / NASM)
                                                           ↓
                                              assembler (ml64) → .obj → .exe
```

## 2. Sintaxis Python-like: ¡Puro x86 sin dolor!

ASM-BIB provee una sintaxis limpia. Todo lo que odias de ensamblador lo hace el compilador por detrás:

```python
@arch('x86_64')
@format('win64')

@section('.data')
    msg = string("Select option: ")
    fmt = string("%d")

@section('.bss')
    opcion = resd(1)

@section('.text')
@export
def main():
    prologue(32)
    print(msg)
    scanf(fmt, opcion)

    mov(eax, dword(opcion))
    @switch(eax)
        @case(1)
            print("Elegiste 1!\n")
            @break
        @case(2)
            print("Elegiste 2!\n")
            @break
        @default
            print("No válido\n")
            @break
    @endswitch
    
    epilogue()
```

## 3. Librería Estándar (`stdlib` integrada en `.pasm`)

La magia del parser de ASM-BIB expande estas macros a código x86 nativo/API invocations, manteniéndote enfocado.

### I/O & Memoria
* `print("string")` → Llama `printf` sin configurar registros explícitos.
* `printf(fmt_str, reg1, reg2...)` → Format String nativo MSVCRT.
* `scanf(fmt_str, dest...)` → User input rápido y seguro con formato C.
* `input(buffer_label, size)` → Helper (traduce a `%s` si es requerido).
* `alloc(size)` → Reserva heap win32 / libc.
* `free(ptr)` → Borra memoria dinamida.

### Punteros y Utilidades (CRT Arrays)
* `strlen(str)` → `repne scasb` optimizado.
* `strcpy(dst, src)`, `strcat(dst, src)` → Movimiento vectorizado.
* `memcpy(dst, src, size)` → `rep movsb` zero-cost loop.
* `memset(dst, val, size)` → `rep stosb`.
* `memcmp(dst, src, size)` → `repe cmpsb`.

### Operaciones Aritméticas (Math)
* `abs(reg)` → `neg` + `cmovs`.
* `min(r1, r2)`, `max(r1, r2)` → Comparadores + CMov nativos.
* `pow(base, exp)` → Ciclos optimizados con `imul`.
* `sqrt(dst, src)` → `sqrtss` (SSE escalar rápido).

### SIMD / Cálculo Vectorial (Juegos y 3D)
* `vec_add(dst, src)`, `vec_sub`, `vec_mul`, `vec_div` → `vaddps`, `vsubps` (AVX puro).
* `dot4(dst, src)` → Producto punto de vectores 4D (`vdpps` unificado).
* `mat4x4_mul(dst, A, B)` → Multiplicación masiva abstractada para matrices de shaders.

## 4. Control Flow y Orientación a Objetos

Adiós etiquetas sueltas:
* `@if(reg, ==, val) / @else / @endif`
* `@loop(reg, n) / @endloop`
* `@while(reg, <, val) / @endwhile`
* `@switch(reg) / @case(x) / @default / @endswitch`
* `@break` y `@continue` dinámicos.

Estructuras en ASM:
```python
@struct
class Float3:
    x = float32(1.0)
    y = float32(0.0)
    z = float32(0.0)
```
Genera `ALIGN 4`, `STRUCT`, `ENDS` en MASM; o `struc` dinámico de NASM.

Conéctate usando:
* `@stdcall` para Win32 antiguo.
* `@fastcall` para Win64 normal.
* `@naked` sin alineación de stack automático.

---
**Build de un solo clic:** `cargo run -- hello.pasm --build --masm`
