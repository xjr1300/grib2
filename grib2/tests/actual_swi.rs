use std::fs::{File, OpenOptions};
use std::io::{stdout, BufWriter, Write};

use grib2::reader::{ActualSwiReader, Grib2ValueIter};

#[test]
#[ignore]
fn actual_swi() {
    // GRIB2ファイルを読み込みCSVファイルに座標を出力
    let input = "../resources/actual_swi.bin";
    let mut reader = ActualSwiReader::new(input).unwrap();
    {
        let handle = stdout().lock();
        let mut writer = BufWriter::new(handle);
        reader.debug_info(&mut writer).unwrap();
    }

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
    write_values(&mut writer, reader.swi_values().unwrap());

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
    write_values(&mut writer, reader.first_tank_values().unwrap());

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
    write_values(&mut writer, reader.second_tank_values().unwrap());
}

fn write_values(writer: &mut BufWriter<File>, value_iter: Grib2ValueIter<'_>) {
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
    }
}
