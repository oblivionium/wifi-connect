use clap::{Parser, Subcommand};
use std::io::{self, Write};
use std::process::{Command as ProcessCommand, Stdio};
use wifi::{load_credentials, save_credentials, Portal};
use zeroize::Zeroizing;

#[derive(Parser)]
#[command(
    name = "wifi",
    about = "Open Wi-Fi Connect or sign in to or out of the college Wi-Fi portal"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    Login,
    Logout,
    Credentials {
        #[command(subcommand)]
        command: CredentialCommand,
    },
}

#[derive(Subcommand)]
enum CredentialCommand {
    Set,
}

fn main() {
    if let Err(error) = run() {
        eprintln!("wifi: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    match Cli::parse().command {
        None => launch_gui()?,
        Some(Command::Login) => {
            let credentials = load_credentials()?;
            Portal::new()?.login(&credentials.username, &credentials.password)?;
            println!("Wi-Fi Connect: connected.");
        }
        Some(Command::Logout) => {
            Portal::new()?.logout()?;
            println!("Wi-Fi Connect: disconnected.");
        }
        Some(Command::Credentials {
            command: CredentialCommand::Set,
        }) => {
            print!("College username: ");
            io::stdout().flush()?;
            let mut username = String::new();
            io::stdin().read_line(&mut username)?;
            let password = Zeroizing::new(rpassword::prompt_password("College password: ")?);
            save_credentials(username.trim(), &password)?;
            println!("Credentials saved in the operating system credential store.");
        }
    }
    Ok(())
}

fn launch_gui() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(target_os = "macos")]
    let mut command = {
        let mut command = ProcessCommand::new("open");
        command.args(["-a", "Wi-Fi Connect"]);
        command
    };

    #[cfg(not(target_os = "macos"))]
    let mut command = {
        #[cfg(target_os = "windows")]
        let gui_name = "wifi-desktop.exe";
        #[cfg(not(target_os = "windows"))]
        let gui_name = "wifi-desktop";

        let current_exe = std::env::current_exe()?;
        let sibling_gui = current_exe.with_file_name(gui_name);
        let command = if sibling_gui.is_file() {
            ProcessCommand::new(sibling_gui)
        } else {
            ProcessCommand::new(gui_name)
        };
        command
    };

    command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;
    Ok(())
}
