mod frontend;
mod ir;
mod targets;
mod emitters;
mod macros;

use std::env;
use std::fs;
use std::process;

use frontend::lexer::Lexer;
use frontend::parser::Parser;
use emitters::{get_emitter, OutputFormat};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("ASM-BIB v1.0 — Universal ASM with Python-like syntax 💀🦈");
        eprintln!("Usage: asm-bib <archivo.pasm> [--nasm|--gas|--masm|--fasm|--flat] [-o output]");
        eprintln!();
        eprintln!("Options:");
        eprintln!("  --nasm     Export NASM Intel syntax (.asm)");
        eprintln!("  --gas      Export GAS AT&T syntax (.s)");
        eprintln!("  --masm     Export MASM Microsoft syntax (.asm)");
        eprintln!("  --fasm     Export FASM syntax (.asm)");
        eprintln!("  --flat     Export flat binary (bootloader) (.bin)");
        eprintln!("  -o FILE    Output file (default: stdout)");
        process::exit(1);
    }

    let input_file = &args[1];
    let mut output_format = OutputFormat::Nasm; // default
    let mut output_file: Option<String> = None;

    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "--nasm" => output_format = OutputFormat::Nasm,
            "--gas" => output_format = OutputFormat::Gas,
            "--masm" => output_format = OutputFormat::Masm,
            "--fasm" => output_format = OutputFormat::Fasm,
            "--flat" => output_format = OutputFormat::Flat,
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

    // Read input
    let source = match fs::read_to_string(input_file) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: cannot read '{}': {}", input_file, e);
            process::exit(1);
        }
    };

    // Lexer
    let mut lexer = Lexer::new(&source);
    let tokens = lexer.tokenize();

    // Parser → IR
    let mut parser = Parser::new(tokens);
    let program = match parser.parse() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("error: parse failed: {}", e);
            process::exit(1);
        }
    };

    // Emit to selected format
    let emitter = get_emitter(output_format);
    let output = emitter.emit(&program);

    // Write output
    match output_file {
        Some(path) => {
            if let Err(e) = fs::write(&path, &output) {
                eprintln!("error: cannot write '{}': {}", path, e);
                process::exit(1);
            }
            eprintln!("OK → {} ({:?})", path, output_format);
        }
        None => {
            print!("{}", output);
        }
    }
}
