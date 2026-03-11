//! Waveform CLI - Direct command-line interface for waveform tools
//!
//! Usage: waveform-cli <command1> [args...] [-- <command2> [args...] ...]
//!
//! Commands:
//!   open_waveform <file_path> [--alias <alias>]
//!   close_waveform <waveform_id>
//!   list_signals <waveform_id> [--pattern pattern] [--hierarchy <prefix>] [--recursive <true|false>] [--limit <n>]
//!   read_signal <waveform_id> <signal_path> [--time-index <idx> | --time-indices <idx1,idx2,...>]
//!   get_signal_info <waveform_id> <signal_path>
//!   find_signal_events <waveform_id> <signal_path> [--start <idx>] [--end <idx>] [--limit <n>]
//!   find_conditional_events <waveform_id> <condition> [--start <idx>] [--end <idx>] [--limit <n>]

use std::collections::HashMap;
use std::path::PathBuf;
use waveform_mcp::{
    find_conditional_events, find_signal_by_path, find_signal_events, get_signal_metadata,
    list_signals, read_signal_values,
};

#[derive(Debug, Clone)]
enum Command {
    OpenWaveform {
        file_path: String,
        alias: Option<String>,
    },
    CloseWaveform {
        waveform_id: String,
    },
    ListSignals {
        waveform_id: String,
        name_pattern: Option<String>,
        hierarchy_prefix: Option<String>,
        recursive: bool,
        limit: Option<isize>,
    },
    ReadSignal {
        waveform_id: String,
        signal_path: String,
        time_index: Option<usize>,
        time_indices: Option<Vec<usize>>,
    },
    GetSignalInfo {
        waveform_id: String,
        signal_path: String,
    },
    FindSignalEvents {
        waveform_id: String,
        signal_path: String,
        start_time_index: Option<usize>,
        end_time_index: Option<usize>,
        limit: Option<isize>,
    },
    FindConditionalEvents {
        waveform_id: String,
        condition: String,
        start_time_index: Option<usize>,
        end_time_index: Option<usize>,
        limit: Option<isize>,
    },
}

fn parse_args() -> Vec<Command> {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.is_empty() {
        print_usage();
        std::process::exit(1);
    }

    // Split args by "--" separator
    let mut command_groups: Vec<Vec<String>> = vec![vec![]];
    for arg in args {
        if arg == "--" {
            command_groups.push(vec![]);
        } else {
            command_groups.last_mut().unwrap().push(arg);
        }
    }

    // Parse each command group
    command_groups
        .into_iter()
        .filter(|g| !g.is_empty())
        .map(parse_command)
        .collect()
}

fn parse_command(group: Vec<String>) -> Command {
    if group.is_empty() {
        eprintln!("Error: Empty command group");
        std::process::exit(1);
    }

    let cmd_name = &group[0];
    let cmd_args = &group[1..];

    match cmd_name.as_str() {
        "open_waveform" => parse_open_waveform(cmd_args),
        "close_waveform" => parse_close_waveform(cmd_args),
        "list_signals" => parse_list_signals(cmd_args),
        "read_signal" => parse_read_signal(cmd_args),
        "get_signal_info" => parse_get_signal_info(cmd_args),
        "find_signal_events" => parse_find_signal_events(cmd_args),
        "find_conditional_events" => parse_find_conditional_events(cmd_args),
        _ => {
            eprintln!("Error: Unknown command '{}'", cmd_name);
            eprintln!("\nAvailable commands:");
            eprintln!("  open_waveform, close_waveform, list_signals, read_signal,");
            eprintln!("  get_signal_info, find_signal_events, find_conditional_events");
            std::process::exit(1);
        }
    }
}

