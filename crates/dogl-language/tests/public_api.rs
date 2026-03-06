use dogl_language::{
    export_bpmn, import_bpmn, parse, to_json, validate, ApplicationError, domain::DoglFile,
    syntax::SyntaxKind,
};

#[test]
fn parse_facade_returns_placeholder_layers() {
    let output = parse("collab Example");

    assert_eq!(output.syntax.source.text, "collab Example");
    assert!(output.syntax.nodes.is_empty());
    assert!(output.syntax.diagnostics.is_empty());
    assert!(output.semantic_file.is_none());
}

#[test]
fn import_bpmn_reuses_parse_placeholder_shape() {
    let output = import_bpmn("<definitions />");

    assert_eq!(output.syntax.source.text, "<definitions />");
    assert_eq!(SyntaxKind::File, SyntaxKind::File);
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
