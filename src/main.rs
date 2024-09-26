use std::fs::File;
use std::io::Write;

mod qoi;

fn main() -> Result<(), std::io::Error> {
    let fname = std::env::args().nth(1).expect("Filename is required");
    let file = File::open(fname)?;
    let decoder = png::Decoder::new(file);
    let mut reader = decoder.read_info()?;

    let mut buf = vec![0; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf).unwrap();

    println! {"{:?}", info};

    let mut data = Vec::from(&buf[..info.buffer_size()]);

    let mut result = qoi::encode(
        info.width,
        info.height,
        info.color_type.samples() as u8,
        &mut data,
    );

    let mut file = File::create("test.qoi")?;
    file.write_all(result.as_mut_slice())?;

    Ok(())
}
