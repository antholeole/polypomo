use std::io::Write;

use interprocess::local_socket::LocalSocketStream;

use crate::server::PolydoroServer;

#[repr(u8)]
pub enum OpCode {
    Toggle = 0,
    Skip = 1
}

pub fn opcode_from_byte(byte: u8) -> OpCode {
    match byte {
        0 => OpCode::Toggle,
        1 => OpCode::Skip,
        // TODO return a result!
        _ => panic!("got bad byte from peer: {}", byte),
    }
}

pub fn send_polydoro_message(polydoro_puid: String, opcode: OpCode) {
    let socket = PolydoroServer::build_socket_path(&polydoro_puid);

    let mut stream = LocalSocketStream::connect(socket).unwrap();
    let buf: [u8; 1] = [opcode as u8]; 
    stream.write(&buf).unwrap();
}
