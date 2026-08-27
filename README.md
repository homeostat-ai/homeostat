<img src="assets/logo.svg" alt="Homeostat" width="180" align="left">

<h3>Homeostat</h3>

<p>A learning-augmented control plane for sharded systems.</p>

[![Rust](https://img.shields.io/badge/rust-2024%20edition-orange?style=flat-square&logo=rust)](https://www.rust-lang.org)
[![Status](https://img.shields.io/badge/status-experimental-blueviolet?style=flat-square)](#project-status)
[![Stars](https://img.shields.io/github/stars/homeostat-ai/homeostat?style=flat-square&logo=github)](https://github.com/homeostat-ai/homeostat)

<br clear="left">

Homeostat observes workload and placement state, predicts the effects of shard operations, and
plans safe balancing actions across heterogeneous distributed systems.

## Features

- **Portable shard model** - Normalize workload, placement, topology, and operation state across systems
- **Pluggable adapters** - Keep read-only data sources separate from system-specific executors
- **Learning-augmented planning** - Predict action outcomes while a constrained planner makes decisions
- **Safe execution** - Enforce stale-snapshot checks, cooldowns, migration budgets, and idempotency
- **Versioned API** - Connect sources, controllers, and executors through gRPC
- **Replayable evaluation** - Compare heuristics and learned policies against reproducible workloads
- **Rust workspace** - Build the control plane as a small set of focused crates

## Architecture

```text
Source  ── ClusterSnapshot ──>  Controller  ── Plan ──>  Executor
  ^                               |                         |
  └──────── observed outcome ─────┴─────────────────────────┘
```

| Crate | Responsibility |
| --- | --- |
| `homeostat-api` | Protobuf definitions and generated gRPC interfaces |
| `homeostat-source` | Read-only state collection and normalization |
| `homeostat-controller` | Policies, models, planning, and the control loop |
| `homeostat-executor` | Validation, idempotent execution, and operation tracking |

Learning augments the control loop; it does not define correctness. Adapters and executors enforce
hard system constraints even when a model is wrong, and `NoOp` remains a valid decision.

## Initial scope

Homeostat starts with shard load balancing for Apache Pulsar, followed by Lyra. The first milestones
are a deterministic simulator, heuristic baselines, reproducible action trajectories, and an outcome
model built with Burn. The APIs remain independent of either system.

## Development

The workspace uses stable Rust with the 2024 edition. Generated API code requires `protoc` on
`PATH`.

```bash
cargo fmt --all --check
cargo test --workspace
```

## Project status

Homeostat is at the design and prototyping stage. Its APIs and architecture will change. It is not
yet intended for production use.