fn parse_open_waveform(args: &[String]) -> Command {
    if args.is_empty() {
        eprintln!("Error: open_waveform requires a file path");
        eprintln!("Usage: open_waveform <file_path> [--alias <alias>]");
        std::process::exit(1);
    }

    let file_path = args[0].clone();
    let mut alias = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--alias" | "-a" => {
                i += 1;
                if i < args.len() {
                    alias = Some(args[i].clone());
                } else {
                    eprintln!("Error: --alias requires a value");
                    std::process::exit(1);
                }
            }
            _ => {
                eprintln!("Error: Unknown option '{}' for open_waveform", args[i]);
                std::process::exit(1);
            }
        }
        i += 1;
    }

    Command::OpenWaveform { file_path, alias }
}

fn parse_close_waveform(args: &[String]) -> Command {
    if args.is_empty() {
        eprintln!("Error: close_waveform requires a waveform_id");
        eprintln!("Usage: close_waveform <waveform_id>");
        std::process::exit(1);
    }

    Command::CloseWaveform {
        waveform_id: args[0].clone(),
    }
}

fn parse_list_signals(args: &[String]) -> Command {
    if args.is_empty() {
        eprintln!("Error: list_signals requires a waveform_id");
        eprintln!("Usage: list_signals <waveform_id> [--pattern <pattern>] [--hierarchy <prefix>] [--recursive <true|false>] [--limit <n>]");
        std::process::exit(1);
    }

    let waveform_id = args[0].clone();
    let mut name_pattern = None;
    let mut hierarchy_prefix = None;
    let mut recursive = true;
    let mut limit = Some(100isize);

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--pattern" | "-p" => {
                i += 1;
                if i < args.len() {
                    name_pattern = Some(args[i].clone());
                } else {
                    eprintln!("Error: --pattern requires a value");
                    std::process::exit(1);
                }
            }
            "--hierarchy" | "-h" => {
                i += 1;
                if i < args.len() {
                    hierarchy_prefix = Some(args[i].clone());
                } else {
                    eprintln!("Error: --hierarchy requires a value");
                    std::process::exit(1);
                }
            }
            "--recursive" | "-r" => {
                i += 1;
                if i < args.len() {
                    recursive = args[i].parse().unwrap_or(true);
                } else {
                    eprintln!("Error: --recursive requires a value (true/false)");
                    std::process::exit(1);
                }
            }
            "--limit" | "-l" => {
                i += 1;
                if i < args.len() {
                    limit = args[i].parse().ok();
                } else {
                    eprintln!("Error: --limit requires a value");
                    std::process::exit(1);
                }
            }
            _ => {
                eprintln!("Error: Unknown option '{}' for list_signals", args[i]);
                std::process::exit(1);
            }
        }
        i += 1;
    }

    Command::ListSignals {
        waveform_id,
        name_pattern,
        hierarchy_prefix,
        recursive,
        limit,
    }
}

fn parse_read_signal(args: &[String]) -> Command {
    if args.len() < 2 {
        eprintln!("Error: read_signal requires waveform_id and signal_path");
        eprintln!("Usage: read_signal <waveform_id> <signal_path> [--time-index <idx> | --time-indices <idx1,idx2,...>]");
        std::process::exit(1);
    }

    let waveform_id = args[0].clone();
    let signal_path = args[1].clone();
    let mut time_index = None;
    let mut time_indices = None;

    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "--time-index" | "-t" => {
                i += 1;
                if i < args.len() {
                    time_index = args[i].parse().ok();
                } else {
                    eprintln!("Error: --time-index requires a value");
                    std::process::exit(1);
                }
            }
            "--time-indices" | "-T" => {
                i += 1;
                if i < args.len() {
                    time_indices =
                        Some(args[i].split(',').filter_map(|s| s.parse().ok()).collect());
                } else {
                    eprintln!("Error: --time-indices requires a value");
                    std::process::exit(1);
                }
            }
            _ => {
                eprintln!("Error: Unknown option '{}' for read_signal", args[i]);
                std::process::exit(1);
            }
        }
        i += 1;
    }

    Command::ReadSignal {
        waveform_id,
        signal_path,
        time_index,
        time_indices,
    }
}

