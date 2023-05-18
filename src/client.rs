use {
    std::io::Write,
    interprocess::local_socket::LocalSocketStream,
    anyhow::{Result, anyhow},
    crate::server::PolydoroServer
};

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
    let socket = PolydoroServer::build_socket_path(&polydoro_puid);

    let mut stream = LocalSocketStream::connect(socket)?;
    let buf: [u8; 1] = [opcode as u8]; 
    stream.write(&buf)?;

    Ok(())
}
