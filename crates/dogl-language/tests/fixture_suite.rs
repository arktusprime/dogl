use dogl_language::{
    domain::{Element, GatewayCode, TaskCode},
    parse,
};

const CALL_ACTIVITY_HAPPY_PATH: &str = include_str!("fixtures/call_activity_happy_path.dogl");
const GENERIC_TASK_HAPPY_PATH: &str = include_str!("fixtures/generic_task_happy_path.dogl");
const CALL_ACTIVITY_COMPACT_SYNTAX: &str =
    include_str!("fixtures/call_activity_expression_block.dogl");
const DO_INLINE_COMMAND: &str = include_str!("fixtures/do_inline_command.dogl");
const DO_EXEC_COMMAND_BLOCK: &str = include_str!("fixtures/do_exec_command_block.dogl");
const GATEWAY_DMN_COMMAND: &str = include_str!("fixtures/gateway_dmn_command.dogl");
const GATEWAY_HAPPY_PATH: &str = include_str!("fixtures/gateway_happy_path.dogl");
const COMMENTS_SUPPORTED: &str = include_str!("fixtures/comments_supported.dogl");
const INLINE_BOUNDS_LAYOUT: &str = include_str!("fixtures/inline_bounds_layout.dogl");
const BOTTOM_LAYOUT_BLOCK: &str = include_str!("fixtures/bottom_layout_block.dogl");
const BAD_INDENTATION: &str = include_str!("fixtures/bad_indentation.dogl");
const UNKNOWN_FLOW_TARGET: &str = include_str!("fixtures/unknown_flow_target.dogl");
const MISSING_CALL_TARGET: &str = include_str!("fixtures/missing_call_expression.dogl");
const LEGACY_AT_COMMAND: &str = include_str!("fixtures/legacy_at_command.dogl");
const BARE_TASK_WITHOUT_CODE: &str = include_str!("fixtures/bare_task_without_code.dogl");
const DUPLICATE_ELEMENT_IDS: &str = include_str!("fixtures/duplicate_element_ids.dogl");
const PARENT_FIXTURE: &str = include_str!("fixtures/parent.dogl");
const CHILD_FIXTURE: &str = include_str!("fixtures/child.dogl");

#[test]
fn call_activity_happy_path_fixture_lowers_to_semantic_domain() {
    let output = parse(CALL_ACTIVITY_HAPPY_PATH);

    assert!(output.syntax.diagnostics.is_empty());
    assert!(output.resolver.diagnostics.is_empty());
    let file = output.semantic_file.expect("semantic file");
    assert_eq!(file.collabs.len(), 1);

    let pool = &file.collabs[0].pools[0];
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
fn generic_task_fixture_parses_without_call_activity_semantics() {
    let output = parse(GENERIC_TASK_HAPPY_PATH);

    assert!(output.syntax.diagnostics.is_empty());
    assert!(output.resolver.diagnostics.is_empty());
    let file = output.semantic_file.expect("semantic file");
    let tasks: Vec<_> = file.collabs[0].pools[0]
        .quadrants
        .iter()
        .flat_map(|quadrant| quadrant.elements.iter())
        .filter_map(|element| match element {
            Element::Task(task) => Some(task),
            _ => None,
        })
        .collect();

    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].id, "Review");
    assert_eq!(tasks[0].code, TaskCode::Generic);
    assert!(tasks[0].expressions.is_empty());
}

#[test]
fn compact_call_activity_syntax_fixture_is_supported() {
    let output = parse(CALL_ACTIVITY_COMPACT_SYNTAX);

    assert!(output.syntax.diagnostics.is_empty());
    assert!(output.resolver.diagnostics.is_empty());
    let file = output.semantic_file.expect("semantic file");
    let call_task = file.collabs[0].pools[0]
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
}

