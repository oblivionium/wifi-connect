use clap::{Parser, Subcommand};
use std::io::{self, Write};
use wifi::{load_credentials, save_credentials, Portal};
use zeroize::Zeroizing;

#[derive(Parser)]
#[command(name = "wifi", about = "Sign in to or out of the college Wi-Fi portal")]
struct Cli {
    #[command(subcommand)]
    command: Command,
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
        Command::Login => {
            let credentials = load_credentials()?;
            Portal::new()?.login(&credentials.username, &credentials.password)?;
            println!("College Wi-Fi connected.");
        }
        Command::Logout => {
            Portal::new()?.logout()?;
            println!("College Wi-Fi disconnected.");
        }
        Command::Credentials {
            command: CredentialCommand::Set,
        } => {
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
