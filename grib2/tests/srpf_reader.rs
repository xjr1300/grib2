use std::fs::{File, OpenOptions};
use std::io::{stdout, BufWriter, Write};

use grib2::reader::{ForecastHour6, Grib2ValueIter, SrpfReader};

#[test]
#[ignore]
fn test_srpf_reader() {
    // GRIB2ファイルを読み込みCSVファイルに座標を出力
    let input = "../resources/srpf.bin";
    let mut reader = SrpfReader::new(input).unwrap();
    let handle = stdout().lock();
    let mut writer = BufWriter::new(handle);
    reader.debug_info(&mut writer).unwrap();
    writer.flush().unwrap();
    let number_of_points = reader.section3().number_of_data_points();

    // 予測値をファイルに出力
    for i in 0..6 {
        let output = format!("../resources/srpf_{}hours.csv", i + 1);
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(output)
            .unwrap();
        let mut writer = BufWriter::new(file);
        writeln!(writer, "longitude,latitude,value").unwrap();
        let forecast_hour = ForecastHour6::try_from(i + 1).unwrap();
        let value_iter = reader.forecast_value_iter(forecast_hour).unwrap();
        write_values(&mut writer, value_iter, number_of_points);
    }
}

fn write_values(
    writer: &mut BufWriter<File>,
    value_iter: Grib2ValueIter<'_, u16>,
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
