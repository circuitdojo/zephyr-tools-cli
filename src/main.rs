use core::time;
use std::{
    fs::File,
    io::{self, Write},
    thread::{self, sleep},
    time::Duration,
};

use chrono::Local;
use serialport::{self, available_ports};

use rusb::{Context, Device, DeviceDescriptor, DeviceHandle, UsbContext};

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
    bl: bool,
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
        bl: pargs.contains(["-b", "--bl"]),
    };

    // It's up to the caller what to do with the remaining arguments.
    let remaining = pargs.finish();
    if !remaining.is_empty() {
        eprintln!("Warning: unused arguments left: {:?}.", remaining);
    }

    Ok(args)
}

fn open_device<T: UsbContext>(
    context: &mut T,
    vid: u16,
    pid: u16,
) -> Option<(Device<T>, DeviceDescriptor, DeviceHandle<T>)> {
    let devices = match context.devices() {
        Ok(d) => d,
        Err(_) => return None,
    };

    for device in devices.iter() {
        let device_desc = match device.device_descriptor() {
            Ok(d) => d,
            Err(_) => continue,
        };

        if device_desc.vendor_id() == vid && device_desc.product_id() == pid {
            match device.open() {
                Ok(handle) => return Some((device, device_desc, handle)),
                Err(e) => eprintln!("Device found but failed to open: {}", e),
            }
        }
    }

    None
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

    if args.bl {
        // Get the device and place into bootloader
        match Context::new() {
            Ok(mut context) => match open_device(&mut context, 0x10c4, 0xea60) {
                Some((_device, _device_desc, handle)) => {
                    println!("Placing nRF9169 Feather into bootloader mode!");

                    // Force both low
                    if let Err(e) = handle.write_control(
                        0x41,
                        0xff,
                        0x37e1,
                        0x0003,
                        &[],
                        Duration::from_millis(100),
                    ) {
                        eprintln!("Error writing command! Err: {}", e);
                        return;
                    }

                    // Small delay
                    sleep(Duration::from_millis(100));

                    // Releaseelease reset
                    if let Err(e) = handle.write_control(
                        0x41,
                        0xff,
                        0x37e1,
                        0x0103,
                        &[],
                        Duration::from_millis(100),
                    ) {
                        eprintln!("Error writing command! Err: {}", e);
                        return;
                    }

                    // Larger delay to catch "button press"
                    sleep(Duration::from_millis(1000));

                    // Release mode
                    if let Err(e) = handle.write_control(
                        0x41,
                        0xff,
                        0x37e1,
                        0x0303,
                        &[],
                        Duration::from_millis(100),
                    ) {
                        eprintln!("Error writing command! Err: {}", e);
                        return;
                    }

                    println!("Done!");
                }
                None => {
                    println!("Could not open device {:04x}:{:04x}", 0x10c4, 0xea60);
                    return;
                }
            },
            Err(e) => eprintln!("Could not initialize libusb: {}", e),
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

                    // Clone the port
                    let mut clone = p.try_clone().expect("Failed to clone");

                    // Read input from keyboard.
                    thread::spawn(move || loop {
                        let mut buffer = String::new();

                        let _ = match io::stdin().read_line(&mut buffer) {
                            Ok(_) => {
                                let l = format!("{}\r\n", buffer.replace('\n', ""));
                                // println!("\n{:?}", l);
                                clone.write(l.as_bytes())
                            }
                            Err(e) => {
                                eprintln!("Error: {}", e);
                                break;
                            }
                        };
                    });

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
