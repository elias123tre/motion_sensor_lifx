use std::net::UdpSocket;
use std::{error::Error, net::ToSocketAddrs};

use lifx_core::{BuildOptions, Message, RawMessage};

// pub fn build(&self) -> Result<RawMessage, Box<dyn Error>> {
//     let options = BuildOptions {
//         target: None,
//         ack_required: false,
//         res_required: false,
//         sequence: 0,
//         source: 0,
//     };

//     let message = match self {
//         Message::ON => Message::LightSetPower {
//             level: 0xFF,
//             duration: 0,
//         },
//         Message::OFF => Message::LightSetPower {
//             level: 0x00,
//             duration: 0,
//         },
//         Message::LEVEL(_) => todo!(),
//     };
//     Ok(RawMessage::build(&options, message).expect("Could not build message"))
// }
// pub fn send<A: ToSocketAddrs>(&self, address: A) -> Result<(), Box<dyn Error>> {
//     let sock = UdpSocket::bind::<A>(address)?;
//     Message::ON.send("192.168.1.1:56700");
//     let bytes = msg.pack().unwrap();
//     sock.send(&bytes)?;

//     Ok(())
// }
