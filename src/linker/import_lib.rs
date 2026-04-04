//! Import Library (.lib) parser — Phase 11
//! Parses Windows import libraries (COFF archive format) to extract
//! DLL name + function name mappings for generating the PE Import Table.

use std::collections::HashMap;

/// An import entry: a function imported from a DLL
#[derive(Debug, Clone)]
pub struct ImportEntry {
    pub name: String,        // Function name (e.g. "ExitProcess")
    pub ordinal_hint: u16,   // Ordinal hint (0 if unknown)
}

/// Parsed import library: DLL → list of imported functions
#[derive(Debug)]
pub struct ImportLib {
    pub dll_name: String,
    pub entries: Vec<ImportEntry>,
}

/// Parse a .lib archive file and extract import definitions.
/// Windows import .lib uses COFF archive format with special import header members.
pub fn parse_import_lib(data: &[u8]) -> Result<Vec<ImportLib>, String> {
    if data.len() < 8 {
        return Err("File too small for archive".into());
    }

    // Check archive signature "!<arch>\n"
    if &data[0..8] != b"!<arch>\n" {
        return Err("Not a valid .lib archive (missing !<arch> signature)".into());
    }

    let mut dll_imports: HashMap<String, Vec<ImportEntry>> = HashMap::new();
    let mut offset = 8usize;

    while offset + 60 <= data.len() {
        // Archive member header: 60 bytes
        let header = &data[offset..offset+60];
        let size_str = String::from_utf8_lossy(&header[48..58]).trim().to_string();
        let member_size: usize = size_str.parse().unwrap_or(0);
        let member_data_start = offset + 60;
        let member_data_end = (member_data_start + member_size).min(data.len());

        if member_data_start + 20 <= member_data_end {
            let member = &data[member_data_start..member_data_end];

            // Check for Import Header (short import format)
            // sig1 = 0x0000, sig2 = 0xFFFF
            if member.len() >= 20 && member[0] == 0 && member[1] == 0 && member[2] == 0xFF && member[3] == 0xFF {
                // Short import header
                let _version = u16::from_le_bytes([member[4], member[5]]);
                let _machine = u16::from_le_bytes([member[6], member[7]]);
                let _timestamp = u32::from_le_bytes([member[8], member[9], member[10], member[11]]);
                let _data_size = u32::from_le_bytes([member[12], member[13], member[14], member[15]]);
                let ordinal_hint = u16::from_le_bytes([member[16], member[17]]);
                let _type_info = u16::from_le_bytes([member[18], member[19]]);

                // After header: null-terminated function name, then null-terminated DLL name
                let names_data = &member[20..];
                let func_end = names_data.iter().position(|&b| b == 0).unwrap_or(names_data.len());
                let func_name = String::from_utf8_lossy(&names_data[..func_end]).to_string();

                let dll_start = func_end + 1;
                if dll_start < names_data.len() {
                    let dll_data = &names_data[dll_start..];
                    let dll_end = dll_data.iter().position(|&b| b == 0).unwrap_or(dll_data.len());
                    let dll_name = String::from_utf8_lossy(&dll_data[..dll_end]).to_string();

                    if !func_name.is_empty() && !dll_name.is_empty() {
                        dll_imports.entry(dll_name)
                            .or_insert_with(Vec::new)
                            .push(ImportEntry { name: func_name, ordinal_hint });
                    }
                }
            }
        }

        // Advance to next member (2-byte aligned)
        offset = member_data_end;
        if offset % 2 != 0 { offset += 1; }
    }

    let mut result: Vec<ImportLib> = dll_imports.into_iter()
        .map(|(dll, entries)| ImportLib { dll_name: dll, entries })
        .collect();
    result.sort_by(|a, b| a.dll_name.to_lowercase().cmp(&b.dll_name.to_lowercase()));
    Ok(result)
}

