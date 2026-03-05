use super::*;

#[test]
fn ui_spec_round_trip() {
    let spec = UiSpec::Container {
        id: "1:1".to_string(),
        name: "Root".to_string(),
        text: String::new(),
        children: vec![UiSpec::Text {
            id: "1:2".to_string(),
            name: "Title".to_string(),
            children: Vec::new(),
            repeat_element_ids: Vec::new(),
        }],
        repeat_element_ids: Vec::new(),
    };

    let json = serde_json::to_string(&spec).unwrap();
    let back: UiSpec = serde_json::from_str(&json).unwrap();
    assert_eq!(spec, back);
}

#[test]
fn ron_serialization_is_stable() {
    let spec = UiSpec::Container {
        id: "1:1".to_string(),
        name: "Root".to_string(),
        text: String::new(),
        children: vec![UiSpec::Vector {
            id: "1:2".to_string(),
            name: "Icon".to_string(),
            children: Vec::new(),
            repeat_element_ids: Vec::new(),
        }],
        repeat_element_ids: Vec::new(),
    };

    let first = spec.to_pretty_ron().unwrap();
    let second = spec.to_pretty_ron().unwrap();
    assert_eq!(first, second);
    assert!(first.contains("Container("));
    assert!(!first.contains("text:"));
}

#[test]
fn container_text_field_omits_when_empty_and_serializes_when_present() {
    let empty_text = UiSpec::Container {
        id: "1:1".to_string(),
        name: "Root".to_string(),
        text: String::new(),
        children: Vec::new(),
        repeat_element_ids: Vec::new(),
    };
    let filled_text = UiSpec::Container {
        id: "1:1".to_string(),
        name: "Root".to_string(),
        text: "Title".to_string(),
        children: Vec::new(),
        repeat_element_ids: Vec::new(),
    };

    let empty_ron = empty_text.to_pretty_ron().unwrap();
    let filled_ron = filled_text.to_pretty_ron().unwrap();

    assert!(!empty_ron.contains("text:"));
    assert!(filled_ron.contains("text: \"Title\""));
}

#[test]
fn leaf_nodes_omit_empty_children_in_ron() {
    let leaf = UiSpec::Text {
        id: "9:9".to_string(),
        name: "Leaf".to_string(),
        children: Vec::new(),
        repeat_element_ids: Vec::new(),
    };

    let ron = leaf.to_pretty_ron().unwrap();
    assert!(!ron.contains("children"));
}

#[test]
fn container_repeat_element_ids_omit_when_empty_and_serialize_when_present() {
    let without_repeat = UiSpec::Container {
        id: "1:1".to_string(),
        name: "Root".to_string(),
        text: String::new(),
        children: vec![UiSpec::Text {
            id: "1:2".to_string(),
            name: "Item".to_string(),
            children: Vec::new(),
            repeat_element_ids: Vec::new(),
        }],
        repeat_element_ids: Vec::new(),
    };
    let with_repeat = UiSpec::Container {
        id: "1:1".to_string(),
        name: "Root".to_string(),
        text: String::new(),
        children: vec![UiSpec::Text {
            id: "1:2".to_string(),
            name: "Item".to_string(),
            children: Vec::new(),
            repeat_element_ids: Vec::new(),
        }],
        repeat_element_ids: vec!["1:2".to_string()],
    };

    let without_ron = without_repeat.to_pretty_ron().unwrap();
    let with_ron = with_repeat.to_pretty_ron().unwrap();

    assert!(!without_ron.contains("repeat_element_ids"));
    assert!(with_ron.contains("repeat_element_ids"));
    assert!(with_ron.contains("\"1:2\""));
}

#[test]
fn leaf_repeat_element_ids_omit_when_empty_and_serialize_when_present() {
    let without_repeat = UiSpec::Text {
        id: "9:9".to_string(),
        name: "Leaf".to_string(),
        children: Vec::new(),
        repeat_element_ids: Vec::new(),
    };
    let with_repeat = UiSpec::Text {
        id: "9:9".to_string(),
        name: "Leaf".to_string(),
        children: Vec::new(),
        repeat_element_ids: vec!["a:1".to_string(), "a:2".to_string()],
    };

    let without_ron = without_repeat.to_pretty_ron().unwrap();
    let with_ron = with_repeat.to_pretty_ron().unwrap();

    assert!(!without_ron.contains("repeat_element_ids"));
    assert!(with_ron.contains("repeat_element_ids"));
    assert!(with_ron.contains("\"a:1\""));
    assert!(with_ron.contains("\"a:2\""));
}

#[test]
fn transform_plan_round_trip_json() {
    let plan = TransformPlan {
        version: TRANSFORM_PLAN_VERSION.to_string(),
        decisions: vec![TransformDecision {
            node_id: "1:2".to_string(),
            suggested_type: SuggestedNodeType::Button,
            child_policy: ChildPolicy {
                mode: ChildPolicyMode::Drop,
                children: Vec::new(),
            },
            confidence: 0.82,
            reason: "Action control".to_string(),
        }],
    };

    let encoded = serde_json::to_string(&plan).expect("transform plan should serialize");
    let decoded: TransformPlan =
        serde_json::from_str(encoded.as_str()).expect("transform plan should deserialize");

    assert_eq!(decoded, plan);
}

