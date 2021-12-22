use serde_json;
use serialport::available_ports;

const HELP: &str = "\
App
USAGE:
  app [OPTIONS]
FLAGS:
  -h, --help            Prints help information
OPTIONS:
  --list, -l            Lists serial ports avaiable
ARGS:
  <INPUT>
";

#[derive(Debug)]
struct AppArgs {
    list: bool,
}

fn parse_args() -> Result<AppArgs, pico_args::Error> {
    let mut pargs = pico_args::Arguments::from_env();

    // Help has a higher priority and should be handled separately.
    if pargs.contains(["-h", "--help"]) {
        print!("{}", HELP);
        std::process::exit(0);
    }

    let args = AppArgs {
        list: pargs.contains(["-l", "--list"]),
    };

    // It's up to the caller what to do with the remaining arguments.
    let remaining = pargs.finish();
    if !remaining.is_empty() {
        eprintln!("Warning: unused arguments left: {:?}.", remaining);
    }

    Ok(args)
}

fn main() {
    let args = match parse_args() {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Error: {}.", e);
            std::process::exit(1);
        }
    };

    if args.list {
        match available_ports() {
            Ok(ports) => {
                // Get the portnames
                let ports: Vec<String> = ports.iter().map(|x| x.port_name.clone()).collect();

                // Turn them into a json string
                let json = serde_json::to_string(&ports).unwrap();

                // Print em
                print!("{}", json);
            }
            Err(e) => {
                eprintln!("{:?}", e);
                eprintln!("Error listing serial ports");
            }
        }
    }
}
