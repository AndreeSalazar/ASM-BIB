# ASM-BIB 💀🦈

**Universal ASM abstraction — Escribe una vez, exporta a TODOS los dialectos.**

> Python-like syntax → NASM / GAS / MASM / FASM / Flat binary

## Un mismo código → todos los targets

```python
@arch('x86_64')
@format('elf')

@section('.text')
@export
def main():
    push(rbp)
    mov(rbp, rsp)
    lea(rcx, msg)
    call(printf)
    xor(eax, eax)
    ret()

@section('.data')
msg = string("Hello from ASM-BIB!\n")
```

## Exportar a cualquier dialecto

```bash
asm-bib hello.pasm --nasm -o hello.asm     # NASM Intel
asm-bib hello.pasm --gas  -o hello.s       # GAS AT&T
asm-bib hello.pasm --masm -o hello.asm     # MASM Microsoft
asm-bib hello.pasm --fasm -o hello.asm     # FASM
asm-bib boot.pasm  --flat -o boot.bin      # Flat binary
```

## Arquitecturas soportadas

| Arch | Registros | Status |
|------|-----------|--------|
| x86-64 | rax..r15, xmm0..15, ymm, zmm | ✅ Base |
| x86-32 | eax..ebp | ✅ Base |
| x86-16 | ax..bp (bootloader) | ✅ Base |
| ARM64 | x0..x30, w0..w30, v0..v31 | ✅ Base |
| RISC-V | x0..x31, f0..f31 | ✅ Base |
| MIPS | $0..$31 | ✅ Base |

## Build

```bash
cargo build --release
```

## Eddi Andreé Salazar Matos — Lima, Perú 🇵🇪 — Techne v1.0
