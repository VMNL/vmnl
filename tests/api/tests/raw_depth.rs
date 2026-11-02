// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Headless raw depth configuration through the public facade.

use vmnl::raw::{CompareOp, DepthState, Pipeline};

#[derive(Clone)]
struct Vertex;

#[test]
fn raw_depth_defaults_are_disabled_and_configuration_is_replaceable() {
    assert_eq!(DepthState::default(), DepthState::Disabled);
    assert_eq!(
        Pipeline::<Vertex>::builder().depth_value(),
        DepthState::Disabled
    );
    for compare_op in [
        CompareOp::Never,
        CompareOp::Less,
        CompareOp::Equal,
        CompareOp::LessOrEqual,
        CompareOp::Greater,
        CompareOp::NotEqual,
        CompareOp::GreaterOrEqual,
        CompareOp::Always,
    ] {
        for write_enable in [false, true] {
            let state = DepthState::Test {
                compare_op,
                write_enable,
            };
            let spec = Pipeline::<Vertex>::builder().depth(state);
            assert_eq!(spec.depth_value(), state);
            assert_eq!(spec.clone().depth_value(), state);
            assert_eq!(
                spec.depth(DepthState::Disabled).depth_value(),
                DepthState::Disabled
            );
        }
    }
}