fn parse_get_signal_info(args: &[String]) -> Command {
    if args.len() < 2 {
        eprintln!("Error: get_signal_info requires waveform_id and signal_path");
        eprintln!("Usage: get_signal_info <waveform_id> <signal_path>");
        std::process::exit(1);
    }

    Command::GetSignalInfo {
        waveform_id: args[0].clone(),
        signal_path: args[1].clone(),
    }
}

fn parse_find_signal_events(args: &[String]) -> Command {
    if args.len() < 2 {
        eprintln!("Error: find_signal_events requires waveform_id and signal_path");
        eprintln!("Usage: find_signal_events <waveform_id> <signal_path> [--start <idx>] [--end <idx>] [--limit <n>]");
        std::process::exit(1);
    }

    let waveform_id = args[0].clone();
    let signal_path = args[1].clone();
    let mut start_time_index = None;
    let mut end_time_index = None;
    let mut limit = Some(100isize);

    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "--start" | "-s" => {
                i += 1;
                if i < args.len() {
                    start_time_index = args[i].parse().ok();
                } else {
                    eprintln!("Error: --start requires a value");
                    std::process::exit(1);
                }
            }
            "--end" | "-e" => {
                i += 1;
                if i < args.len() {
                    end_time_index = args[i].parse().ok();
                } else {
                    eprintln!("Error: --end requires a value");
                    std::process::exit(1);
                }
            }
            "--limit" | "-l" => {
                i += 1;
                if i < args.len() {
                    limit = args[i].parse().ok();
                } else {
                    eprintln!("Error: --limit requires a value");
                    std::process::exit(1);
                }
            }
            _ => {
                eprintln!("Error: Unknown option '{}' for find_signal_events", args[i]);
                std::process::exit(1);
            }
        }
        i += 1;
    }

    Command::FindSignalEvents {
        waveform_id,
        signal_path,
        start_time_index,
        end_time_index,
        limit,
    }
}

fn parse_find_conditional_events(args: &[String]) -> Command {
    if args.len() < 2 {
        eprintln!("Error: find_conditional_events requires waveform_id and condition");
        eprintln!("Usage: find_conditional_events <waveform_id> <condition> [--start <idx>] [--end <idx>] [--limit <n>]");
        std::process::exit(1);
    }

    let waveform_id = args[0].clone();
    let condition = args[1].clone();
    let mut start_time_index = None;
    let mut end_time_index = None;
    let mut limit = Some(100isize);

    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "--start" | "-s" => {
                i += 1;
                if i < args.len() {
                    start_time_index = args[i].parse().ok();
                } else {
                    eprintln!("Error: --start requires a value");
                    std::process::exit(1);
                }
            }
            "--end" | "-e" => {
                i += 1;
                if i < args.len() {
                    end_time_index = args[i].parse().ok();
                } else {
                    eprintln!("Error: --end requires a value");
                    std::process::exit(1);
                }
            }
            "--limit" | "-l" => {
                i += 1;
                if i < args.len() {
                    limit = args[i].parse().ok();
                } else {
                    eprintln!("Error: --limit requires a value");
                    std::process::exit(1);
                }
            }
            _ => {
                eprintln!(
                    "Error: Unknown option '{}' for find_conditional_events",
                    args[i]
                );
                std::process::exit(1);
            }
        }
        i += 1;
    }

    Command::FindConditionalEvents {
        waveform_id,
        condition,
        start_time_index,
        end_time_index,
        limit,
    }
}

