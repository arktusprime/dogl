use dogl_language::{
    apply_layout, export_bpmn, import_bpmn, layout_parse_output, parse, render_dogl, to_json,
    validate, validate_for_layout, validate_parse_output, ApplicationError, domain::DoglFile,
    domain::{Bounds, Element, GatewayCode, Identifiable, TaskCode},
    resolver::{
        BindingSummary, LoweredSemanticFile, NameResolutionSummary, NormalizationPass,
        ResolverDiagnostic, ResolverDiagnosticSeverity, ResolverOutput,
    },
    syntax::{
        ParseDiagnostic, ParseDiagnosticSeverity, RecoveryKind, RecoveryNode, SourcePosition, Span,
        SyntaxDocument, SyntaxKind, SyntaxNode, SyntaxNodeId, SyntaxToken, SyntaxTrivia,
        SyntaxTriviaKind, TokenKind, TokenRange, TriviaRange, UnresolvedName, UnresolvedNameKind,
    },
};

const VALID_CALL_ACTIVITY: &str = r#"collab RefundProcess
    == MainPool
        -- Ops
            || Default
                (s) Start
                    => ChildProcess
                [call] ChildProcess
                    => Done
                (e) Done
"#;

const VALID_DO_INLINE_COMMAND: &str = r#"collab CommandProcess
    == MainPool
        -- Ops
            || Default
                [] ReviewOrder [do] check amount
                    => Done
                (e) Done
"#;

const VALID_DO_EXEC_COMMAND_BLOCK: &str = r#"collab CommandProcess
    == MainPool
        -- Ops
            || Default
                [] ReviewOrder
                    [do.exec] validateOrder(order.id)
                    => Done
                (e) Done
"#;

const VALID_GATEWAY_DMN_COMMAND: &str = r#"collab DecisionProcess
    == MainPool
        -- Ops
            || Default
                <x> RouteOrder [dmn] OrderRouting
                    => Done
                (e) Done
"#;

const INVALID_BARE_TASK: &str = r#"collab SimpleProcess
    == MainPool
        -- Ops
            || Default
                (s) Start
                    => Review
                Review
                    => Done
                (e) Done
"#;

const VALID_GATEWAY_SYNTAX: &str = r#"collab GatewayProcess
    == MainPool
        -- Ops
            || Default
                (s) Start
                    => Route
                <x> Route
                    => Done
                (e) Done
"#;

const VALID_COMMENT_SYNTAX: &str = r#"// file comment
collab CommentedProcess // collab comment
    == MainPool
        -- Ops
            || Default
                (s) Start
                    => Done // flow comment
                (e) Done
"#;

// Quoted labels differ from name_from_id(id) so render and BPMN export keep explicit names.
const VALID_QUOTED_DISPLAY_NAME_SYNTAX: &str = r#"collab AliasProcess
    == MainPool
        -- Ops
            || Default
                (s) StartOrder "Begin order"
                    => ReviewOrder
                [] ReviewOrder "Check order" [do] check amount
                    => RouteOrder
                <x> RouteOrder "Routing"
                    => ChildProcess
                [call] ChildProcess "Initiation"
                    => Done
                (e) Done "Completed"
"#;

const INLINE_LAYOUT_SOURCE: &str = r#"collab LayoutProcess
    == MainPool {0 0 600 320}
        -- Ops {0 40 600 80}
            || Default {120 0 180 320}
                (s) Start {80 140 36 36}
                    => Review
                [] Review {180 132 100 52}
                    => Route
                <x> Route {340 136 50 50}
                    => Done
                (e) Done {460 140 36 36}
"#;

const BOTTOM_LAYOUT_SOURCE: &str = r#"collab LayoutProcess
    == MainPool
        -- Ops
            || Default
                (s) Start
                    => Review
                [] Review
                    => Route
                <x> Route
                    => Done
                (e) Done

layout
    == MainPool {0 0 600 320}
        -- Ops {0 40 600 80}
            || Default {120 0 180 320}
                (s) Start {80 140 36 36}
                [] Review {180 132 100 52}
                <x> Route {340 136 50 50}
                (e) Done {460 140 36 36}
"#;

