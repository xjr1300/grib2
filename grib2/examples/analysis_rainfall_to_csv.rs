use std::fs::OpenOptions;
use std::io::{BufWriter, Write};

use gsjp::{Coordinate, Mesh, Mesh3};

use grib2::reader::AnalysisRainfallReader;

fn main() {
    let input = "resources/analysis_rainfall.bin";
    let mut reader = AnalysisRainfallReader::new(input).unwrap();
    let output = "resources/analysis_rainfall_with_mesh3.csv";
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(output)
        .unwrap();
    let mut writer = BufWriter::new(file);
    writeln!(writer, "code,longitude,latitude,value").unwrap();
    for value in reader.values().unwrap() {
        let value = value.unwrap();
        if value.value.is_some() {
            let coord = Coordinate::new(value.lat, value.lon).unwrap();
            let mesh3 = Mesh3::from_coordinate(coord).unwrap();
            writeln!(
                writer,
                "{},{:.6},{:.6},{}",
                mesh3.code(),
                value.lon,
                value.lat,
                value.value.unwrap()
            )
            .unwrap();
        }
    }
    writer.flush().unwrap();
}
