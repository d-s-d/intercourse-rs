//! This is the entry point ot the library network-simulator; dependees only see
//! what is exposed here.
//!
//! The network-simulator library is a naive

#![allow(dead_code)]
pub mod network;

use std::collections::HashMap;

/// A router receives packets and makes a decision on how to foward those
/// packets. The canonical router is stateless and just a function that maps a
/// packet onto the corresponding interface.
trait Router {
    // Here we could pin down the receiver to be just &self which, in principle,
    // would ensure that the state of the router does not change. However, we
    // want to allow for stateful routers.
    fn route_packet(&mut self, iface_state: HashMap<NodeAddress, &mut Interface>);
}

/// Address that uniquely identifies a node in the network.
// Note: The wrapped field is not marked `pub`, thus only methods inside this
// module can access it. It also does not appear in the documentation.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NodeAddress([u16; 2]);

// in : Hop -> Q<Packet>
// out: Hop -> Q<Packet>

// RouterFn: Vec((Interface, Packet)) -> Vec((Interface, Packet))

/// A stateless reliable router routes packets reliably without checking
/// consistency of incoming packets.
struct StatelessReliableRouter();

impl Router for StatelessReliableRouter {
    fn route_packet(&mut self, _state: HashMap<NodeAddress, &mut Interface>) {
        unimplemented!()
    }
}

/*
networkspec.add_router(NodeAddress(x), )
*/

pub struct Packet {
    path: Vec<NodeAddress>,
    current_hop: usize,
    payload: PacketPayload,
}

pub enum PacketPayload {
    Payload(String),
    Control(String),
}

struct Interface();
