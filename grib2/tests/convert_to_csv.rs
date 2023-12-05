use std::fs::{File, OpenOptions};
use std::io::{stdout, BufRead, BufReader, BufWriter, Write};

use grib2::reader::AnalysisRainfallReader;

#[test]
#[ignore]
fn convert_to_csv() {
    // GRIB2ファイルを読み込みCSVファイルに座標を出力
    let input = "../resources/analysis_rainfall.bin";
    let mut reader = AnalysisRainfallReader::new(input).unwrap();
    {
        let handle = stdout().lock();
        let mut writer = BufWriter::new(handle);
        reader.debug_info(&mut writer).unwrap();
    }

    let output = "../resources/analysis_rainfall.csv";
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(output)
        .unwrap();
    let mut writer = BufWriter::new(file);
    writeln!(writer, "longitude,latitude,value").unwrap();
    let mut number_of_points = 0_u64;
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
        number_of_points += 1;
    }
    writer.flush().unwrap();
    println!("number of reading points: {}", number_of_points);

    // 検証
    let expected = "../resources/analysis_rainfall.org.csv";
    let mut o_reader = BufReader::new(File::open(output).unwrap());
    let e_reader = BufReader::new(File::open(expected).unwrap());
    for e_line in e_reader.lines() {
        let mut o_line = String::new();
        let o_bytes = o_reader.read_line(&mut o_line);
        match o_bytes {
            Ok(0) => {
                // 出力したファイルの行数が、予期したファイルの行数より少ない
                assert!(false, "the output file is shorter than the expected file");
            }
            Ok(_) => {
                // 出力したファイルの行が、予期したファイルの行と一致するか確認
                assert_eq!(e_line.unwrap(), o_line.trim_end());
            }
            Err(_) => {
                assert!(false, "unexpected error raised");
            }
        }
    }
    let mut o_line = String::new();
    let o_bytes = o_reader.read_line(&mut o_line);
    match o_bytes {
        Ok(0) => {
            // 出力したファイルの行数と、予期したファイルの行数が一致
        }
        Ok(_) => {
            // 出力したファイルの行数が、予期したファイルの行数より多い
            assert!(false, "the output file is longer than the expected file");
        }
        Err(_) => {
            assert!(false, "unexpected error raised");
        }
    }

    // 出力したファイルと、予期したファイルの内容が完全に一致
    println!("the output file is completely same as the expected file");
}
