use std::env;
use std::process::{Command, Stdio};
use sysinfo::{ProcessesToUpdate, System};
//
// CONFIG (EDIT THESE)
//
// Exact playback device names as shown in SoundVolumeView:
const DEVICE_VM_MODE: &str = r"7.1 Surround Sound";
const DEVICE_DIRECT: &str = r"USB Audio Device";
const SOUNDVOLUMEVIEW_EXE: &str = r"C:\Tools\SoundVolumeView\SoundVolumeView.exe";
const VOICEMEETER_EXE: &str = r"C:\Program Files (x86)\VB\Voicemeeter\voicemeeter8.exe";
//
// END CONFIG
//

fn check_one(label: &str, path: &str) -> std::io::Result<()> {
    println!("Checking for {} at: {}", label, path);

    let out = Command::new("cmd")
        .args(["/C", "dir", "/b", path])
        .output()?;

    println!("Exit code: {:?}", out.status.code());
    if !out.stdout.is_empty() {
        println!("STDOUT:\n{}", String::from_utf8_lossy(&out.stdout));
    }
    if !out.stderr.is_empty() {
        println!("STDERR:\n{}", String::from_utf8_lossy(&out.stderr));
    }

    if out.status.success() {
        println!("OK: {} exists", label);
    } else {
        println!("MISSING: {}", label);
    }
    println!("----------------------------------------");
    Ok(())
}

fn check_paths() -> std::io::Result<()> {
    check_one("SVV", SOUNDVOLUMEVIEW_EXE)?;
    check_one("Voicemeeter", VOICEMEETER_EXE)?;
    Ok(())
}

fn set_default_device(device_name: &str) -> std::io::Result<()> {
    for role in ["0", "1", "2"] {
        let status = Command::new(SOUNDVOLUMEVIEW_EXE)
            .args(["/SetDefault", device_name, role])
            .status()?;

        if !status.success() {
            eprintln!(
                "ERROR: SetDefault failed (device='{}', role={}, status={})",
                device_name, role, status
            );
        }
    }
    Ok(())
}



fn is_running(substr: &str) -> bool {
    let mut sys = System::new();
    sys.refresh_processes(ProcessesToUpdate::All, true);
    let target = substr.to_lowercase();
    sys.processes().values().any(|p| {
        p.name()
            .to_string_lossy()
            .to_lowercase()
            .contains(&target)
    })
}

fn start_voicemeeter() -> std::io::Result<()> {
    if is_running("voicemeeter") {
        println!("Voicemeeter is already running.");
        return Ok(());
    }

    Command::new(VOICEMEETER_EXE)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;

    println!("Started Voicemeeter.");
    Ok(())
}



fn export_devices_csv() -> std::io::Result<()> {
    // Ensure folder exists
    let _ = std::fs::create_dir_all(r"C:\Temp");

    let status = Command::new(SOUNDVOLUMEVIEW_EXE)
        .args(["/scomma", r"C:\Temp\svv_devices.csv"])
        .status()?;

    if !status.success() {
        eprintln!("ERROR: failed to export device list (status={})", status);
    } else {
        println!("Wrote: C:\\Temp\\svv_devices.csv");
    }

    Ok(())
}

fn print_usage() {
    eprintln!("Usage: audio_profiles <vm|direct|check|list>");
    eprintln!("  vm     -> set default device to Voicemeeter + start Voicemeeter");
    eprintln!("  direct -> set default device to your headset directly (no VM needed)");
    eprintln!("  check  -> verify that required executables exist");
    eprintln!("  list   -> export device list to C:\\Temp\\svv_devices.csv");
}


fn main() -> std::io::Result<()> {
    println!("audio_profiles build: {}", env!("CARGO_PKG_VERSION"));

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        print_usage();
        std::process::exit(1);
    }
    if args[1] == "check" {
        check_paths()?;
        return Ok(());
    }
    if args[1] == "list" {
        export_devices_csv()?;
        return Ok(());
    }
    match args[1].as_str() {
        "vm" => {
            set_default_device(DEVICE_VM_MODE)?;
            start_voicemeeter()?;
            println!("OK: switched to VM/7.1 mode.");
        }
        "direct" => {
            set_default_device(DEVICE_DIRECT)?;
            println!("OK: switched to Direct headset mode.");
        }
        _ => { print_usage(); std::process::exit(1); }
    }
    Ok(())
}