#[test]
fn transform_plan_validate_rejects_missing_node_id() {
    let pre_layout = UiSpec::Container {
        id: "1:1".to_string(),
        name: "Root".to_string(),
        text: String::new(),
        children: vec![UiSpec::Text {
            id: "1:2".to_string(),
            name: "Title".to_string(),
            children: Vec::new(),
            repeat_element_ids: Vec::new(),
        }],
        repeat_element_ids: Vec::new(),
    };
    let plan = TransformPlan {
        version: TRANSFORM_PLAN_VERSION.to_string(),
        decisions: vec![TransformDecision {
            node_id: "9:9".to_string(),
            suggested_type: SuggestedNodeType::Button,
            child_policy: ChildPolicy {
                mode: ChildPolicyMode::Keep,
                children: Vec::new(),
            },
            confidence: 0.6,
            reason: "Not present".to_string(),
        }],
    };

    let err = plan
        .validate_against_pre_layout(&pre_layout)
        .expect_err("validation should fail");
    assert_eq!(
        err,
        TransformPlanValidationError::DecisionNodeNotFound("9:9".to_string())
    );
}

#[test]
fn transform_plan_validate_rejects_replace_with_unknown_child() {
    let pre_layout = UiSpec::Container {
        id: "1:1".to_string(),
        name: "Root".to_string(),
        text: String::new(),
        children: vec![
            UiSpec::Container {
                id: "1:2".to_string(),
                name: "Card".to_string(),
                text: String::new(),
                children: vec![UiSpec::Text {
                    id: "1:3".to_string(),
                    name: "Label".to_string(),
                    children: Vec::new(),
                    repeat_element_ids: Vec::new(),
                }],
                repeat_element_ids: Vec::new(),
            },
            UiSpec::Text {
                id: "1:4".to_string(),
                name: "Footer".to_string(),
                children: Vec::new(),
                repeat_element_ids: Vec::new(),
            },
        ],
        repeat_element_ids: Vec::new(),
    };
    let plan = TransformPlan {
        version: TRANSFORM_PLAN_VERSION.to_string(),
        decisions: vec![TransformDecision {
            node_id: "1:2".to_string(),
            suggested_type: SuggestedNodeType::HStack,
            child_policy: ChildPolicy {
                mode: ChildPolicyMode::ReplaceWith,
                children: vec!["9:9".to_string()],
            },
            confidence: 0.71,
            reason: "Force children".to_string(),
        }],
    };

    let err = plan
        .validate_against_pre_layout(&pre_layout)
        .expect_err("validation should fail");
    assert_eq!(
        err,
        TransformPlanValidationError::ReplacementChildNotFound {
            node_id: "1:2".to_string(),
            child_id: "9:9".to_string(),
        }
    );
}

#[test]
fn transform_plan_validate_rejects_non_replace_mode_with_children() {
    let pre_layout = UiSpec::Container {
        id: "1:1".to_string(),
        name: "Root".to_string(),
        text: String::new(),
        children: vec![UiSpec::Text {
            id: "1:2".to_string(),
            name: "Label".to_string(),
            children: Vec::new(),
            repeat_element_ids: Vec::new(),
        }],
        repeat_element_ids: Vec::new(),
    };
    let plan = TransformPlan {
        version: TRANSFORM_PLAN_VERSION.to_string(),
        decisions: vec![TransformDecision {
            node_id: "1:1".to_string(),
            suggested_type: SuggestedNodeType::VStack,
            child_policy: ChildPolicy {
                mode: ChildPolicyMode::Keep,
                children: vec!["1:2".to_string()],
            },
            confidence: 0.65,
            reason: "Invalid with keep".to_string(),
        }],
    };

    let err = plan
        .validate_against_pre_layout(&pre_layout)
        .expect_err("validation should fail");
    assert_eq!(
        err,
        TransformPlanValidationError::UnexpectedChildrenForMode {
            node_id: "1:1".to_string(),
            mode: ChildPolicyMode::Keep,
        }
    );
}

#[test]
fn apply_transform_plan_drop_removes_children_and_sets_button_type() {
    let pre_layout = UiSpec::Container {
        id: "1:1".to_string(),
        name: "Root".to_string(),
        text: String::new(),
        children: vec![UiSpec::Container {
            id: "1:2".to_string(),
            name: "CTA".to_string(),
            text: String::new(),
            children: vec![UiSpec::Text {
                id: "1:3".to_string(),
                name: "Buy".to_string(),
                children: Vec::new(),
                repeat_element_ids: Vec::new(),
            }],
            repeat_element_ids: Vec::new(),
        }],
        repeat_element_ids: Vec::new(),
    };
    let plan = TransformPlan {
        version: TRANSFORM_PLAN_VERSION.to_string(),
        decisions: vec![TransformDecision {
            node_id: "1:2".to_string(),
            suggested_type: SuggestedNodeType::Button,
            child_policy: ChildPolicy {
                mode: ChildPolicyMode::Drop,
                children: Vec::new(),
            },
            confidence: 0.9,
            reason: "Leaf control".to_string(),
        }],
    };

    let transformed = apply_transform_plan(&pre_layout, &plan).expect("transform should succeed");
    let button = &transformed.children()[0];
    assert_eq!(button.node_type(), NodeType::Button);
    assert!(button.children().is_empty());
}

