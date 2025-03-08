mod compiler;

use std::fs;
use std::env;
use std::process::Command;
use crate::compiler::Compiler;
use std::path::Path;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: BRS <file.brs>");
        std::process::exit(1);
    }

    let file = Path::new(&args[1]); // No need for `as_str()`

    let mut out_dir = String::from("./");

    if let Some(out_id) = args.iter().position(|s| s == "--outdir") {
        if let Some(dir) = args.get(out_id + 1) {
            out_dir = dir.clone(); // Use owned string to avoid borrowing issues
        }
    }

    let file_name = file.file_name().unwrap().to_str().unwrap();
    let out_asm_file = format!("{}{}", out_dir, file_name.replace(".brs", ".asm"));
    let out_o_file = format!("{}{}", out_dir, file_name.replace(".brs", ".o"));

    println!("In file {}", file.to_str().unwrap());
    println!("Out file {}", out_asm_file);
    println!("Out file {}", out_o_file);

    let contents = fs::read_to_string(file)
        .expect("Should have been able to read the file");

    println!("With text:\n{contents}");
    let assembly: Option<String>;
    {
        let mut compiler = Compiler::new();
        assembly = compiler.compile(file.to_str().unwrap(), contents.as_str());
    }
    match assembly {
        Some(assembly_code) => {
            println!("Assembly:\n{assembly_code}");

            // Write the assembly code to the file
            fs::write(&out_asm_file, assembly_code).expect("Unable to write file");

            let arch = std::env::var("MY_TARGET_ARCH").unwrap_or_else(|_| env::consts::ARCH.to_string());
            let os = std::env::var("MY_TARGET_OS").unwrap_or_else(|_| env::consts::OS.to_string());
        

            let current_dir = env::current_dir().expect("Failed to get current directory");

            match os.as_str() {
                "macos" => {
                    match arch.as_str() {
                        "x86_64" => {
                            run_command(
                                Command::new("nasm")
                                    .current_dir(&current_dir)
                                    .arg("-f")
                                    .arg("macho64")
                                    .arg(&out_asm_file)
                            );

                            let ld_output = run_command(
                                Command::new("ld")
                                    .current_dir(&current_dir)
                                    .arg("-arch")
                                    .arg(if arch == "x86_64" { "x86_64" } else { "arm64" })
                                    .arg("-macos_version_min")
                                    .arg("11.0.0")
                                    .arg("-o")
                                    .arg(format!("{}out", out_dir))
                                    .arg(&out_o_file)
                                    .arg("-e")
                                    .arg("_start")
                                    .arg("-static")
                            );

                            println!("{}", ld_output);
                        },
                        "aarch64" => {
                            run_command(
                                Command::new("as")
                                    .current_dir(&current_dir)
                                    .arg("-arch")
                                    .arg("arm64")
                                    .arg("-o")
                                    .arg(&out_o_file)
                                    .arg(&out_asm_file)
                            );

                            let ld_output = run_command(
                                Command::new("ld")
                                    .current_dir(&current_dir)
                                    .arg("-arch")
                                    .arg(if arch == "x86_64" { "x86_64" } else { "arm64" })
                                    .arg("-macos_version_min")
                                    .arg("11.0.0")
                                    .arg("-o")
                                    .arg(format!("{}out", out_dir))
                                    .arg(&out_o_file)
                                    .arg("-e")
                                    .arg("_start")
                                    .arg("-lSystem")
                                    .arg("-syslibroot")
                                    .arg(&get_macos_sdk_path())
                            );

                            println!("{}", ld_output);
                        },
                        _ => eprintln!("Unsupported architecture"),
                    }
                },
                "linux" => {
                    match arch.as_str() {
                        "x86_64" => {
                            run_command(
                                Command::new("nasm")
                                    .current_dir(&current_dir)
                                    .arg("-f")
                                    .arg("elf64")
                                    .arg(&out_asm_file)
                                    .arg("-o")
                                    .arg(&out_o_file)
                            );
                        },
                        "aarch64" => {
                            run_command(
                                Command::new("aarch64-linux-gnu-as")
                                    .current_dir(&current_dir)
                                    .arg("-o")
                                    .arg(&out_o_file)
                                    .arg(&out_asm_file)
                            );
                        },
                        _ => eprintln!("Unsupported architecture"),
                    }

                    let mut ld_command = if arch == "aarch64" {
                        Command::new("aarch64-linux-gnu-ld")
                    } else {
                        Command::new("ld")
                    };

                    let ld_output = run_command(
                        ld_command
                            .current_dir(&current_dir)
                            .arg("-o")
                            .arg(format!("{}out", out_dir))
                            .arg(&out_o_file)
                            .arg("-e")
                            .arg("_start")
                            .arg("-static")
                    );

                    println!("{}", ld_output);
                },
                _ => eprintln!("Unsupported OS"),
            }

        }
        None => {
            std::process::exit(1);
        }
    }
}

fn run_command(cmd: &mut Command) -> String {
    let output = cmd.output().expect("Failed to execute command");
    if !output.status.success() {
        eprintln!("Command failed: {}", String::from_utf8_lossy(&output.stderr));
        std::process::exit(1);
    }
    String::from_utf8_lossy(&output.stdout).to_string()
}

fn get_macos_sdk_path() -> String {
    let mut cmd = Command::new("xcrun");
    cmd.arg("--sdk")
       .arg("macosx")
       .arg("--show-sdk-path");
    let sdk_path = run_command(&mut cmd).trim().to_string();
    if sdk_path.is_empty() {
        eprintln!("xcrun returned an empty SDK path.");
        std::process::exit(1);
    }
    sdk_path
}