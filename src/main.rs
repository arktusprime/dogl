use std::{
    env, fs,
    path::{Path, PathBuf},
};

use dogl_language::{export_bpmn, layout_parse_output, parse, render_dogl, validate_for_layout};

const MAX_REPORTED_DIAGNOSTICS: usize = 50;

fn main() {
    if let Err(message) = run() {
        eprintln!("{message}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.len() != 2 {
        return Err(usage().to_string());
    }

    let path = Path::new(&args[1]);
    match args[0].as_str() {
        "layout" => run_layout(path),
        "export-bpmn" => run_export_bpmn(path),
        "build-dir" => run_build_dir(path),
        _ => Err(usage().to_string()),
    }
}

fn run_layout(path: &Path) -> Result<(), String> {
    let result = process_dogl_file(path)?;
    fs::write(path, result.rendered_dogl)
        .map_err(|err| format!("failed to write `{}`: {err}", path.display()))?;
    println!("Layout written to `{}`", path.display());
    Ok(())
}

fn run_export_bpmn(path: &Path) -> Result<(), String> {
    let output = parse_source_file(path)?;
    let validation = validate_for_layout(&output);
    if !validation.can_run_layout {
        return Err(format_validation_diagnostics(&validation.report.diagnostics));
    }

    let file = output
        .semantic_file
        .as_ref()
        .ok_or_else(|| "parse did not produce a semantic file".to_string())?;
    let export = export_bpmn(file).map_err(render_application_error)?;
    let output_path = path.with_extension("bpmn");
    fs::write(&output_path, export.xml)
        .map_err(|err| format!("failed to write `{}`: {err}", output_path.display()))?;
    println!("BPMN written to `{}`", output_path.display());
    Ok(())
}

fn run_build_dir(path: &Path) -> Result<(), String> {
    if !path.is_dir() {
        return Err(format!("`{}` is not a directory", path.display()));
    }

    let files = collect_dogl_files(path)?;
    if files.is_empty() {
        return Err(format!("no `.dogl` files found under `{}`", path.display()));
    }

    for file in files {
        let result =
            process_dogl_file(&file).map_err(|message| format!("{}: {message}", file.display()))?;
        fs::write(&file, result.rendered_dogl)
            .map_err(|err| format!("failed to write `{}`: {err}", file.display()))?;

        let output_path = file.with_extension("bpmn");
        fs::write(&output_path, result.bpmn_xml)
            .map_err(|err| format!("failed to write `{}`: {err}", output_path.display()))?;

        println!(
            "Processed `{}` and wrote `{}`",
            file.display(),
            output_path.display()
        );
    }

    Ok(())
}

fn collect_dogl_files(path: &Path) -> Result<Vec<PathBuf>, String> {
    let mut files = Vec::new();
    collect_dogl_files_recursive(path, &mut files)?;
    files.sort();
    Ok(files)
}

fn collect_dogl_files_recursive(path: &Path, files: &mut Vec<PathBuf>) -> Result<(), String> {
    for entry in fs::read_dir(path)
        .map_err(|err| format!("failed to read directory `{}`: {err}", path.display()))?
    {
        let entry =
            entry.map_err(|err| format!("failed to read entry in `{}`: {err}", path.display()))?;
        let entry_path = entry.path();
        let metadata = entry
            .metadata()
            .map_err(|err| format!("failed to read metadata for `{}`: {err}", entry_path.display()))?;
        if metadata.is_dir() {
            collect_dogl_files_recursive(&entry_path, files)?;
        } else if entry_path.extension().is_some_and(|ext| ext == "dogl") {
            files.push(entry_path);
        }
    }
    Ok(())
}

fn process_dogl_file(path: &Path) -> Result<ProcessedDoglFile, String> {
    let output = parse_source_file(path)?;
    let stage = layout_parse_output(&output).map_err(render_application_error)?;

    if !stage.validation.can_run_layout {
        return Err(format_validation_diagnostics(&stage.validation.report.diagnostics));
    }

    let laid_out_file = stage
        .laid_out_file
        .ok_or_else(|| "layout stage did not produce a semantic file".to_string())?;
    let rendered_dogl = render_dogl(&laid_out_file).map_err(render_application_error)?;
    let bpmn_xml = export_bpmn(&laid_out_file)
        .map_err(render_application_error)?
        .xml;

    Ok(ProcessedDoglFile {
        rendered_dogl,
        bpmn_xml,
    })
}

fn parse_source_file(path: &Path) -> Result<dogl_language::ParseOutput, String> {
    let source = fs::read_to_string(path)
        .map_err(|err| format!("failed to read `{}`: {err}", path.display()))?;

    let output = parse(&source);
    if !output.syntax.diagnostics.is_empty() {
        return Err(format_diagnostics(
            "syntax diagnostics:",
            output
                .syntax
                .diagnostics
                .iter()
                .map(|diagnostic| diagnostic.message.as_str()),
        ));
    }
    if !output.resolver.diagnostics.is_empty() {
        return Err(format_diagnostics(
            "resolver diagnostics:",
            output
                .resolver
                .diagnostics
                .iter()
                .map(|diagnostic| diagnostic.message.as_str()),
        ));
    }
    Ok(output)
}

fn format_validation_diagnostics(
    diagnostics: &[dogl_language::validation::ValidationDiagnostic],
) -> String {
    format_diagnostics(
        "validation diagnostics:",
        diagnostics.iter().map(|diagnostic| diagnostic.message.as_str()),
    )
}

fn render_application_error(err: dogl_language::ApplicationError) -> String {
    match err {
        dogl_language::ApplicationError::NotImplemented(name) => {
            format!("application surface `{name}` is not implemented")
        }
        dogl_language::ApplicationError::Layout(message) => message,
        dogl_language::ApplicationError::Serialize(message) => message,
    }
}

fn usage() -> &'static str {
    "usage: dogl <layout|export-bpmn|build-dir> <path-to-file.dogl|path-to-directory>"
}

struct ProcessedDoglFile {
    rendered_dogl: String,
    bpmn_xml: String,
}

fn format_diagnostics<'a>(
    header: &str,
    messages: impl Iterator<Item = &'a str>,
) -> String {
    let collected: Vec<_> = messages.collect();
    let total = collected.len();
    let mut lines = vec![header.to_string()];
    for message in collected.iter().take(MAX_REPORTED_DIAGNOSTICS) {
        lines.push(format!("- {message}"));
    }
    if total > MAX_REPORTED_DIAGNOSTICS {
        lines.push(format!(
            "- ... {} more diagnostics omitted",
            total - MAX_REPORTED_DIAGNOSTICS
        ));
    }
    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::format_diagnostics;

    #[test]
    fn diagnostic_output_is_capped() {
        let messages: Vec<String> = (0..60).map(|index| format!("problem {index}")).collect();
        let rendered = format_diagnostics("syntax diagnostics:", messages.iter().map(|s| s.as_str()));

        assert!(rendered.contains("syntax diagnostics:"));
        assert!(rendered.contains("problem 0"));
        assert!(rendered.contains("problem 49"));
        assert!(!rendered.contains("problem 59"));
        assert!(rendered.contains("10 more diagnostics omitted"));
    }
}
