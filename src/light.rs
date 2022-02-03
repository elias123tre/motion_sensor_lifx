use std::error::Error;
use std::net::ToSocketAddrs;
use std::net::UdpSocket;

use lifx_core::{BuildOptions, Message, RawMessage};

#[derive(Debug)]
pub struct Light<A: ToSocketAddrs> {
    pub device: A,
    pub socket: UdpSocket,
    pub options: BuildOptions,
}

impl<A: ToSocketAddrs> Light<A>
where
    A: Copy,
{
    /// Create new light with ip address `device` (see [`ToSocketAddrs`]) and optional BuildOptions for message header
    pub fn new(device: A) -> Result<Self, std::io::Error> {
        let socket = UdpSocket::bind("[::]:0")?;
        socket.connect(device)?;
        let options = BuildOptions::default();

        Ok(Self {
            device,
            socket,
            options,
        })
    }

    /// Get [`RawMessage`] from [`Message`]
    pub fn raw_message(&self, message: Message) -> Result<RawMessage, Box<dyn Error>> {
        Ok(RawMessage::build(&self.options, message)?)
    }

    /// Send `message` to self
    pub fn send(&self, message: Message) -> Result<(), Box<dyn Error>> {
        let bytes = self.raw_message(message)?.pack()?;
        self.socket.send_to(&bytes, self.device)?;
        Ok(())
    }

    pub fn receive(&self) -> Result<Message, Box<dyn Error>> {
        Ok(Message::GetInfo)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    const TAKLAMPA: &str = "192.168.1.99:56700";
    #[allow(dead_code)]
    const LIFXZ: &str = "192.168.1.45:56700";

    #[test]
    fn test_connect() {
        let light = Light::new(TAKLAMPA).unwrap();
        assert_eq!(light.device, TAKLAMPA);
    }

    #[test]
    fn test_raw_message() {
        let light = Light::new(TAKLAMPA).unwrap();
        let message = Message::LightSetPower {
            level: 0xFF,
            duration: 0,
        };
        let raw_message = light.raw_message(message).unwrap();
        raw_message.validate();
        assert_eq!(raw_message.packed_size(), 42);
        assert_eq!(
            raw_message.pack().unwrap(),
            vec![
                42, 0, 0, 52, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 117, 0, 0, 0, 255, 255, 0, 0, 0, 0
            ]
        );
    }

    #[test]
    fn test_get_info() {
        let light = Light::new(TAKLAMPA).unwrap();
        let message = Message::GetInfo;
        light.send(message).unwrap();
    }

    #[test]
    fn test_turn_on() {
        let light = Light::new(TAKLAMPA).unwrap();
        let message = Message::LightSetPower {
            level: u16::MAX,
            duration: 0,
        };
        light.send(message).unwrap();
    }

    #[test]
    fn test_turn_off() {
        let light = Light::new(TAKLAMPA).unwrap();
        let message = Message::LightSetPower {
            level: u16::MIN,
            duration: 0,
        };
        light.send(message).unwrap();
    }
}
