#[derive(Clone, Debug, PartialEq)]
pub struct OutcomeSample {
    pub observed_at_unix_ms: i64,
    pub scenario_id: String,
    pub features: Vec<f32>,
    pub target: f32,
}

#[derive(Clone, Debug)]
pub struct OutcomeDataset {
    pub feature_names: Vec<String>,
    pub samples: Vec<OutcomeSample>,
}

#[derive(Clone, Debug)]
pub struct DatasetSplit {
    pub training: Vec<OutcomeSample>,
    pub validation: Vec<OutcomeSample>,
    pub purged_examples: usize,
}