#[test]
fn do_inline_command_fixture_is_supported() {
    let output = parse(DO_INLINE_COMMAND);

    assert!(output.syntax.diagnostics.is_empty());
    assert!(output.resolver.diagnostics.is_empty());
    let file = output.semantic_file.expect("semantic file");
    let task = file.collabs[0].pools[0]
        .quadrants
        .iter()
        .flat_map(|quadrant| quadrant.elements.iter())
        .find_map(|element| match element {
            Element::Task(task) if task.id == "ReviewOrder" => Some(task),
            _ => None,
        })
        .expect("task");

    assert_eq!(task.code, TaskCode::Generic);
    assert_eq!(task.expressions.len(), 1);
    assert_eq!(task.expressions[0].key, "do");
    assert_eq!(task.expressions[0].value, "check amount");
}

#[test]
fn do_exec_command_block_fixture_is_supported() {
    let output = parse(DO_EXEC_COMMAND_BLOCK);

    assert!(output.syntax.diagnostics.is_empty());
    assert!(output.resolver.diagnostics.is_empty());
    let file = output.semantic_file.expect("semantic file");
    let task = file.collabs[0].pools[0]
        .quadrants
        .iter()
        .flat_map(|quadrant| quadrant.elements.iter())
        .find_map(|element| match element {
            Element::Task(task) if task.id == "ReviewOrder" => Some(task),
            _ => None,
        })
        .expect("task");

    assert_eq!(task.expressions.len(), 1);
    assert_eq!(task.expressions[0].key, "do.exec");
    assert_eq!(task.expressions[0].value, "validateOrder(order.id)");
}

#[test]
fn gateway_dmn_command_fixture_is_supported() {
    let output = parse(GATEWAY_DMN_COMMAND);

    assert!(output.syntax.diagnostics.is_empty());
    assert!(output.resolver.diagnostics.is_empty());
    let file = output.semantic_file.expect("semantic file");
    let gateway = file.collabs[0].pools[0]
        .quadrants
        .iter()
        .flat_map(|quadrant| quadrant.elements.iter())
        .find_map(|element| match element {
            Element::Gateway(gateway) if gateway.id == "RouteOrder" => Some(gateway),
            _ => None,
        })
        .expect("gateway");

    assert_eq!(gateway.dmn_ref.as_deref(), Some("OrderRouting"));
    assert_eq!(gateway.expressions.len(), 1);
    assert_eq!(gateway.expressions[0].key, "dmn");
    assert_eq!(gateway.expressions[0].value, "OrderRouting");
}

#[test]
fn gateway_happy_path_fixture_lowers_supported_gateway_forms() {
    let output = parse(GATEWAY_HAPPY_PATH);

    assert!(output.syntax.diagnostics.is_empty());
    assert!(output.resolver.diagnostics.is_empty());
    let file = output.semantic_file.expect("semantic file");
    let gateways: Vec<_> = file.collabs[0].pools[0]
        .quadrants
        .iter()
        .flat_map(|quadrant| quadrant.elements.iter())
        .filter_map(|element| match element {
            Element::Gateway(gateway) => Some(gateway),
            _ => None,
        })
        .collect();

    assert_eq!(gateways.len(), 3);
    assert_eq!(gateways[0].id, "RouteInclusive");
    assert_eq!(gateways[0].code, GatewayCode::Inclusive);
    assert_eq!(gateways[1].id, "RouteExclusive");
    assert_eq!(gateways[1].code, GatewayCode::Exclusive);
    assert_eq!(gateways[2].id, "RouteParallel");
    assert_eq!(gateways[2].code, GatewayCode::Parallel);
}

#[test]
fn slash_comments_fixture_is_accepted_as_trivia() {
    let output = parse(COMMENTS_SUPPORTED);

    assert!(output.syntax.diagnostics.is_empty());
    assert!(output.resolver.diagnostics.is_empty());
    assert!(output.semantic_file.is_some());
    assert!(output.syntax.comments().count() >= 4);
}

#[test]
fn inline_bounds_fixture_populates_semantic_layout() {
    let output = parse(INLINE_BOUNDS_LAYOUT);

    assert!(output.syntax.diagnostics.is_empty());
    assert!(output.resolver.diagnostics.is_empty());
    let file = output.semantic_file.expect("semantic file");
    assert!(file.collabs[0].layout.is_some());
}

