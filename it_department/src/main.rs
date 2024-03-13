use clap::{Parser, Subcommand};
use it_company::{pc_directory::get_directory, person::{EmailAddr, EmailParseError}};

#[derive(Parser)]
#[command(about, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Command
}

#[derive(Subcommand)]
enum Command {
    SendEmail {
        #[arg(long, value_parser = parse_email)]
        to: Option<EmailAddr>,
    },
    Search {
        #[arg(long)]
        first: Option<String>,

        #[arg(long)]
        last: Option<String>,
    }
}

fn parse_email(s: &str) -> Result<EmailAddr, EmailParseError> {
    EmailAddr::try_from(s)
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Command::SendEmail { to } => {
            println!("You want to send an email to {to:?}");
        },
        Command::Search { first, last } => {
            let (first, last) = (first.unwrap_or_default(), last.unwrap_or_default());
            println!("You want to list all computers of {first} {last}");
        },
    }

    let _dir = get_directory();


    println!("Hello, world!");
}