#[test]
fn apply_transform_plan_keep_preserves_children_and_sets_hstack_type() {
    let pre_layout = UiSpec::Container {
        id: "1:1".to_string(),
        name: "Root".to_string(),
        text: String::new(),
        children: vec![UiSpec::Container {
            id: "1:2".to_string(),
            name: "Row".to_string(),
            text: String::new(),
            children: vec![
                UiSpec::Text {
                    id: "1:3".to_string(),
                    name: "Left".to_string(),
                    children: Vec::new(),
                    repeat_element_ids: Vec::new(),
                },
                UiSpec::Text {
                    id: "1:4".to_string(),
                    name: "Right".to_string(),
                    children: Vec::new(),
                    repeat_element_ids: Vec::new(),
                },
            ],
            repeat_element_ids: Vec::new(),
        }],
        repeat_element_ids: Vec::new(),
    };
    let plan = TransformPlan {
        version: TRANSFORM_PLAN_VERSION.to_string(),
        decisions: vec![TransformDecision {
            node_id: "1:2".to_string(),
            suggested_type: SuggestedNodeType::HStack,
            child_policy: ChildPolicy {
                mode: ChildPolicyMode::Keep,
                children: Vec::new(),
            },
            confidence: 0.78,
            reason: "Horizontal alignment".to_string(),
        }],
    };

    let transformed = apply_transform_plan(&pre_layout, &plan).expect("transform should succeed");
    let row = &transformed.children()[0];
    assert_eq!(row.node_type(), NodeType::HStack);
    assert_eq!(row.children().len(), 2);
    assert_eq!(row.children()[0].id(), "1:3");
    assert_eq!(row.children()[1].id(), "1:4");
}

#[test]
fn apply_transform_plan_replace_with_rewires_children_order() {
    let pre_layout = UiSpec::Container {
        id: "1:1".to_string(),
        name: "Root".to_string(),
        text: String::new(),
        children: vec![UiSpec::Container {
            id: "1:2".to_string(),
            name: "Scroll Region".to_string(),
            text: String::new(),
            children: vec![
                UiSpec::Text {
                    id: "1:3".to_string(),
                    name: "A".to_string(),
                    children: Vec::new(),
                    repeat_element_ids: Vec::new(),
                },
                UiSpec::Image {
                    id: "1:4".to_string(),
                    name: "Preview".to_string(),
                    children: Vec::new(),
                    repeat_element_ids: Vec::new(),
                },
            ],
            repeat_element_ids: Vec::new(),
        }],
        repeat_element_ids: Vec::new(),
    };
    let plan = TransformPlan {
        version: TRANSFORM_PLAN_VERSION.to_string(),
        decisions: vec![TransformDecision {
            node_id: "1:2".to_string(),
            suggested_type: SuggestedNodeType::ScrollView,
            child_policy: ChildPolicy {
                mode: ChildPolicyMode::ReplaceWith,
                children: vec!["1:4".to_string()],
            },
            confidence: 0.74,
            reason: "Content subset".to_string(),
        }],
    };

    let transformed = apply_transform_plan(&pre_layout, &plan).expect("transform should succeed");
    let scroll = &transformed.children()[0];
    assert_eq!(scroll.node_type(), NodeType::ScrollView);
    assert_eq!(scroll.children().len(), 1);
    assert_eq!(scroll.children()[0].id(), "1:4");
}

#[test]
fn apply_transform_plan_preserves_repeat_element_ids_when_type_changes() {
    let pre_layout = UiSpec::Container {
        id: "1:1".to_string(),
        name: "Root".to_string(),
        text: String::new(),
        children: vec![UiSpec::Container {
            id: "1:2".to_string(),
            name: "Card".to_string(),
            text: String::new(),
            children: Vec::new(),
            repeat_element_ids: vec!["card:1".to_string(), "card:2".to_string()],
        }],
        repeat_element_ids: Vec::new(),
    };
    let plan = TransformPlan {
        version: TRANSFORM_PLAN_VERSION.to_string(),
        decisions: vec![TransformDecision {
            node_id: "1:2".to_string(),
            suggested_type: SuggestedNodeType::VStack,
            child_policy: ChildPolicy {
                mode: ChildPolicyMode::Keep,
                children: Vec::new(),
            },
            confidence: 0.84,
            reason: "Treat card as vertical stack".to_string(),
        }],
    };

    let transformed = apply_transform_plan(&pre_layout, &plan).expect("transform should succeed");
    let card = &transformed.children()[0];
    assert_eq!(card.node_type(), NodeType::VStack);
    assert_eq!(
        card.repeat_element_ids(),
        &["card:1".to_string(), "card:2".to_string()]
    );
}

#[test]
fn build_ui_spec_preserves_original_node_ids() {
    let normalized = figma_normalizer::NormalizationOutput {
        document: figma_normalizer::NormalizedDocument {
            schema_version: figma_normalizer::NORMALIZED_SCHEMA_VERSION.to_string(),
            source: figma_normalizer::NormalizedSource {
                file_key: "abc123".to_string(),
                root_node_id: "1:1".to_string(),
                figma_api_version: figma_normalizer::FIGMA_API_VERSION.to_string(),
            },
            nodes: vec![
                container_node("1:1", vec!["2:1".to_string(), "3:1".to_string()]),
                text_node("2:1"),
                vector_node("3:1"),
            ],
        },
        warnings: Vec::new(),
    };

    let spec = build_pre_layout_spec(&normalized).expect("build should succeed");
    assert_eq!(spec.id(), "1:1");
    assert_eq!(spec.node_type(), NodeType::Container);
    assert_eq!(spec.children().len(), 2);
    assert_eq!(spec.children()[0].id(), "2:1");
    assert_eq!(spec.children()[0].node_type(), NodeType::Text);
    assert_eq!(spec.children()[1].id(), "3:1");
    assert_eq!(spec.children()[1].node_type(), NodeType::Vector);
}

