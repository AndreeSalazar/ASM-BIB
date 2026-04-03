# PASM High-Level Examples

Python + C → ASM: Write simple OOP code, get real x86 assembly.

## How it works

```
print("Hello")  →  .pasm  →  MASM (.asm)  →  ml64  →  .exe
print("Hello")  →  .pasm  →  NASM (.asm)  →  nasm  →  .obj → link → .exe
```

## Examples

| File | Description | Target |
|------|-------------|--------|
| `hello_masm.pasm` | Hello World for MASM (Windows x64) | `--masm` |
| `hello_nasm.pasm` | Hello World for NASM (Windows x64) | `--nasm` |
| `oop_masm.pasm` | OOP classes for MASM | `--masm` |
| `oop_nasm.pasm` | OOP classes for NASM | `--nasm` |

## Build & Run

### MASM (Windows)
```bash
# Compile .pasm → .asm
cargo run -- examples/pasm_highlevel/hello_masm.pasm --masm -o hello.asm

# Assemble + Link → .exe
ml64 /c /nologo hello.asm
link /SUBSYSTEM:CONSOLE /ENTRY:main hello.obj kernel32.lib msvcrt.lib
hello.exe
```

### NASM (Windows)
```bash
# Compile .pasm → .asm
cargo run -- examples/pasm_highlevel/hello_nasm.pasm --nasm -o hello.asm

# Assemble + Link → .exe
nasm -f win64 hello.asm -o hello.obj
link /SUBSYSTEM:CONSOLE /ENTRY:main hello.obj kernel32.lib msvcrt.lib
hello.exe
```

### Auto-build (single command)
```bash
cargo run -- examples/pasm_highlevel/hello_masm.pasm --masm --build
```
