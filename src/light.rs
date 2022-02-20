use std::error::Error;
use std::fmt;
use std::net::ToSocketAddrs;
use std::net::UdpSocket;
use std::time::Duration;

use lifx_core::HSBK;
use lifx_core::{BuildOptions, Message, RawMessage};

use crate::SOCKET_TIMEOUT;

/// Minimum light brightness (to that is still on/visible)
///
/// `328 = 0x148 = 2% of 0xFFFF`
pub const MIN: u16 = (u16::MAX as f64 / 200_f64 + 0.5_f64) as u16; //
/// Maximum light brightness
///
/// `65535 = 0xFFFF = 100% of 0xFFFF`
pub const MAX: u16 = u16::MAX;

#[derive(Debug, Clone, PartialEq)]
pub struct WrongMessageError(Message);
impl fmt::Display for WrongMessageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "wrong message received from light after get message: {:?}",
            self.0
        )
    }
}
impl Error for WrongMessageError {}

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

    pub fn change_color(
        &self,
        change: fn(HSBK) -> HSBK,
        duration: Duration,
    ) -> Result<(), Box<dyn Error>> {
        self.send(Message::LightGet)?;
        match self.receive()? {
            Message::LightState { color, .. } => {
                let new_color = change(color);
                self.send(Message::LightSetColor {
                    color: new_color,
                    duration: duration.as_millis() as u32,
                    reserved: 0,
                })?;
                Ok(())
            }
            msg => Err(Box::new(WrongMessageError(msg))),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::TAKLAMPA;

    use super::*;
    use lifx_core::{EchoPayload, Service, HSBK};

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
    fn test_get_color() {
        let light = Light::new(TAKLAMPA).unwrap();
        light.send(Message::LightGet).unwrap();
        let response = light.receive().unwrap();
        println!("{:#?}", response);
        match response {
            Message::LightState { label, .. } if label == *"Taklampa" => {}
            _ => panic!(),
        }
    }

    #[test]
    fn test_all_info() {
        let light = Light::new(TAKLAMPA).unwrap();
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
            light.send(message).unwrap();
            let response = light.receive().unwrap();
            println!("{:#?}", response);
        }
    }

    #[test]
    #[ignore = "changes light state"]
    fn test_set_color() {
        let light = Light::new(TAKLAMPA).unwrap();
        light
            .send(Message::LightSetColor {
                color: HSBK {
                    hue: 0,
                    saturation: 0,
                    brightness: 0xFFFF / 2,
                    kelvin: 3500,
                },
                duration: 0,
                reserved: 0,
            })
            .unwrap();
    }
}
