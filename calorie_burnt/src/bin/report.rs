use std::{fs::File, io::Write, path::PathBuf};

use calorie_burnt::Sex;
use linfa::traits::Predict;

#[derive(Debug)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Deserialize),
    serde(rename_all = "PascalCase")
)]
struct TestDataCsv {
    user_id: String,
    gender: Sex,
    age: u8,
    height: f64,
    weight: f64,
    body_temp: f64,
    heart_rate: u8,
    duration: u64,
    calories: f64,
}

#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Report {
    user: String,
    expected: f64,
    actual: f64,
    precision: f64,
}

#[derive(Debug, clap::Parser)]
pub struct Args {
    /// Input csv file location
    #[arg(default_value_os_t = std::env::current_dir().unwrap_or_default().join("input.csv"), required = false)]
    pub input: PathBuf,
    /// Output csv file. _Note_: will truncate old file if exists
    #[arg(default_value_os_t = std::env::current_dir().unwrap_or_default().join("output.csv"), required = false)]
    pub output: PathBuf,
    /// Model json file
    #[arg(default_value_os_t = std::env::current_dir().unwrap_or_default().join("calorie_burnt.json"), required = false)]
    pub model: PathBuf,
    /// Don't save changes
    #[arg(short, long, default_value_t = false, required = false)]
    pub dry: bool,
    /// Print result to stdout
    #[arg(short, long, default_value_t = false, required = false)]
    pub print: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let Args {
        input,
        output,
        model,
        dry,
        print,
    } = <Args as clap::Parser>::parse();

    let mut rdr = csv::Reader::from_reader(
        File::open(input).map_err(|e| format!("Failed to read input file. Reason: {e}"))?,
    );

    let mut wrt = csv::Writer::from_path(&output);

    let data = rdr
        .deserialize()
        .filter_map(|this| this.ok())
        .collect::<Vec<_>>();

    let total_data = data.len();

    println!("Total: {total_data} records",);

    let model = serde_json::from_reader::<_, linfa_trees::DecisionTree<f64, usize>>(
        std::fs::File::open(model).map_err(|e| format!("Failed to open model. Reason {e}"))?,
    )
    .map_err(|e| format!("Failed to init model. Reason {e}"))?;

    let mut io = match print {
        true => {
            let io = std::io::stdout();

            let mut io = io.lock();

            io.write_all("user, expected,actual,precision".as_bytes())?;

            Some(io)
        }
        false => None,
    };

    // 0 - Threshold for precision
    // 1 - Count for this 10% precision category
    // 2 -  Count for all precision >= threshold
    let mut threshold = [
        (0.9, 0, 0),
        (0.8, 0, 0),
        (0.7, 0, 0),
        (0.6, 0, 0),
        (0.5, 0, 0),
        (0.4, 0, 0),
        (0.3, 0, 0),
        (0.2, 0, 0),
        (0.1, 0, 0),
    ];

    for TestDataCsv {
        user_id,
        gender,
        age,
        height,
        weight,
        body_temp,
        heart_rate,
        calories: expected_calories,
        duration,
    } in data
    {
        let data = ndarray::Array2::from_shape_vec(
            (1, 7),
            vec![
                f64::from(gender),
                age as f64,
                height,
                weight,
                body_temp,
                heart_rate as f64,
                duration as f64,
            ],
        )
        .expect("can't fail");

        let prediction = model.predict(&data);
        let actual_calories = prediction
            .first()
            .map(|this| *this as f64)
            .expect("always have element");

        let precision = match expected_calories < actual_calories {
            true => expected_calories / actual_calories,
            false => actual_calories / expected_calories,
        };

        let precision = match precision.is_sign_negative() {
            true => precision * -1.0,
            false => precision,
        };

        if let Some((_, count, _)) = threshold.iter_mut().find(|(th, _, _)| precision > *th) {
            *count += 1;
        }

        for (_, _, count) in threshold.iter_mut().filter(|(th, _, _)| precision > *th) {
            *count += 1;
        }

        if let Some(io) = &mut io {
            io.write_fmt(format_args!(
                "{user_id},{expected_calories},{actual_calories},{precision}"
            ))?;
        }

        if dry {
            continue;
        }

        if let Ok(wrt) = &mut wrt {
            wrt.serialize(Report {
                user: user_id,
                expected: expected_calories,
                actual: actual_calories,
                precision,
            })?;
        }
    }

    for (f, count, count_total) in threshold {
        println!(
            "> {f}: {count:5} records | total: {count_total:5} - {}%",
            ((count_total as f64 / total_data as f64) * 100.0).floor()
        )
    }

    if !dry {
        println!("Saving to {}", output.to_string_lossy());
        wrt?.flush()?;
    }

    println!("Done!");

    Ok(())
}