fn print_usage() {
    println!("waveform-cli - Command-line interface for waveform analysis");
    println!();
    println!("Usage: waveform-cli <command1> [args...] [-- <command2> [args...] ...]");
    println!();
    println!("Commands can be chained using '--' as a separator.");
    println!();
    println!("Commands:");
    println!("  open_waveform <file_path> [--alias <alias>]");
    println!("    Open a VCD or FST waveform file");
    println!();
    println!("  close_waveform <waveform_id>");
    println!("    Close a waveform and free its memory");
    println!();
    println!("  list_signals <waveform_id> [--pattern <pattern>] [--hierarchy <prefix>] [--recursive <true|false>] [--limit <n>]");
    println!("    List signals matching optional pattern");
    println!();
    println!("  read_signal <waveform_id> <signal_path> [--time-index <idx> | --time-indices <idx1,idx2,...>]");
    println!("    Read signal values at specific time indices");
    println!();
    println!("  get_signal_info <waveform_id> <signal_path>");
    println!("    Get metadata about a signal");
    println!();
    println!("  find_signal_events <waveform_id> <signal_path> [--start <idx>] [--end <idx>] [--limit <n>]");
    println!("    Find all changes (events) of a signal in a time range");
    println!();
    println!("  find_conditional_events <waveform_id> <condition> [--start <idx>] [--end <idx>] [--limit <n>]");
    println!("    Find events where a condition is satisfied");
    println!("    Condition syntax supports: signal paths, ~ (NOT), & (AND), | (OR),");
    println!("    ^ (XOR), &&, ||, ==, !=, $past(), bit extraction, Verilog literals");
    println!();
    println!("Examples:");
    println!("  waveform-cli open_waveform test.vcd");
    println!("  waveform-cli open_waveform test.fst --alias mywave -- list_signals mywave --pattern clock");
    println!("  waveform-cli open_waveform test.vcd -- read_signal test top.clk --time-index 0");
    println!(
        "  waveform-cli open_waveform test.vcd -- find_signal_events test top.reset --limit 10"
    );
}

struct WaveformStore {
    waveforms: HashMap<String, wellen::simple::Waveform>,
}

impl WaveformStore {
    fn new() -> Self {
        Self {
            waveforms: HashMap::new(),
        }
    }

    fn open_waveform(&mut self, file_path: &str, alias: Option<String>) -> Result<String, String> {
        let path = PathBuf::from(file_path);

        if !path.exists() {
            return Err(format!("File not found: {}", file_path));
        }

        let waveform =
            wellen::simple::read(&path).map_err(|e| format!("Failed to read waveform: {}", e))?;

        let id = alias.unwrap_or_else(|| {
            path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string()
        });

        self.waveforms.insert(id.clone(), waveform);
        Ok(id)
    }

    fn close_waveform(&mut self, waveform_id: &str) -> Result<(), String> {
        match self.waveforms.remove(waveform_id) {
            Some(_) => Ok(()),
            None => Err(format!("Waveform not found: {}", waveform_id)),
        }
    }

    fn get(&self, waveform_id: &str) -> Option<&wellen::simple::Waveform> {
        self.waveforms.get(waveform_id)
    }

    fn get_mut(&mut self, waveform_id: &str) -> Option<&mut wellen::simple::Waveform> {
        self.waveforms.get_mut(waveform_id)
    }
}

