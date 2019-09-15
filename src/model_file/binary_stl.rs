use std::fs::File;
use model_file::data::*;
use std::io;
use std::io::Read;
use std::io::BufReader;
use byteorder::LittleEndian;
use byteorder::ByteOrder;
use byteorder::ReadBytesExt;
use std::error;
use std::fmt;

#[derive(Debug)]
pub enum StlError {
    HeaderBytes(usize),
    TrianglesBytes(usize, usize),
    IO(io::Error)
}

impl error::Error for StlError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::IO(e) => Some(e),
            _ => None
        }
    }
}

impl From<io::Error> for StlError {
    fn from(error : io::Error) -> Self {
        StlError::IO(error)
    }
}

impl fmt::Display for StlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::HeaderBytes(n) =>
                write!(f, "Incomplete header, expected 80 bytes, got {}", n),
            Self::TrianglesBytes(e, g) =>
                write!(f, "Incomplete triangle data, expected {} bytes, got {}",
                       e, g),
            Self::IO(e) => write!(f, "{}", e)
        }
    }
}

fn read_vertex(buf : &[u8]) -> Vertex {
    let elem_size = 4;
    let x : f64 = LittleEndian::read_f32(&buf[0..elem_size]) as f64;
    let y : f64 = LittleEndian::read_f32(&buf[elem_size..elem_size * 2]) as f64;
    let z : f64 = LittleEndian::read_f32(&buf[elem_size * 2..elem_size * 3]) as f64;

    [x, y, z]
}

fn read_triangle(buf : &[u8]) -> FreeTriangle {
    let vertex_size = 12;
    let _normal = read_vertex(&buf[0..vertex_size]);
    let a = read_vertex(&buf[vertex_size..vertex_size * 2]);
    let b = read_vertex(&buf[vertex_size * 2..vertex_size * 3]);
    let c = read_vertex(&buf[vertex_size * 3..vertex_size * 4]);

    [a, b, c]
}

type StlResult<T> = Result<T, StlError>;

pub fn load(fh : &File) -> StlResult<FreeSurface> {
    let mut header_buf = [0u8; 80];
    let mut reader = BufReader::new(fh);
    let num = reader.read(&mut header_buf)?;

    if num != 80 {
        return Err(StlError::HeaderBytes(num));
    }

    let num_triangles = reader.read_u32::<LittleEndian>()? as usize;

    const TRIANGLE_SIZE : usize =
        4/*bytes per float*/
        *3/*floats per vector*/
        *4/*vectors per triangle (normal + 3 points)*/
        +2;/*attribute bytes*/

    let expected_triangle_bytes = TRIANGLE_SIZE * num_triangles;
    let mut bytes_so_far = 0;

    let mut surface = FreeSurface::new();
    let mut triangle_buf = [0u8; TRIANGLE_SIZE];

    for _i in 0..num_triangles {
        let mut this_bytes = 0;

        while this_bytes  < TRIANGLE_SIZE {
            let num = reader.read(&mut triangle_buf[this_bytes..])?;
            bytes_so_far += num;
            this_bytes += num;

            if num == 0 {
                return Err(StlError::TrianglesBytes(expected_triangle_bytes,
                                                    bytes_so_far));
            }
        }

        surface.push(read_triangle(&triangle_buf));
    }

    Ok(surface)
}
