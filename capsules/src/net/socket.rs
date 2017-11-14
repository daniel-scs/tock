use net::ip::{IPAddr, IP6Header};
use core::cell::{Cell};
use kernel::common::list::{List, ListLink, ListNode};

// An address is the name of a communication endpoint
#[derive(Copy, Clone, Debug)]
pub struct UdpAddress {
    ip_addr: IPAddr,
    port: u16,
}

// A socket represents a local communication endpoint.
// It contains an address that is matched against the destination
// of incoming packets, and a reference to the client who should
// receive matching packets.
pub struct UdpSocket<'a> {
  address: Cell<Option<UdpAddress>>,
  client: &'a UdpClient<'a>,
  next_link: ListLink<'a, UdpSocket<'a>>,
}

impl<'a> UdpSocket<'a> {
    pub fn new(client: &'a UdpClient<'a>) -> Self {
        UdpSocket {
            address: Cell::new(None),
            client: client,
            next_link: ListLink::empty(),
        }
    }

    pub fn bind(&self, address: UdpAddress) {
        self.address.set(Some(address));
    }

    pub fn close(&self) {
        self.address.set(None);
    }
}

impl<'a> ListNode<'a, UdpSocket<'a>> for UdpSocket<'a> {
    fn next(&'a self) -> &'a ListLink<'a, UdpSocket<'a>> {
        &self.next_link
    }
}

// A client of the networking subsystem should implement this trait
// in order to receive UDP packets.
pub trait UdpClient<'a> {
    fn recv(&'a self, ip_header: IP6Header, ip_payload: &'a mut [u8]);
}

// A representation of all local UDP endpoints
pub struct Udp<'a> {
  sockets: List<'a, UdpSocket<'a>>
}

impl<'a> Udp<'a> {
    pub fn new() -> Udp<'a> {
        Udp { sockets: List::new() }
    }

    // Link a socket into the UDP networking system.
    // If the socket's address field is non-empty, "bind" is implicit here
    pub fn socket_add(&self, socket: &'a mut UdpSocket<'a>) {
        self.sockets.push_head(socket);
    }

    // Process an incoming UDP packet: If a matching socket is found,
    // pass it to the associated client and return true; else return false.
    pub fn receive(_ip_header: IP6Header, _ip_payload: &'a mut [u8]) -> bool {
        // XXX: Search addresses in self.sockets for a matching address
        // and port number, and pass the packet to any matching client.

        return false;
    }
}
