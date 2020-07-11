use plotters::prelude::*;
use serde::Deserialize;
use std::fs::{read_dir, File};
use std::io::prelude::*;
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
    Hospitalization_Rate: f64,
}

#[derive(Debug)]
enum AppError {
    ConfigurationNotFound,
    InvalidConfiguration,
    InvalidDatapath,
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
            Hospitalization_Rate: 0.0,
        }
    }
}

impl DailyReport {
    fn get_prop(&self, name: &str) -> Option<f64> {
        match name.to_ascii_lowercase().as_str() {
            "confirmed" => Some(self.Confirmed as f64),
            "deaths" => self.Deaths.map(|x| x as f64),
            "recovered" => self.Recovered.map(|x| x as f64),
            "people_tested" => self.People_Tested.map(|x| x as f64),
            "hospitalization_rate" => Some(self.Hospitalization_Rate),
            _ => None,
        }
    }
}

#[derive(Deserialize, Clone)]
struct Config {
    state: String,
    statistic: String,
    datapath: String,
}

impl std::cmp::PartialOrd for DailyReport {
    fn partial_cmp(&self, other: &DailyReport) -> Option<std::cmp::Ordering> {
        Some(self.Confirmed.cmp(&other.Confirmed))
    }
}

fn load_configuration() -> AppResult<Config> {
    let mut config_file = File::open("config.toml").map_err(|_| AppError::ConfigurationNotFound)?;
    let mut raw_config = String::new();
    config_file.read_to_string(&mut raw_config).unwrap();
    toml::from_str(&raw_config).map_err(|_| AppError::InvalidConfiguration)
}

fn generate_plot(config: &Config, data: Vec<DailyReport>) {
    let root = BitMapBackend::new("output.png", (640, 480)).into_drawing_area();
    root.fill(&WHITE).unwrap();
    let root = root.margin(10, 10, 10, 10);

    let max = data
        .iter()
        .map(|rep| rep.get_prop(&config.statistic).unwrap_or(0.0) as usize)
        .max().unwrap_or(0) + 1;
    let mut chart = ChartBuilder::on(&root)
        // Set the caption of the chart
        .caption(
            format!("{}: {} vs DAYS ELAPSED", config.state, config.statistic.to_uppercase()),
            ("sans-serif", 40).into_font(),
        )
        // Set the size of the label region
        .x_label_area_size(20)
        .y_label_area_size(40)
        // Finally attach a coordinate on the drawing area and make a chart context
        .build_ranged(0..data.len(), 0.0..max as f64)
        .unwrap();

    chart
        .configure_mesh()
        // We can customize the maximum number of labels allowed for each axis
        .x_labels(5)
        .y_labels(5)
        .y_desc("log10[# People]")
        // We can also change the format of the label text
        .y_label_formatter(&|x| format!("{:.3}", x))
        .draw()
        .unwrap();

    chart
        .draw_series(LineSeries::new(
            (0..data.len()).zip(
                data.iter()
                    .map(|rep| rep.get_prop(&config.statistic).unwrap_or(0.0)),
            ),
            &RED,
        ))
        .unwrap();
}
// 6, 7, 8
// confirmed, deaths, recovered
fn main() -> AppResult<()> {
    let config = load_configuration()?;

    let sources = read_dir(&config.datapath).map_err(|_| AppError::InvalidDatapath)?;
    let mut reports: Vec<DailyReport> = Vec::new();
    for file in sources {
        if let Ok(d) = file {
            let data = File::open(d.path()).map_err(|_| AppError::InvalidConfiguration)?;
            let mut rdr = csv::Reader::from_reader(data);
            reports.push(
                rdr.deserialize()
                    .filter(|record: &Result<DailyReport, csv::Error>| {
                        record.is_ok() && record.as_ref().unwrap().Province_State == config.state
                    })
                    .nth(0)
                    .unwrap_or(Ok(DailyReport::default()))
                    .unwrap(),
            )
        }
    }

    reports.sort_by(|a, b| a.partial_cmp(&b).unwrap());

    generate_plot(&config, reports);
    Ok(())
}
