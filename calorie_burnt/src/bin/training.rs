use calorie_burnt::Sex;
use csv::Reader;
use linfa::{Dataset, traits::Fit};
use ndarray::Array2;
use std::error::Error;
use std::path::{Path, PathBuf};

const ARRAY_SIZE: usize = 8;
const TRAINING_SLICE: usize = ARRAY_SIZE - 1;

#[derive(Debug)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Deserialize),
    serde(rename_all = "PascalCase")
)]
struct TestDataCsv {
    #[allow(dead_code)]
    user_id: String,
    gender: Sex,
    age: u8,
    height: f64,
    weight: f64,
    body_temp: f64,
    heart_rate: u8,
    calories: f64,
    duration: u64,
}

#[derive(Debug, clap::Parser)]
pub struct Args {
    /// Input csv file location with training data
    #[arg(default_value_os_t = std::env::current_dir().unwrap_or_default().join("input.csv"), required = false)]
    pub input: PathBuf,
    /// Output model file. _Note_: will truncate old file if exists
    #[arg(default_value_os_t = std::env::current_dir().unwrap_or_default().join("calorie_burnt.json"), required = false)]
    pub output: PathBuf,
    /// Don't save changes
    #[arg(short, long, default_value_t = false, required = false)]
    pub dry: bool,
    /// Print result to stdout
    #[arg(short, long, default_value_t = false, required = false)]
    pub print: bool,
}

fn load_data(file_path: impl AsRef<Path>) -> Result<Array2<f64>, Box<dyn Error>> {
    let mut reader = Reader::from_path(file_path.as_ref())
        .map_err(|e| format!("Failed to open input file. Reason: {e}"))?;

    let data: Vec<_> = reader
        .deserialize::<TestDataCsv>()
        .filter_map(|r| r.ok())
        .flat_map(
            |TestDataCsv {
                 user_id: _,
                 gender,
                 age,
                 height,
                 weight,
                 body_temp,
                 heart_rate,
                 calories,
                 duration,
             }| {
                [
                    f64::from(gender),
                    age as f64,
                    height,
                    weight,
                    body_temp,
                    heart_rate as f64,
                    duration as f64,
                    calories,
                ]
            },
        )
        .collect();

    Ok(
        Array2::from_shape_vec((data.len() / ARRAY_SIZE, ARRAY_SIZE), data)
            .map_err(|e| format!("Failed to init dataset vector. Reason: {e}"))?,
    )
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let Args {
        input,
        output,
        dry,
        print,
    } = <Args as clap::Parser>::parse();

    let array = load_data(input)?;

    let (data, targets) = (
        array.slice(ndarray::s![.., 0..TRAINING_SLICE]).to_owned(),
        array.column(TRAINING_SLICE).to_owned(),
    );

    println!("Number of records for training: {}", data.len());

    let feature_names = vec![
        "sex",
        "age",
        "height",
        "weight",
        "body_temp",
        "heart_rate",
        "duration",
    ];

    let train = Dataset::new(data, targets)
        .with_feature_names(feature_names)
        .map_targets(|this| this.floor() as usize);

    let model = linfa_trees::DecisionTree::params()
        .fit(&train)
        .map_err(|e| format!("Failed to fit dataset to model. Reason: {e}"))?;

    if print {
        println!(
            "{}",
            serde_json::to_string_pretty(&model).expect("serde serialization can't fail")
        );
    }

    if !dry {
        println!("Save to {}", output.to_string_lossy());

        std::fs::write(
            output,
            serde_json::to_string_pretty(&model).expect("serde serialization can't fail"),
        )
        .map_err(|e| format!("Failed to save mode. Reason: {e}"))?;
    }

    println!("Done!");

    Ok(())
}