fn execute_command(store: &mut WaveformStore, cmd: Command) -> Result<String, String> {
    match cmd {
        Command::OpenWaveform { file_path, alias } => {
            let id = store.open_waveform(&file_path, alias)?;
            Ok(format!("Waveform opened successfully with id: {}", id))
        }
        Command::CloseWaveform { waveform_id } => {
            store.close_waveform(&waveform_id)?;
            Ok(format!("Waveform '{}' closed successfully", waveform_id))
        }
        Command::ListSignals {
            waveform_id,
            name_pattern,
            hierarchy_prefix,
            recursive,
            limit,
        } => {
            let waveform = store
                .get(&waveform_id)
                .ok_or_else(|| format!("Waveform not found: {}", waveform_id))?;

            let hierarchy = waveform.hierarchy();
            let signals = list_signals(
                hierarchy,
                name_pattern.as_deref(),
                hierarchy_prefix.as_deref(),
                recursive,
                limit,
            );

            Ok(format!(
                "Found {} signals:\n{}",
                signals.len(),
                signals.join("\n")
            ))
        }
        Command::ReadSignal {
            waveform_id,
            signal_path,
            time_index,
            time_indices,
        } => {
            let waveform = store
                .get_mut(&waveform_id)
                .ok_or_else(|| format!("Waveform not found: {}", waveform_id))?;

            let hierarchy = waveform.hierarchy();
            let signal_ref = find_signal_by_path(hierarchy, &signal_path)
                .ok_or_else(|| format!("Signal not found: {}", signal_path))?;

            waveform.load_signals(&[signal_ref]);

            let indices_to_read: Vec<usize> = if let Some(ref indices) = time_indices {
                indices.clone()
            } else if let Some(index) = time_index {
                vec![index]
            } else {
                return Err("Either time_index or time_indices must be provided".to_string());
            };

            let results = read_signal_values(waveform, signal_ref, &indices_to_read)
                .map_err(|e| format!("Error reading signal: {}", e))?;

            Ok(results.join("\n"))
        }
        Command::GetSignalInfo {
            waveform_id,
            signal_path,
        } => {
            let waveform = store
                .get(&waveform_id)
                .ok_or_else(|| format!("Waveform not found: {}", waveform_id))?;

            let hierarchy = waveform.hierarchy();
            let info = get_signal_metadata(hierarchy, &signal_path)
                .map_err(|e| format!("Error getting signal info: {}", e))?;

            Ok(info)
        }
        Command::FindSignalEvents {
            waveform_id,
            signal_path,
            start_time_index,
            end_time_index,
            limit,
        } => {
            let waveform = store
                .get_mut(&waveform_id)
                .ok_or_else(|| format!("Waveform not found: {}", waveform_id))?;

            let hierarchy = waveform.hierarchy();
            let signal_ref = find_signal_by_path(hierarchy, &signal_path)
                .ok_or_else(|| format!("Signal not found: {}", signal_path))?;

            waveform.load_signals(&[signal_ref]);

            let time_table = waveform.time_table();
            let start_idx = start_time_index.unwrap_or(0);
            let end_idx = end_time_index.unwrap_or(time_table.len().saturating_sub(1));
            let lim = limit.unwrap_or(-1);

            let events = find_signal_events(waveform, signal_ref, start_idx, end_idx, lim)
                .map_err(|e| format!("Error finding signal events: {}", e))?;

            Ok(format!(
                "Found {} events for signal '{}' (time range: {} to {}):\n{}",
                events.len(),
                signal_path,
                start_idx,
                end_idx,
                events.join("\n")
            ))
        }
        Command::FindConditionalEvents {
            waveform_id,
            condition,
            start_time_index,
            end_time_index,
            limit,
        } => {
            let waveform = store
                .get_mut(&waveform_id)
                .ok_or_else(|| format!("Waveform not found: {}", waveform_id))?;

            let time_table = waveform.time_table();
            let start_idx = start_time_index.unwrap_or(0);
            let end_idx = end_time_index.unwrap_or(time_table.len().saturating_sub(1));
            let lim = limit.unwrap_or(-1);

            let events = find_conditional_events(waveform, &condition, start_idx, end_idx, lim)
                .map_err(|e| format!("Error finding conditional events: {}", e))?;

            Ok(format!(
                "Found {} events for condition '{}' (time range: {} to {}):\n{}",
                events.len(),
                condition,
                start_idx,
                end_idx,
                events.join("\n")
            ))
        }
    }
}

fn main() {
    let commands = parse_args();
    let mut store = WaveformStore::new();

    for (i, cmd) in commands.iter().enumerate() {
        if i > 0 {
            println!();
        }

        match execute_command(&mut store, cmd.clone()) {
            Ok(output) => println!("{}", output),
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
    }
}
