use std::{path::*, io::*, time::*};
use rawloader::*;

fn print_file_info(raw: &RawImage) {
    if let Some(exif) = &raw.exif {
        println!("    Exif:");
        let tags = exif.get_tags();
        for tag in tags {
            println!("        {:?} = {}", tag, exif.to_string(tag).unwrap_or("".to_string()));
        }
    }
}

fn f32_slices_eq(s1: &[f32], s2: &[f32]) -> bool {
    if s1.len() != s2.len() { return false; }
    for (&v1, &v2) in s1.iter().zip(s2) {
        if v1.is_nan() && v2.is_nan() { continue; }
        if v1.is_nan() || v2.is_nan() { return false; }
        if v1 != v2 { return false; }
    }
    return true;
}

fn test_file(file_name: &PathBuf) -> std::io::Result<()> {
    println!("File: {}", file_name.as_os_str().to_str().unwrap_or(""));
    let mut file = std::fs::File::open(file_name)?;

    file.seek(SeekFrom::Start(0))?;
    let _whole = decode(&mut file).unwrap();

    file.seek(SeekFrom::Start(0))?;
    let start = Instant::now();
    let whole = decode(&mut file).unwrap();
    let whole_time = start.elapsed().as_secs_f64();

    print_file_info(&whole);

    file.seek(SeekFrom::Start(0))?;
    let start = Instant::now();
    let exif_only = decode_exif_only(&mut file).unwrap();
    let exif_only_time = start.elapsed().as_secs_f64();

    println!(
        "whole_time = {}s, exif_only_time = {}s, diff={}",
        whole_time, exif_only_time, whole_time/exif_only_time
    );

    assert!(whole.make        == exif_only.make);
    assert!(whole.model       == exif_only.model);
    assert!(whole.clean_make  == exif_only.clean_make);
    assert!(whole.clean_model == exif_only.clean_model);
    assert!(whole.width       == exif_only.width);
    assert!(whole.height      == exif_only.height);
    assert!(whole.cpp         == exif_only.cpp);
    assert!(whole.xyz_to_cam  == exif_only.xyz_to_cam);
    assert!(whole.cfa         == exif_only.cfa);
    assert!(whole.crops       == exif_only.crops);
    assert!(whole.orientation == exif_only.orientation);
    assert!(f32_slices_eq(&whole.wb_coeffs, &exif_only.wb_coeffs));

    match &exif_only.data {
        RawImageData::Integer(data) =>
            assert!(data.len() <= 1),
        RawImageData::Float(data) =>
            assert!(data.len() <= 1),
    };

    Ok(())
}

fn main() -> std::io::Result<()> {
    let env: Vec<_> = std::env::args().collect();

    if env.len() != 2 {
        println!("exif <directory with RAW files>");
        return Ok(());
    }

    let files = std::fs::read_dir(&env[1])?
        .filter_map(|entry| match entry {
            Ok(entry) if entry.path().is_file() =>
                Some(entry.path()),
            _ =>
                None,
        });

    for file in files {
        test_file(&file)?;
    }

    Ok(())
}