use std::fs::{File, OpenOptions};
use std::io::{stdout, BufWriter, Write};

use grib2::reader::{AswiReader, Grib2ValueIter};

#[test]
#[ignore]
fn test_aswi_reader() {
    // GRIB2ファイルを読み込みCSVファイルに座標を出力
    let input = "../resources/actual_swi.bin";
    let mut reader = AswiReader::new(input).unwrap();
    let handle = stdout().lock();
    let mut writer = BufWriter::new(handle);
    reader.debug_info(&mut writer).unwrap();
    writer.flush().unwrap();
    let number_of_points = reader.section3().number_of_data_points();

    // 土壌雨量指数を出力
    let output = "../resources/actual_swi_0.csv";
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(output)
        .unwrap();
    let mut writer = BufWriter::new(file);
    writeln!(writer, "longitude,latitude,value").unwrap();
    write_values(&mut writer, reader.swi_values().unwrap(), number_of_points);

    // 第一タンクの値を出力
    let output = "../resources/actual_swi_1.csv";
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(output)
        .unwrap();
    let mut writer = BufWriter::new(file);
    writeln!(writer, "longitude,latitude,value").unwrap();
    write_values(
        &mut writer,
        reader.first_tank_values().unwrap(),
        number_of_points,
    );

    // 第二タンクの値を出力
    let output = "../resources/actual_swi_2.csv";
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(output)
        .unwrap();
    let mut writer = BufWriter::new(file);
    writeln!(writer, "longitude,latitude,value").unwrap();
    write_values(
        &mut writer,
        reader.second_tank_values().unwrap(),
        number_of_points,
    );
}

fn write_values(
    writer: &mut BufWriter<File>,
    value_iter: Grib2ValueIter<'_>,
    number_of_points: u32,
) {
    let mut number_of_reads = 0_u32;
    for value in value_iter {
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
        number_of_reads += 1;
    }
    assert_eq!(
        number_of_points, number_of_reads,
        "ファイルには{}個のデータが記録されているとされていますが、{}個のデータを読み込みました。\
        ファイルが誤っているか、バグがあります。",
        number_of_points, number_of_reads
    );
}