#[test]
fn build_ui_spec_marks_image_fill_nodes_as_image() {
    let normalized = figma_normalizer::NormalizationOutput {
        document: figma_normalizer::NormalizedDocument {
            schema_version: figma_normalizer::NORMALIZED_SCHEMA_VERSION.to_string(),
            source: figma_normalizer::NormalizedSource {
                file_key: "abc123".to_string(),
                root_node_id: "1:1".to_string(),
                figma_api_version: figma_normalizer::FIGMA_API_VERSION.to_string(),
            },
            nodes: vec![
                container_node("1:1", vec!["2:1".to_string(), "3:1".to_string()]),
                image_node("2:1"),
                text_node("3:1"),
            ],
        },
        warnings: Vec::new(),
    };

    let spec = build_pre_layout_spec(&normalized).expect("build should succeed");
    assert_eq!(spec.node_type(), NodeType::Container);
    assert_eq!(spec.children()[0].node_type(), NodeType::Image);
}

#[test]
fn build_ui_spec_collapses_container_with_single_text_child_into_text_field() {
    let normalized = figma_normalizer::NormalizationOutput {
        document: figma_normalizer::NormalizedDocument {
            schema_version: figma_normalizer::NORMALIZED_SCHEMA_VERSION.to_string(),
            source: figma_normalizer::NormalizedSource {
                file_key: "abc123".to_string(),
                root_node_id: "1:1".to_string(),
                figma_api_version: figma_normalizer::FIGMA_API_VERSION.to_string(),
            },
            nodes: vec![
                container_node("1:1", vec!["2:1".to_string()]),
                text_node("2:1"),
            ],
        },
        warnings: Vec::new(),
    };

    let spec = build_pre_layout_spec(&normalized).expect("build should succeed");
    assert_eq!(spec.node_type(), NodeType::Container);
    assert!(spec.children().is_empty());
    match spec {
        UiSpec::Container { text, .. } => assert_eq!(text, "Text"),
        _ => panic!("expected container"),
    }
}

#[test]
fn build_ui_spec_omits_invisible_nodes() {
    let normalized = figma_normalizer::NormalizationOutput {
        document: figma_normalizer::NormalizedDocument {
            schema_version: figma_normalizer::NORMALIZED_SCHEMA_VERSION.to_string(),
            source: figma_normalizer::NormalizedSource {
                file_key: "abc123".to_string(),
                root_node_id: "1:1".to_string(),
                figma_api_version: figma_normalizer::FIGMA_API_VERSION.to_string(),
            },
            nodes: vec![
                container_node(
                    "1:1",
                    vec!["2:1".to_string(), "3:1".to_string(), "4:1".to_string()],
                ),
                text_node("2:1"),
                hidden_text_node("3:1"),
                hidden_container_node("4:1", vec!["5:1".to_string()]),
                text_node("5:1"),
            ],
        },
        warnings: Vec::new(),
    };

    let spec = build_pre_layout_spec(&normalized).expect("build should succeed");
    let child_ids = spec
        .children()
        .iter()
        .map(|child| child.id().to_string())
        .collect::<Vec<_>>();

    assert!(child_ids.is_empty());
    match spec {
        UiSpec::Container { text, .. } => assert_eq!(text, "Text"),
        _ => panic!("expected container"),
    }
}

#[test]
fn build_ui_spec_rejects_missing_root_node() {
    let normalized = figma_normalizer::NormalizationOutput {
        document: figma_normalizer::NormalizedDocument {
            schema_version: figma_normalizer::NORMALIZED_SCHEMA_VERSION.to_string(),
            source: figma_normalizer::NormalizedSource {
                file_key: "abc123".to_string(),
                root_node_id: "missing-root".to_string(),
                figma_api_version: figma_normalizer::FIGMA_API_VERSION.to_string(),
            },
            nodes: vec![text_node("2:1")],
        },
        warnings: Vec::new(),
    };

    let err = build_pre_layout_spec(&normalized).expect_err("missing root should fail");
    assert!(
        err.to_string()
            .contains("missing normalized root node: missing-root")
    );
}

#[test]
fn build_ui_spec_maps_unknown_leaf_node_kind_to_vector() {
    let normalized = figma_normalizer::NormalizationOutput {
        document: figma_normalizer::NormalizedDocument {
            schema_version: figma_normalizer::NORMALIZED_SCHEMA_VERSION.to_string(),
            source: figma_normalizer::NormalizedSource {
                file_key: "abc123".to_string(),
                root_node_id: "1:1".to_string(),
                figma_api_version: figma_normalizer::FIGMA_API_VERSION.to_string(),
            },
            nodes: vec![figma_normalizer::NormalizedNode {
                id: "1:1".to_string(),
                parent_id: None,
                name: "Unknown".to_string(),
                kind: figma_normalizer::NodeKind::Unknown,
                visible: true,
                bounds: figma_normalizer::Bounds {
                    x: 0.0,
                    y: 0.0,
                    w: 10.0,
                    h: 10.0,
                },
                layout: None,
                constraints: None,
                style: figma_normalizer::NodeStyle {
                    opacity: 1.0,
                    corner_radius: None,
                    fills: Vec::new(),
                    strokes: Vec::new(),
                },
                component: figma_normalizer::ComponentMetadata {
                    component_id: None,
                    component_set_id: None,
                    instance_of: None,
                    variant_properties: Vec::new(),
                },
                passthrough_fields: std::collections::BTreeMap::new(),
                children: Vec::new(),
            }],
        },
        warnings: Vec::new(),
    };

    let spec = build_pre_layout_spec(&normalized).expect("build should succeed");
    assert_eq!(spec.node_type(), NodeType::Vector);
}

