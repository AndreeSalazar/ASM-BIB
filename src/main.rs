mod frontend;
mod ir;
mod targets;
mod emitters;
mod macros;

use std::env;
use std::fs;
use std::process;
use std::time::Instant;

use frontend::lexer::{Lexer, Token};
use frontend::parser::Parser;
use emitters::{get_emitter, OutputFormat};
use ir::*;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("ASM-BIB v2.0 — x86 ASM with Python-like syntax 💀🦈");
        eprintln!("Usage: asm-bib <archivo.pasm> [--nasm|--masm] [--step] [-o output]");
        eprintln!();
        eprintln!("Options:");
        eprintln!("  --nasm     Export NASM Intel syntax (.asm)");
        eprintln!("  --masm     Export MASM Microsoft syntax (.asm)");
        eprintln!("  --step     Show pipeline steps (debug mode)");
        eprintln!("  -o FILE    Output file (default: stdout)");
        process::exit(1);
    }

    let input_file = &args[1];
    let mut output_format = OutputFormat::Nasm; // default
    let mut output_file: Option<String> = None;
    let mut step_mode = false;

    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "--nasm" => output_format = OutputFormat::Nasm,
            "--masm" => output_format = OutputFormat::Masm,
            "--step" => step_mode = true,
            "-o" => {
                i += 1;
                if i < args.len() {
                    output_file = Some(args[i].clone());
                } else {
                    eprintln!("error: -o requires a filename");
                    process::exit(1);
                }
            }
            other => {
                eprintln!("warning: unknown option '{}'", other);
            }
        }
        i += 1;
    }

    let total_start = Instant::now();

    // ── Step 1: Read source ─────────────────────────────────────────────
    let t0 = Instant::now();
    let source = match fs::read_to_string(input_file) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: cannot read '{}': {}", input_file, e);
            process::exit(1);
        }
    };
    let read_us = t0.elapsed().as_micros();

    if step_mode {
        let line_count = source.lines().count();
        let byte_count = source.len();
        eprintln!("┌─ STEP 1 · Source ─────────────────────────────────────────");
        eprintln!("│  file:  {}", input_file);
        eprintln!("│  lines: {}    bytes: {}", line_count, byte_count);
        eprintln!("│  time:  {}µs", read_us);
        eprintln!("│");
    }

    // ── Step 2: Lexer ───────────────────────────────────────────────────
    let t1 = Instant::now();
    let mut lexer = Lexer::new(&source);
    let tokens = lexer.tokenize();
    let lex_us = t1.elapsed().as_micros();

    if step_mode {
        let total_tokens = tokens.len();
        let idents = tokens.iter().filter(|t| matches!(t, Token::Ident(_))).count();
        let integers = tokens.iter().filter(|t| matches!(t, Token::Integer(_) | Token::HexInteger(_))).count();
        let floats = tokens.iter().filter(|t| matches!(t, Token::FloatLiteral(_))).count();
        let strings = tokens.iter().filter(|t| matches!(t, Token::StringLiteral(_))).count();
        let keywords = tokens.iter().filter(|t| matches!(t,
            Token::Def | Token::Fn | Token::Struct | Token::Enum |
            Token::Use | Token::Let | Token::Const | Token::Static |
            Token::Extern | Token::Pub | Token::Inline | Token::Volatile |
            Token::Unsafe | Token::Naked | Token::Asm |
            Token::If | Token::Else | Token::While | Token::For |
            Token::Loop | Token::Break | Token::Continue | Token::Return
        )).count();
        let decorators = tokens.iter().filter(|t| matches!(t, Token::At)).count();
        let comments = tokens.iter().filter(|t| matches!(t, Token::Comment(_))).count();

        eprintln!("├─ STEP 2 · Lexer ──────────────────────────────────────────");
        eprintln!("│  tokens:     {}", total_tokens);
        eprintln!("│  idents:     {}    keywords: {}    decorators: {}", idents, keywords, decorators);
        eprintln!("│  integers:   {}    floats: {}    strings: {}", integers, floats, strings);
        eprintln!("│  comments:   {}", comments);
        eprintln!("│  time:       {}µs", lex_us);
        eprintln!("│");
    }

    // ── Step 3: Parser → IR ─────────────────────────────────────────────
    let t2 = Instant::now();
    let mut parser = Parser::new(tokens);
    let program = match parser.parse() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("error: parse failed: {}", e);
            process::exit(1);
        }
    };
    let parse_us = t2.elapsed().as_micros();

    if step_mode {
        let sec_count = program.sections.len();
        let func_count: usize = program.sections.iter().map(|s| s.functions.len()).sum();
        let data_count: usize = program.sections.iter().map(|s| s.data.len()).sum();
        let instr_count: usize = program.sections.iter()
            .flat_map(|s| s.functions.iter())
            .flat_map(|f| f.instructions.iter())
            .filter(|item| matches!(item, FunctionItem::Instruction(_)))
            .count();
        let label_count: usize = program.sections.iter()
            .flat_map(|s| s.functions.iter())
            .flat_map(|f| f.instructions.iter())
            .filter(|item| matches!(item, FunctionItem::Label(_)))
            .count();
        let exported: Vec<&str> = program.sections.iter()
            .flat_map(|s| s.functions.iter())
            .filter(|f| f.exported)
            .map(|f| f.name.as_str())
            .collect();

        eprintln!("├─ STEP 3 · Parser → IR ────────────────────────────────────");
        eprintln!("│  arch:       {:?}", program.arch);
        eprintln!("│  format:     {}", program.format);
        if let Some(org) = program.org {
            eprintln!("│  org:        0x{:X}", org);
        }
        eprintln!("│  sections:   {}", sec_count);
        for sec in &program.sections {
            let sec_name = match &sec.kind {
                SectionKind::Text => ".text",
                SectionKind::Data => ".data",
                SectionKind::Bss => ".bss",
                SectionKind::Rodata => ".rodata",
                SectionKind::Custom(n) => n.as_str(),
            };
            eprintln!("│    {} → {} functions, {} data items",
                sec_name, sec.functions.len(), sec.data.len());
        }
        eprintln!("│  functions:  {}    exported: [{}]", func_count, exported.join(", "));
        eprintln!("│  instructions: {}    labels: {}    data: {}",
            instr_count, label_count, data_count);
        if !program.structs.is_empty() {
            eprintln!("│  structs:    {}", program.structs.len());
        }
        if !program.enums.is_empty() {
            eprintln!("│  enums:      {}", program.enums.len());
        }
        if !program.constants.is_empty() {
            eprintln!("│  constants:  {}", program.constants.len());
        }
        if !program.externs.is_empty() {
            eprintln!("│  externs:    {}", program.externs.len());
        }
        eprintln!("│  time:       {}µs", parse_us);
        eprintln!("│");
    }

    // ── Step 4: Emit ────────────────────────────────────────────────────
    let t3 = Instant::now();
    let emitter = get_emitter(output_format);
    let output = emitter.emit(&program);
    let emit_us = t3.elapsed().as_micros();

    if step_mode {
        let out_lines = output.lines().count();
        let out_bytes = output.len();
        let format_name = match output_format {
            OutputFormat::Nasm => "NASM (Intel)",
            OutputFormat::Masm => "MASM (Microsoft)",
        };
        eprintln!("├─ STEP 4 · Emit → {} ─────────────────────────────", format_name);
        eprintln!("│  output:     {} lines, {} bytes", out_lines, out_bytes);
        eprintln!("│  time:       {}µs", emit_us);
        eprintln!("│");
    }

    // ── Write output ────────────────────────────────────────────────────
    match output_file {
        Some(ref path) => {
            if let Err(e) = fs::write(path, &output) {
                eprintln!("error: cannot write '{}': {}", path, e);
                process::exit(1);
            }
            if step_mode {
                let total_us = total_start.elapsed().as_micros();
                eprintln!("└─ DONE ────────────────────────────────────────────────────");
                eprintln!("   {} → {} ({:?})", input_file, path, output_format);
                eprintln!("   total: {}µs", total_us);
            } else {
                eprintln!("OK → {} ({:?})", path, output_format);
            }
        }
        None => {
            print!("{}", output);
            if step_mode {
                let total_us = total_start.elapsed().as_micros();
                eprintln!("└─ DONE ({:?}) total: {}µs ─────────────────────────────────",
                    output_format, total_us);
            }
        }
    }
}
