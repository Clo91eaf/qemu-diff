mod gdb;
mod protocol;

use gdb::gdb_connect_qemu;
use std::process::{Command, Stdio};

fn difftest_init(port: u16) {
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

    gdb_connect_qemu(port);
}

pub fn difftest_memcpy(dest: &mut [u8], src: &[u8]) {
    todo!("difftest_memcpy")
}

pub fn difftest_regcpy(dest: &mut [u8], src: &[u8]) {
    todo!("difftest_regcpy")
}

pub fn difftest_exec() {
    todo!("difftest_exec")
}

pub fn difftest_error() {
    todo!("difftest_error")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_difftest_init() {
        difftest_init(1145);
    }
}
