use std::{env, fs, path::Path};

use dogl_language::{layout_parse_output, parse, render_dogl};

fn main() {
    if let Err(message) = run() {
        eprintln!("{message}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.len() != 2 || args[0] != "layout" {
        return Err("usage: dogl layout <path-to-file.dogl>".to_string());
    }

    let path = Path::new(&args[1]);
    let source = fs::read_to_string(path)
        .map_err(|err| format!("failed to read `{}`: {err}", path.display()))?;

    let output = parse(&source);
    if !output.syntax.diagnostics.is_empty() {
        let mut lines = vec!["syntax diagnostics:".to_string()];
        for diagnostic in &output.syntax.diagnostics {
            lines.push(format!("- {}", diagnostic.message));
        }
        return Err(lines.join("\n"));
    }
    if !output.resolver.diagnostics.is_empty() {
        let mut lines = vec!["resolver diagnostics:".to_string()];
        for diagnostic in &output.resolver.diagnostics {
            lines.push(format!("- {}", diagnostic.message));
        }
        return Err(lines.join("\n"));
    }

    let stage = layout_parse_output(&output).map_err(|err| match err {
        dogl_language::ApplicationError::NotImplemented(name) => {
            format!("application surface `{name}` is not implemented")
        }
        dogl_language::ApplicationError::Layout(message) => message,
        dogl_language::ApplicationError::Serialize(message) => message,
    })?;

    if !stage.validation.can_run_layout {
        let mut lines = vec!["validation diagnostics:".to_string()];
        for diagnostic in &stage.validation.report.diagnostics {
            lines.push(format!("- {}", diagnostic.message));
        }
        return Err(lines.join("\n"));
    }

    let laid_out_file = stage
        .laid_out_file
        .ok_or_else(|| "layout stage did not produce a semantic file".to_string())?;
    let rendered = render_dogl(&laid_out_file).map_err(|err| match err {
        dogl_language::ApplicationError::NotImplemented(name) => {
            format!("application surface `{name}` is not implemented")
        }
        dogl_language::ApplicationError::Layout(message) => message,
        dogl_language::ApplicationError::Serialize(message) => message,
    })?;

    fs::write(path, rendered)
        .map_err(|err| format!("failed to write `{}`: {err}", path.display()))?;
    println!("Layout written to `{}`", path.display());
    Ok(())
}