#[test]
fn build_ui_spec_maps_unknown_node_with_children_to_container() {
    let normalized = figma_normalizer::NormalizationOutput {
        document: figma_normalizer::NormalizedDocument {
            schema_version: figma_normalizer::NORMALIZED_SCHEMA_VERSION.to_string(),
            source: figma_normalizer::NormalizedSource {
                file_key: "abc123".to_string(),
                root_node_id: "1:1".to_string(),
                figma_api_version: figma_normalizer::FIGMA_API_VERSION.to_string(),
            },
            nodes: vec![
                figma_normalizer::NormalizedNode {
                    id: "1:1".to_string(),
                    parent_id: None,
                    name: "Unknown Parent".to_string(),
                    kind: figma_normalizer::NodeKind::Unknown,
                    visible: true,
                    bounds: figma_normalizer::Bounds {
                        x: 0.0,
                        y: 0.0,
                        w: 10.0,
                        h: 10.0,
                    },
                    layout: None,
                    constraints: None,
                    style: figma_normalizer::NodeStyle {
                        opacity: 1.0,
                        corner_radius: None,
                        fills: Vec::new(),
                        strokes: Vec::new(),
                    },
                    component: figma_normalizer::ComponentMetadata {
                        component_id: None,
                        component_set_id: None,
                        instance_of: None,
                        variant_properties: Vec::new(),
                    },
                    passthrough_fields: std::collections::BTreeMap::new(),
                    children: vec!["2:1".to_string()],
                },
                text_node("2:1"),
            ],
        },
        warnings: Vec::new(),
    };

    let spec = build_pre_layout_spec(&normalized).expect("build should succeed");
    assert_eq!(spec.node_type(), NodeType::Container);
    assert!(spec.children().is_empty());
    match spec {
        UiSpec::Container { text, .. } => assert_eq!(text, "Text"),
        _ => panic!("expected container"),
    }
}

#[test]
fn build_ui_spec_maps_instance_kind_to_instance() {
    let normalized = figma_normalizer::NormalizationOutput {
        document: figma_normalizer::NormalizedDocument {
            schema_version: figma_normalizer::NORMALIZED_SCHEMA_VERSION.to_string(),
            source: figma_normalizer::NormalizedSource {
                file_key: "abc123".to_string(),
                root_node_id: "1:1".to_string(),
                figma_api_version: figma_normalizer::FIGMA_API_VERSION.to_string(),
            },
            nodes: vec![
                container_node("1:1", vec!["2:1".to_string()]),
                instance_node("2:1", vec!["3:1".to_string()]),
                text_node("3:1"),
            ],
        },
        warnings: Vec::new(),
    };

    let spec = build_pre_layout_spec(&normalized).expect("build should succeed");
    assert_eq!(spec.children()[0].node_type(), NodeType::Instance);
    assert_eq!(spec.children()[0].children().len(), 1);
    assert_eq!(spec.children()[0].children()[0].node_type(), NodeType::Text);
}

#[test]
fn build_ui_spec_maps_rectangle_kind_to_shape() {
    let normalized = figma_normalizer::NormalizationOutput {
        document: figma_normalizer::NormalizedDocument {
            schema_version: figma_normalizer::NORMALIZED_SCHEMA_VERSION.to_string(),
            source: figma_normalizer::NormalizedSource {
                file_key: "abc123".to_string(),
                root_node_id: "1:1".to_string(),
                figma_api_version: figma_normalizer::FIGMA_API_VERSION.to_string(),
            },
            nodes: vec![
                container_node("1:1", vec!["2:1".to_string(), "3:1".to_string()]),
                rectangle_node("2:1"),
                text_node("3:1"),
            ],
        },
        warnings: Vec::new(),
    };

    let spec = build_pre_layout_spec(&normalized).expect("build should succeed");
    assert_eq!(spec.node_type(), NodeType::Container);
    assert_eq!(spec.children()[0].node_type(), NodeType::Shape);
}

#[test]
fn build_ui_spec_collapses_container_with_all_shape_children_to_shape() {
    let normalized = figma_normalizer::NormalizationOutput {
        document: figma_normalizer::NormalizedDocument {
            schema_version: figma_normalizer::NORMALIZED_SCHEMA_VERSION.to_string(),
            source: figma_normalizer::NormalizedSource {
                file_key: "abc123".to_string(),
                root_node_id: "1:1".to_string(),
                figma_api_version: figma_normalizer::FIGMA_API_VERSION.to_string(),
            },
            nodes: vec![
                container_node("1:1", vec!["2:1".to_string(), "3:1".to_string()]),
                rectangle_node("2:1"),
                rectangle_node("3:1"),
            ],
        },
        warnings: Vec::new(),
    };

    let spec = build_pre_layout_spec(&normalized).expect("build should succeed");
    assert_eq!(spec.node_type(), NodeType::Shape);
    assert!(spec.children().is_empty());
}

#[test]
fn build_ui_spec_collapses_container_with_one_image_and_shapes_to_image() {
    let normalized = figma_normalizer::NormalizationOutput {
        document: figma_normalizer::NormalizedDocument {
            schema_version: figma_normalizer::NORMALIZED_SCHEMA_VERSION.to_string(),
            source: figma_normalizer::NormalizedSource {
                file_key: "abc123".to_string(),
                root_node_id: "1:1".to_string(),
                figma_api_version: figma_normalizer::FIGMA_API_VERSION.to_string(),
            },
            nodes: vec![
                container_node(
                    "1:1",
                    vec!["2:1".to_string(), "3:1".to_string(), "4:1".to_string()],
                ),
                image_node("2:1"),
                rectangle_node("3:1"),
                rectangle_node("4:1"),
            ],
        },
        warnings: Vec::new(),
    };

    let spec = build_pre_layout_spec(&normalized).expect("build should succeed");
    assert_eq!(spec.node_type(), NodeType::Image);
    assert!(spec.children().is_empty());
}