const INVALID_VALIDATION_ORPHAN_TASK: &str = r#"collab InvalidProcess
    == MainPool
        -- Ops
            || Default
                [] Review
                (e) Done
"#;

const INVALID_END_EVENT_WITH_OUTGOING_FLOW: &str = r#"collab InvalidProcess
    == MainPool
        -- Ops
            || Default
                (s) Start
                    => Done
                (e) Done
                    => AfterDone
                [] AfterDone
"#;

const INVALID_COMPONENT_WITHOUT_BOUNDARY_EVENTS: &str = r#"collab InvalidProcess
    == MainPool
        -- Ops
            || Default
                [] Review
                    => Approve
                [] Approve
                    => Review
"#;

const VALID_TWO_GRAPHS_IN_ONE_POOL: &str = r#"collab MultiGraphProcess
    == MainPool
        -- Ops
            || Default
                (s) StartA
                    => ReviewA
                [] ReviewA
                    => DoneA
                (e) DoneA
                (s) StartB
                    => ReviewB
                [] ReviewB
                    => DoneB
                (e) DoneB
"#;

const LAYOUT_BASIC_CHAIN: &str = include_str!("fixtures/layout_basic_chain.dogl");
const LAYOUT_MULTIPLE_LANES: &str = include_str!("fixtures/layout_multiple_lanes.dogl");
const LAYOUT_LANE_EXPANSION: &str = include_str!("fixtures/layout_lane_expansion.dogl");
const LAYOUT_GATEWAY_FANOUT: &str = include_str!("fixtures/layout_gateway_fanout.dogl");
const LAYOUT_GATEWAY_MERGE: &str = include_str!("fixtures/layout_gateway_merge.dogl");
const LAYOUT_BACKWARD_CONNECTION: &str = include_str!("fixtures/layout_backward_connection.dogl");

#[test]
fn parse_returns_populated_layers_for_valid_call_activity_input() {
    let output = parse(VALID_CALL_ACTIVITY);

    assert_eq!(output.syntax.source.text, VALID_CALL_ACTIVITY);
    assert!(!output.syntax.tokens.is_empty());
    assert!(!output.syntax.nodes.is_empty());
    assert!(output.syntax.root.is_some());
    assert!(output.syntax.trivia.is_empty());
    assert!(output.syntax.diagnostics.is_empty());

    let semantic_file = output.semantic_file.as_ref().expect("semantic file");
    assert_eq!(semantic_file.collabs.len(), 1);
    let collab = &semantic_file.collabs[0];
    assert_eq!(collab.id, "RefundProcess");
    assert_eq!(collab.pools.len(), 1);

    let pool = &collab.pools[0];
    assert_eq!(pool.sequence_flows.len(), 2);
    let call_task = pool
        .quadrants
        .iter()
        .flat_map(|quadrant| quadrant.elements.iter())
        .find_map(|element| match element {
            Element::Task(task) if task.id == "ChildProcess" => Some(task),
            _ => None,
        })
        .expect("call task");
    assert_eq!(call_task.code, TaskCode::CallActivity);
    assert_eq!(call_task.call_target.as_deref(), Some("ChildProcess"));
    assert!(call_task.expressions.is_empty());
}

#[test]
fn parse_accepts_do_inline_command_in_public_api() {
    let output = parse(VALID_DO_INLINE_COMMAND);

    assert!(output.syntax.diagnostics.is_empty());
    assert!(output.resolver.diagnostics.is_empty());
    let semantic_file = output.semantic_file.as_ref().expect("semantic file");
    let pool = &semantic_file.collabs[0].pools[0];
    let task = pool
        .quadrants
        .iter()
        .flat_map(|quadrant| quadrant.elements.iter())
        .find_map(|element| match element {
            Element::Task(task) if task.id == "ReviewOrder" => Some(task),
            _ => None,
        })
        .expect("task");

    assert_eq!(task.expressions[0].key, "do");
    assert_eq!(task.expressions[0].value, "check amount");
}

