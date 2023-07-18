use crate::protocol::{self, GdbConn};
use std::io::Write;

pub fn gdb_connect_qemu(port: u16) {
    loop {
        if protocol::gdb_begin_inet("localhost", port).is_ok() {
            break;
        }
    }
}

fn gdb_memcpy_to_qemu_small(conn: &mut GdbConn, dest: u32, src: &[u8]) {
    let mut buf = Vec::with_capacity(src.len() * 2 + 128);
    write!(buf, "M0x{:x},{:x}:", dest, src.len()).unwrap();
    write!(
        buf,
        "{:x}",
        u8::from_str_radix(std::str::from_utf8(src).unwrap(), 16).unwrap()
    );

    conn.send(&buf).unwrap();

    let reply = String::from_utf8(conn.recv().unwrap()).unwrap();

    assert_eq!(reply, "OK");
}

fn gdb_memcpy_to_qemu(conn: &mut GdbConn, dest: u32, src: &[u8]) {
    const MTU: usize = 1500;
    let mut offset = 0;

    while offset < src.len() {
        let chunk_len = std::cmp::min(MTU, src.len() - offset);
        let chunk = &src[offset..offset + chunk_len];
        gdb_memcpy_to_qemu_small(conn, dest + offset as u32, chunk);
        offset += MTU;
    }
}

fn gdb_getregs(conn: &mut GdbConn, r: &mut [u32; 43]) {
    // Send the "g" packet to request registers data
    conn.send(b"g").unwrap();

    // Receive the registers data from GDB
    let mut response = conn.recv().unwrap();

    // Parse the received data and update the registers array
    for reg in r.iter_mut() {
        let (reg_value_str, rest) = response.split_at(8);
        *reg = u32::from_str_radix(&String::from_utf8_lossy(reg_value_str), 16).unwrap();
        response = rest.to_vec();
    }
}

fn gdb_setregs(conn: &mut GdbConn, r: &[u32; 43]) {
    let mut buf = Vec::with_capacity(r.len() * 8 + 1);
    buf.push(b'G');
    for reg in r.iter() {
        write!(buf, "{:08x}", reg).unwrap();
    }

    conn.send(&buf).unwrap();

    let reply = String::from_utf8(conn.recv().unwrap()).unwrap();

    assert_eq!(reply, "OK");
}

fn gdb_si(conn: &mut GdbConn) {
    let buf = "vCont;s:1";
    conn.send(buf.as_bytes()).unwrap();
}
