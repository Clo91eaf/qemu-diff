use crate::protocol::{self, GdbConn};
use std::io::Write;

// Connect to QEMU's GDB server
pub fn gdb_connect_qemu(port: u16) -> GdbConn {
    loop {
        let conn =  protocol::gdb_begin_inet("localhost", port);
        if conn.is_ok() {
            return conn.unwrap();
        }
    }
}

fn gdb_memcpy_to_qemu_small(conn: &mut GdbConn, dest: u32, src: &[u8]) {
    let mut buf = Vec::with_capacity(src.len() * 2 + 128);
    write!(buf, "M0x{:x},{:x}:", dest, src.len()).unwrap();
    write!(buf, "{:x}", u8::from_str_radix(std::str::from_utf8(src).unwrap(), 16).unwrap()).unwrap();

    conn.send(&buf).unwrap();

    let reply = String::from_utf8(conn.recv().unwrap()).unwrap();

    assert_eq!(reply, "OK");
}

pub fn gdb_memcpy_to_qemu(conn: &mut Option<GdbConn>, dest: u32, src: &[u8]) {
    const MTU: usize = 1500;
    let mut offset = 0;

    while offset < src.len() {
        let chunk_len = std::cmp::min(MTU, src.len() - offset);
        let chunk = &src[offset..offset + chunk_len];
        gdb_memcpy_to_qemu_small(conn.as_mut().unwrap(), dest + offset as u32, chunk);
        offset += MTU;
    }
}

pub fn gdb_getregs(conn: &mut Option<GdbConn>, r: &mut [u32; 43]) {
    // Send the "g" packet to request registers data
    conn.as_mut().unwrap().send(b"g").unwrap();

    // Receive the registers data from GDB
    let mut response = conn.as_mut().unwrap().recv().unwrap();

    // Parse the received data and update the registers array
    for reg in r.iter_mut() {
        let (reg_value_str, rest) = response.split_at(8);
        *reg = u32::from_str_radix(&String::from_utf8_lossy(reg_value_str), 16).unwrap();
        response = rest.to_vec();
    }
}

pub fn gdb_setregs(conn: &mut Option<GdbConn>, r: &[u32; 43]) {
    let mut buf = Vec::with_capacity(r.len() * 8 + 1);
    buf.push(b'G');
    for reg in r.iter() {
        write!(buf, "{:08x}", reg).unwrap();
    }

    conn.as_mut().unwrap().send(&buf).unwrap();

    let reply = String::from_utf8(conn.as_mut().unwrap().recv().unwrap()).unwrap();

    assert_eq!(reply, "OK");
}

pub fn gdb_si(conn: &mut Option<GdbConn>) {
    conn.as_mut().unwrap().send(b"vCont;s:1").unwrap();
}
