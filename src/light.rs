use std::error::Error;
use std::net::ToSocketAddrs;
use std::net::UdpSocket;

use lifx_core::{BuildOptions, Message, RawMessage};

use crate::SOCKET_TIMEOUT;

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
        // "[::]:0" for all addresses
        let socket = UdpSocket::bind("[::]:0")?;
        socket.connect(device)?;
        socket.set_read_timeout(Some(SOCKET_TIMEOUT))?;
        socket.set_write_timeout(Some(SOCKET_TIMEOUT))?;
        let options = BuildOptions::default();

        Ok(Self {
            device,
            socket,
            options,
        })
    }

    /// Get [`RawMessage`] from [`Message`]
    pub fn raw_message(&self, message: Message) -> Result<RawMessage, Box<dyn Error>> {
        Ok(RawMessage::build(&self.options, message.clone())?)
    }

    /// Send `message` to self
    pub fn send(&self, message: Message) -> Result<(), Box<dyn Error>> {
        let bytes = self.raw_message(message)?.pack()?;
        self.socket.send(&bytes)?;
        Ok(())
    }

    pub fn receive(&self) -> Result<Message, Box<dyn Error>> {
        let mut buf = [0; 1024];
        self.socket.recv(&mut buf)?;
        let raw = RawMessage::unpack(&buf)?;
        Ok(Message::from_raw(&raw)?)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use lifx_core::{EchoPayload, Service};

    const TAKLAMPA: &str = "192.168.1.11:56700";
    #[allow(dead_code)]
    const LIFXZ: &str = "192.168.1.12:56700";

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
    fn test_echo() {
        let light = Light::new(TAKLAMPA).unwrap();
        let payload = [5; 64];
        let message = Message::EchoRequest {
            payload: EchoPayload(payload),
        };
        light.send(message.clone()).unwrap();
        let response = light.receive().unwrap();
        assert!(matches!(response, Message::EchoResponse { .. }));
        if let Message::EchoResponse {
            payload: EchoPayload(resp_payload),
        } = response
        {
            assert_eq!(payload, resp_payload);
        };
    }

    #[test]
    fn test_service() {
        let light = Light::new(TAKLAMPA).unwrap();
        light.send(Message::GetService).unwrap();
        let response = light.receive().unwrap();
        if let Message::StateService { port, service } = response {
            assert_eq!(port, 56700);
            assert_eq!(service, Service::UDP);
        } else {
            panic!("No StateService response from GetService");
        }
    }

    #[test]
    #[ignore = "requires lifx device on network"]
    fn test_all_info() {
        for message in [
            Message::GetGroup,
            Message::GetHostFirmware,
            Message::GetHostInfo,
            Message::GetInfo,
            Message::GetLabel,
            Message::GetLocation,
            Message::GetPower,
            Message::GetService,
            Message::GetVersion,
            Message::GetWifiFirmware,
            Message::GetWifiInfo,
        ] {
            let light = Light::new(TAKLAMPA).unwrap();
            light.send(message).unwrap();
            let response = light.receive().unwrap();
            println!("{:#?}", response);
        }
    }

    #[test]
    #[ignore = "requires lifx device on network"]
    fn test_turn_on() {
        let light = Light::new(TAKLAMPA).unwrap();
        let message = Message::LightSetPower {
            level: u16::MAX,
            duration: 0,
        };
        light.send(message).unwrap();
    }

    #[test]
    #[ignore = "requires lifx device on network"]
    fn test_turn_off() {
        let light = Light::new(TAKLAMPA).unwrap();
        let message = Message::LightSetPower {
            level: u16::MIN,
            duration: 0,
        };
        light.send(message).unwrap();
    }
}