#[test]
fn parse_accepts_do_exec_block_command_in_public_api() {
    let output = parse(VALID_DO_EXEC_COMMAND_BLOCK);

    assert!(output.syntax.diagnostics.is_empty());
    assert!(output.resolver.diagnostics.is_empty());
    let semantic_file = output.semantic_file.as_ref().expect("semantic file");
    let pool = &semantic_file.collabs[0].pools[0];
    let task = pool
        .quadrants
        .iter()
        .flat_map(|quadrant| quadrant.elements.iter())
        .find_map(|element| match element {
            Element::Task(task) if task.id == "ReviewOrder" => Some(task),
            _ => None,
        })
        .expect("task");

    assert_eq!(task.expressions[0].key, "do.exec");
    assert_eq!(task.expressions[0].value, "validateOrder(order.id)");
}

#[test]
fn parse_accepts_gateway_forms_in_public_api() {
    let output = parse(VALID_GATEWAY_SYNTAX);

    assert!(output.syntax.diagnostics.is_empty());
    assert!(output.resolver.diagnostics.is_empty());
    let semantic_file = output.semantic_file.as_ref().expect("semantic file");
    let pool = &semantic_file.collabs[0].pools[0];
    let gateway = pool
        .quadrants
        .iter()
        .flat_map(|quadrant| quadrant.elements.iter())
        .find_map(|element| match element {
            Element::Gateway(gateway) if gateway.id == "Route" => Some(gateway),
            _ => None,
        })
        .expect("gateway");

    assert_eq!(gateway.code, GatewayCode::Exclusive);
}

#[test]
fn parse_accepts_gateway_dmn_command_in_public_api() {
    let output = parse(VALID_GATEWAY_DMN_COMMAND);

    assert!(output.syntax.diagnostics.is_empty());
    assert!(output.resolver.diagnostics.is_empty());
    let semantic_file = output.semantic_file.as_ref().expect("semantic file");
    let pool = &semantic_file.collabs[0].pools[0];
    let gateway = pool
        .quadrants
        .iter()
        .flat_map(|quadrant| quadrant.elements.iter())
        .find_map(|element| match element {
            Element::Gateway(gateway) if gateway.id == "RouteOrder" => Some(gateway),
            _ => None,
        })
        .expect("gateway");

    assert_eq!(gateway.dmn_ref.as_deref(), Some("OrderRouting"));
}

#[test]
fn parse_accepts_slash_comments_in_public_api() {
    let output = parse(VALID_COMMENT_SYNTAX);

    assert!(output.syntax.diagnostics.is_empty());
    assert!(output.resolver.diagnostics.is_empty());
    assert!(output.semantic_file.is_some());
    assert!(output.syntax.comments().count() >= 2);
}

#[test]
fn parse_accepts_quoted_display_names_while_preserving_ascii_ids() {
    let output = parse(VALID_QUOTED_DISPLAY_NAME_SYNTAX);

    assert!(output.syntax.diagnostics.is_empty());
    assert!(output.resolver.diagnostics.is_empty());
    let semantic_file = output.semantic_file.as_ref().expect("semantic file");
    let pool = &semantic_file.collabs[0].pools[0];

    let start = pool
        .quadrants
        .iter()
        .flat_map(|quadrant| quadrant.elements.iter())
        .find_map(|element| match element {
            Element::Event(event) if event.id == "StartOrder" => Some(event),
            _ => None,
        })
        .expect("start event");
    assert_eq!(start.id, "StartOrder");
    assert_eq!(start.name, "Begin order");

    let review = pool
        .quadrants
        .iter()
        .flat_map(|quadrant| quadrant.elements.iter())
        .find_map(|element| match element {
            Element::Task(task) if task.id == "ReviewOrder" => Some(task),
            _ => None,
        })
        .expect("review task");
    assert_eq!(review.name, "Check order");
    assert_eq!(review.expressions[0].key, "do");

    let route = pool
        .quadrants
        .iter()
        .flat_map(|quadrant| quadrant.elements.iter())
        .find_map(|element| match element {
            Element::Gateway(gateway) if gateway.id == "RouteOrder" => Some(gateway),
            _ => None,
        })
        .expect("gateway");
    assert_eq!(route.name, "Routing");

    let call = pool
        .quadrants
        .iter()
        .flat_map(|quadrant| quadrant.elements.iter())
        .find_map(|element| match element {
            Element::Task(task) if task.id == "ChildProcess" => Some(task),
            _ => None,
        })
        .expect("call activity");
    assert_eq!(call.name, "Initiation");
    assert_eq!(call.call_target.as_deref(), Some("ChildProcess"));
    assert_eq!(pool.sequence_flows.len(), 4);
}

