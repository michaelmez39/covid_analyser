use plotters::prelude::*;
use std::io::prelude::*;
use serde::Deserialize;
use std::fs::{read_dir, File};
// Province_State,
// Country_Region,Last_Update,Lat,Long_,Confirmed,Deaths,
// Recovered,Active,FIPS,Incident_Rate,People_Tested,People_Hospitalized,
// Mortality_Rate,UID,ISO3,Testing_Rate,Hospitalization_Rate
#[derive(Debug, Deserialize, PartialEq)]
#[allow(non_snake_case)]
struct DailyReport {
    Province_State: String,
    Confirmed: u64,
    Deaths: Option<u64>,
    Recovered: Option<u64>,
    People_Tested: Option<u64>,
    Hospitalization_Rate: f64
}

#[derive(Debug)]
enum AppError {
    ConfigurationNotFound,
    InvalidConfiguration,
    InvalidData
}

type AppResult<T> = Result<T, AppError>;

impl Default for DailyReport {
    fn default() -> Self {
        DailyReport {
            Province_State: "".to_owned(),
            Confirmed: 0,
            Deaths: Some(0),
            Recovered: Some(0),
            People_Tested: Some(0),
            Hospitalization_Rate: 0.0
        }
    }
}

#[derive(Deserialize)]
struct Config {
    state: String,
    statistic: String,
    datapath: String
}

impl std::cmp::PartialOrd for DailyReport {
    fn partial_cmp(&self, other: &DailyReport) -> Option<std::cmp::Ordering> {
        Some(self.Confirmed.cmp(&other.Confirmed))
    }
}

fn load_configuration() -> AppResult<Config> {
    let mut config_file = File::open("config.toml").map_err(|x| AppError::ConfigurationNotFound)?;
    let mut raw_config = String::new();
    config_file.read_to_string(&mut raw_config).unwrap();
    toml::from_str(&raw_config).map_err(|x| AppError::InvalidConfiguration)
}
// 6, 7, 8
// confirmed, deaths, recovered
fn main() -> AppResult<()> {
    let config = load_configuration();
    
    let data_path = config.datapath;
    let mut confirmed: Vec<DailyReport> = Vec::new();
    let reports = read_dir(data_path).expect("COVID_PATH invalid");
    let mut num_reports = 0;
    for file in reports {
        if let Ok(d) = file {
            num_reports += 1;
            let data = File::open(d.path()).map_err(|x| AppError::InvalidConfiguration)?;
            let mut rdr = csv::Reader::from_reader(data);
            confirmed.push(
                rdr.deserialize()
                    .filter(|record: &Result<DailyReport, csv::Error>| {
                        record.is_ok() && record.as_ref().unwrap().Province_State == config.state
                    })
                    .nth(0).unwrap_or(Ok(DailyReport::default())).unwrap()
            )
        }
    }
    confirmed.sort_by(|a,b| a.Confirmed.cmp(&b.Confirmed));
    let confirmed: Vec<(i32, DailyReport)> = confirmed.into_iter().enumerate().map(|(a, b)| (a as i32, b)).collect();
    println!("{:?}", confirmed);
    let root = BitMapBackend::new("output.png", (640, 480)).into_drawing_area();
    root.fill(&WHITE).unwrap();
    let root = root.margin(10, 10, 10, 10);
    let mut chart = ChartBuilder::on(&root)
        // Set the caption of the chart
        .caption("Florida Hospitiliaztion Rate", ("sans-serif", 40).into_font())
        // Set the size of the label region
        .x_label_area_size(20)
        .y_label_area_size(40)
        // Finally attach a coordinate on the drawing area and make a chart context
        .build_ranged(0..num_reports, 0.0..20.0)
        .unwrap();

    chart
        .configure_mesh()
        // We can customize the maximum number of labels allowed for each axis
        .x_labels(5)
        .y_labels(5)
        .y_desc("log10[# People]")
        // We can also change the format of the label text
        .y_label_formatter(&|x| format!("{:.3}", x))
        .draw().unwrap();

    chart.draw_series(LineSeries::new(
        confirmed.iter().map(|x| (x.0, (x.1).Hospitalization_Rate)),
        &RED,
    )).unwrap();
    Ok(())
}