#[test]
fn build_ui_spec_collapses_instance_with_one_image_and_shapes_to_image() {
    let normalized = figma_normalizer::NormalizationOutput {
        document: figma_normalizer::NormalizedDocument {
            schema_version: figma_normalizer::NORMALIZED_SCHEMA_VERSION.to_string(),
            source: figma_normalizer::NormalizedSource {
                file_key: "abc123".to_string(),
                root_node_id: "1:1".to_string(),
                figma_api_version: figma_normalizer::FIGMA_API_VERSION.to_string(),
            },
            nodes: vec![
                container_node("1:1", vec!["2:1".to_string(), "6:1".to_string()]),
                instance_node(
                    "2:1",
                    vec!["3:1".to_string(), "4:1".to_string(), "5:1".to_string()],
                ),
                image_node("3:1"),
                rectangle_node("4:1"),
                rectangle_node("5:1"),
                text_node("6:1"),
            ],
        },
        warnings: Vec::new(),
    };

    let spec = build_pre_layout_spec(&normalized).expect("build should succeed");
    let collapsed = &spec.children()[0];
    assert_eq!(collapsed.node_type(), NodeType::Image);
    assert!(collapsed.children().is_empty());
}

#[test]
fn build_ui_spec_collapses_container_with_single_image_child_to_image() {
    let normalized = figma_normalizer::NormalizationOutput {
        document: figma_normalizer::NormalizedDocument {
            schema_version: figma_normalizer::NORMALIZED_SCHEMA_VERSION.to_string(),
            source: figma_normalizer::NormalizedSource {
                file_key: "abc123".to_string(),
                root_node_id: "1:1".to_string(),
                figma_api_version: figma_normalizer::FIGMA_API_VERSION.to_string(),
            },
            nodes: vec![
                container_node("1:1", vec!["2:1".to_string()]),
                image_node("2:1"),
            ],
        },
        warnings: Vec::new(),
    };

    let spec = build_pre_layout_spec(&normalized).expect("build should succeed");
    assert_eq!(spec.node_type(), NodeType::Image);
    assert!(spec.children().is_empty());
}

#[test]
fn build_ui_spec_collapses_instance_with_single_image_child_to_image() {
    let normalized = figma_normalizer::NormalizationOutput {
        document: figma_normalizer::NormalizedDocument {
            schema_version: figma_normalizer::NORMALIZED_SCHEMA_VERSION.to_string(),
            source: figma_normalizer::NormalizedSource {
                file_key: "abc123".to_string(),
                root_node_id: "1:1".to_string(),
                figma_api_version: figma_normalizer::FIGMA_API_VERSION.to_string(),
            },
            nodes: vec![
                container_node("1:1", vec!["2:1".to_string(), "4:1".to_string()]),
                instance_node("2:1", vec!["3:1".to_string()]),
                image_node("3:1"),
                text_node("4:1"),
            ],
        },
        warnings: Vec::new(),
    };

    let spec = build_pre_layout_spec(&normalized).expect("build should succeed");
    let collapsed = &spec.children()[0];
    assert_eq!(collapsed.node_type(), NodeType::Image);
    assert!(collapsed.children().is_empty());
}

#[test]
fn build_ui_spec_collapses_container_with_one_vector_and_shapes_to_vector() {
    let normalized = figma_normalizer::NormalizationOutput {
        document: figma_normalizer::NormalizedDocument {
            schema_version: figma_normalizer::NORMALIZED_SCHEMA_VERSION.to_string(),
            source: figma_normalizer::NormalizedSource {
                file_key: "abc123".to_string(),
                root_node_id: "1:1".to_string(),
                figma_api_version: figma_normalizer::FIGMA_API_VERSION.to_string(),
            },
            nodes: vec![
                container_node(
                    "1:1",
                    vec!["2:1".to_string(), "3:1".to_string(), "4:1".to_string()],
                ),
                vector_node("2:1"),
                rectangle_node("3:1"),
                rectangle_node("4:1"),
            ],
        },
        warnings: Vec::new(),
    };

    let spec = build_pre_layout_spec(&normalized).expect("build should succeed");
    assert_eq!(spec.node_type(), NodeType::Vector);
    assert!(spec.children().is_empty());
}

#[test]
fn build_ui_spec_drops_children_for_instance_with_one_vector_and_shapes() {
    let normalized = figma_normalizer::NormalizationOutput {
        document: figma_normalizer::NormalizedDocument {
            schema_version: figma_normalizer::NORMALIZED_SCHEMA_VERSION.to_string(),
            source: figma_normalizer::NormalizedSource {
                file_key: "abc123".to_string(),
                root_node_id: "1:1".to_string(),
                figma_api_version: figma_normalizer::FIGMA_API_VERSION.to_string(),
            },
            nodes: vec![
                container_node("1:1", vec!["2:1".to_string(), "6:1".to_string()]),
                instance_node(
                    "2:1",
                    vec!["3:1".to_string(), "4:1".to_string(), "5:1".to_string()],
                ),
                vector_node("3:1"),
                rectangle_node("4:1"),
                rectangle_node("5:1"),
                text_node("6:1"),
            ],
        },
        warnings: Vec::new(),
    };

    let spec = build_pre_layout_spec(&normalized).expect("build should succeed");
    let collapsed = &spec.children()[0];
    assert_eq!(collapsed.node_type(), NodeType::Instance);
    assert!(collapsed.children().is_empty());
}