#[test]
fn bottom_layout_block_fixture_populates_semantic_layout() {
    let output = parse(BOTTOM_LAYOUT_BLOCK);

    assert!(output.syntax.diagnostics.is_empty());
    assert!(output.resolver.diagnostics.is_empty());
    let file = output.semantic_file.expect("semantic file");
    assert!(file.collabs[0].layout.is_some());
}

#[test]
fn inline_and_bottom_layout_modes_lower_to_same_layout() {
    let inline = parse(INLINE_BOUNDS_LAYOUT);
    let bottom = parse(BOTTOM_LAYOUT_BLOCK);

    assert!(inline.syntax.diagnostics.is_empty());
    assert!(inline.resolver.diagnostics.is_empty());
    assert!(bottom.syntax.diagnostics.is_empty());
    assert!(bottom.resolver.diagnostics.is_empty());

    let inline_file = inline.semantic_file.expect("inline semantic file");
    let bottom_file = bottom.semantic_file.expect("bottom semantic file");
    assert_eq!(inline_file.collabs[0].layout, bottom_file.collabs[0].layout);
}

#[test]
fn bad_indentation_fixture_reports_syntax_errors() {
    let output = parse(BAD_INDENTATION);

    assert!(!output.syntax.diagnostics.is_empty());
    assert!(output.semantic_file.is_none());
}

#[test]
fn unknown_flow_target_fixture_reports_resolver_errors() {
    let output = parse(UNKNOWN_FLOW_TARGET);

    assert!(output.syntax.diagnostics.is_empty());
    assert!(!output.resolver.diagnostics.is_empty());
    assert!(output.semantic_file.is_none());
}

#[test]
fn missing_call_target_fixture_reports_syntax_errors() {
    let output = parse(MISSING_CALL_TARGET);

    assert!(!output.syntax.diagnostics.is_empty());
    assert!(output.semantic_file.is_none());
}

#[test]
fn legacy_at_command_fixture_reports_syntax_errors() {
    let output = parse(LEGACY_AT_COMMAND);

    assert!(!output.syntax.diagnostics.is_empty());
    assert!(output.semantic_file.is_none());
}

#[test]
fn bare_task_without_code_fixture_reports_syntax_errors() {
    let output = parse(BARE_TASK_WITHOUT_CODE);

    assert!(!output.syntax.diagnostics.is_empty());
    assert!(output.semantic_file.is_none());
}

#[test]
fn duplicate_element_ids_fixture_reports_resolver_errors() {
    let output = parse(DUPLICATE_ELEMENT_IDS);

    assert!(output.syntax.diagnostics.is_empty());
    assert!(!output.resolver.diagnostics.is_empty());
    assert!(output.semantic_file.is_none());
}

#[test]
fn parent_and_child_fixtures_parse_independently_for_next_phase() {
    let parent = parse(PARENT_FIXTURE);
    let child = parse(CHILD_FIXTURE);

    assert!(parent.syntax.diagnostics.is_empty());
    assert!(parent.resolver.diagnostics.is_empty());
    assert!(child.syntax.diagnostics.is_empty());
    assert!(child.resolver.diagnostics.is_empty());

    let parent_file = parent.semantic_file.expect("parent semantic file");
    let child_file = child.semantic_file.expect("child semantic file");
    assert_eq!(parent_file.collabs[0].id, "ParentProcess");
    assert_eq!(child_file.collabs[0].id, "ChildProcess");

    let parent_call = parent_file.collabs[0].pools[0]
        .quadrants
        .iter()
        .flat_map(|quadrant| quadrant.elements.iter())
        .find_map(|element| match element {
            Element::Task(task) if task.id == "ChildProcess" => Some(task),
            _ => None,
        })
        .expect("parent call task");

    assert_eq!(parent_call.code, TaskCode::CallActivity);
    assert_eq!(parent_call.call_target.as_deref(), Some(child_file.collabs[0].id.as_str()));
}