#[test]
fn import_bpmn_returns_not_implemented() {
    let result = import_bpmn("<definitions />");
    assert_eq!(result, Err(ApplicationError::NotImplemented("import_bpmn")));
}

#[test]
fn parse_keeps_resolver_stages_explicit_for_valid_input() {
    let output = parse(VALID_CALL_ACTIVITY);

    assert_eq!(output.resolver.bindings.bound_names, 7);
    assert_eq!(output.resolver.bindings.unresolved_names, 0);
    assert_eq!(output.resolver.resolution.resolved_references, 2);
    assert_eq!(output.resolver.resolution.unresolved_references, 0);
    assert_eq!(
        output.resolver.normalization_passes,
        vec![NormalizationPass::new("mvp_lowering")]
    );
    assert!(output.resolver.diagnostics.is_empty());
    assert_eq!(output.resolver.lowering.semantic_file, output.semantic_file);
}

#[test]
fn parse_rejects_bare_task_syntax_in_public_api() {
    let output = parse(INVALID_BARE_TASK);

    assert!(output.semantic_file.is_none());
    assert!(!output.syntax.diagnostics.is_empty());
}

#[test]
fn parse_populates_layout_from_inline_bounds_in_public_api() {
    let output = parse(INLINE_LAYOUT_SOURCE);

    assert!(output.syntax.diagnostics.is_empty());
    assert!(output.resolver.diagnostics.is_empty());
    let semantic_file = output.semantic_file.as_ref().expect("semantic file");
    assert!(semantic_file.collabs[0].layout.is_some());
}

#[test]
fn parse_populates_layout_from_bottom_layout_block_in_public_api() {
    let output = parse(BOTTOM_LAYOUT_SOURCE);

    assert!(output.syntax.diagnostics.is_empty());
    assert!(output.resolver.diagnostics.is_empty());
    let semantic_file = output.semantic_file.as_ref().expect("semantic file");
    assert!(semantic_file.collabs[0].layout.is_some());
}

#[test]
fn parse_surfaces_resolver_errors_for_invalid_call_activity_input() {
    let output = parse(
        r#"collab RefundProcess
    == MainPool
        -- Ops
            || Default
                [call]
                    => MissingTarget
"#,
    );

    assert!(output.semantic_file.is_none());
    assert!(!output.syntax.diagnostics.is_empty());
}

#[test]
fn parse_rejects_legacy_at_commands_in_public_api() {
    let output = parse(
        r#"collab LegacyProcess
    == MainPool
        -- Ops
            || Default
                [] ReviewOrder @do check amount
                    => RouteOrder
                <x> RouteOrder @dmn: "OrderRouting"
                    => ChildProcess
                [call] ChildProcess @call: "ChildProcess"
                    => Done
                (e) Done
"#,
    );

    assert!(output.semantic_file.is_none());
    assert!(!output.syntax.diagnostics.is_empty());
}

#[test]
fn resolver_contracts_preserve_stage_specific_placeholder_data() {
    let normalization = NormalizationPass::new("canonicalize_names");
    let diagnostic = ResolverDiagnostic::new(
        ResolverDiagnosticSeverity::Warning,
        "placeholder resolver diagnostic",
    );
    let output = ResolverOutput {
        bindings: BindingSummary {
            bound_names: 2,
            unresolved_names: 1,
        },
        resolution: NameResolutionSummary {
            resolved_references: 3,
            unresolved_references: 1,
        },
        normalization_passes: vec![normalization.clone()],
        lowering: LoweredSemanticFile { semantic_file: None },
        diagnostics: vec![diagnostic.clone()],
    };

    assert_eq!(output.bindings.bound_names, 2);
    assert_eq!(output.bindings.unresolved_names, 1);
    assert_eq!(output.resolution.resolved_references, 3);
    assert_eq!(output.resolution.unresolved_references, 1);
    assert_eq!(output.normalization_passes, vec![normalization]);
    assert!(output.lowering.semantic_file.is_none());
    assert_eq!(output.diagnostics, vec![diagnostic]);
}