/// Build import definitions from well-known Windows library names
/// without actually reading .lib files. Covers kernel32, user32, msvcrt.
pub fn builtin_imports_for(lib_name: &str) -> Option<ImportLib> {
    let lower = lib_name.to_lowercase();
    let lower = lower.trim_end_matches(".lib");

    match lower {
        "kernel32" => Some(ImportLib {
            dll_name: "kernel32.dll".into(),
            entries: [
                "ExitProcess", "GetModuleHandleA", "GetModuleHandleW",
                "GetStdHandle", "WriteFile", "ReadFile",
                "WriteConsoleA", "WriteConsoleW", "ReadConsoleA",
                "VirtualAlloc", "VirtualFree", "VirtualProtect",
                "HeapAlloc", "HeapFree", "GetProcessHeap",
                "GetLastError", "SetLastError",
                "CreateFileA", "CreateFileW", "CloseHandle",
                "GetCommandLineA", "GetCommandLineW",
                "Sleep", "CreateThread", "ExitThread",
                "GetProcAddress", "LoadLibraryA", "LoadLibraryW", "FreeLibrary",
                "GetSystemInfo", "QueryPerformanceCounter", "QueryPerformanceFrequency",
                "GetCurrentProcess", "GetCurrentProcessId",
                "GetCurrentThread", "GetCurrentThreadId",
                "InitializeCriticalSection", "EnterCriticalSection",
                "LeaveCriticalSection", "DeleteCriticalSection",
                "TlsAlloc", "TlsFree", "TlsGetValue", "TlsSetValue",
                "FlushFileBuffers", "SetFilePointer",
                "GetFileSize", "GetFileSizeEx",
                "MultiByteToWideChar", "WideCharToMultiByte",
            ].iter().map(|&s| ImportEntry { name: s.into(), ordinal_hint: 0 }).collect(),
        }),
        "user32" => Some(ImportLib {
            dll_name: "user32.dll".into(),
            entries: [
                "MessageBoxA", "MessageBoxW",
                "CreateWindowExA", "CreateWindowExW",
                "ShowWindow", "UpdateWindow", "DestroyWindow",
                "DefWindowProcA", "DefWindowProcW",
                "PostQuitMessage", "GetMessageA", "GetMessageW",
                "TranslateMessage", "DispatchMessageA", "DispatchMessageW",
                "RegisterClassExA", "RegisterClassExW",
                "LoadCursorA", "LoadCursorW",
                "LoadIconA", "LoadIconW",
                "SendMessageA", "SendMessageW",
                "SetWindowTextA", "SetWindowTextW",
                "GetDC", "ReleaseDC", "BeginPaint", "EndPaint",
                "InvalidateRect", "GetClientRect",
                "SetTimer", "KillTimer",
                "PostMessageA", "PostMessageW",
            ].iter().map(|&s| ImportEntry { name: s.into(), ordinal_hint: 0 }).collect(),
        }),
        "msvcrt" => Some(ImportLib {
            dll_name: "msvcrt.dll".into(),
            entries: [
                "printf", "scanf", "sprintf", "sscanf", "fprintf", "fscanf",
                "puts", "putchar", "getchar", "gets",
                "malloc", "calloc", "realloc", "free",
                "memcpy", "memset", "memmove", "memcmp",
                "strlen", "strcpy", "strncpy", "strcat", "strncat", "strcmp", "strncmp",
                "atoi", "atof", "atol",
                "exit", "_exit", "abort",
                "fopen", "fclose", "fread", "fwrite", "fseek", "ftell", "fflush",
                "rand", "srand", "time",
                "abs", "labs",
                "_beginthreadex", "_endthreadex",
            ].iter().map(|&s| ImportEntry { name: s.into(), ordinal_hint: 0 }).collect(),
        }),
        "ucrt" | "api-ms-win-crt-runtime-l1-1-0" => Some(ImportLib {
            dll_name: "api-ms-win-crt-runtime-l1-1-0.dll".into(),
            entries: [
                "_exit", "exit", "abort", "_cexit",
                "_initterm", "_initterm_e",
                "_get_initial_narrow_environment",
                "__p___argc", "__p___argv",
                "_configure_narrow_argv", "_initialize_narrow_environment",
                "_c_exit", "_register_thread_local_exe_atexit_callback",
                "_set_app_type", "_seh_filter_exe",
            ].iter().map(|&s| ImportEntry { name: s.into(), ordinal_hint: 0 }).collect(),
        }),
        _ => None,
    }
}
