use std::io::{self, BufReader, Read, Write};
use std::net::{TcpStream, ToSocketAddrs};

pub struct GdbConn {
    reader: BufReader<TcpStream>,
    writer: TcpStream,
    ack: bool,
}

impl GdbConn {
    pub fn new(reader: TcpStream, writer: TcpStream) -> Self {
        Self {
            reader: BufReader::new(reader),
            writer,
            ack: true,
        }
    }

    fn send_packet(&mut self, command: &[u8]) -> io::Result<()> {
        let sum: u8 = command.iter().copied().sum();
        write!(self.writer, "${}", String::from_utf8_lossy(command))?;
        write!(self.writer, "#{:02X}", sum)?;
        self.writer.flush()?;
        Ok(())
    }

    fn recv_packet(&mut self) -> io::Result<(Vec<u8>, bool)> {
        let mut reply = Vec::with_capacity(4096);
        let mut sum = 0u8;
        let mut escape = false;
        let mut buf = [0; 1];

        // Fast-forward to the start of packet
        loop {
            self.reader.read_exact(&mut buf)?;
            match buf[0] {
                b'$' => break,
                _ => continue,
            }
        }

        loop {
            self.reader.read_exact(&mut buf)?;
            match buf[0] {
                b'$' => {
                    // New packet, start over...
                    reply.clear();
                    sum = 0;
                    escape = false;
                }
                b'#' => {
                    // End of packet
                    let mut checksum = [0u8; 2];
                    self.reader.read_exact(&mut checksum)?;

                    let checksum_str = std::str::from_utf8(&checksum).unwrap();

                    let ret_sum_ok = sum == u8::from_str_radix(checksum_str, 16).unwrap();

                    return Ok((reply, ret_sum_ok));
                }
                b'}' => {
                    escape = true;
                }
                b'*' => {
                    // Run-length-encoding
                    // The next character tells how many times to repeat the last
                    // character we saw. The count is added to 29, so that the
                    // minimum-beneficial RLE 3 is the first printable character ' '.
                    // The count character can't be >126 or '$'/'#' packet markers.
                    if !reply.is_empty() {
                        let mut buf = [0; 1];
                        self.reader.read_exact(&mut buf)?;
                        let count = buf[0] - 29;
                        let last_byte = *reply.last().unwrap();
                        reply.extend(std::iter::repeat(last_byte).take(count as usize));
                        sum += count;
                    }
                }
                c => {
                    // XOR an escaped character
                    let c = if escape { c ^ 0x20 } else { c };
                    escape = false;
                    reply.push(c);
                    sum += c;
                }
            }
        }
    }

    pub fn send(&mut self, command: &[u8]) -> io::Result<()> {
        loop {
            self.send_packet(command)?;
            if !self.ack {
                break;
            }

            // Look for '+' ACK or '-' NACK/resend
            let mut buf = [0; 1];
            self.reader.read_exact(&mut buf)?;
            match buf[0] {
                b'+' => {
                    break;
                }
                _ => continue,
            }
        }

        Ok(())
    }

    pub fn recv(&mut self) -> io::Result<Vec<u8>> {
        let mut reply = Vec::new();

        loop {
            let (data, ret_sum_ok) = self.recv_packet()?;
            reply.extend_from_slice(&data);

            if !self.ack {
                break;
            }

            // Send +/- depending on checksum result, retry if needed
            let ack = if ret_sum_ok { b'+' } else { b'-' };
            self.writer.write_all(&[ack])?;
            self.writer.flush()?;

            if ret_sum_ok {
                break;
            }
        }

        Ok(reply)
    }
}

pub fn gdb_begin_inet(addr: &str, port: u16) -> io::Result<GdbConn> {
    let mut addr_iter = (addr, port).to_socket_addrs()?;
    let remote_addr = addr_iter
        .next()
        .ok_or_else(|| io::Error::new(io::ErrorKind::AddrNotAvailable, "No address found"))?;
    let stream = TcpStream::connect(remote_addr)?;

    // Disable Nagle's algorithm
    stream.set_nodelay(true)?;

    // Disable SIGPIPE
    let conn = GdbConn::new(stream.try_clone()?, stream);

    Ok(conn)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connect() {
        let port = 1145;
        assert_eq!(gdb_begin_inet("localhost", port).is_ok(), true);
    }
}