#[test]
fn validation_facade_returns_empty_report_for_valid_semantic_file() {
    let output = parse(VALID_CALL_ACTIVITY);
    let file = output.semantic_file.as_ref().expect("semantic file");
    let report = validate(file);

    assert!(!report.has_errors());
    assert!(report.diagnostics.is_empty());
}

#[test]
fn validate_parse_output_surfaces_source_linked_validation_errors() {
    let output = parse(INVALID_VALIDATION_ORPHAN_TASK);

    assert!(output.syntax.diagnostics.is_empty());
    assert!(output.resolver.diagnostics.is_empty());

    let report = validate_parse_output(&output);
    assert!(report.has_errors());

    let diagnostic = report
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.metadata.code == Some("DOGL2207"))
        .expect("missing incoming validation diagnostic");
    let span = diagnostic.span.expect("source span");

    assert_eq!(span.start.line, 5);
    assert_eq!(span.start.column, 17);
    assert!(report
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.metadata.code == Some("DOGL2208")));
}

#[test]
fn validate_for_layout_blocks_layout_when_validation_fails() {
    let output = parse(INVALID_END_EVENT_WITH_OUTGOING_FLOW);

    assert!(output.syntax.diagnostics.is_empty());
    assert!(output.resolver.diagnostics.is_empty());

    let validation = validate_for_layout(&output);
    assert!(!validation.can_run_layout);
    assert!(validation.report.has_errors());
    assert!(validation
        .report
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.metadata.code == Some("DOGL2202")));
}

#[test]
fn validate_for_layout_allows_layout_when_validation_succeeds() {
    let output = parse(VALID_CALL_ACTIVITY);
    let validation = validate_for_layout(&output);

    assert!(validation.can_run_layout);
    assert!(validation.report.diagnostics.is_empty());
}

#[test]
fn validate_parse_output_rejects_component_without_start_and_end_events() {
    let output = parse(INVALID_COMPONENT_WITHOUT_BOUNDARY_EVENTS);

    assert!(output.syntax.diagnostics.is_empty());
    assert!(output.resolver.diagnostics.is_empty());

    let report = validate_parse_output(&output);
    assert!(report.has_errors());
    assert!(report
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.metadata.code == Some("DOGL2210")));
    assert!(report
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.metadata.code == Some("DOGL2211")));
}

#[test]
fn validate_for_layout_allows_multiple_independent_graphs_per_pool() {
    let output = parse(VALID_TWO_GRAPHS_IN_ONE_POOL);

    assert!(output.syntax.diagnostics.is_empty());
    assert!(output.resolver.diagnostics.is_empty());

    let validation = validate_for_layout(&output);
    assert!(validation.can_run_layout);
    assert!(validation.report.diagnostics.is_empty());
}

#[test]
fn layout_parse_output_places_basic_chain_left_to_right() {
    let output = parse(LAYOUT_BASIC_CHAIN);

    let stage = layout_parse_output(&output).expect("layout stage");
    assert!(stage.validation.can_run_layout);
    let file = stage.laid_out_file.expect("laid out file");

    let start = element_bounds(&file, "Start");
    let review = element_bounds(&file, "Review");
    let done = element_bounds(&file, "Done");

    assert!(start.x() < review.x());
    assert!(review.x() < done.x());
    assert_eq!(center_y(&start), center_y(&review));
    assert_eq!(center_y(&review), center_y(&done));
    assert!(pool_bounds(&file).w() > 0.0);
    assert!(lane_bounds(&file, "Ops").h() > 0.0);
}

#[test]
fn layout_parse_output_separates_lanes_vertically() {
    let output = parse(LAYOUT_MULTIPLE_LANES);

    let stage = layout_parse_output(&output).expect("layout stage");
    let file = stage.laid_out_file.expect("laid out file");
    let start_ops = element_bounds(&file, "StartOps");
    let review_sales = element_bounds(&file, "ReviewInSales");
    let ops_lane = lane_bounds(&file, "Ops");
    let sales_lane = lane_bounds(&file, "Sales");

    assert!(review_sales.x() > start_ops.x());
    assert!(sales_lane.y() > ops_lane.y());
    assert!(review_sales.y() >= sales_lane.y());
}

