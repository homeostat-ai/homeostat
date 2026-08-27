<p align="center">
  <img src="assets/homeostat-logo.png" alt="Homeostat beaver mascot balancing distributed shards" width="240">
</p>

<h1 align="center">Homeostat</h1>

<p align="center">
  <strong>A learning-augmented control plane for sharded systems.</strong>
</p>

Homeostat observes workload and placement state, predicts the effects of shard operations, and
plans safe balancing actions across heterogeneous distributed systems.

The project starts with shard load balancing. Its first integrations are planned for Apache Pulsar
and Lyra, while its APIs remain independent of either system.

> [!IMPORTANT]
> Homeostat is at the design and prototyping stage. Its APIs and architecture will change.

## Why Homeostat?

Load balancing in a stateful sharded system is not request routing. Moving a shard can transfer
ownership, copy state, warm caches, reconnect clients, or wait for replicas to catch up. The cost
and settling time vary by system, workload, and cluster.

Homeostat separates this control loop into three roles:

```text
Source  ── ClusterSnapshot ──>  Controller  ── Plan ──>  Executor
  ^                               |                         |
  └──────── observed outcome ─────┴─────────────────────────┘
```

- **Source** observes a system through read-only interfaces and produces a normalized snapshot.
- **Controller** evaluates candidate actions with heuristics or learned models and builds a plan.
- **Executor** validates preconditions and performs system-specific operations with write access.

All components communicate through a versioned gRPC API.

## Repository layout

| Crate | Responsibility |
| --- | --- |
| `homeostat-api` | Protobuf definitions and generated gRPC interfaces |
| `homeostat-source` | Read-only state collection and normalization |
| `homeostat-controller` | Policies, models, planning, and the control loop |
| `homeostat-executor` | Validation, idempotent execution, and operation tracking |

Planning, model inference, health assessment, simulation, and replay remain modules inside the
controller until an independent deployment or release boundary requires otherwise.

## Initial scope

Homeostat initially focuses on:

- shard workload and placement observation;
- candidate shard movement generation;
- action outcome and settling-time prediction;
- constrained plan selection;
- cooldowns and migration budgets;
- idempotent execution with stale-snapshot protection;
- deterministic baselines before learned policies;
- Pulsar as the first adapter and Lyra as the second.

Homeostat is not intended to be a general-purpose Kubernetes or SRE agent.

## Design principles

1. **Learning augments control; it does not define correctness.** System adapters and the executor
   enforce hard constraints even when a model is wrong.
2. **No operation is a valid decision.** The controller can choose `NoOp` when expected benefit or
   confidence is insufficient.
3. **Observations are versioned.** Every plan references the cluster revision from which it was
   derived, allowing the executor to reject stale decisions.
4. **Effects are learned before policies.** Early models predict the outcome of `(state, action)`;
   a constrained planner remains responsible for choosing an action.
5. **Portability is tested, not assumed.** Shared abstractions must be validated against at least
   two systems with different shard movement semantics.

## Roadmap

- [ ] Define the canonical state, action, plan, and operation lifecycle APIs.
- [ ] Build a deterministic simulator and replay format.
- [ ] Implement a Pulsar source and executor.
- [ ] Establish heuristic load-balancing baselines.
- [ ] Collect reproducible `(state, action, outcome)` trajectories.
- [ ] Train an outcome model with Burn.
- [ ] Add constrained planning, shadow mode, and canary execution.
- [ ] Implement the Lyra adapter and evaluate cross-system transfer.
- [ ] Explore health assessment and guarded model escalation.

## Development

The workspace uses stable Rust with the 2024 edition.

```bash
cargo fmt --all --check
cargo test --workspace
```

The generated API currently requires `protoc` to be installed and available on `PATH`.

## Project status

Homeostat is an early-stage research and engineering project. Design discussions, workload traces,
and reproducible evaluation will be treated as first-class project artifacts as the implementation
develops.
