use std::{env, fs, path::PathBuf};

fn main() {
    println!("cargo:rerun-if-changed=partitions.csv");

    let out = PathBuf::from(env::var("OUT_DIR").unwrap());

    let csv = fs::read_to_string("partitions.csv").unwrap();
    let mut ld = String::new();

    for line in csv.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let cols: Vec<_> = line.split(',').map(str::trim).collect();
        if cols.len() < 5 {
            continue;
        }

        let name = cols[0];
        let (off, sz) = (parse_size(cols[3]), parse_size(cols[4]));
        let end = off + sz;
        ld.push_str(&format!(
            "__{name}_start = {off:#010X};\n\
             __{name}_end   = {end:#010X};\n\n"
        ));
    }

    fs::write(out.join("partitions.x"), ld).unwrap();

    println!(
        "cargo:rustc-link-arg=-T{}",
        out.join("partitions.x").display()
    );
    println!("cargo:rustc-link-arg=-Tlinkall.x");
    println!("cargo:rustc-link-arg-bins=-Tdefmt.x");
}

fn parse_size(s: &str) -> u32 {
    let s = s.trim();
    if let Some(h) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        u32::from_str_radix(h, 16).unwrap()
    } else if let Some(k) = s.strip_suffix(['K', 'k']) {
        k.parse::<u32>().unwrap() * 1024
    } else if let Some(m) = s.strip_suffix(['M', 'm']) {
        m.parse::<u32>().unwrap() * 1024 * 1024
    } else {
        s.parse().unwrap()
    }
}