#[test]
fn layout_parse_output_expands_lane_for_multiple_start_rows() {
    let output = parse(LAYOUT_LANE_EXPANSION);

    let stage = layout_parse_output(&output).expect("layout stage");
    let file = stage.laid_out_file.expect("laid out file");
    let start_a = element_bounds(&file, "StartA");
    let start_b = element_bounds(&file, "StartB");
    let lane = lane_bounds(&file, "Ops");

    assert!(start_b.y() > start_a.y());
    assert!(lane.h() > 160.0);
}

#[test]
fn layout_parse_output_applies_gateway_fan_out_rule() {
    let output = parse(LAYOUT_GATEWAY_FANOUT);

    let stage = layout_parse_output(&output).expect("layout stage");
    let file = stage.laid_out_file.expect("laid out file");
    let route = element_bounds(&file, "Route");
    let approve = element_bounds(&file, "Approve");
    let reject = element_bounds(&file, "Reject");

    assert!(approve.x() > route.x());
    assert_eq!(approve.x(), reject.x());
    assert!(reject.y() > approve.y());
}

#[test]
fn layout_parse_output_places_merge_gateway_from_first_source() {
    let output = parse(LAYOUT_GATEWAY_MERGE);

    let stage = layout_parse_output(&output).expect("layout stage");
    let file = stage.laid_out_file.expect("laid out file");
    let start_a = element_bounds(&file, "StartA");
    let start_b = element_bounds(&file, "StartB");
    let merge = element_bounds(&file, "Merge");
    let done = element_bounds(&file, "Done");

    assert!(center_y(&start_b) > center_y(&start_a));
    assert_eq!(center_y(&merge), center_y(&start_a));
    assert!(merge.x() > start_a.x());
    assert!(done.x() > merge.x());
}

#[test]
fn layout_parse_output_keeps_backward_links_from_repositioning_targets() {
    let output = parse(LAYOUT_BACKWARD_CONNECTION);

    let stage = layout_parse_output(&output).expect("layout stage");
    let file = stage.laid_out_file.expect("laid out file");
    let review = element_bounds(&file, "Review");
    let route = element_bounds(&file, "Route");
    let rework = element_bounds(&file, "Rework");

    assert!(route.x() > review.x());
    assert!(rework.x() > route.x());
    assert!(review.x() < rework.x());
}

#[test]
fn apply_layout_and_render_dogl_are_deterministic() {
    let output = parse(LAYOUT_GATEWAY_FANOUT);
    let file = output.semantic_file.as_ref().expect("semantic file");

    let first = apply_layout(file).expect("first layout");
    let second = apply_layout(file).expect("second layout");
    let first_rendered = render_dogl(&first).expect("first render");
    let second_rendered = render_dogl(&second).expect("second render");

    assert_eq!(first, second);
    assert_eq!(first_rendered, second_rendered);
}

#[test]
fn render_dogl_replaces_previous_layout_with_canonical_layout_block() {
    let output = parse(INLINE_LAYOUT_SOURCE);
    let stage = layout_parse_output(&output).expect("layout stage");
    let file = stage.laid_out_file.expect("laid out file");

    let rendered = render_dogl(&file).expect("rendered source");
    assert!(rendered.contains("\nlayout\n"));
    assert!(!rendered.contains("[] Review {180 132 100 52}"));
    assert_eq!(rendered.matches("\nlayout\n").count(), 1);

    let reparsed = parse(&rendered);
    assert!(reparsed.syntax.diagnostics.is_empty());
    assert!(reparsed.resolver.diagnostics.is_empty());
    assert!(reparsed.semantic_file.as_ref().and_then(|file| file.collabs[0].layout.as_ref()).is_some());
}

