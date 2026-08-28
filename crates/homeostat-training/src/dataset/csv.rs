use std::path::Path;

use crate::error::DatasetError;

use super::{OutcomeDataset, OutcomeSample};

const TIMESTAMP_COLUMN: &str = "observed_at_unix_ms";
const SCENARIO_COLUMN: &str = "scenario_id";
const TARGET_COLUMN: &str = "target";

impl OutcomeDataset {
    pub fn from_csv(path: impl AsRef<Path>) -> Result<Self, DatasetError> {
        let mut reader = csv::Reader::from_path(path)?;
        let headers = reader.headers()?.clone();

        let timestamp_index = required_column(&headers, TIMESTAMP_COLUMN)?;
        let scenario_index = required_column(&headers, SCENARIO_COLUMN)?;
        let target_index = required_column(&headers, TARGET_COLUMN)?;
        let feature_indices = headers
            .iter()
            .enumerate()
            .filter(|(index, _)| {
                *index != timestamp_index && *index != scenario_index && *index != target_index
            })
            .collect::<Vec<_>>();

        if feature_indices.is_empty() {
            return Err(DatasetError::NoFeatures);
        }

        let feature_names = feature_indices
            .iter()
            .map(|(_, name)| (*name).to_owned())
            .collect::<Vec<_>>();
        let mut samples = Vec::new();

        for (record_index, record) in reader.records().enumerate() {
            let row = record_index + 2;
            let record = record?;
            let observed_at_unix_ms =
                parse_field::<i64>(&record, timestamp_index, row, TIMESTAMP_COLUMN)?;
            let scenario_id = record
                .get(scenario_index)
                .unwrap_or_default()
                .trim()
                .to_owned();
            if scenario_id.is_empty() {
                return Err(DatasetError::EmptyScenario { row });
            }

            let target = parse_finite_f32(&record, target_index, row, TARGET_COLUMN)?;
            let features = feature_indices
                .iter()
                .map(|(index, name)| parse_finite_f32(&record, *index, row, name))
                .collect::<Result<Vec<_>, _>>()?;

            samples.push(OutcomeSample {
                observed_at_unix_ms,
                scenario_id,
                features,
                target,
            });
        }

        if samples.len() < 3 {
            return Err(DatasetError::NotEnoughSamples(samples.len()));
        }

        samples.sort_by_key(|sample| sample.observed_at_unix_ms);
        Ok(Self {
            feature_names,
            samples,
        })
    }
}

fn required_column(headers: &csv::StringRecord, name: &'static str) -> Result<usize, DatasetError> {
    headers
        .iter()
        .position(|header| header == name)
        .ok_or(DatasetError::MissingColumn(name))
}

fn parse_field<T>(
    record: &csv::StringRecord,
    index: usize,
    row: usize,
    column: &str,
) -> Result<T, DatasetError>
where
    T: std::str::FromStr,
{
    let value = record.get(index).unwrap_or_default();
    value.parse().map_err(|_| DatasetError::InvalidField {
        row,
        column: column.to_owned(),
        value: value.to_owned(),
    })
}

fn parse_finite_f32(
    record: &csv::StringRecord,
    index: usize,
    row: usize,
    column: &str,
) -> Result<f32, DatasetError> {
    let value = parse_field::<f32>(record, index, row, column)?;
    if !value.is_finite() {
        return Err(DatasetError::NonFiniteField {
            row,
            column: column.to_owned(),
        });
    }
    Ok(value)
}
