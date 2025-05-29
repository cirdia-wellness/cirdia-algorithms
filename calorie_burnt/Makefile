generate_report:
	cargo run --bin report  --features="binary" -- assets/exercise.csv ../tmp/report_calorie_burnt.csv assets/calorie_burnt.json

train_model:
	cargo run --bin training --features="binary" -- assets/training_data.csv assets/calorie_burnt.json