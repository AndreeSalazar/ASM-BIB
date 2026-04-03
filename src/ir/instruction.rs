use super::register::Register;

/// x86 opcodes — complete MASM canon (16/32/64-bit + SSE/AVX/AVX2)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Opcode {
    // === Movement ===
    Mov, Movzx, Movsx, Lea, Xchg, Push, Pop,
    Pushf, Popf, Pushad, Popad,
    Enter, // ENTER imm, imm

    // === Arithmetic ===
    Add, Sub, Mul, Imul, Div, Idiv, Inc, Dec, Neg, Adc, Sbb,

    // === Logic ===
    And, Or, Xor, Not, Shl, Shr, Sar, Rol, Ror, Rcl, Rcr,

    // === Comparison ===
    Cmp, Test,

    // === Jumps ===
    Jmp, Je, Jne, Jl, Jle, Jg, Jge, Jb, Jbe, Ja, Jae, Js, Jns, Jo, Jno, Jp, Jnp,
    Jcxz, Jecxz, Jrcxz,

    // === Loop ===
    Loop, Loope, Loopne,

    // === Call/Return ===
    Call, Ret, Leave,

    // === String ops ===
    RepMovsb, RepMovsw, RepMovsd, RepMovsq,
    RepStosb, RepStosw, RepStosd, RepStosq,
    Scasb, Scasw, Scasd,
    RepeCmpsb, RepeCmpsw, RepeCmpsd,
    RepneScasb, RepneScasw, RepneScasd,
    Movsb, Movsw, Movsd, Movsq,
    Stosb, Stosw, Stosd, Stosq,
    Cmpsb, Cmpsw, Cmpsd,
    Lodsb, Lodsw, Lodsd, Lodsq,
    Cld, Std,

    // === System ===
    Syscall, Int, Hlt, Cli, Sti, Nop, Cpuid, Iretq,
    Rdtsc, Rdtscp,

    // === Bit manipulation ===
    Bt, Bts, Btr, Btc, Bsf, Bsr,
    Popcnt, Lzcnt, Tzcnt,

    // === Byte swap / atomic ===
    Bswap, Xadd, Cmpxchg, Cmpxchg8b, Cmpxchg16b,

    // === SETcc ===
    Sete, Setne, Setl, Setle, Setg, Setge, Setb, Setbe, Seta, Setae, Sets, Setns,

    // === Conditional moves ===
    Cmove, Cmovne, Cmovl, Cmovle, Cmovg, Cmovge, Cmovb, Cmovbe, Cmova, Cmovae,
    Cmovs, Cmovns,

    // === SSE packed float ===
    Movaps, Movups, Addps, Subps, Mulps, Divps, Minps, Maxps, Xorps,
    Andps, Orps, Andnps,
    Sqrtps, Rsqrtps, Rcpps,
    Cmpps, Shufps, Unpcklps, Unpckhps,

    // === SSE scalar float ===
    Movss, Addss, Subss, Mulss, Divss, Sqrtss,
    Minss, Maxss, Cmpss,
    Comiss, Ucomiss,
    Cvtsi2ss, Cvtss2si, Cvttss2si,

    // === SSE2 packed double ===
    Movapd, Movupd, Addpd, Subpd, Mulpd, Divpd, Minpd, Maxpd, Xorpd,
    Andpd, Orpd, Andnpd,
    Sqrtpd, Cmppd, Shufpd,

    // === SSE2 scalar double ===
    Movsd2, // we use Movsd2 to not conflict with string movsd
    Addsd, Subsd, Mulsd, Divsd, Sqrtsd,
    Minsd, Maxsd, Cmpsd2,
    Comisd, Ucomisd,
    Cvtsi2sd, Cvtsd2si, Cvttsd2si,
    Cvtss2sd, Cvtsd2ss,

    // === SSE2 integer ===
    Movdqa, Movdqu,
    Paddb, Paddw, Paddd, Paddq,
    Psubb, Psubw, Psubd, Psubq,
    Pmullw, Pmulld, Pmuludq,
    Pand, Por, Pxor, Pandn,
    Pcmpeqb, Pcmpeqw, Pcmpeqd,
    Pcmpgtb, Pcmpgtw, Pcmpgtd,
    Psllw, Pslld, Psllq,
    Psrlw, Psrld, Psrlq,
    Psraw, Psrad,
    Pshufb, Pshufd, Pshufhw, Pshuflw,
    Punpcklbw, Punpckhbw, Punpcklwd, Punpckhwd, Punpckldq, Punpckhdq,

    // === SSE data movement ===
    Movd, Movq,

    // === AVX packed float ===
    Vmovaps, Vmovups, Vaddps, Vsubps, Vmulps, Vdivps, Vxorps,
    Vandps, Vorps, Vandnps,
    Vminps, Vmaxps, Vsqrtps,
    Vcmpps, Vshufps,

    // === AVX scalar float ===
    Vmovss, Vaddss, Vsubss, Vmulss, Vdivss, Vsqrtss,

    // === AVX packed double ===
    Vmovapd, Vmovupd, Vaddpd, Vsubpd, Vmulpd, Vdivpd, Vxorpd,

    // === AVX scalar double ===
    Vmovsd, Vaddsd, Vsubsd, Vmulsd, Vdivsd, Vsqrtsd,

    // === AVX integer ===
    Vmovdqa, Vmovdqu,
    Vpaddb, Vpaddw, Vpaddd, Vpaddq,
    Vpsubb, Vpsubw, Vpsubd, Vpsubq,
    Vpmullw, Vpmulld,
    Vpand, Vpor, Vpxor, Vpandn,

    // === AVX special ===
    Vdpps, Vdppd,
    Vbroadcastss, Vbroadcastsd,
    Vperm2f128, Vinsertf128, Vextractf128,
    Vzeroall, Vzeroupper,

    // === FMA ===
    Vfmadd132ps, Vfmadd213ps, Vfmadd231ps,
    Vfmadd132ss, Vfmadd213ss, Vfmadd231ss,
    Vfmadd132pd, Vfmadd213pd, Vfmadd231pd,
    Vfmadd132sd, Vfmadd213sd, Vfmadd231sd,

    // === Misc ===
    Cqo, Cdq, Cbw, Cwd, Cwde,
    Lahf, Sahf,
    Xlat,
}

