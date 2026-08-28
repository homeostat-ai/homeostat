use std::fmt::Debug;

use homeostat_api::v1::{
    Action, DataQuality, HorizontalScale, MergeShards, MoveShard, NoOp, NodeGroupObservation,
    NodeObservation, Observation, Placement, PlacementRole, ResourceCapacity, ShardCapabilities,
    ShardObservation, ShardState, SplitShard, TransferLeadership, VerticalScale, Workload, action,
};
use prost::Message;

fn assert_round_trip<T>(value: &T)
where
    T: Message + Default + PartialEq + Debug,
{
    let bytes = value.encode_to_vec();
    let decoded = T::decode(bytes.as_slice()).expect("contract should decode");

    assert_eq!(&decoded, value);
}

#[test]
fn observation_represents_scaling_and_sharding_inputs() {
    let observation = Observation {
        observation_id: "observation-1".to_owned(),
        system_id: "system-1".to_owned(),
        revision: 7,
        observed_at_unix_ms: 1_700_000_000_000,
        node_groups: vec![NodeGroupObservation {
            node_group_id: "workers".to_owned(),
            desired_nodes: Some(3),
            ready_nodes: Some(3),
            min_nodes: Some(1),
            max_nodes: Some(10),
            node_capacity: Some(ResourceCapacity {
                cpu_cores: Some(4.0),
                memory_bytes: Some(8 * 1024 * 1024 * 1024),
                ..Default::default()
            }),
            ..Default::default()
        }],
        nodes: vec![
            NodeObservation {
                node_id: "node-1".to_owned(),
                shard_count: Some(1),
                ..Default::default()
            },
            NodeObservation {
                node_id: "node-2".to_owned(),
                shard_count: Some(1),
                ..Default::default()
            },
            NodeObservation {
                node_id: "node-3".to_owned(),
                shard_count: Some(1),
                ..Default::default()
            },
        ],
        shards: vec![ShardObservation {
            shard_id: "shard-1".to_owned(),
            workload: Some(Workload {
                read_ops_per_sec: Some(100.0),
                write_ops_per_sec: Some(50.0),
                active_readers: Some(4),
                active_writers: Some(2),
                ..Default::default()
            }),
            state_bytes: Some(1024 * 1024),
            capabilities: Some(ShardCapabilities {
                movable: true,
                splittable: true,
                mergeable: true,
            }),
            state: ShardState::Active.into(),
            ..Default::default()
        }],
        placements: vec![
            Placement {
                shard_id: "shard-1".to_owned(),
                node_id: "node-1".to_owned(),
                role: PlacementRole::Leader.into(),
            },
            Placement {
                shard_id: "shard-1".to_owned(),
                node_id: "node-2".to_owned(),
                role: PlacementRole::Follower.into(),
            },
            Placement {
                shard_id: "shard-1".to_owned(),
                node_id: "node-3".to_owned(),
                role: PlacementRole::Observer.into(),
            },
        ],
        data_quality: Some(DataQuality {
            complete: true,
            ..Default::default()
        }),
        ..Default::default()
    };

    assert_round_trip(&observation);
}

#[test]
fn action_contract_represents_every_mvp_action() {
    let actions = [
        Action {
            action_id: "horizontal".to_owned(),
            expected_revision: Some(7),
            kind: Some(action::Kind::HorizontalScale(HorizontalScale {
                node_group_id: "workers".to_owned(),
                target_node_count: 4,
            })),
        },
        Action {
            action_id: "vertical".to_owned(),
            expected_revision: Some(7),
            kind: Some(action::Kind::VerticalScale(VerticalScale {
                node_id: "node-1".to_owned(),
                target_capacity: Some(ResourceCapacity {
                    cpu_cores: Some(8.0),
                    memory_bytes: Some(16 * 1024 * 1024 * 1024),
                    ..Default::default()
                }),
            })),
        },
        Action {
            action_id: "split".to_owned(),
            expected_revision: Some(7),
            kind: Some(action::Kind::SplitShard(SplitShard {
                shard_id: "shard-1".to_owned(),
                split_points: vec!["midpoint".to_owned()],
            })),
        },
        Action {
            action_id: "move".to_owned(),
            expected_revision: Some(7),
            kind: Some(action::Kind::MoveShard(MoveShard {
                shard_id: "shard-2".to_owned(),
                source_node_id: "node-1".to_owned(),
                target_node_id: "node-2".to_owned(),
            })),
        },
        Action {
            action_id: "transfer-leadership".to_owned(),
            expected_revision: Some(7),
            kind: Some(action::Kind::TransferLeadership(TransferLeadership {
                shard_id: "shard-2".to_owned(),
                target_node_id: "node-2".to_owned(),
            })),
        },
        Action {
            action_id: "merge".to_owned(),
            expected_revision: Some(7),
            kind: Some(action::Kind::MergeShards(MergeShards {
                shard_ids: vec!["shard-3".to_owned(), "shard-4".to_owned()],
            })),
        },
        Action {
            action_id: "no-op".to_owned(),
            expected_revision: Some(7),
            kind: Some(action::Kind::NoOp(NoOp {})),
        },
    ];

    for action in actions {
        assert_round_trip(&action);
    }
}
