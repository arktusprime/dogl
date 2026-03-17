use dogl_language::{
    export_bpmn, import_bpmn, parse, to_json, validate, validate_for_layout,
    validate_parse_output, ApplicationError, domain::DoglFile,
    domain::{Element, GatewayCode, TaskCode},
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
fn import_bpmn_reuses_parse_contract_and_exposes_syntax_errors() {
    let output = import_bpmn("<definitions />");

    assert_eq!(output.syntax.source.text, "<definitions />");
    assert!(!output.syntax.tokens.is_empty());
    assert!(output.semantic_file.is_none());
    assert!(!output.syntax.diagnostics.is_empty());
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
fn export_and_json_facades_are_explicitly_not_implemented_yet() {
    let file = DoglFile::new(vec![]);

    assert_eq!(
        to_json(&file),
        Err(ApplicationError::NotImplemented("to_json"))
    );
    assert_eq!(
        export_bpmn(&file),
        Err(ApplicationError::NotImplemented("export_bpmn"))
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
    assert_eq!(document.comments().count(), 1);
    assert_eq!(document.trivia_slice(token.leading_trivia), Some(&document.trivia[0..1]));
}
