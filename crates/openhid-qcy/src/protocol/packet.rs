//! QCY 0xFF packet framing and parsing.
//!
//! A command frame sent to characteristic `00001001` or received on `00001002`
//! has the structure:
//! ```text
//! [0xFF, body_len, opcode, param_len, params...]
//! ```
//! Multiple commands can be chained within a single framed packet payload.

/// A single decoded command opcode and its parameters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QcyCommand {
    pub opcode: u8,
    pub parameters: Vec<u8>,
}

/// Pack a single command into a 0xFF-framed buffer.
#[must_use]
pub fn pack_command(opcode: u8, parameters: &[u8]) -> Vec<u8> {
    let body_len = (parameters.len() + 2) as u8;
    let mut packet = Vec::with_capacity(parameters.len() + 4);
    packet.push(0xFF);
    packet.push(body_len);
    packet.push(opcode);
    packet.push(parameters.len() as u8);
    packet.extend_from_slice(parameters);
    packet
}

/// Parse one or more chained commands from a 0xFF-framed packet buffer.
#[must_use]
pub fn parse_packet(packet: &[u8]) -> Vec<QcyCommand> {
    if packet.len() < 4 || packet[0] != 0xFF {
        return Vec::new();
    }

    let body_len = packet[1] as usize;
    if body_len + 2 != packet.len() {
        return Vec::new();
    }

    let mut commands = Vec::new();
    let mut offset = 2;

    while offset < packet.len() {
        if offset + 2 > packet.len() {
            break;
        }
        let opcode = packet[offset];
        let param_len = packet[offset + 1] as usize;
        offset += 2;

        if offset + param_len > packet.len() {
            break;
        }

        commands.push(QcyCommand {
            opcode,
            parameters: packet[offset..offset + param_len].to_vec(),
        });
        offset += param_len;
    }

    commands
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packs_and_unpacks_single_command() {
        let packet = pack_command(0x17, &[0x01, 0x05, 0x00]);
        assert_eq!(packet, vec![0xFF, 0x05, 0x17, 0x03, 0x01, 0x05, 0x00]);

        let commands = parse_packet(&packet);
        assert_eq!(commands.len(), 1);
        assert_eq!(commands[0].opcode, 0x17);
        assert_eq!(commands[0].parameters, vec![0x01, 0x05, 0x00]);
    }

    #[test]
    fn parses_chained_response_commands() {
        // [0xFF, 0x06, (0x09, 0x01, 0x02), (0x10, 0x01, 0x01)]
        let packet = [0xFF, 0x06, 0x09, 0x01, 0x02, 0x10, 0x01, 0x01];
        let commands = parse_packet(&packet);
        assert_eq!(commands.len(), 2);
        assert_eq!(commands[0].opcode, 0x09);
        assert_eq!(commands[0].parameters, vec![0x02]);
        assert_eq!(commands[1].opcode, 0x10);
        assert_eq!(commands[1].parameters, vec![0x01]);
    }

    #[test]
    fn rejects_malformed_packets() {
        assert!(parse_packet(&[]).is_empty());
        assert!(parse_packet(&[0xFE, 0x02, 0x01, 0x00]).is_empty());
        assert!(parse_packet(&[0xFF, 0x05, 0x01, 0x00]).is_empty()); // truncated
    }
}
