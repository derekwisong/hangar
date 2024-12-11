use hangar::resource_path;
use hangar::garmin;

// A Garmin EIS file for a Mooney M20J
const SAMPLE_CSV: &str = "log_231104_084813_KPOU.csv";

#[test]
fn test_read_csv_metadata() -> Result<(), String> {
    let path = resource_path(SAMPLE_CSV);
    let header = garmin::GarminEISLogHeader::from_csv(&path).map_err(|e| e.to_string())?;

    assert_eq!(header.metadata["log_version"], "1.03");
    assert_eq!(header.metadata["airframe_name"], "Mooney M20J");
    assert_eq!(header.metadata["tail_number"], "N12345");
    Ok(())
}

// test reading raw column names from a Garmin EIS csv file
#[test]
fn read_csv_raw_column_names() -> Result<(), String> {
    let path = resource_path(SAMPLE_CSV);
    let header = garmin::GarminEISLogHeader::from_csv(&path).map_err(|e| e.to_string())?;
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
    Ok(())
}

// test reading the column names from a Garmin EIS csv file
#[test]
fn read_csv_column_names() -> Result<(), String> {
    let path = resource_path("log_231104_084813_KPOU.csv");
    let header = garmin::GarminEISLogHeader::from_csv(&path).map_err(|e| e.to_string())?;
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
    Ok(())
}

#[test]
fn read_eis_data() -> Result<(), String> {
    let path = resource_path(SAMPLE_CSV);
    let eis = garmin::GarminEISLog::from_csv(&path).map_err(|e| e.to_string())?;
    // check that the headers are loaded
    assert_eq!(eis.header.metadata["log_version"], "1.03");
    assert_eq!(eis.header.metadata["airframe_name"], "Mooney M20J");
    // check that the data are loaded
    assert_eq!(eis.data.shape(), (3676, 72));
    Ok(())
}