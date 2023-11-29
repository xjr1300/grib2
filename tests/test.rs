use grib2::reader::Grib2Reader;

#[test]
#[ignore]
fn test() {
    let input = "resources/sample.bin";
    //let output = "fixtures/sample.csv";
    //let expected = "fixtures/sample.org.csv";
    let mut reader = Grib2Reader::new(input).unwrap();
}
