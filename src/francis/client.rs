use std::io::Cursor;

use async_std::{
    io::{self, ReadExt, WriteExt},
    net::{TcpStream, ToSocketAddrs},
};
use byteorder::WriteBytesExt;
use byteorder::{BigEndian, ReadBytesExt};
use futures_util::FutureExt;
use rand::{Rng, seq::SliceRandom};
use wgpu::util::align_to;

pub struct Francis {
    width: u16,
    height: u16,
    x: u16,
    y: u16,
    stream: TcpStream,
    buffer: Option<Vec<u8>>,
}

const ITEMS: usize = 5000;
impl Francis {
    pub async fn new(
        addr: String,
        w: Option<u16>,
        h: Option<u16>,
    ) -> io::Result<Self> {
        let mut stream = TcpStream::connect("10.1.0.196:10001").await.unwrap();
        let mut buf = Vec::new();
        let _ = stream.read_to_end(&mut buf).await;
        println!("{:?}", buf);

        let sections = usize::from_be_bytes(buf[0..8].try_into().unwrap());
        println!("{}", sections);
        let mut section_idx = 8;
        let mut info_vec = vec![];
        for _ in 0..sections {
            let width = u16::from_be_bytes(buf[section_idx..section_idx+2].try_into().unwrap());
            let height = u16::from_be_bytes(buf[section_idx+2..section_idx+4].try_into().unwrap());
            let x = u16::from_be_bytes(buf[section_idx+4..section_idx+6].try_into().unwrap());
            let y = u16::from_be_bytes(buf[section_idx+6..section_idx+8].try_into().unwrap());
            let port = usize::from_be_bytes(buf[section_idx+8..section_idx+16].try_into().unwrap());

            info_vec.push((width, height, x, y, port));

            section_idx += 16;
        }
        
        println!("{:?}", addr);
        let section: usize = addr.parse().unwrap();
        let info = info_vec[section];
        println!("{:?}", info);

        let stream = TcpStream::connect("10.1.0.196:8000").await?;

        // let mut buf = [0u8; 4];
        // stream.read_exact(&mut buf).await?;
        // let mut rdr = Cursor::new(buf);
        // let mut width = rdr.read_u16::<BigEndian>().unwrap();
        // let mut height = rdr.read_u16::<BigEndian>().unwrap();

        // if let Some(w) = w {
        //     width = w;
        // }
        // if let Some(h) = h {
        //     height = h;
        // }

        // let width = align_to(width, 64);
        Ok(Self {
            width: info.0,
            height: info.1,
            x: info.2,
            y: info.3,
            stream,
            buffer: None,
        })
    }

    pub fn width(&self) -> u32 {
        self.width as u32
    }

    pub fn height(&self) -> u32 {
        self.height as u32
    }

    pub async fn write(&mut self, buf: Vec<u8>, bytes_per_pixel: usize) -> io::Result<()> {
        let mut rng = rand::thread_rng();

        debug_assert_eq!(
            buf.len(),
            bytes_per_pixel * self.width as usize * self.height as usize
        );

        let mut cursor = Cursor::new([0; 7 * ITEMS]);
        let mut i = 0;

        let mut x_arr = (0..self.width).collect::<Vec<u16>>();
        let mut y_arr = (0..self.height).collect::<Vec<u16>>();

        x_arr.shuffle(&mut rng);

        for x in x_arr {
            y_arr.shuffle(&mut rng);
            for y in &y_arr {
                let random = rng.gen::<f32>();

                if random >= 0.9 {
                    continue;
                }

                let index = (*y as usize * self.width as usize + x as usize) * bytes_per_pixel;

                let b = buf[index + 0];
                let g = buf[index + 1];
                let r = buf[index + 2];

                if let Some(old) = &self.buffer {
                    let or = old[index + 0];
                    let og = old[index + 1];
                    let ob = old[index + 2];
                    if or == r && og == g && ob == b {
                        continue;
                    }
                }

                cursor.write_u16::<BigEndian>(x + self.x).unwrap();
                cursor.write_u16::<BigEndian>(y + self.y).unwrap();

                cursor.write_u8(r).unwrap();
                cursor.write_u8(g).unwrap();
                cursor.write_u8(b).unwrap();
                i += 1;

                if i == ITEMS {
                    self.stream.write_all(cursor.get_ref()).await?;
                    cursor.set_position(0);
                    i = 0;
                }
            }
        }

        self.stream.write_all(cursor.get_ref()).await?;
        self.stream.flush().await?;
        cursor.set_position(0);

        self.buffer = Some(buf);

        Ok(())
    }
}
