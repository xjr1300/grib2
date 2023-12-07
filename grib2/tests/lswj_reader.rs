use std::fs::OpenOptions;
use std::io::{stdout, BufWriter, Write};

use grib2::reader::LswjReader;

#[test]
#[ignore]
fn test_lswj_reader() {
    // GRIB2ファイルを読み込みCSVファイルに座標を出力
    let input = "../resources/lswj.bin";
    let mut reader = LswjReader::new(input).unwrap();
    let handle = stdout().lock();
    let mut writer = BufWriter::new(handle);
    reader.debug_info(&mut writer).unwrap();
    writer.flush().unwrap();

    let output = "../resources/lswj.csv";
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(output)
        .unwrap();
    let mut writer = BufWriter::new(file);
    writeln!(writer, "longitude,latitude,value").unwrap();
    let mut number_of_read = 0_u32;
    for value in reader.values().unwrap() {
        let value = value.unwrap();
        if value.value.is_some() {
            writeln!(
                writer,
                "{:.6},{:.6},{}",
                value.lon,
                value.lat,
                value.value.unwrap()
            )
            .unwrap();
        }
        number_of_read += 1;
    }
    writer.flush().unwrap();
    assert_eq!(
        reader.section3().number_of_data_points(),
        number_of_read,
        "ファイルには{}個のデータが記録されているとされていますが、{}個のデータを読み込みました。\
        ファイルが誤っているか、バグがあります。",
        reader.section3().number_of_data_points(),
        number_of_read
    );
}
