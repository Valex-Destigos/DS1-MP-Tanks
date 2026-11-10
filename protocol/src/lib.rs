use glam::Vec2;
use postcard::Error;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub enum Packet {
    Input(InputFrame),
    TankSnap(TankSnapshot),
    ServerSnap(ServerSnapshot),
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct InputFrame {
    pub seq: u32,
    pub move_dir: Vec2,
    pub shoot: bool,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct TankSnapshot {
    pub id: u32,
    pub position: Vec2,
    pub rotation: f32,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct ServerSnapshot {
    pub tick: u64,
    pub tanks: Vec<TankSnapshot>,
}

pub fn serialize_packet(packet: &Packet) -> Result<Vec<u8>, Error> {
    postcard::to_allocvec(packet)
}

pub fn deserialize_packet(bytes: &[u8]) -> Result<Packet, Error> {
    postcard::from_bytes(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> Vec<Packet> {
        let mut packets = Vec::with_capacity(3);

        let frame = InputFrame {
            seq: 1,
            move_dir: Vec2::new(1.0, 3.0),
            shoot: false,
        };

        let tank1_snap = TankSnapshot {
            id: 420,
            position: Vec2::new(345.67, 111.222),
            rotation: 0.69,
        };

        let tank2_snap = TankSnapshot {
            id: 69,
            position: Vec2::new(2340.344, 12.432),
            rotation: 0.333,
        };

        let server_snap = ServerSnapshot {
            tick: 1000,
            tanks: vec![tank1_snap.clone(), tank2_snap.clone()],
        };

        packets.push(Packet::Input(frame));
        packets.push(Packet::TankSnap(tank1_snap));
        packets.push(Packet::TankSnap(tank2_snap));
        packets.push(Packet::ServerSnap(server_snap));

        packets
    }

    #[test]
    fn test_serde() {
        let packets = setup();

        for packet in packets {
            let serialized = serialize_packet(&packet)
                .unwrap_or_else(|e| panic!("Serialization failed with following error: {}", e));
            let roundtrip_packet = deserialize_packet(&serialized)
                .unwrap_or_else(|e| panic!("Deserialization failed with following error: {}", e));

            assert_eq!(packet, roundtrip_packet);
        }
    }
}
