use std::{
    io::{Read, Write},
    net::TcpStream,
};

use color_eyre::eyre::{Context, Result};
use protocol::net::{ClientPacket, ServerPacket};

fn main() -> Result<()> {
    color_eyre::install()?;
    let mut stream = TcpStream::connect("127.0.0.1:8080")?;

    write_packet(
        &mut stream,
        ClientPacket::JoinRequest {
            username: "BOB".into(),
        },
    )?;

    loop {
        let packet = read_packet(&mut stream)
            .with_context(|| "while listening for packets from the server")?;
        dbg!(packet);
    }
}

fn read_packet<R: Read>(rd: &mut R) -> Result<ServerPacket> {
    let mut len_buf = [0u8; 4];
    rd.read_exact(&mut len_buf)?;
    let len = u32::from_be_bytes(len_buf) as usize;

    let mut buf = vec![0u8; len];
    rd.read_exact(&mut buf)?;

    Ok(postcard::from_bytes(&buf)?)
}

fn write_packet<W: Write>(wr: &mut W, packet: ClientPacket) -> Result<()> {
    let buf = postcard::to_allocvec(&packet).unwrap();
    let len = buf.len() as u32;

    wr.write_all(&len.to_be_bytes())?;
    wr.write_all(&buf)?;

    Ok(())
}
