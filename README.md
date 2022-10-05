# Motion Sensor Lifx

## Automatic deployment (Windows only)

Make sure you have a ssh profile called `pi` configured in your `~/.ssh/config` file (preferably with ssh key authentication) and a network SMB drive under `\\raspi\RaspberryPi` to allow automatic deployment.

### Compile & deploy release

Running the file `deploy.ps1` with Powershell will cross-compile the cargo project, copy the binary, set +x permission and restart the systemd service.

### Run the program via terminal

Make sure the systemd service is stopped then `cargo run`.

### Build or test from VS Code

Run the default build task `CTRL+SHIFT+B` to build the binary.

Run the default test task `CTRL+SHIFT+G` to build the binary, copy it to the server and then run the tests.

## [IO Architecture Diagram](https://whimsical.com/lifx-pir-diagram-LWt2r7TCdW55KH7i5Y7EtW)

## Implemented features:

- [x] Abstract timer to struct
- [x] Socket connection to Lifx lamp
- [x] UDP Packet generation and sending
- [x] Unit tests for modules
- [x] Update only certain HSBK fields
- [x] Algorithm for dimming when timer has passed
- [ ] Get timeout config externally with function cache
- [ ] Turning timer on or off at certain times
- [ ] Turning on or off with API
- [ ] Rust Github action to build and test
- [ ] [tokio-rs/tracing](https://github.com/tokio-rs/tracing/blob/master/examples/examples/appender-multifile.rs) for logging
