use std::{env, fs, process::ExitCode};

fn u16_at(b: &[u8], o: usize) -> u16 {
    u16::from_le_bytes([b[o], b[o + 1]])
}

fn u32_at(b: &[u8], o: usize) -> u32 {
    u32::from_le_bytes([b[o], b[o + 1], b[o + 2], b[o + 3]])
}

fn align(v: u32, a: u32) -> u32 {
    v.div_ceil(a) * a
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("usage: add_section <in.dll> <out.dll>");
        return ExitCode::FAILURE;
    }
    let mut b = match fs::read(&args[1]) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::FAILURE;
        }
    };
    let pe = u32_at(&b, 0x3c) as usize;
    let nsec = u16_at(&b, pe + 6) as usize;
    let opt_size = u16_at(&b, pe + 20) as usize;
    let opt = pe + 24;
    let sec_align = u32_at(&b, opt + 32);
    let file_align = u32_at(&b, opt + 36);
    let size_of_image = u32_at(&b, opt + 56);
    let table = opt + opt_size;
    let new_hdr = table + nsec * 40;
    let size_of_headers = u32_at(&b, opt + 60) as usize;
    if new_hdr + 40 > size_of_headers {
        eprintln!("no room for another section header");
        return ExitCode::FAILURE;
    }
    let data = b"amml extra import section\0".to_vec();
    let raw_ptr = align(b.len() as u32, file_align);
    let raw_size = align(data.len() as u32, file_align);
    let va = align(size_of_image, sec_align);
    b.resize(raw_ptr as usize, 0);
    b.extend_from_slice(&data);
    b.resize((raw_ptr + raw_size) as usize, 0);

    let mut hdr = [0u8; 40];
    hdr[..8].copy_from_slice(b".amml\0\0\0");
    hdr[8..12].copy_from_slice(&(data.len() as u32).to_le_bytes());
    hdr[12..16].copy_from_slice(&va.to_le_bytes());
    hdr[16..20].copy_from_slice(&raw_size.to_le_bytes());
    hdr[20..24].copy_from_slice(&raw_ptr.to_le_bytes());
    // initialized data, readable
    hdr[36..40].copy_from_slice(&0x4000_0040u32.to_le_bytes());
    b[new_hdr..new_hdr + 40].copy_from_slice(&hdr);

    b[pe + 6..pe + 8].copy_from_slice(&((nsec + 1) as u16).to_le_bytes());
    let new_image = align(va + data.len() as u32, sec_align);
    b[opt + 56..opt + 60].copy_from_slice(&new_image.to_le_bytes());
    match fs::write(&args[2], &b) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{e}");
            ExitCode::FAILURE
        }
    }
}
