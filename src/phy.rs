extern crate mio;

use std::net::SocketAddr;
use mio::net::UdpSocket;
use mio::{Events, Interest, Poll, Token};
use core::time::Duration;
use zt::core::{Node, PhyProvider};
use failure::Fallible;

const MAIN: Token = Token(0);
const SECONDARY: Token = Token(1);

#[derive(Debug)]
pub struct Phy {
    main: UdpSocket,
    secondary: UdpSocket,
    poll: Poll,
}

impl Phy {
    pub fn new(port: u16, secondary_port: u16) -> Fallible<Phy> {
        let mut main = UdpSocket::bind(format!("0.0.0.0:{}", port).parse()?)?;
        let mut secondary = UdpSocket::bind(format!("0.0.0.0:{}", secondary_port).parse()?)?;
        let poll = Poll::new()?;

        poll.registry().register(&mut main, MAIN, Interest::READABLE)?;
        poll.registry().register(&mut secondary, SECONDARY, Interest::READABLE)?;

        Ok(Self {
            main: main,
            secondary: secondary,
            poll: poll,
        })
    }

    pub fn poll(&mut self, node: &Node) -> Fallible<()> {
        let mut events = Events::with_capacity(1024);

        self.poll.poll(&mut events, Some(Duration::from_millis(200)))?;

        for event in &events {
            let mut buf = [0u8; 2048];
            if let Some((len, addr)) = match event.token() {
                MAIN => Some(self.main.recv_from(&mut buf).unwrap()),
                SECONDARY => Some(self.secondary.recv_from(&mut buf).unwrap()),
                Token(_) => None,
            } {
                let res = node.process_wire_packet(self, &buf, len, &addr, event.token().0 as i64);
                if let Err(error) = res {
                    println!("process_wire_packet failed: {}", error);
                }
            }
        };
        Ok(())
    }
}

impl PhyProvider for Phy {
    fn send(&self, address: &SocketAddr, socket: i64, buf: &[u8]) -> usize {
        match Token(socket as usize) {
            MAIN => self.main.send_to(buf, address.clone()).unwrap(),
            SECONDARY => self.secondary.send_to(buf, address.clone()).unwrap(),
            Token(_) => 0usize,
        }
    }

    fn send_all(&self, address: &SocketAddr, buf: &[u8]) -> usize {
        self.main.send_to(buf, address.clone()).unwrap();
        self.secondary.send_to(buf, address.clone()).unwrap();
        0
    }
}
