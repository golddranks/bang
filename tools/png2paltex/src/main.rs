use std::{
    collections::HashMap,
    env::args,
    ffi::OsStr,
    fs::{read_dir, write},
    io::{Read, Write, stdin, stdout},
    ops::Range,
    path::Path,
};

use png::ColorType;

use paltex::{Color, PalTex};

fn from_png(input: impl Read) -> (PalTex, Range<usize>) {
    let decoder = png::Decoder::new(input);
    let mut reader = decoder.read_info().expect("Failed to read PNG info");
    let mut buf = vec![0; reader.output_buffer_size()];
    let info = reader
        .next_frame(&mut buf)
        .expect("Failed to read PNG image data");
    let width = info.width as usize;
    reader.finish().expect("Failed to finish reading PNG");
    let bytes = &buf[..info.buffer_size()];
    assert!(info.color_type == ColorType::Rgba);

    let mut color_map = HashMap::new();
    let mut palette = Vec::new();
    color_map.insert(Color::TRANSPARENT, 0);
    let transp_idx = 0;

    let mut data = Vec::new();
    let mut left = width;
    let mut right = 0;

    for row in bytes.chunks_exact(4 * width) {
        let mut leftmost_opaque = width;
        let mut rightmost_opaque = 0;
        for (i, pixel) in row.chunks_exact(4).enumerate() {
            let color = Color::from_rgba_u8(pixel[0..4].try_into().unwrap());
            let color_idx = *color_map.entry(color).or_insert_with(|| {
                palette.push(color);
                palette.len() // Implicit transparent as 0th index, so len() == latest index
            }) as u8;
            if color_idx != transp_idx {
                rightmost_opaque = i;
            }
            if color_idx != transp_idx && i < leftmost_opaque {
                leftmost_opaque = i;
            }
            data.push(color_idx);
        }
        left = left.min(leftmost_opaque);
        right = right.max(rightmost_opaque + 1);
    }

    assert!(palette.len() < 15);
    let height = (data.len() / width) as u32;
    let width = width as u32;

    (
        PalTex {
            palette,
            width,
            height,
            data,
        },
        left..right,
    )
}

fn crop(paltex: &PalTex, range: Range<usize>) -> PalTex {
    let mut data: Vec<u8> = Vec::with_capacity(paltex.data.len());

    for row in paltex.data.chunks_exact(paltex.width as usize) {
        if row.iter().all(|&color_idx| color_idx == 0) {
            continue;
        }
        data.extend(&row[range.clone()]);
    }

    let crop_img_width = (range.end - range.start) as u32;
    let crop_img_height = data.len() as u32 / crop_img_width;

    PalTex {
        width: crop_img_width,
        height: crop_img_height,
        palette: paltex.palette.clone(),
        data,
    }
}

fn convert(input: impl Read) -> Vec<u8> {
    let (paltex, range) = from_png(input);

    eprintln!(
        "Palette length: {} RGBA colors (+ 1 implicit transparent)",
        paltex.palette.len()
    );
    eprintln!("Original image width: {} pixels", paltex.width);
    eprintln!("Original image height: {} pixels", paltex.height);

    let cropped = crop(&paltex, range);

    eprintln!("Cropped image length: {} pixels", cropped.width);
    eprintln!("Cropped image width: {} pixels", cropped.height);

    let mut encoded_output = Vec::new();
    paltex::encode(&cropped, &mut encoded_output);
    encoded_output
}

fn convert_file(path: &Path) {
    let paltex_path = path.with_extension("paltex");
    let fname = paltex_path.file_name().unwrap();
    let input = std::fs::File::open(path).unwrap();
    let encoded_output = convert(input);
    eprintln!("Writing {fname:?}.");
    write(fname, encoded_output).unwrap();
}

fn main() {
    if let Some(path) = args().nth(1) {
        let path = Path::new(&path);
        let target_ext = OsStr::new("png");
        if path.is_dir() {
            eprintln!("Looking for png files in {path:?}.");
            for path in read_dir(path).unwrap_or_else(|e| {
                panic!("Failed to read directory: {path:?}: {e}");
            }) {
                let path = path.unwrap().path();
                if let Some(ext) = path.extension()
                    && ext == target_ext
                {
                    convert_file(&path);
                }
            }
        } else if path.is_file()
            && let Some(ext) = path.extension()
            && ext == target_ext
        {
            convert_file(path);
        }
    } else {
        eprintln!("Converting input from stdin to stdout.");
        let input = stdin().lock();
        let encoded_output = convert(input);

        stdout()
            .write_all(&encoded_output)
            .expect("Failed to write to stdout");
    }
}
