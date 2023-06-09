use std::{io::{Cursor, Write, Read, self}, net::{SocketAddr, IpAddr}};
use byteorder::{WriteBytesExt, BigEndian, ReadBytesExt};

pub trait Serializable {
    type Output;

    fn write(&self, buf: &mut Vec<u8>) -> io::Result<()>;
    fn read(buf: &mut Cursor<&[u8]>) -> io::Result<Self::Output>;

    fn size(&self) -> usize;
}

pub fn write_byte_arr<const N: usize>(buf: &mut Vec<u8>, arr: &[u8; N]) -> io::Result<()> {
    Ok(buf.write_all(arr)?)
}

pub fn read_byte_arr<const N: usize>(buf: &mut Cursor<&[u8]>) -> io::Result<[u8; N]> {
    let mut arr = [0; N];
    buf.read_exact(&mut arr)?;
    Ok(arr)
}

pub fn write_arr<T: Serializable, const N: usize>(buf: &mut Vec<u8>, arr: &[T; N]) -> io::Result<()> {
    //buf.write_u16::<BigEndian>(N as u16)?;
    Ok(for t in arr.iter() {
        t.write(buf)?;
    })
}

pub fn read_arr<T: Serializable<Output = T> + Copy + Default, const N: usize>(buf: &mut Cursor<&[u8]>) -> io::Result<[T; N]> {
    let mut arr = [T::default(); N];
    for i in 0..N {
        arr[i] = T::read(buf)?;
    }
    Ok(arr)
}

pub fn write_byte_vec(buf: &mut Vec<u8>, vec: &Vec<u8>) -> io::Result<()> {
    buf.write_u16::<BigEndian>(vec.len() as u16)?;
    Ok(buf.write_all(vec)?)
}

pub fn read_byte_vec(buf: &mut Cursor<&[u8]>) -> io::Result<Vec<u8>> {
    let len = buf.read_u16::<BigEndian>()?;
    let mut vec = vec![0u8; len as usize];
    buf.read_exact(&mut vec)?;
    Ok(vec)
}

pub fn write_vec<T: Serializable>(buf: &mut Vec<u8>, vec: &Vec<T>) -> io::Result<()> {
    buf.write_u32::<BigEndian>(vec.len() as u32)?;
    for i in vec.iter() {
        i.write(buf)?;
    }
    Ok(())
}

pub fn read_vec<T: Serializable<Output = T>>(buf: &mut Cursor<&[u8]>) -> io::Result<Vec<T>> {
    let len = buf.read_u32::<BigEndian>()? as usize;
    let mut vec = Vec::with_capacity(len);
    for _ in 0..len {
        vec.push(T::read(buf)?);
    }
    Ok(vec)
}

pub fn write_string(buf: &mut Vec<u8>, str: &str) -> io::Result<()> {
    let bytes = &str.as_bytes().to_vec();
    write_byte_vec(buf, bytes)
}

pub fn read_string(buf: &mut Cursor<&[u8]>) -> io::Result<String> {
    let bytes = read_byte_vec(buf)?;
    let string = String::from_utf8(bytes).unwrap();
    Ok(string) // TODO: Check err...
}

pub fn write_sock_addr(buf: &mut Vec<u8>, addr: &SocketAddr) -> io::Result<()> {
    match addr.ip() {
        std::net::IpAddr::V4(ip) => {
            buf.write_u8(0)?;
            write_byte_arr::<4>(buf, &ip.octets())
        },
        std::net::IpAddr::V6(ip) =>  {
            buf.write_u8(1)?;
            write_byte_arr::<16>(buf, &ip.octets())
        }
    }?;
    buf.write_u16::<BigEndian>(addr.port())?;
    Ok(())
}

pub fn read_sock_addr(buf: &mut Cursor<&[u8]>) -> io::Result<SocketAddr> {
    let ip_addr_type = buf.read_u8()?;
    let ip_addr = match ip_addr_type {
        0 => IpAddr::V4(read_byte_arr::<4>(buf)?.into()),
        1 => IpAddr::V6(read_byte_arr::<16>(buf)?.into()),
        n @ _ => panic!("Index out of bounds: {n}.")
    };    
    let port = buf.read_u16::<BigEndian>()?;
    Ok((ip_addr, port).into())
}

pub fn get_sock_addr_size(addr: &SocketAddr) -> usize {
    (if addr.is_ipv4() {
        4
    } else {
        16
    }) + 2
}

// WARNING: write_instant()/read_instant() not usable, as Instant uses
// floating types internally, which can not be routinely parsed without losing
// data integrity.

// pub fn write_instant(buf: &mut Vec<u8>, time: Instant) -> io::Result {
//     Ok(buf.write_f32::<BigEndian>(time.elapsed().as_secs_f32())?)
// }
// pub fn read_instant(buf: &mut Cursor<&[u8]>) -> io::Result<Instant> {
//     // TODO: Improve serialization
//     let elapsed = Duration::from_secs_f32(buf.read_f32::<BigEndian>()?);
//     Ok(Instant::now().checked_sub(elapsed).unwrap())
// }

pub trait Serializer : Serializable {
    fn serialize(&self) -> io::Result<Vec<u8>> {
        let mut buf = vec![];
        self.write(&mut buf)?;
        Ok(buf)
    }
    fn deserialize(buf: &[u8]) -> io::Result<Self::Output> {
        Ok(Self::read(&mut Cursor::new(&buf))?)
    }
}
