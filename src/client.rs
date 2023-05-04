use std::io::Write;

use interprocess::local_socket::LocalSocketStream;

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

pub fn send_polydoro_message(polypomo_puid: String, opcode: OpCode) {
    let mut stream = LocalSocketStream::connect(polypomo_puid).unwrap();
    let buf: [u8; 1] = [opcode as u8]; 
    stream.write(&buf).unwrap();
}
