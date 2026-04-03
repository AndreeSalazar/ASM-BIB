# Tareas v0.3 - Estabilización de MASM & Paridad Funcional

- [x] Extender Builtins (scanf, pow, sqrt, memset, math vectorizado)
- [x] Resolver limitantes del `Lexer` para reconocer floats en structs
- [x] Actualizar `Parser` para emitir control flow básico (Switch/Case, Continue)
- [x] Solucionar CallingConv enum en Funciones (Stdcall, Fastcall, Cdecl, Naked)
- [x] Soportar `@struct` definitions con DataDefs compuestos
- [x] Actualizaciones en Emitters (masm.rs, nasm.rs) para lidiar con `FunctionItem::RawDirective` y alineación
- [x] Crear un demo completo `complete_demo.pasm` que use todas las features juntas
- [x] Agregar demo `struct_demo.pasm` para tests AVX SIMD y arreglos custom
- [x] Actualizar `README.md`
- [x] Validar que `cargo build` no de errores críticos de parser
