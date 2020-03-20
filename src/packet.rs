//! Packet definitions

use crate::nineteen;
use crate::packet::event::EventPacket;
use crate::packet::lap::LapPacket;
use bytes::{Buf, BytesMut};
use std::io::{Cursor, Error, ErrorKind};

pub mod event;
pub mod header;
pub mod lap;

/// A packet published by an F1 game.
///
/// The F1 games publish different packets with different data at different intervals. Each of these
/// packets is decoded from UDP to their respective representation in this Rust crate. The `Packet`
/// enum lists all packets that can be expected, and that a client should handle.
pub enum Packet {
    /// The F1 games send event packets whenever certain events occur in a session. Some event
    /// packets carry a payload with more information about the event.
    Event(EventPacket),

    /// Lap data packets provide information about each car in a session, and are sent at an
    /// interval that can be configured in the game.
    Lap(LapPacket),

    /// Packet from F1 2019
    Nineteen(nineteen::Packet),
}

/// Ensure a packet has the expected size
///
/// Modern F1 games send their packets over UDP. Depending on their size, these packets might be
/// split into multiple UDP fragments. The decoder collects these fragments, and asks the codec if
/// enough data has been received to decode a packet.
///
/// The sizes of the packets sent by F1 games are part of the API specification, and can be used to
/// determine if a full packet has ben received. This function takes a cursor to the raw data and
/// the expected size of the packet, and returns an error if not enough data is ready to decode the
/// complete packet.
pub(crate) fn ensure_packet_size(
    expected_size: usize,
    cursor: &mut Cursor<&mut BytesMut>,
) -> Result<(), Error> {
    if cursor.remaining() < expected_size {
        Err(Error::new(
            ErrorKind::UnexpectedEof,
            format!(
                "Packet is expected to have a size of {} bytes, but was {}.",
                expected_size,
                cursor.remaining()
            ),
        ))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::packet::ensure_packet_size;
    use bytes::{Buf, BufMut, BytesMut};
    use std::io::{Cursor, Error};

    struct Packet {
        counter: u8,
    }

    const PACKET_SIZE: usize = 1;

    fn decode_packet(cursor: &mut Cursor<&mut BytesMut>) -> Result<Packet, Error> {
        ensure_packet_size(PACKET_SIZE, cursor)?;

        Ok(Packet {
            counter: cursor.get_u8(),
        })
    }

    #[test]
    fn ensure_packet_size_correctly() {
        let mut bytes = BytesMut::with_capacity(1);
        bytes.put_u8(0);

        let mut cursor = Cursor::new(&mut bytes);

        let packet = decode_packet(&mut cursor).unwrap();
        assert_eq!(0, packet.counter);
    }

    #[test]
    fn ensure_packet_size_with_error() {
        let mut bytes = BytesMut::with_capacity(0);
        let mut cursor = Cursor::new(&mut bytes);

        let packet = decode_packet(&mut cursor);
        assert!(packet.is_err());
    }
}
