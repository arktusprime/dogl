use dogl_language::{
    export_bpmn, import_bpmn, parse, to_json, validate, ApplicationError, domain::DoglFile,
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

#[test]
fn parse_facade_returns_placeholder_layers() {
    let output = parse("collab Example");

    assert_eq!(output.syntax.source.text, "collab Example");
    assert!(output.syntax.tokens.is_empty());
    assert!(output.syntax.nodes.is_empty());
    assert!(output.syntax.trivia.is_empty());
    assert!(output.syntax.unresolved_names.is_empty());
    assert!(output.syntax.recoveries.is_empty());
    assert!(output.syntax.diagnostics.is_empty());
    assert!(output.semantic_file.is_none());
}

#[test]
fn import_bpmn_reuses_parse_placeholder_shape() {
    let output = import_bpmn("<definitions />");

    assert_eq!(output.syntax.source.text, "<definitions />");
    assert!(output.syntax.tokens.is_empty());
    assert!(output.syntax.nodes.is_empty());
    assert!(output.syntax.root.is_none());
    assert!(output.semantic_file.is_none());
}

#[test]
fn parse_facade_keeps_resolver_stages_explicit() {
    let output = parse("collab Example");

    assert_eq!(output.resolver, ResolverOutput::default());
    assert_eq!(output.resolver.bindings, BindingSummary::default());
    assert_eq!(output.resolver.resolution, NameResolutionSummary::default());
    assert!(output.resolver.normalization_passes.is_empty());
    assert_eq!(output.resolver.lowering, LoweredSemanticFile::default());
    assert!(output.resolver.lowering.semantic_file.is_none());
    assert!(output.resolver.diagnostics.is_empty());
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
fn validation_facade_returns_empty_report_for_placeholder_domain() {
    let file = DoglFile::new(vec![]);
    let report = validate(&file);

    assert!(report.diagnostics.is_empty());
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

    let comment = SyntaxTrivia::new(SyntaxTriviaKind::Comment, span, "# note");
    let whitespace = SyntaxTrivia::new(SyntaxTriviaKind::Whitespace, span, " ");
    let token = SyntaxToken::new(TokenKind::Keyword, span, "collab")
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
    let node = SyntaxNode::new(SyntaxKind::Collaboration)
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

    assert_eq!(token.kind, TokenKind::Keyword);
    assert_eq!(token.leading_trivia, TriviaRange::new(0, 1));
    assert_eq!(comment.kind, SyntaxTriviaKind::Comment);
    assert_eq!(comment.text, "# note");
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
