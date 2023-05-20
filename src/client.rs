use {
    std::io::Write,
    interprocess::local_socket::LocalSocketStream,
    anyhow::{Result, anyhow},
};

#[derive(Eq, PartialEq, Debug, Clone)]
#[repr(u8)]
pub enum OpCode {
    Toggle = 0,
    Skip = 1
}

pub fn opcode_from_byte(byte: u8) -> Result<OpCode> {
    match byte {
        0 => Ok(OpCode::Toggle),
        1 => Ok(OpCode::Skip),
        _ => Err(anyhow!("Got bad opcode from peer: {}", byte)),
    }
}

pub fn send_polydoro_message(polydoro_puid: String, opcode: OpCode) -> Result<()> {
    let mut stream = LocalSocketStream::connect(polydoro_puid)?;
    let buf: [u8; 1] = [opcode as u8]; 
    stream.write(&buf)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opcode_from_byte_maps_to_opcode() {
        let opcodes = vec![OpCode::Skip, OpCode::Toggle];

        opcodes
            .into_iter()
        .for_each(|opcode| assert_eq!(
            opcode.clone(), 
            opcode_from_byte(opcode as u8).unwrap()
        ));

        assert!(opcode_from_byte(3 as u8).is_err());
    }
}
