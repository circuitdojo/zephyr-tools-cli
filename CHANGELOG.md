## [0.1.7]

Allow checking of version

Changed:
* Version now shows in help dialogue
* Version now shows with -v command

## [0.1.6]

Fixing panic condition for platforms that don't support libusb

Changed:
* Removed/replaced panic! with eprintln!
* Moved Done! print out

## [0.1.5]

Allowing to put select devices into bootloader mode

Changed:
* nRF9160 Feather can now be placed into bootloader mode via this CLI util.


## [0.1.4]

Allowing for writing to serial session.

Changed:
* Monitoring stdin and allowing write of string to device


## [0.1.3]

Fixing issue where not all characters are printed

Changed:
* Use of raw buffers rather than "lines" since not all data returned will be a nice null terminated or \n string

## [0.1.2]

Option to save to file.

Added:
* Option to save output to file as well.

## [0.1.1]

Opening serial port.

### Added:
* Opening of serial port
* Following of serial port if it gets disconnected.


## [0.1.0]

Initial release.

### Added:

* List functionality directly to JSON for easy parsing.