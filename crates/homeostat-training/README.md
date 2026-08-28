# Homeostat training

The first training pipeline fits a small MLP that estimates the outcome of a
candidate action. Each CSV row contains a timestamp, a scenario identifier, a
target, and a fixed-width numeric feature vector:

```csv
observed_at_unix_ms,scenario_id,target,cpu_mean_1h,cpu_max_24h,action_move_shard
1787846400000,move-shard-001,0.61,0.72,0.91,1
1787850000000,move-shard-002,0.55,0.68,0.88,1
```

All columns except `observed_at_unix_ms`, `scenario_id`, and `target` are model
features in header order. Action kinds and optional values should be encoded by
the dataset builder before training, for example with one-hot and presence-mask
features.

Run training with:

```console
cargo run --release -p homeostat-cli --features training -- model train \
  --dataset path/to/outcomes.csv \
  --artifact-dir homeostat-model-v1
```

The `training` CLI feature is opt-in. A normal release build of `homeostat-cli`
does not link the training crate and exposes only the controller commands.

The trainer sorts rows by time, creates a chronological 80/20 split, removes a
24-hour-and-15-minute leakage window before validation, fits normalization only
on the training rows, and saves the weights, model options, normalization,
training options, and evaluation manifest in the artifact directory.
