// A Garmin EIS file for a Mooney M20J
const SAMPLE_CSV: &str = "log_231104_084813_KPOU.csv";

fn resource_path(filename: &str) -> std::path::PathBuf {
    let mut d = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.push("resources");
    d.push(filename);
    d
}


#[test]
fn test_read_csv_metadata() {
    let path = resource_path(SAMPLE_CSV);
    let header = hangar::EISHeader::from_csv(&path).unwrap();

    assert_eq!(header.metadata["log_version"], "1.03");
    assert_eq!(header.metadata["airframe_name"], "Mooney M20J");
    assert_eq!(header.metadata["tail_number"], "N12345");
}

// test reading raw column names from a Garmin EIS csv file
#[test]
fn read_csv_raw_column_names() {
    let path = resource_path(SAMPLE_CSV);
    let header = hangar::EISHeader::from_csv(&path).unwrap();
    let raw_columns = header.columns.iter().map(|c| c.raw_name()).collect::<Vec<&str>>();
    assert_eq!(
        raw_columns,
        vec![
            "  Lcl Date",
            " Lcl Time",
            " UTCOfst",
            " AtvWpt",
            "     Latitude",
            "    Longitude",
            "    AltB",
            " BaroA",
            "  AltMSL",
            "   OAT",
            "    IAS",
            " GndSpd",
            "    VSpd",
            "  Pitch",
            "   Roll",
            " LatAc",
            " NormAc",
            "   HDG",
            "   TRK",
            " bus1volts",
            " alt1amps",
            "  FQtyL",
            "  FQtyR",
            " FQtyLlbs",
            " FQtyRlbs",
            " E1 FFlow",
            " E1 FPres",
            " E1 OilT",
            " E1 OilP",
            " E1 MAP",
            " E1 RPM",
            " E1 %Pwr",
            " E1 CHT1",
            " E1 CHT2",
            " E1 CHT3",
            " E1 CHT4",
            " E1 CHT CLD",
            " E1 EGT1",
            " E1 EGT2",
            " E1 EGT3",
            " E1 EGT4",
            "  AltGPS",
            " TAS",
            " HSIS",
            "    CRS",
            "   NAV1",
            "   NAV2",
            "   HCDI",
            "   VCDI",
            " WndSpd",
            " WndDr",
            " WptDst",
            " WptBrg",
            " MagVar",
            " AfcsOn",
            " RollM",
            " PitchM",
            " RollC",
            " PitchC",
            " VSpdG",
            " GPSfix",
            "  HAL",
            "   VAL",
            " HPLwas",
            " HPLfd",
            " VPLwas",
            " AltPress",
            " OnGrnd",
            "     LogIdx",
            "      SysTime",
            " LonAc",
            " LogCheck"
        ]
    );
}

// test reading the column names from a Garmin EIS csv file
#[test]
fn read_csv_column_names() {
    let path = resource_path("log_231104_084813_KPOU.csv");
    let header = hangar::EISHeader::from_csv(&path).unwrap();
    let columns = header.columns.iter().map(|c| c.name()).collect::<Vec<&str>>();
    assert_eq!(
        columns,
        vec![
            "Lcl Date",
            "Lcl Time",
            "UTCOfst",
            "AtvWpt",
            "Latitude",
            "Longitude",
            "AltB",
            "BaroA",
            "AltMSL",
            "OAT",
            "IAS",
            "GndSpd",
            "VSpd",
            "Pitch",
            "Roll",
            "LatAc",
            "NormAc",
            "HDG",
            "TRK",
            "bus1volts",
            "alt1amps",
            "FQtyL",
            "FQtyR",
            "FQtyLlbs",
            "FQtyRlbs",
            "E1 FFlow",
            "E1 FPres",
            "E1 OilT",
            "E1 OilP",
            "E1 MAP",
            "E1 RPM",
            "E1 %Pwr",
            "E1 CHT1",
            "E1 CHT2",
            "E1 CHT3",
            "E1 CHT4",
            "E1 CHT CLD",
            "E1 EGT1",
            "E1 EGT2",
            "E1 EGT3",
            "E1 EGT4",
            "AltGPS",
            "TAS",
            "HSIS",
            "CRS",
            "NAV1",
            "NAV2",
            "HCDI",
            "VCDI",
            "WndSpd",
            "WndDr",
            "WptDst",
            "WptBrg",
            "MagVar",
            "AfcsOn",
            "RollM",
            "PitchM",
            "RollC",
            "PitchC",
            "VSpdG",
            "GPSfix",
            "HAL",
            "VAL",
            "HPLwas",
            "HPLfd",
            "VPLwas",
            "AltPress",
            "OnGrnd",
            "LogIdx",
            "SysTime",
            "LonAc",
            "LogCheck"
        ]
    );
}
