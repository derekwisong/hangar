const SAMPLE_CSV: &str = "/mnt/d/Flying/Flight Logs/log_231104_084813_KPOU.csv";

#[test]
fn test_read_csv_metadata() {
    let path = std::path::Path::new(SAMPLE_CSV);
    let header = hangar::EISHeader::from_csv(&path).unwrap();
    assert_eq!(header.metadata["log_version"], "1.03");
    assert_eq!(header.metadata["airframe_name"], "Mooney M20J");
    assert_eq!(header.metadata["tail_number"], "N355RL");
}

#[test]
fn test_read_csv_metadata2() {
    let path = std::path::Path::new(SAMPLE_CSV);
    let header = hangar::EISHeader::from_csv2(&path).unwrap();
    assert_eq!(header.metadata["log_version"], "1.03");
    assert_eq!(header.metadata["airframe_name"], "Mooney M20J");
    assert_eq!(header.metadata["tail_number"], "N355RL");
    let col = header.columns.get(0).unwrap();
    assert_eq!(col.raw_name(), "  Lcl Date");
    assert_eq!(col.name(), "Lcl Date");
    assert_eq!(col.unit(), "yyy-mm-dd");
}

// test reading csv columns
#[test]
fn test_read_csv_columns() {
    let path = std::path::Path::new(SAMPLE_CSV);
    let columns = hangar::read_csv_columns(&path).unwrap();
    assert_eq!(
        columns,
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
