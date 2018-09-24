use std::fs::File;
use model_file::data::*;
use std::io;
use std::io::Read;
use byteorder::LittleEndian;
use byteorder::ByteOrder;
use byteorder::ReadBytesExt;

#[derive(Fail, Debug)]
pub enum StlError {
    #[fail(display = "Incomplete header, expected 80 bytes, got {}", _0)]
    HeaderBytes(usize),
    #[fail(display = "Incomplete triangle data, expected {} bytes, got {}", _0, _1)]
    TrianglesBytes(usize, usize),
    #[fail(display = "{}", _0)]
    IO(#[fail(cause)] io::Error)
}

impl From<io::Error> for StlError {
    fn from(error : io::Error) -> Self {
        StlError::IO(error)
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
    let a = read_vertex(&buf[0..vertex_size]);
    let b = read_vertex(&buf[vertex_size..vertex_size * 2]);
    let c = read_vertex(&buf[vertex_size * 2..vertex_size * 3]);

    [a, b, c]
}

type StlResult<T> = Result<T, StlError>;

pub fn load(mut fh : File) -> StlResult<FreeSurface> {
    let mut header_buf = [0u8; 80];
    let num = fh.read(&mut header_buf)?;

    if num != 80 {
        return Err(StlError::HeaderBytes(num));
    }

    let num_triangles = fh.read_u32::<LittleEndian>()? as usize;

    const TRIANGLE_SIZE : usize =
        4/*bytes per float*/
        *3/*floats per vector*/
        *4;/*vectors per triangle (normal + 3 points)*/

    let expected_triangle_bytes = TRIANGLE_SIZE * num_triangles;
    let mut bytes_so_far = 0;

    let mut surface = FreeSurface::new();
    
    for _i in 0..num_triangles {
        let mut triangle_buf = [0u8; TRIANGLE_SIZE];

        let num = fh.read(&mut triangle_buf)?;
        bytes_so_far += num;
        
        if num != TRIANGLE_SIZE {
            return Err(StlError::TrianglesBytes(expected_triangle_bytes,
                                                bytes_so_far));
        }
        
        surface.push(read_triangle(&triangle_buf[4..]));
    }

    let _attrib = fh.read_u16::<LittleEndian>()?;
    
    Ok(surface)
}
