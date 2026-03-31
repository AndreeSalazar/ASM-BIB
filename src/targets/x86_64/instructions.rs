pub const X86_64_INSTRUCTIONS: &[&str] = &[
    "mov","movzx","movsx","lea","xchg","push","pop",
    "add","sub","mul","imul","div","idiv","inc","dec","neg",
    "and","or","xor","not","shl","shr","sar","rol","ror",
    "cmp","test",
    "jmp","je","jne","jl","jle","jg","jge","jb","jbe","ja","jae",
    "call","ret","leave",
    "syscall","int","hlt","cli","sti","nop","cpuid","iretq",
    "rep movsb","rep stosb","scasb",
    "movaps","movups","addps","mulps","xorps",
    "vmovaps","vaddps","vmulps",
];
