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
        eprintln!("  --build / --exe  Assemble + link → .exe (requires ml64/nasm + link.exe)");
        eprintln!("  --obj      Assemble only → .obj (requires ml64/nasm)");
        eprintln!("  -o FILE    Output file (default: stdout)");
        process::exit(1);
    }

    let input_file = &args[1];
    let mut output_format = OutputFormat::Nasm; // default
    let mut output_file: Option<String> = None;
    let mut step_mode = false;
    let mut build_exe = false;
    let mut build_dll = false;
    let mut build_obj = false;
    let mut internal_native = false;

    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "--nasm" => output_format = OutputFormat::Nasm,
            "--masm" => output_format = OutputFormat::Masm,
            "--native" => internal_native = true,
            "--step" => step_mode = true,
            "--build" | "--exe" => build_exe = true,
            "--dll" => { build_exe = true; build_dll = true; }
            "--obj" => build_obj = true,
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
            Token::Loop | Token::Break | Token::Continue | Token::Return |
            Token::Class
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

    // ── Build: assemble + link → .exe ───────────────────────────────────
    // Auto-detect VS Build Tools paths on Windows
    fn find_vs_tool(name: &str) -> Option<std::path::PathBuf> {
        // Check PATH first
        if let Ok(output) = std::process::Command::new("where").arg(name).output() {
            if output.status.success() {
                let s = String::from_utf8_lossy(&output.stdout);
                if let Some(line) = s.lines().next() {
                    let p = std::path::PathBuf::from(line.trim());
                    if p.exists() { return Some(p); }
                }
            }
        }
        // Search VS Build Tools directories
        let roots = [
            r"C:\Program Files (x86)\Microsoft Visual Studio",
            r"C:\Program Files\Microsoft Visual Studio",
        ];
        for root in &roots {
            let root_path = std::path::Path::new(root);
            if !root_path.exists() { continue; }
            if let Ok(entries) = std::fs::read_dir(root_path) {
                for year in entries.flatten() {
                    let host_dir = year.path().join("BuildTools").join("VC").join("Tools").join("MSVC");
                    if let Ok(versions) = std::fs::read_dir(&host_dir) {
                        for ver in versions.flatten() {
                            let tool = ver.path().join("bin").join("Hostx64").join("x64").join(name);
                            if tool.exists() { return Some(tool); }
                        }
                    }
                    // Also check Community/Professional/Enterprise
                    for edition in &["Community", "Professional", "Enterprise"] {
                        let host_dir = year.path().join(edition).join("VC").join("Tools").join("MSVC");
                        if let Ok(versions) = std::fs::read_dir(&host_dir) {
                            for ver in versions.flatten() {
                                let tool = ver.path().join("bin").join("Hostx64").join("x64").join(name);
                                if tool.exists() { return Some(tool); }
                            }
                        }
                    }
                }
            }
        }
        None
    }

    if build_exe || build_obj {
        let asm_path = match output_file {
            Some(ref p) => p.clone(),
            None => {
                // Auto-generate output filename
                let base = input_file.trim_end_matches(".pasm");
                let path = format!("{}.asm", base);
                fs::write(&path, &output).unwrap_or_else(|e| {
                    eprintln!("error: cannot write '{}': {}", path, e);
                    process::exit(1);
                });
                path
            }
        };

        let base_name = asm_path.trim_end_matches(".asm");
        let obj_path = format!("{}.obj", base_name);
        let exe_path = format!("{}.exe", base_name);

        eprintln!("┌─ BUILD ────────────────────────────────────────────────────");

        // Step 1: Assemble
        let assemble_ok = if internal_native {
            eprintln!("│  [1/2] (Internal Native) Generating {}", obj_path);
            let mut coff = crate::targets::coff::CoffObject::new(true); // Always x64 for now
            match coff.encode_program(&program) {
                Ok(bytes) => {
                    if let Err(e) = fs::write(&obj_path, bytes) {
                        eprintln!("│  ❌ Failed to write obj: {}", e);
                        false
                    } else {
                        eprintln!("│  ✅ Assembled OK (Native COFF)");
                        true
                    }
                }
                Err(e) => {
                    eprintln!("│  ❌ Native encoding failed: {}", e);
                    false
                }
            }
        } else {
            match output_format {
                OutputFormat::Masm => {
                    let ml64 = find_vs_tool("ml64.exe");
                    match ml64 {
                        Some(ref ml64_path) => {
                            eprintln!("│  [1/2] {} /c /nologo {}", ml64_path.display(), asm_path);
                            let result = std::process::Command::new(ml64_path)
                                .args(&["/c", "/nologo", &format!("/Fo{}", obj_path), &asm_path])
                                .output();
                            match result {
                                Ok(out) => {
                                    if !out.status.success() {
                                        let err = String::from_utf8_lossy(&out.stderr);
                                        let stdout = String::from_utf8_lossy(&out.stdout);
                                        eprintln!("│  ❌ ml64 failed:");
                                        for line in format!("{}{}", stdout, err).lines() {
                                            if !line.trim().is_empty() {
                                                eprintln!("│     {}", line);
                                            }
                                        }
                                        false
                                    } else {
                                        eprintln!("│  ✅ Assembled OK");
                                        true
                                    }
                                }
                                Err(e) => {
                                    eprintln!("│  ❌ ml64 exec error: {}", e);
                                    false
                                }
                            }
                        }
                        None => {
                            eprintln!("│  ❌ ml64.exe not found — install Visual Studio Build Tools");
                            false
                        }
                    }
                }
            OutputFormat::Nasm => {
                eprintln!("│  [1/2] nasm -f win64 {} -o {}", asm_path, obj_path);
                let result = std::process::Command::new("nasm")
                    .args(&["-f", "win64", &asm_path, "-o", &obj_path])
                    .output();
                match result {
                    Ok(out) => {
                        if !out.status.success() {
                            let err = String::from_utf8_lossy(&out.stderr);
                            eprintln!("│  ❌ nasm failed: {}", err);
                            false
                        } else {
                            eprintln!("│  ✅ Assembled OK");
                            true
                        }
                    }
                            Err(_) => {
                        eprintln!("│  ❌ nasm not found — install NASM and add to PATH");
                        false
                    }
                }
            }
        }
    };


        // Step 2: Link
        if assemble_ok && build_exe {
            let linker = find_vs_tool("link.exe");

            // Build LIB path for linker: MSVC libs + Windows SDK libs
            fn find_lib_paths() -> Vec<String> {
                let mut paths = Vec::new();
                let vs_roots = [
                    r"C:\Program Files (x86)\Microsoft Visual Studio",
                    r"C:\Program Files\Microsoft Visual Studio",
                ];
                for root in &vs_roots {
                    let root_path = std::path::Path::new(root);
                    if !root_path.exists() { continue; }
                    if let Ok(years) = std::fs::read_dir(root_path) {
                        for year in years.flatten() {
                            for edition in &["BuildTools", "Community", "Professional", "Enterprise"] {
                                let msvc = year.path().join(edition).join("VC").join("Tools").join("MSVC");
                                if let Ok(versions) = std::fs::read_dir(&msvc) {
                                    for ver in versions.flatten() {
                                        let lib = ver.path().join("lib").join("x64");
                                        if lib.exists() { paths.push(lib.to_string_lossy().to_string()); }
                                    }
                                }
                            }
                        }
                    }
                }
                // Windows SDK
                let sdk = std::path::Path::new(r"C:\Program Files (x86)\Windows Kits\10\Lib");
                if let Ok(versions) = std::fs::read_dir(sdk) {
                    for ver in versions.flatten() {
                        let um = ver.path().join("um").join("x64");
                        if um.exists() { paths.push(um.to_string_lossy().to_string()); }
                        let ucrt = ver.path().join("ucrt").join("x64");
                        if ucrt.exists() { paths.push(ucrt.to_string_lossy().to_string()); }
                    }
                }
                paths
            }

            match linker {
                Some(ref link_path) => {
                    let exe_disp = if build_dll { format!("{}.dll", base_name) } else { exe_path.clone() };
                    eprintln!("│  [2/2] link /{} /ENTRY:{} → {}", if build_dll { "DLL" } else { "SUBSYSTEM:CONSOLE" }, if build_dll { "_DllMainCRTStartup" } else { "main" }, exe_disp);
                    let lib_paths = find_lib_paths();
                    let mut link_args: Vec<String> = vec![
                        if build_dll { "/DLL".to_string() } else { "/SUBSYSTEM:CONSOLE".to_string() },
                        if build_dll { "/ENTRY:_DllMainCRTStartup".to_string() } else { "/ENTRY:main".to_string() },
                        obj_path.clone(),
                        "kernel32.lib".to_string(),
                        "msvcrt.lib".to_string(),
                        "ucrt.lib".to_string(),
                        "legacy_stdio_definitions.lib".to_string(),
                        format!("/OUT:{}", if build_dll { format!("{}.dll", base_name) } else { exe_path.clone() }),
                        "/NOLOGO".to_string(),
                    ];
                    for lp in &lib_paths {
                        link_args.push(format!("/LIBPATH:{}", lp));
                    }

                    let result = std::process::Command::new(link_path)
                        .args(&link_args)
                        .output();

                    match result {
                        Ok(out) => {
                            if !out.status.success() {
                                let err = String::from_utf8_lossy(&out.stderr);
                                let stdout = String::from_utf8_lossy(&out.stdout);
                                eprintln!("│  ❌ link failed:");
                                for line in format!("{}{}", stdout, err).lines() {
                                    if !line.trim().is_empty() {
                                        eprintln!("│     {}", line);
                                    }
                                }
                            } else {
                                eprintln!("│  ✅ Linked OK");
                                eprintln!("└─ OUTPUT: {}", exe_path);
                            }
                        }
                        Err(e) => {
                            eprintln!("│  ❌ link exec error: {}", e);
                        }
                    }
                }
                None => {
                    eprintln!("│  ❌ link.exe not found — install Visual Studio Build Tools");
                }
            }

            // Clean up .obj if we only wanted an exe
            let _ = fs::remove_file(&obj_path);
        } else if assemble_ok && build_obj {
            eprintln!("│  ✅ Output: {}", obj_path);
        }
    }
}