#[test]
fn build_ui_spec_drops_children_for_instance_with_multiple_vectors_and_shapes() {
    let normalized = figma_normalizer::NormalizationOutput {
        document: figma_normalizer::NormalizedDocument {
            schema_version: figma_normalizer::NORMALIZED_SCHEMA_VERSION.to_string(),
            source: figma_normalizer::NormalizedSource {
                file_key: "abc123".to_string(),
                root_node_id: "1:1".to_string(),
                figma_api_version: figma_normalizer::FIGMA_API_VERSION.to_string(),
            },
            nodes: vec![
                container_node("1:1", vec!["2:1".to_string(), "7:1".to_string()]),
                instance_node(
                    "2:1",
                    vec![
                        "3:1".to_string(),
                        "4:1".to_string(),
                        "5:1".to_string(),
                        "6:1".to_string(),
                    ],
                ),
                vector_node("3:1"),
                vector_node("4:1"),
                rectangle_node("5:1"),
                rectangle_node("6:1"),
                text_node("7:1"),
            ],
        },
        warnings: Vec::new(),
    };

    let spec = build_pre_layout_spec(&normalized).expect("build should succeed");
    let collapsed = &spec.children()[0];
    assert_eq!(collapsed.node_type(), NodeType::Instance);
    assert!(collapsed.children().is_empty());
}

#[test]
fn build_ui_spec_collapses_container_with_single_vector_child_to_vector() {
    let normalized = figma_normalizer::NormalizationOutput {
        document: figma_normalizer::NormalizedDocument {
            schema_version: figma_normalizer::NORMALIZED_SCHEMA_VERSION.to_string(),
            source: figma_normalizer::NormalizedSource {
                file_key: "abc123".to_string(),
                root_node_id: "1:1".to_string(),
                figma_api_version: figma_normalizer::FIGMA_API_VERSION.to_string(),
            },
            nodes: vec![
                container_node("1:1", vec!["2:1".to_string()]),
                vector_node("2:1"),
            ],
        },
        warnings: Vec::new(),
    };

    let spec = build_pre_layout_spec(&normalized).expect("build should succeed");
    assert_eq!(spec.node_type(), NodeType::Vector);
    assert!(spec.children().is_empty());
}

#[test]
fn build_ui_spec_collapses_container_with_multiple_vector_children_to_vector() {
    let normalized = figma_normalizer::NormalizationOutput {
        document: figma_normalizer::NormalizedDocument {
            schema_version: figma_normalizer::NORMALIZED_SCHEMA_VERSION.to_string(),
            source: figma_normalizer::NormalizedSource {
                file_key: "abc123".to_string(),
                root_node_id: "1:1".to_string(),
                figma_api_version: figma_normalizer::FIGMA_API_VERSION.to_string(),
            },
            nodes: vec![
                container_node("1:1", vec!["2:1".to_string(), "3:1".to_string()]),
                vector_node("2:1"),
                vector_node("3:1"),
            ],
        },
        warnings: Vec::new(),
    };

    let spec = build_pre_layout_spec(&normalized).expect("build should succeed");
    assert_eq!(spec.node_type(), NodeType::Vector);
    assert!(spec.children().is_empty());
}

#[test]
fn build_ui_spec_collapses_instance_with_single_vector_child_to_vector() {
    let normalized = figma_normalizer::NormalizationOutput {
        document: figma_normalizer::NormalizedDocument {
            schema_version: figma_normalizer::NORMALIZED_SCHEMA_VERSION.to_string(),
            source: figma_normalizer::NormalizedSource {
                file_key: "abc123".to_string(),
                root_node_id: "1:1".to_string(),
                figma_api_version: figma_normalizer::FIGMA_API_VERSION.to_string(),
            },
            nodes: vec![
                container_node("1:1", vec!["2:1".to_string(), "4:1".to_string()]),
                instance_node("2:1", vec!["3:1".to_string()]),
                vector_node("3:1"),
                text_node("4:1"),
            ],
        },
        warnings: Vec::new(),
    };

    let spec = build_pre_layout_spec(&normalized).expect("build should succeed");
    let collapsed = &spec.children()[0];
    assert_eq!(collapsed.node_type(), NodeType::Vector);
    assert!(collapsed.children().is_empty());
}

#[test]
fn build_ui_spec_collapses_instance_with_multiple_vector_children_to_vector() {
    let normalized = figma_normalizer::NormalizationOutput {
        document: figma_normalizer::NormalizedDocument {
            schema_version: figma_normalizer::NORMALIZED_SCHEMA_VERSION.to_string(),
            source: figma_normalizer::NormalizedSource {
                file_key: "abc123".to_string(),
                root_node_id: "1:1".to_string(),
                figma_api_version: figma_normalizer::FIGMA_API_VERSION.to_string(),
            },
            nodes: vec![
                container_node("1:1", vec!["2:1".to_string(), "5:1".to_string()]),
                instance_node("2:1", vec!["3:1".to_string(), "4:1".to_string()]),
                vector_node("3:1"),
                vector_node("4:1"),
                text_node("5:1"),
            ],
        },
        warnings: Vec::new(),
    };

    let spec = build_pre_layout_spec(&normalized).expect("build should succeed");
    let collapsed = &spec.children()[0];
    assert_eq!(collapsed.node_type(), NodeType::Vector);
    assert!(collapsed.children().is_empty());
}

fn container_node(id: &str, children: Vec<String>) -> figma_normalizer::NormalizedNode {
    figma_normalizer::NormalizedNode {
        id: id.to_string(),
        parent_id: None,
        name: "Container".to_string(),
        kind: figma_normalizer::NodeKind::Frame,
        visible: true,
        bounds: figma_normalizer::Bounds {
            x: 0.0,
            y: 0.0,
            w: 300.0,
            h: 300.0,
        },
        layout: Some(figma_normalizer::LayoutMetadata {
            mode: figma_normalizer::LayoutMode::Vertical,
            primary_align: figma_normalizer::Align::Start,
            cross_align: figma_normalizer::Align::Stretch,
            item_spacing: 12.0,
            padding: figma_normalizer::Padding {
                top: 0.0,
                right: 0.0,
                bottom: 0.0,
                left: 0.0,
            },
        }),
        constraints: None,
        style: figma_normalizer::NodeStyle {
            opacity: 1.0,
            corner_radius: Some(8.0),
            fills: Vec::new(),
            strokes: Vec::new(),
        },
        component: figma_normalizer::ComponentMetadata {
            component_id: None,
            component_set_id: None,
            instance_of: None,
            variant_properties: Vec::new(),
        },
        passthrough_fields: std::collections::BTreeMap::new(),
        children,
    }
}