#[test]
fn render_dogl_preserves_explicit_quoted_display_names() {
    let output = parse(VALID_QUOTED_DISPLAY_NAME_SYNTAX);
    let file = output.semantic_file.as_ref().expect("semantic file");
    let laid_out = apply_layout(file).expect("laid out file");

    let rendered = render_dogl(&laid_out).expect("rendered source");
    assert!(rendered.contains(r#"[] ReviewOrder "Check order" [do] check amount"#));
    assert!(rendered.contains(r#"[call] ChildProcess "Initiation""#));
    assert!(rendered.contains(r#"<x> RouteOrder "Routing""#));

    let reparsed = parse(&rendered);
    assert!(reparsed.syntax.diagnostics.is_empty());
    assert!(reparsed.resolver.diagnostics.is_empty());
    let semantic_file = reparsed.semantic_file.as_ref().expect("semantic file");
    let pool = &semantic_file.collabs[0].pools[0];
    let review = pool
        .quadrants
        .iter()
        .flat_map(|quadrant| quadrant.elements.iter())
        .find_map(|element| match element {
            Element::Task(task) if task.id == "ReviewOrder" => Some(task),
            _ => None,
        })
        .expect("review task");
    assert_eq!(review.name, "Check order");
}

#[test]
fn layout_parse_output_blocks_layout_when_validation_fails() {
    let output = parse(INVALID_VALIDATION_ORPHAN_TASK);

    let stage = layout_parse_output(&output).expect("layout stage");
    assert!(!stage.validation.can_run_layout);
    assert!(stage.laid_out_file.is_none());
}

#[test]
fn export_bpmn_emits_minimal_xml_for_laid_out_file() {
    let output = parse(BOTTOM_LAYOUT_SOURCE);
    let file = output.semantic_file.as_ref().expect("semantic file");
    let export = export_bpmn(file).expect("bpmn export");

    assert!(export.xml.contains(r#"<bpmn:definitions"#));
    assert!(export.xml.contains(r#"<bpmn:collaboration"#));
    assert!(export.xml.contains(r#"<bpmn:process"#));
    assert!(export.xml.contains(r#"<bpmn:laneSet"#));
    assert!(export.xml.contains(r#"<bpmn:sequenceFlow"#));
    assert!(export.xml.contains(r#"<bpmndi:BPMNDiagram"#));
    assert!(export.xml.contains(r#"<dc:Bounds"#));
}

#[test]
fn export_bpmn_emits_call_activity_and_called_element() {
    let output = parse(VALID_CALL_ACTIVITY);
    let file = output.semantic_file.as_ref().expect("semantic file");
    let laid_out = apply_layout(file).expect("laid out file");
    let export = export_bpmn(&laid_out).expect("bpmn export");

    assert!(export.xml.contains(r#"<bpmn:callActivity"#));
    assert!(export.xml.contains(r#"name="ChildProcess""#));
    assert!(!export.xml.contains(r#"name="Child process""#));
    assert!(export
        .xml
        .contains(r#"calledElement="Process_ChildProcess""#));
}

#[test]
fn export_bpmn_uses_explicit_display_names_in_xml() {
    let output = parse(VALID_QUOTED_DISPLAY_NAME_SYNTAX);
    let file = output.semantic_file.as_ref().expect("semantic file");
    let laid_out = apply_layout(file).expect("laid out file");
    let export = export_bpmn(&laid_out).expect("bpmn export");

    assert!(export.xml.contains(r#"name="Begin order""#));
    assert!(export.xml.contains(r#"name="Check order""#));
    assert!(export.xml.contains(r#"name="Routing""#));
    assert!(export.xml.contains(r#"name="Initiation""#));
    assert!(export
        .xml
        .contains(r#"calledElement="Process_ChildProcess""#));
}

#[test]
fn export_bpmn_fails_fast_when_layout_is_missing() {
    let output = parse(VALID_CALL_ACTIVITY);
    let file = output.semantic_file.as_ref().expect("semantic file");

    assert!(matches!(
        export_bpmn(file),
        Err(ApplicationError::Serialize(message))
            if message.contains("requires an existing layout")
    ));
}

#[test]
fn json_facade_is_explicitly_not_implemented_yet() {
    let file = DoglFile::new(vec![]);

    assert_eq!(
        to_json(&file),
        Err(ApplicationError::NotImplemented("to_json"))
    );
}

#[test]
fn syntax_contracts_reserve_source_fidelity_and_recovery_shapes() {
    let span = Span::new(
        SourcePosition {
            offset: 0,
            line: 1,
            column: 1,
        },
        SourcePosition {
            offset: 6,
            line: 1,
            column: 7,
        },
    );

    let comment = SyntaxTrivia::new(SyntaxTriviaKind::Comment, span, "// note");
    let whitespace = SyntaxTrivia::new(SyntaxTriviaKind::Whitespace, span, " ");
    let token = SyntaxToken::new(TokenKind::KeywordCollab, span, "collab")
        .with_leading_trivia(TriviaRange::new(0, 1))
        .with_trailing_trivia(TriviaRange::new(1, 2));
    let unresolved = UnresolvedName::new(UnresolvedNameKind::Reference, "Example").with_span(span);
    let recovery = RecoveryNode::new(RecoveryKind::IncompleteNode, "placeholder recovery")
        .with_span(span);
    let diagnostic = ParseDiagnostic::new(ParseDiagnosticSeverity::Warning, "placeholder")
        .with_code("DOGL0001")
        .mark_recovered()
        .with_related_span(span)
        .with_span(span);
    let node = SyntaxNode::new(SyntaxKind::Collab)
        .with_span(span)
        .with_token_range(TokenRange::new(0, 1))
        .with_text_name("Example")
        .with_display_name("Example label")
        .with_children(vec![SyntaxNodeId(1)])
        .mark_recovered();
    let document = SyntaxDocument {
        source: dogl_language::syntax::SourceFile::new("collab Example"),
        tokens: vec![token.clone()],
        nodes: vec![node.clone()],
        root: Some(SyntaxNodeId(0)),
        trivia: vec![comment.clone(), whitespace],
        unresolved_names: vec![unresolved.clone()],
        recoveries: vec![recovery.clone()],
        diagnostics: vec![diagnostic.clone()],
    };

    assert_eq!(token.kind, TokenKind::KeywordCollab);
    assert_eq!(token.leading_trivia, TriviaRange::new(0, 1));
    assert_eq!(comment.kind, SyntaxTriviaKind::Comment);
    assert_eq!(comment.text, "// note");
    assert_eq!(unresolved.kind, UnresolvedNameKind::Reference);
    assert_eq!(recovery.kind, RecoveryKind::IncompleteNode);
    assert_eq!(diagnostic.metadata.code, Some("DOGL0001"));
    assert!(diagnostic.metadata.recovered);
    assert_eq!(diagnostic.metadata.related_spans, vec![span]);
    assert_eq!(document.root_node(), Some(&node));
    assert_eq!(document.node(SyntaxNodeId(0)), Some(&node));
    assert_eq!(node.display_name.as_deref(), Some("Example label"));
    assert_eq!(document.comments().count(), 1);
    assert_eq!(document.trivia_slice(token.leading_trivia), Some(&document.trivia[0..1]));
}

fn element_bounds(file: &DoglFile, element_id: &str) -> Bounds {
    let pool = &file.collabs[0].pools[0];
    let element = pool
        .quadrants
        .iter()
        .flat_map(|quadrant| quadrant.elements.iter())
        .find(|element| element.id() == element_id)
        .expect("element");
    file.collabs[0]
        .layout
        .as_ref()
        .and_then(|layout| layout.get(element.uid()))
        .cloned()
        .expect("element bounds")
}

fn lane_bounds(file: &DoglFile, lane_id: &str) -> Bounds {
    let lane = file.collabs[0].pools[0]
        .lanes
        .iter()
        .find(|lane| lane.id == lane_id)
        .expect("lane");
    file.collabs[0]
        .layout
        .as_ref()
        .and_then(|layout| layout.get(lane.uid))
        .cloned()
        .expect("lane bounds")
}

fn pool_bounds(file: &DoglFile) -> Bounds {
    let pool = &file.collabs[0].pools[0];
    file.collabs[0]
        .layout
        .as_ref()
        .and_then(|layout| layout.get(pool.uid))
        .cloned()
        .expect("pool bounds")
}

fn center_y(bounds: &Bounds) -> i64 {
    ((bounds.y() + bounds.h() / 2.0) * 100.0).round() as i64
}
