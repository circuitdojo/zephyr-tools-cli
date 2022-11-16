# Zephyr Tools CLI

```
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
```

Allows you to:

1. Monitor serial devices and save the output to file. 
2. Includes the use of a `--follow` command which will reconnect to a target device if disconnected/unplugged.
3. Send commands to the CP2012 for automagic bootloading on the nRF9160 Feather

The latest version ships with the Circuit Dojo VSCode Zephyr Tools Plugin. 

## License

Apache 2.0
