mod gdb;
mod protocol;

use gdb::{gdb_connect_qemu, gdb_getregs, gdb_memcpy_to_qemu, gdb_setregs, gdb_si};
use protocol::GdbConn;
use std::process::{Command, Stdio};

pub struct Difftest {
    conn: Option<GdbConn>,
}

impl Difftest {
    pub fn new() -> Self {
        Difftest { conn: None }
    }

    pub fn init(mut self, port: u16) {
        // start qemu
        let _ = Command::new("qemu-system-riscv64")
            .arg("-S")
            .arg("-gdb")
            .arg("tcp::{port}")
            .arg("-nographic")
            .arg("-serial")
            .arg("none")
            .arg("-monitor")
            .arg("none")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn();

        self.conn = Some(gdb_connect_qemu(port));
    }

    pub fn memcpy(&mut self, dest: &mut [u8], src: &[u8]) {
        gdb_memcpy_to_qemu(&mut self.conn, dest.as_ptr() as u32, src);
    }

    // if direction == true, copy from src to qemu
    pub fn regcpy(&mut self, r: &mut [u32; 43], direction: bool) {
        if direction {
            gdb_setregs(&mut self.conn, r);
        } else {
            gdb_getregs(&mut self.conn, r);
        }
    }

    pub fn exec(&mut self, n: u64) {
        (0..n).for_each(|_| gdb_si(&mut self.conn));
    }
}