impl Opcode {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            // Movement
            "mov" => Some(Opcode::Mov), "movzx" => Some(Opcode::Movzx),
            "movsx" => Some(Opcode::Movsx), "lea" => Some(Opcode::Lea),
            "xchg" => Some(Opcode::Xchg), "push" => Some(Opcode::Push),
            "pop" => Some(Opcode::Pop),
            "pushf" | "pushfq" => Some(Opcode::Pushf),
            "popf" | "popfq" => Some(Opcode::Popf),
            "pushad" | "pusha" => Some(Opcode::Pushad),
            "popad" | "popa" => Some(Opcode::Popad),
            "enter" => Some(Opcode::Enter),
            // Arithmetic
            "add" => Some(Opcode::Add), "sub" => Some(Opcode::Sub),
            "mul" => Some(Opcode::Mul), "imul" => Some(Opcode::Imul),
            "div" => Some(Opcode::Div), "idiv" => Some(Opcode::Idiv),
            "inc" => Some(Opcode::Inc), "dec" => Some(Opcode::Dec),
            "neg" => Some(Opcode::Neg),
            "adc" => Some(Opcode::Adc), "sbb" => Some(Opcode::Sbb),
            // Logic
            "and" => Some(Opcode::And), "or" => Some(Opcode::Or),
            "xor" => Some(Opcode::Xor), "not" => Some(Opcode::Not),
            "shl" => Some(Opcode::Shl), "shr" => Some(Opcode::Shr),
            "sar" => Some(Opcode::Sar), "rol" => Some(Opcode::Rol),
            "ror" => Some(Opcode::Ror),
            "rcl" => Some(Opcode::Rcl), "rcr" => Some(Opcode::Rcr),
            // Comparison
            "cmp" => Some(Opcode::Cmp), "test" => Some(Opcode::Test),
            // Jumps
            "jmp" => Some(Opcode::Jmp),
            "je" | "jz" => Some(Opcode::Je), "jne" | "jnz" => Some(Opcode::Jne),
            "jl" | "jnge" => Some(Opcode::Jl), "jle" | "jng" => Some(Opcode::Jle),
            "jg" | "jnle" => Some(Opcode::Jg), "jge" | "jnl" => Some(Opcode::Jge),
            "jb" | "jc" | "jnae" => Some(Opcode::Jb), "jbe" | "jna" => Some(Opcode::Jbe),
            "ja" | "jnbe" => Some(Opcode::Ja), "jae" | "jnb" | "jnc" => Some(Opcode::Jae),
            "js" => Some(Opcode::Js), "jns" => Some(Opcode::Jns),
            "jo" => Some(Opcode::Jo), "jno" => Some(Opcode::Jno),
            "jp" | "jpe" => Some(Opcode::Jp), "jnp" | "jpo" => Some(Opcode::Jnp),
            "jcxz" => Some(Opcode::Jcxz), "jecxz" => Some(Opcode::Jecxz),
            "jrcxz" => Some(Opcode::Jrcxz),
            // Loop
            "loop" => Some(Opcode::Loop),
            "loope" | "loopz" => Some(Opcode::Loope),
            "loopne" | "loopnz" => Some(Opcode::Loopne),
            // Call/Return
            "call" => Some(Opcode::Call), "ret" => Some(Opcode::Ret),
            "leave" => Some(Opcode::Leave),
            // String ops
            "rep movsb" => Some(Opcode::RepMovsb), "rep movsw" => Some(Opcode::RepMovsw),
            "rep movsd" => Some(Opcode::RepMovsd), "rep movsq" => Some(Opcode::RepMovsq),
            "rep stosb" => Some(Opcode::RepStosb), "rep stosw" => Some(Opcode::RepStosw),
            "rep stosd" => Some(Opcode::RepStosd), "rep stosq" => Some(Opcode::RepStosq),
            "scasb" => Some(Opcode::Scasb), "scasw" => Some(Opcode::Scasw),
            "scasd" => Some(Opcode::Scasd),
            "repe cmpsb" => Some(Opcode::RepeCmpsb), "repe cmpsw" => Some(Opcode::RepeCmpsw),
            "repe cmpsd" => Some(Opcode::RepeCmpsd),
            "repne scasb" => Some(Opcode::RepneScasb), "repne scasw" => Some(Opcode::RepneScasw),
            "repne scasd" => Some(Opcode::RepneScasd),
            "movsb" => Some(Opcode::Movsb), "movsw" => Some(Opcode::Movsw),
            "movsd" => Some(Opcode::Movsd), "movsq" => Some(Opcode::Movsq),
            "stosb" => Some(Opcode::Stosb), "stosw" => Some(Opcode::Stosw),
            "stosd" => Some(Opcode::Stosd), "stosq" => Some(Opcode::Stosq),
            "cmpsb" => Some(Opcode::Cmpsb), "cmpsw" => Some(Opcode::Cmpsw),
            "cmpsd" => Some(Opcode::Cmpsd),
            "lodsb" => Some(Opcode::Lodsb), "lodsw" => Some(Opcode::Lodsw),
            "lodsd" => Some(Opcode::Lodsd), "lodsq" => Some(Opcode::Lodsq),
            "cld" => Some(Opcode::Cld), "std" => Some(Opcode::Std),
            // System
            "syscall" => Some(Opcode::Syscall), "int" => Some(Opcode::Int),
            "hlt" => Some(Opcode::Hlt), "cli" => Some(Opcode::Cli),
            "sti" => Some(Opcode::Sti), "nop" => Some(Opcode::Nop),
            "cpuid" => Some(Opcode::Cpuid), "iretq" => Some(Opcode::Iretq),
            "rdtsc" => Some(Opcode::Rdtsc), "rdtscp" => Some(Opcode::Rdtscp),
            // Bit manipulation
            "bt" => Some(Opcode::Bt), "bts" => Some(Opcode::Bts),
            "btr" => Some(Opcode::Btr), "btc" => Some(Opcode::Btc),
            "bsf" => Some(Opcode::Bsf), "bsr" => Some(Opcode::Bsr),
            "popcnt" => Some(Opcode::Popcnt), "lzcnt" => Some(Opcode::Lzcnt),
            "tzcnt" => Some(Opcode::Tzcnt),
            // Byte swap / atomic
            "bswap" => Some(Opcode::Bswap), "xadd" => Some(Opcode::Xadd),
            "cmpxchg" => Some(Opcode::Cmpxchg),
            "cmpxchg8b" => Some(Opcode::Cmpxchg8b),
            "cmpxchg16b" => Some(Opcode::Cmpxchg16b),
            // SETcc
            "sete" | "setz" => Some(Opcode::Sete),
            "setne" | "setnz" => Some(Opcode::Setne),
            "setl" | "setnge" => Some(Opcode::Setl),
            "setle" | "setng" => Some(Opcode::Setle),
            "setg" | "setnle" => Some(Opcode::Setg),
            "setge" | "setnl" => Some(Opcode::Setge),
            "setb" | "setc" | "setnae" => Some(Opcode::Setb),
            "setbe" | "setna" => Some(Opcode::Setbe),
            "seta" | "setnbe" => Some(Opcode::Seta),
            "setae" | "setnb" | "setnc" => Some(Opcode::Setae),
            "sets" => Some(Opcode::Sets), "setns" => Some(Opcode::Setns),
            // Conditional moves
            "cmove" | "cmovz" => Some(Opcode::Cmove),
            "cmovne" | "cmovnz" => Some(Opcode::Cmovne),
            "cmovl" => Some(Opcode::Cmovl), "cmovle" => Some(Opcode::Cmovle),
            "cmovg" => Some(Opcode::Cmovg), "cmovge" => Some(Opcode::Cmovge),
            "cmovb" => Some(Opcode::Cmovb), "cmovbe" => Some(Opcode::Cmovbe),
            "cmova" => Some(Opcode::Cmova), "cmovae" => Some(Opcode::Cmovae),
            "cmovs" => Some(Opcode::Cmovs), "cmovns" => Some(Opcode::Cmovns),
            // SSE packed float
            "movaps" => Some(Opcode::Movaps), "movups" => Some(Opcode::Movups),
            "addps" => Some(Opcode::Addps), "subps" => Some(Opcode::Subps),
            "mulps" => Some(Opcode::Mulps), "divps" => Some(Opcode::Divps),
            "minps" => Some(Opcode::Minps), "maxps" => Some(Opcode::Maxps),
            "xorps" => Some(Opcode::Xorps),
            "andps" => Some(Opcode::Andps), "orps" => Some(Opcode::Orps),
            "andnps" => Some(Opcode::Andnps),
            "sqrtps" => Some(Opcode::Sqrtps), "rsqrtps" => Some(Opcode::Rsqrtps),
            "rcpps" => Some(Opcode::Rcpps),
            "cmpps" => Some(Opcode::Cmpps), "shufps" => Some(Opcode::Shufps),
            "unpcklps" => Some(Opcode::Unpcklps), "unpckhps" => Some(Opcode::Unpckhps),
            // SSE scalar float
            "movss" => Some(Opcode::Movss), "addss" => Some(Opcode::Addss),
            "subss" => Some(Opcode::Subss), "mulss" => Some(Opcode::Mulss),
            "divss" => Some(Opcode::Divss), "sqrtss" => Some(Opcode::Sqrtss),
            "minss" => Some(Opcode::Minss), "maxss" => Some(Opcode::Maxss),
            "cmpss" => Some(Opcode::Cmpss),
            "comiss" => Some(Opcode::Comiss), "ucomiss" => Some(Opcode::Ucomiss),
            "cvtsi2ss" => Some(Opcode::Cvtsi2ss), "cvtss2si" => Some(Opcode::Cvtss2si),
            "cvttss2si" => Some(Opcode::Cvttss2si),
            // SSE2 packed double
            "movapd" => Some(Opcode::Movapd), "movupd" => Some(Opcode::Movupd),
            "addpd" => Some(Opcode::Addpd), "subpd" => Some(Opcode::Subpd),
            "mulpd" => Some(Opcode::Mulpd), "divpd" => Some(Opcode::Divpd),
            "minpd" => Some(Opcode::Minpd), "maxpd" => Some(Opcode::Maxpd),
            "xorpd" => Some(Opcode::Xorpd),
            "andpd" => Some(Opcode::Andpd), "orpd" => Some(Opcode::Orpd),
            "andnpd" => Some(Opcode::Andnpd),
            "sqrtpd" => Some(Opcode::Sqrtpd), "cmppd" => Some(Opcode::Cmppd),
            "shufpd" => Some(Opcode::Shufpd),
            // SSE2 scalar double
            "addsd" => Some(Opcode::Addsd), "subsd" => Some(Opcode::Subsd),
            "mulsd" => Some(Opcode::Mulsd), "divsd" => Some(Opcode::Divsd),
            "sqrtsd" => Some(Opcode::Sqrtsd),
            "minsd" => Some(Opcode::Minsd), "maxsd" => Some(Opcode::Maxsd),
            "comisd" => Some(Opcode::Comisd), "ucomisd" => Some(Opcode::Ucomisd),
            "cvtsi2sd" => Some(Opcode::Cvtsi2sd), "cvtsd2si" => Some(Opcode::Cvtsd2si),
            "cvttsd2si" => Some(Opcode::Cvttsd2si),
            "cvtss2sd" => Some(Opcode::Cvtss2sd), "cvtsd2ss" => Some(Opcode::Cvtsd2ss),
            // SSE2 integer
            "movdqa" => Some(Opcode::Movdqa), "movdqu" => Some(Opcode::Movdqu),
            "paddb" => Some(Opcode::Paddb), "paddw" => Some(Opcode::Paddw),
            "paddd" => Some(Opcode::Paddd), "paddq" => Some(Opcode::Paddq),
            "psubb" => Some(Opcode::Psubb), "psubw" => Some(Opcode::Psubw),
            "psubd" => Some(Opcode::Psubd), "psubq" => Some(Opcode::Psubq),
            "pmullw" => Some(Opcode::Pmullw), "pmulld" => Some(Opcode::Pmulld),
            "pmuludq" => Some(Opcode::Pmuludq),
            "pand" => Some(Opcode::Pand), "por" => Some(Opcode::Por),
            "pxor" => Some(Opcode::Pxor), "pandn" => Some(Opcode::Pandn),
            "pcmpeqb" => Some(Opcode::Pcmpeqb), "pcmpeqw" => Some(Opcode::Pcmpeqw),
            "pcmpeqd" => Some(Opcode::Pcmpeqd),
            "pcmpgtb" => Some(Opcode::Pcmpgtb), "pcmpgtw" => Some(Opcode::Pcmpgtw),
            "pcmpgtd" => Some(Opcode::Pcmpgtd),
            "psllw" => Some(Opcode::Psllw), "pslld" => Some(Opcode::Pslld),
            "psllq" => Some(Opcode::Psllq),
            "psrlw" => Some(Opcode::Psrlw), "psrld" => Some(Opcode::Psrld),
            "psrlq" => Some(Opcode::Psrlq),
            "psraw" => Some(Opcode::Psraw), "psrad" => Some(Opcode::Psrad),
            "pshufb" => Some(Opcode::Pshufb), "pshufd" => Some(Opcode::Pshufd),
            "pshufhw" => Some(Opcode::Pshufhw), "pshuflw" => Some(Opcode::Pshuflw),
            "punpcklbw" => Some(Opcode::Punpcklbw), "punpckhbw" => Some(Opcode::Punpckhbw),
            "punpcklwd" => Some(Opcode::Punpcklwd), "punpckhwd" => Some(Opcode::Punpckhwd),
            "punpckldq" => Some(Opcode::Punpckldq), "punpckhdq" => Some(Opcode::Punpckhdq),
            // SSE data movement
            "movd" => Some(Opcode::Movd), "movq" => Some(Opcode::Movq),
            // AVX packed float
            "vmovaps" => Some(Opcode::Vmovaps), "vmovups" => Some(Opcode::Vmovups),
            "vaddps" => Some(Opcode::Vaddps), "vsubps" => Some(Opcode::Vsubps),
            "vmulps" => Some(Opcode::Vmulps), "vdivps" => Some(Opcode::Vdivps),
            "vxorps" => Some(Opcode::Vxorps),
            "vandps" => Some(Opcode::Vandps), "vorps" => Some(Opcode::Vorps),
            "vandnps" => Some(Opcode::Vandnps),
            "vminps" => Some(Opcode::Vminps), "vmaxps" => Some(Opcode::Vmaxps),
            "vsqrtps" => Some(Opcode::Vsqrtps),
            "vcmpps" => Some(Opcode::Vcmpps), "vshufps" => Some(Opcode::Vshufps),
            // AVX scalar float
            "vmovss" => Some(Opcode::Vmovss), "vaddss" => Some(Opcode::Vaddss),
            "vsubss" => Some(Opcode::Vsubss), "vmulss" => Some(Opcode::Vmulss),
            "vdivss" => Some(Opcode::Vdivss), "vsqrtss" => Some(Opcode::Vsqrtss),
            // AVX packed double
            "vmovapd" => Some(Opcode::Vmovapd), "vmovupd" => Some(Opcode::Vmovupd),
            "vaddpd" => Some(Opcode::Vaddpd), "vsubpd" => Some(Opcode::Vsubpd),
            "vmulpd" => Some(Opcode::Vmulpd), "vdivpd" => Some(Opcode::Vdivpd),
            "vxorpd" => Some(Opcode::Vxorpd),
            // AVX scalar double
            "vmovsd" => Some(Opcode::Vmovsd), "vaddsd" => Some(Opcode::Vaddsd),
            "vsubsd" => Some(Opcode::Vsubsd), "vmulsd" => Some(Opcode::Vmulsd),
            "vdivsd" => Some(Opcode::Vdivsd), "vsqrtsd" => Some(Opcode::Vsqrtsd),
            // AVX integer
            "vmovdqa" => Some(Opcode::Vmovdqa), "vmovdqu" => Some(Opcode::Vmovdqu),
            "vpaddb" => Some(Opcode::Vpaddb), "vpaddw" => Some(Opcode::Vpaddw),
            "vpaddd" => Some(Opcode::Vpaddd), "vpaddq" => Some(Opcode::Vpaddq),
            "vpsubb" => Some(Opcode::Vpsubb), "vpsubw" => Some(Opcode::Vpsubw),
            "vpsubd" => Some(Opcode::Vpsubd), "vpsubq" => Some(Opcode::Vpsubq),
            "vpmullw" => Some(Opcode::Vpmullw), "vpmulld" => Some(Opcode::Vpmulld),
            "vpand" => Some(Opcode::Vpand), "vpor" => Some(Opcode::Vpor),
            "vpxor" => Some(Opcode::Vpxor), "vpandn" => Some(Opcode::Vpandn),
            // AVX special
            "vdpps" => Some(Opcode::Vdpps), "vdppd" => Some(Opcode::Vdppd),
            "vbroadcastss" => Some(Opcode::Vbroadcastss),
            "vbroadcastsd" => Some(Opcode::Vbroadcastsd),
            "vperm2f128" => Some(Opcode::Vperm2f128),
            "vinsertf128" => Some(Opcode::Vinsertf128),
            "vextractf128" => Some(Opcode::Vextractf128),
            "vzeroall" => Some(Opcode::Vzeroall), "vzeroupper" => Some(Opcode::Vzeroupper),
            // FMA
            "vfmadd132ps" => Some(Opcode::Vfmadd132ps),
            "vfmadd213ps" => Some(Opcode::Vfmadd213ps),
            "vfmadd231ps" => Some(Opcode::Vfmadd231ps),
            "vfmadd132ss" => Some(Opcode::Vfmadd132ss),
            "vfmadd213ss" => Some(Opcode::Vfmadd213ss),
            "vfmadd231ss" => Some(Opcode::Vfmadd231ss),
            "vfmadd132pd" => Some(Opcode::Vfmadd132pd),
            "vfmadd213pd" => Some(Opcode::Vfmadd213pd),
            "vfmadd231pd" => Some(Opcode::Vfmadd231pd),
            "vfmadd132sd" => Some(Opcode::Vfmadd132sd),
            "vfmadd213sd" => Some(Opcode::Vfmadd213sd),
            "vfmadd231sd" => Some(Opcode::Vfmadd231sd),
            // Misc
            "cqo" => Some(Opcode::Cqo), "cdq" => Some(Opcode::Cdq),
            "cbw" => Some(Opcode::Cbw), "cwd" => Some(Opcode::Cwd),
            "cwde" => Some(Opcode::Cwde),
            "lahf" => Some(Opcode::Lahf), "sahf" => Some(Opcode::Sahf),
            "xlat" | "xlatb" => Some(Opcode::Xlat),
            _ => None,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            // Movement
            Opcode::Mov => "mov", Opcode::Movzx => "movzx", Opcode::Movsx => "movsx",
            Opcode::Lea => "lea", Opcode::Xchg => "xchg",
            Opcode::Push => "push", Opcode::Pop => "pop",
            Opcode::Pushf => "pushfq", Opcode::Popf => "popfq",
            Opcode::Pushad => "pushad", Opcode::Popad => "popad",
            Opcode::Enter => "enter",
            // Arithmetic
            Opcode::Add => "add", Opcode::Sub => "sub",
            Opcode::Mul => "mul", Opcode::Imul => "imul",
            Opcode::Div => "div", Opcode::Idiv => "idiv",
            Opcode::Inc => "inc", Opcode::Dec => "dec", Opcode::Neg => "neg",
            Opcode::Adc => "adc", Opcode::Sbb => "sbb",
            // Logic
            Opcode::And => "and", Opcode::Or => "or",
            Opcode::Xor => "xor", Opcode::Not => "not",
            Opcode::Shl => "shl", Opcode::Shr => "shr",
            Opcode::Sar => "sar", Opcode::Rol => "rol", Opcode::Ror => "ror",
            Opcode::Rcl => "rcl", Opcode::Rcr => "rcr",
            // Comparison
            Opcode::Cmp => "cmp", Opcode::Test => "test",
            // Jumps
            Opcode::Jmp => "jmp", Opcode::Je => "je", Opcode::Jne => "jne",
            Opcode::Jl => "jl", Opcode::Jle => "jle",
            Opcode::Jg => "jg", Opcode::Jge => "jge",
            Opcode::Jb => "jb", Opcode::Jbe => "jbe",
            Opcode::Ja => "ja", Opcode::Jae => "jae",
            Opcode::Js => "js", Opcode::Jns => "jns",
            Opcode::Jo => "jo", Opcode::Jno => "jno",
            Opcode::Jp => "jp", Opcode::Jnp => "jnp",
            Opcode::Jcxz => "jcxz", Opcode::Jecxz => "jecxz", Opcode::Jrcxz => "jrcxz",
            // Loop
            Opcode::Loop => "loop", Opcode::Loope => "loope", Opcode::Loopne => "loopne",
            // Call/Return
            Opcode::Call => "call", Opcode::Ret => "ret", Opcode::Leave => "leave",
            // String ops
            Opcode::RepMovsb => "rep movsb", Opcode::RepMovsw => "rep movsw",
            Opcode::RepMovsd => "rep movsd", Opcode::RepMovsq => "rep movsq",
            Opcode::RepStosb => "rep stosb", Opcode::RepStosw => "rep stosw",
            Opcode::RepStosd => "rep stosd", Opcode::RepStosq => "rep stosq",
            Opcode::Scasb => "scasb", Opcode::Scasw => "scasw", Opcode::Scasd => "scasd",
            Opcode::RepeCmpsb => "repe cmpsb", Opcode::RepeCmpsw => "repe cmpsw",
            Opcode::RepeCmpsd => "repe cmpsd",
            Opcode::RepneScasb => "repne scasb", Opcode::RepneScasw => "repne scasw",
            Opcode::RepneScasd => "repne scasd",
            Opcode::Movsb => "movsb", Opcode::Movsw => "movsw",
            Opcode::Movsd => "movsd", Opcode::Movsq => "movsq",
            Opcode::Stosb => "stosb", Opcode::Stosw => "stosw",
            Opcode::Stosd => "stosd", Opcode::Stosq => "stosq",
            Opcode::Cmpsb => "cmpsb", Opcode::Cmpsw => "cmpsw", Opcode::Cmpsd => "cmpsd",
            Opcode::Lodsb => "lodsb", Opcode::Lodsw => "lodsw",
            Opcode::Lodsd => "lodsd", Opcode::Lodsq => "lodsq",
            Opcode::Cld => "cld", Opcode::Std => "std",
            // System
            Opcode::Syscall => "syscall", Opcode::Int => "int",
            Opcode::Hlt => "hlt", Opcode::Cli => "cli",
            Opcode::Sti => "sti", Opcode::Nop => "nop",
            Opcode::Cpuid => "cpuid", Opcode::Iretq => "iretq",
            Opcode::Rdtsc => "rdtsc", Opcode::Rdtscp => "rdtscp",
            // Bit manipulation
            Opcode::Bt => "bt", Opcode::Bts => "bts",
            Opcode::Btr => "btr", Opcode::Btc => "btc",
            Opcode::Bsf => "bsf", Opcode::Bsr => "bsr",
            Opcode::Popcnt => "popcnt", Opcode::Lzcnt => "lzcnt",
            Opcode::Tzcnt => "tzcnt",
            // Byte swap / atomic
            Opcode::Bswap => "bswap", Opcode::Xadd => "xadd",
            Opcode::Cmpxchg => "cmpxchg",
            Opcode::Cmpxchg8b => "cmpxchg8b", Opcode::Cmpxchg16b => "cmpxchg16b",
            // SETcc
            Opcode::Sete => "sete", Opcode::Setne => "setne",
            Opcode::Setl => "setl", Opcode::Setle => "setle",
            Opcode::Setg => "setg", Opcode::Setge => "setge",
            Opcode::Setb => "setb", Opcode::Setbe => "setbe",
            Opcode::Seta => "seta", Opcode::Setae => "setae",
            Opcode::Sets => "sets", Opcode::Setns => "setns",
            // Conditional moves
            Opcode::Cmove => "cmove", Opcode::Cmovne => "cmovne",
            Opcode::Cmovl => "cmovl", Opcode::Cmovle => "cmovle",
            Opcode::Cmovg => "cmovg", Opcode::Cmovge => "cmovge",
            Opcode::Cmovb => "cmovb", Opcode::Cmovbe => "cmovbe",
            Opcode::Cmova => "cmova", Opcode::Cmovae => "cmovae",
            Opcode::Cmovs => "cmovs", Opcode::Cmovns => "cmovns",
            // SSE packed float
            Opcode::Movaps => "movaps", Opcode::Movups => "movups",
            Opcode::Addps => "addps", Opcode::Subps => "subps",
            Opcode::Mulps => "mulps", Opcode::Divps => "divps",
            Opcode::Minps => "minps", Opcode::Maxps => "maxps",
            Opcode::Xorps => "xorps",
            Opcode::Andps => "andps", Opcode::Orps => "orps", Opcode::Andnps => "andnps",
            Opcode::Sqrtps => "sqrtps", Opcode::Rsqrtps => "rsqrtps", Opcode::Rcpps => "rcpps",
            Opcode::Cmpps => "cmpps", Opcode::Shufps => "shufps",
            Opcode::Unpcklps => "unpcklps", Opcode::Unpckhps => "unpckhps",
            // SSE scalar float
            Opcode::Movss => "movss", Opcode::Addss => "addss",
            Opcode::Subss => "subss", Opcode::Mulss => "mulss",
            Opcode::Divss => "divss", Opcode::Sqrtss => "sqrtss",
            Opcode::Minss => "minss", Opcode::Maxss => "maxss", Opcode::Cmpss => "cmpss",
            Opcode::Comiss => "comiss", Opcode::Ucomiss => "ucomiss",
            Opcode::Cvtsi2ss => "cvtsi2ss", Opcode::Cvtss2si => "cvtss2si",
            Opcode::Cvttss2si => "cvttss2si",
            // SSE2 packed double
            Opcode::Movapd => "movapd", Opcode::Movupd => "movupd",
            Opcode::Addpd => "addpd", Opcode::Subpd => "subpd",
            Opcode::Mulpd => "mulpd", Opcode::Divpd => "divpd",
            Opcode::Minpd => "minpd", Opcode::Maxpd => "maxpd",
            Opcode::Xorpd => "xorpd",
            Opcode::Andpd => "andpd", Opcode::Orpd => "orpd", Opcode::Andnpd => "andnpd",
            Opcode::Sqrtpd => "sqrtpd", Opcode::Cmppd => "cmppd", Opcode::Shufpd => "shufpd",
            // SSE2 scalar double
            Opcode::Movsd2 => "movsd", Opcode::Addsd => "addsd", Opcode::Subsd => "subsd",
            Opcode::Mulsd => "mulsd", Opcode::Divsd => "divsd", Opcode::Sqrtsd => "sqrtsd",
            Opcode::Minsd => "minsd", Opcode::Maxsd => "maxsd", Opcode::Cmpsd2 => "cmpsd",
            Opcode::Comisd => "comisd", Opcode::Ucomisd => "ucomisd",
            Opcode::Cvtsi2sd => "cvtsi2sd", Opcode::Cvtsd2si => "cvtsd2si",
            Opcode::Cvttsd2si => "cvttsd2si",
            Opcode::Cvtss2sd => "cvtss2sd", Opcode::Cvtsd2ss => "cvtsd2ss",
            // SSE2 integer
            Opcode::Movdqa => "movdqa", Opcode::Movdqu => "movdqu",
            Opcode::Paddb => "paddb", Opcode::Paddw => "paddw",
            Opcode::Paddd => "paddd", Opcode::Paddq => "paddq",
            Opcode::Psubb => "psubb", Opcode::Psubw => "psubw",
            Opcode::Psubd => "psubd", Opcode::Psubq => "psubq",
            Opcode::Pmullw => "pmullw", Opcode::Pmulld => "pmulld",
            Opcode::Pmuludq => "pmuludq",
            Opcode::Pand => "pand", Opcode::Por => "por",
            Opcode::Pxor => "pxor", Opcode::Pandn => "pandn",
            Opcode::Pcmpeqb => "pcmpeqb", Opcode::Pcmpeqw => "pcmpeqw",
            Opcode::Pcmpeqd => "pcmpeqd",
            Opcode::Pcmpgtb => "pcmpgtb", Opcode::Pcmpgtw => "pcmpgtw",
            Opcode::Pcmpgtd => "pcmpgtd",
            Opcode::Psllw => "psllw", Opcode::Pslld => "pslld", Opcode::Psllq => "psllq",
            Opcode::Psrlw => "psrlw", Opcode::Psrld => "psrld", Opcode::Psrlq => "psrlq",
            Opcode::Psraw => "psraw", Opcode::Psrad => "psrad",
            Opcode::Pshufb => "pshufb", Opcode::Pshufd => "pshufd",
            Opcode::Pshufhw => "pshufhw", Opcode::Pshuflw => "pshuflw",
            Opcode::Punpcklbw => "punpcklbw", Opcode::Punpckhbw => "punpckhbw",
            Opcode::Punpcklwd => "punpcklwd", Opcode::Punpckhwd => "punpckhwd",
            Opcode::Punpckldq => "punpckldq", Opcode::Punpckhdq => "punpckhdq",
            // SSE data movement
            Opcode::Movd => "movd", Opcode::Movq => "movq",
            // AVX packed float
            Opcode::Vmovaps => "vmovaps", Opcode::Vmovups => "vmovups",
            Opcode::Vaddps => "vaddps", Opcode::Vsubps => "vsubps",
            Opcode::Vmulps => "vmulps", Opcode::Vdivps => "vdivps",
            Opcode::Vxorps => "vxorps",
            Opcode::Vandps => "vandps", Opcode::Vorps => "vorps", Opcode::Vandnps => "vandnps",
            Opcode::Vminps => "vminps", Opcode::Vmaxps => "vmaxps",
            Opcode::Vsqrtps => "vsqrtps",
            Opcode::Vcmpps => "vcmpps", Opcode::Vshufps => "vshufps",
            // AVX scalar float
            Opcode::Vmovss => "vmovss", Opcode::Vaddss => "vaddss",
            Opcode::Vsubss => "vsubss", Opcode::Vmulss => "vmulss",
            Opcode::Vdivss => "vdivss", Opcode::Vsqrtss => "vsqrtss",
            // AVX packed double
            Opcode::Vmovapd => "vmovapd", Opcode::Vmovupd => "vmovupd",
            Opcode::Vaddpd => "vaddpd", Opcode::Vsubpd => "vsubpd",
            Opcode::Vmulpd => "vmulpd", Opcode::Vdivpd => "vdivpd",
            Opcode::Vxorpd => "vxorpd",
            // AVX scalar double
            Opcode::Vmovsd => "vmovsd", Opcode::Vaddsd => "vaddsd",
            Opcode::Vsubsd => "vsubsd", Opcode::Vmulsd => "vmulsd",
            Opcode::Vdivsd => "vdivsd", Opcode::Vsqrtsd => "vsqrtsd",
            // AVX integer
            Opcode::Vmovdqa => "vmovdqa", Opcode::Vmovdqu => "vmovdqu",
            Opcode::Vpaddb => "vpaddb", Opcode::Vpaddw => "vpaddw",
            Opcode::Vpaddd => "vpaddd", Opcode::Vpaddq => "vpaddq",
            Opcode::Vpsubb => "vpsubb", Opcode::Vpsubw => "vpsubw",
            Opcode::Vpsubd => "vpsubd", Opcode::Vpsubq => "vpsubq",
            Opcode::Vpmullw => "vpmullw", Opcode::Vpmulld => "vpmulld",
            Opcode::Vpand => "vpand", Opcode::Vpor => "vpor",
            Opcode::Vpxor => "vpxor", Opcode::Vpandn => "vpandn",
            // AVX special
            Opcode::Vdpps => "vdpps", Opcode::Vdppd => "vdppd",
            Opcode::Vbroadcastss => "vbroadcastss", Opcode::Vbroadcastsd => "vbroadcastsd",
            Opcode::Vperm2f128 => "vperm2f128",
            Opcode::Vinsertf128 => "vinsertf128", Opcode::Vextractf128 => "vextractf128",
            Opcode::Vzeroall => "vzeroall", Opcode::Vzeroupper => "vzeroupper",
            // FMA
            Opcode::Vfmadd132ps => "vfmadd132ps", Opcode::Vfmadd213ps => "vfmadd213ps",
            Opcode::Vfmadd231ps => "vfmadd231ps",
            Opcode::Vfmadd132ss => "vfmadd132ss", Opcode::Vfmadd213ss => "vfmadd213ss",
            Opcode::Vfmadd231ss => "vfmadd231ss",
            Opcode::Vfmadd132pd => "vfmadd132pd", Opcode::Vfmadd213pd => "vfmadd213pd",
            Opcode::Vfmadd231pd => "vfmadd231pd",
            Opcode::Vfmadd132sd => "vfmadd132sd", Opcode::Vfmadd213sd => "vfmadd213sd",
            Opcode::Vfmadd231sd => "vfmadd231sd",
            // Misc
            Opcode::Cqo => "cqo", Opcode::Cdq => "cdq", Opcode::Cbw => "cbw",
            Opcode::Cwd => "cwd", Opcode::Cwde => "cwde",
            Opcode::Lahf => "lahf", Opcode::Sahf => "sahf",
            Opcode::Xlat => "xlatb",
        }
    }
}

/// An operand to an instruction
#[derive(Debug, Clone, PartialEq)]
pub enum Operand {
    Reg(Register),
    Imm(i64),
    Label(String),
    Memory {
        base: Option<Register>,
        index: Option<Register>,
        scale: u8,
        disp: i64,
    },
    StringLit(String),
}

/// A single normalized instruction in the IR
#[derive(Debug, Clone)]
pub struct Instruction {
    pub opcode: Opcode,
    pub operands: Vec<Operand>,
}

impl Instruction {
    pub fn new(opcode: Opcode, operands: Vec<Operand>) -> Self {
        Self { opcode, operands }
    }

    pub fn zero(opcode: Opcode) -> Self {
        Self { opcode, operands: vec![] }
    }

    pub fn one(opcode: Opcode, op: Operand) -> Self {
        Self { opcode, operands: vec![op] }
    }

    pub fn two(opcode: Opcode, dst: Operand, src: Operand) -> Self {
        Self { opcode, operands: vec![dst, src] }
    }

    pub fn three(opcode: Opcode, a: Operand, b: Operand, c: Operand) -> Self {
        Self { opcode, operands: vec![a, b, c] }
    }
}