fn text_node(id: &str) -> figma_normalizer::NormalizedNode {
    figma_normalizer::NormalizedNode {
        id: id.to_string(),
        parent_id: None,
        name: "Text".to_string(),
        kind: figma_normalizer::NodeKind::Text,
        visible: true,
        bounds: figma_normalizer::Bounds {
            x: 16.0,
            y: 16.0,
            w: 120.0,
            h: 28.0,
        },
        layout: None,
        constraints: None,
        style: figma_normalizer::NodeStyle {
            opacity: 1.0,
            corner_radius: None,
            fills: Vec::new(),
            strokes: Vec::new(),
        },
        component: figma_normalizer::ComponentMetadata {
            component_id: None,
            component_set_id: None,
            instance_of: None,
            variant_properties: Vec::new(),
        },
        passthrough_fields: std::collections::BTreeMap::new(),
        children: Vec::new(),
    }
}

fn vector_node(id: &str) -> figma_normalizer::NormalizedNode {
    figma_normalizer::NormalizedNode {
        id: id.to_string(),
        parent_id: None,
        name: "Vector".to_string(),
        kind: figma_normalizer::NodeKind::Vector,
        visible: true,
        bounds: figma_normalizer::Bounds {
            x: 0.0,
            y: 0.0,
            w: 24.0,
            h: 24.0,
        },
        layout: None,
        constraints: None,
        style: figma_normalizer::NodeStyle {
            opacity: 1.0,
            corner_radius: None,
            fills: Vec::new(),
            strokes: Vec::new(),
        },
        component: figma_normalizer::ComponentMetadata {
            component_id: None,
            component_set_id: None,
            instance_of: None,
            variant_properties: Vec::new(),
        },
        passthrough_fields: std::collections::BTreeMap::new(),
        children: Vec::new(),
    }
}

fn rectangle_node(id: &str) -> figma_normalizer::NormalizedNode {
    figma_normalizer::NormalizedNode {
        id: id.to_string(),
        parent_id: None,
        name: "Shape".to_string(),
        kind: figma_normalizer::NodeKind::Rectangle,
        visible: true,
        bounds: figma_normalizer::Bounds {
            x: 0.0,
            y: 0.0,
            w: 24.0,
            h: 24.0,
        },
        layout: None,
        constraints: None,
        style: figma_normalizer::NodeStyle {
            opacity: 1.0,
            corner_radius: None,
            fills: Vec::new(),
            strokes: Vec::new(),
        },
        component: figma_normalizer::ComponentMetadata {
            component_id: None,
            component_set_id: None,
            instance_of: None,
            variant_properties: Vec::new(),
        },
        passthrough_fields: std::collections::BTreeMap::new(),
        children: Vec::new(),
    }
}

fn image_node(id: &str) -> figma_normalizer::NormalizedNode {
    figma_normalizer::NormalizedNode {
        id: id.to_string(),
        parent_id: None,
        name: "Avatar Graphic".to_string(),
        kind: figma_normalizer::NodeKind::Rectangle,
        visible: true,
        bounds: figma_normalizer::Bounds {
            x: 0.0,
            y: 0.0,
            w: 64.0,
            h: 64.0,
        },
        layout: None,
        constraints: None,
        style: figma_normalizer::NodeStyle {
            opacity: 1.0,
            corner_radius: None,
            fills: vec![figma_normalizer::Paint {
                kind: figma_normalizer::PaintKind::Image,
                color: None,
                image_ref: Some("img-ref".to_string()),
            }],
            strokes: Vec::new(),
        },
        component: figma_normalizer::ComponentMetadata {
            component_id: None,
            component_set_id: None,
            instance_of: None,
            variant_properties: Vec::new(),
        },
        passthrough_fields: std::collections::BTreeMap::new(),
        children: Vec::new(),
    }
}

fn instance_node(id: &str, children: Vec<String>) -> figma_normalizer::NormalizedNode {
    figma_normalizer::NormalizedNode {
        id: id.to_string(),
        parent_id: None,
        name: "Button Instance".to_string(),
        kind: figma_normalizer::NodeKind::Instance,
        visible: true,
        bounds: figma_normalizer::Bounds {
            x: 0.0,
            y: 0.0,
            w: 100.0,
            h: 40.0,
        },
        layout: None,
        constraints: None,
        style: figma_normalizer::NodeStyle {
            opacity: 1.0,
            corner_radius: None,
            fills: Vec::new(),
            strokes: Vec::new(),
        },
        component: figma_normalizer::ComponentMetadata {
            component_id: None,
            component_set_id: None,
            instance_of: Some("42:7".to_string()),
            variant_properties: Vec::new(),
        },
        passthrough_fields: std::collections::BTreeMap::new(),
        children,
    }
}

fn hidden_text_node(id: &str) -> figma_normalizer::NormalizedNode {
    let mut node = text_node(id);
    node.visible = false;
    node
}

fn hidden_container_node(id: &str, children: Vec<String>) -> figma_normalizer::NormalizedNode {
    let mut node = container_node(id, children);
    node.visible = false;
    node
}
