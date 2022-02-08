# Motion Sensor Lifx

Left to implement:

- [x] Abstract timer to struct
- [x] Socket connection to Lifx lamp
- [x] UDP Packet generation and sending
- [x] Unit tests for modules
- [ ] Update only certain HSBK fields (use `Option<u16>`)
- [ ] Get timeout config externally with function cache
- [ ] Algorithm for dimming when timer has passed
- [ ] Turning timer on or off at certain times
- [ ] Turning on or off with API
- [ ] Rust Github action to build and test
- [ ] [tokio-rs/tracing](https://github.com/tokio-rs/tracing/blob/master/examples/examples/appender-multifile.rs) for logging
