use core::time;
use std::{
    fs::File,
    io::{self, Write},
    thread::sleep,
    time::Duration,
};

use chrono::Local;
use serde_json;
use serialport::{self, available_ports};

const HELP: &str = "\
App
USAGE:
  app [OPTIONS]
FLAGS:
  -h, --help            Prints help information
OPTIONS:
  --list, -l            Lists serial ports avaiable
  --follow,             Follow serial port if it disconnects
  --port,               Port to connect to
  --baud,               Baud to use (default 115200)
  --save, -s            Automatically save output to file
ARGS:
  <INPUT>
";

#[derive(Debug)]
struct AppArgs {
    list: bool,
    port: Option<String>,
    baud: u32,
    follow: bool,
    save: bool,
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
        port: pargs.opt_value_from_str("--port")?,
        baud: pargs.value_from_str("--baud").or(Ok(115_200))?,
        follow: pargs.contains(["-f", "--follow"]),
        save: pargs.contains(["-s", "--save"]),
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

    // Show list of possible ports
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

        return;
    }

    if args.port.is_some() {
        print!("Connecting..");
        io::stdout().flush().unwrap();

        let port_name = args.port.unwrap();

        // Open file if active
        let mut file = match args.save {
            true => {
                let time = Local::now();

                match File::create(format!("log-{}.txt", time.to_rfc3339())) {
                    Ok(f) => Some(f),
                    Err(_) => None,
                }
            }
            false => None,
        };

        loop {
            // Open with settings
            let port = serialport::new(&port_name, args.baud)
                .timeout(time::Duration::from_millis(10))
                .open();

            // Then watch the buffer until terminated..
            match port {
                Ok(mut p) => {
                    // Start incoming data on a new line
                    println!("\nConnected to {}!", port_name);

                    let mut buf: Vec<u8> = vec![0; 1000];
                    loop {
                        match p.read(buf.as_mut_slice()) {
                            Ok(t) => {
                                io::stdout().write_all(&buf[..t]).unwrap();

                                // Save to file as well..
                                if let Some(ref mut f) = file {
                                    let _ = f.write(&buf[..t]);
                                    let _ = f.flush();
                                }
                            }
                            Err(ref e) if e.kind() == io::ErrorKind::TimedOut => continue,
                            Err(e) => {
                                if args.follow {
                                    break;
                                } else {
                                    eprintln!("Error: {}", e);
                                    std::process::exit(1);
                                }
                            }
                        }
                    }
                }
                Err(_e) => {
                    if !args.follow {
                        eprintln!("Unable to connect to {} with baud {}", port_name, args.baud);
                        std::process::exit(1);
                    }
                }
            }

            // Print that we're waiting..
            print!(".");
            io::stdout().flush().unwrap();
            sleep(Duration::from_secs(1));
        }
    }

    // Otherwise print help
    print!("{}", HELP);
}
