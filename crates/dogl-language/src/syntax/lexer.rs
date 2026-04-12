use super::{
    diagnostics::{ParseDiagnostic, ParseDiagnosticSeverity},
    nodes::SyntaxDocument,
    source::{SourceFile, SourcePosition, Span},
    tokens::{SyntaxToken, SyntaxTrivia, SyntaxTriviaKind, TokenKind},
};

pub fn lex(source: &str) -> SyntaxDocument {
    let mut lexer = Lexer::new(source);
    lexer.lex();
    lexer.document
}

struct Lexer<'a> {
    source: &'a str,
    document: SyntaxDocument,
    indent_stack: Vec<usize>,
}

impl<'a> Lexer<'a> {
    fn new(source: &'a str) -> Self {
        Self {
            source,
            document: SyntaxDocument::new(SourceFile::new(source)),
            indent_stack: vec![0],
        }
    }

    fn lex(&mut self) {
        let mut offset = 0usize;

        for (line_idx, raw_line) in self.source.split_inclusive('\n').enumerate() {
            let line_number = line_idx + 1;
            let line_without_newline = raw_line.trim_end_matches(['\r', '\n']);
            let line_len = line_without_newline.len();
            let indent = self.count_indent(line_without_newline, line_number, offset);
            let content = &line_without_newline[indent..];

            if content.trim().is_empty() {
                self.push_newline(line_number, offset + line_len, raw_line.ends_with('\n'));
                offset += raw_line.len();
                continue;
            }

            if content.trim_start().starts_with("//") {
                let comment_start = indent + (content.len() - content.trim_start().len());
                self.push_trivia(
                    SyntaxTriviaKind::Comment,
                    line_number,
                    offset + comment_start,
                    offset + line_len,
                    &line_without_newline[comment_start..],
                );
                self.push_newline(line_number, offset + line_len, raw_line.ends_with('\n'));
                offset += raw_line.len();
                continue;
            }

            self.adjust_indent(indent, line_number, offset);
            self.lex_line_content(line_without_newline, line_number, offset, indent);
            self.push_newline(line_number, offset + line_len, raw_line.ends_with('\n'));
            offset += raw_line.len();
        }

        while self.indent_stack.len() > 1 {
            self.indent_stack.pop();
            let position = self.position_for_offset(self.source.len());
            self.document
                .push_token(SyntaxToken::new(TokenKind::Dedent, Span::new(position, position), ""));
        }

        let position = self.position_for_offset(self.source.len());
        self.document
            .push_token(SyntaxToken::new(TokenKind::Eof, Span::new(position, position), ""));
    }

    fn count_indent(&mut self, line: &str, line_number: usize, line_offset: usize) -> usize {
        let mut indent = 0usize;

        for (idx, ch) in line.char_indices() {
            match ch {
                ' ' => indent += 1,
                '\t' => {
                    self.document.diagnostics.push(
                        ParseDiagnostic::new(
                            ParseDiagnosticSeverity::Error,
                            "Tabs are not supported for indentation",
                        )
                        .with_span(self.span(line_number, line_offset + idx, line_offset + idx + 1))
                        .with_code("DOGL1001"),
                    );
                }
                _ => break,
            }
        }

        indent
    }

    fn adjust_indent(&mut self, indent: usize, line_number: usize, line_offset: usize) {
        let current = *self.indent_stack.last().unwrap_or(&0);
        if indent > current {
            self.indent_stack.push(indent);
            let span = self.span(line_number, line_offset, line_offset + indent);
            self.document
                .push_token(SyntaxToken::new(TokenKind::Indent, span, ""));
            return;
        }

        if indent == current {
            return;
        }

        while self.indent_stack.len() > 1 && indent < *self.indent_stack.last().unwrap() {
            self.indent_stack.pop();
            let span = self.span(line_number, line_offset, line_offset + indent);
            self.document
                .push_token(SyntaxToken::new(TokenKind::Dedent, span, ""));
        }

        let new_current = *self.indent_stack.last().unwrap_or(&0);
        if indent != new_current {
            self.document.diagnostics.push(
                ParseDiagnostic::new(
                    ParseDiagnosticSeverity::Error,
                    "Indentation does not match any previous block level",
                )
                .with_span(self.span(line_number, line_offset, line_offset + indent))
                .with_code("DOGL1002"),
            );
            self.indent_stack.push(indent);
            let span = self.span(line_number, line_offset, line_offset + indent);
            self.document
                .push_token(SyntaxToken::new(TokenKind::Indent, span, ""));
        }
    }

    fn lex_line_content(
        &mut self,
        line: &str,
        line_number: usize,
        line_offset: usize,
        indent: usize,
    ) {
        let bytes = line.as_bytes();
        let mut idx = indent;
        while idx < line.len() {
            let ch = line[idx..].chars().next().unwrap();
            match ch {
                ' ' => {
                    idx += 1;
                }
                '/' if line[idx..].starts_with("//") => {
                    self.push_trivia(
                        SyntaxTriviaKind::Comment,
                        line_number,
                        line_offset + idx,
                        line_offset + line.len(),
                        &line[idx..],
                    );
                    break;
                }
                '=' if line[idx..].starts_with("==") => {
                    self.push_token(TokenKind::PoolMarker, line_number, line_offset + idx, 2, "==");
                    idx += 2;
                }
                '-' if line[idx..].starts_with("--") => {
                    self.push_token(TokenKind::LaneMarker, line_number, line_offset + idx, 2, "--");
                    idx += 2;
                }
                '|' if line[idx..].starts_with("||") => {
                    self.push_token(TokenKind::StageMarker, line_number, line_offset + idx, 2, "||");
                    idx += 2;
                }
                '<' if line[idx..].starts_with("<eb>") => {
                    self.push_token(
                        TokenKind::GatewayEventBasedMarker,
                        line_number,
                        line_offset + idx,
                        4,
                        "<eb>",
                    );
                    idx += 4;
                }
                '<' if line[idx..].starts_with("<x>") => {
                    self.push_token(
                        TokenKind::GatewayExclusiveMarker,
                        line_number,
                        line_offset + idx,
                        3,
                        "<x>",
                    );
                    idx += 3;
                }
                '<' if line[idx..].starts_with("<p>") => {
                    self.push_token(
                        TokenKind::GatewayParallelMarker,
                        line_number,
                        line_offset + idx,
                        3,
                        "<p>",
                    );
                    idx += 3;
                }
                '<' if line[idx..].starts_with("<c>") => {
                    self.push_token(
                        TokenKind::GatewayComplexMarker,
                        line_number,
                        line_offset + idx,
                        3,
                        "<c>",
                    );
                    idx += 3;
                }
                '<' if line[idx..].starts_with("<i>") => {
                    self.push_token(
                        TokenKind::GatewayInclusiveMarker,
                        line_number,
                        line_offset + idx,
                        3,
                        "<i>",
                    );
                    idx += 3;
                }
                '<' if line[idx..].starts_with("<>") => {
                    self.push_token(
                        TokenKind::GatewayMarker,
                        line_number,
                        line_offset + idx,
                        2,
                        "<>",
                    );
                    idx += 2;
                }
                '=' if line[idx..].starts_with("=>") => {
                    self.push_token(TokenKind::FlowArrow, line_number, line_offset + idx, 2, "=>");
                    idx += 2;
                }
                '-' if line[idx..].starts_with("->") => {
                    self.push_token(TokenKind::FlowArrow, line_number, line_offset + idx, 2, "->");
                    idx += 2;
                }
                '~' if line[idx..].starts_with("~>") => {
                    self.push_token(TokenKind::FlowArrow, line_number, line_offset + idx, 2, "~>");
                    idx += 2;
                }
                '.' if line[idx..].starts_with(".>") => {
                    self.push_token(TokenKind::FlowArrow, line_number, line_offset + idx, 2, ".>");
                    idx += 2;
                }
                '(' if line[idx..].starts_with("(s)") => {
                    self.push_token(
                        TokenKind::EventStartMarker,
                        line_number,
                        line_offset + idx,
                        3,
                        "(s)",
                    );
                    idx += 3;
                }
                '(' if line[idx..].starts_with("(i)") => {
                    self.push_token(
                        TokenKind::EventIntermediateMarker,
                        line_number,
                        line_offset + idx,
                        3,
                        "(i)",
                    );
                    idx += 3;
                }
                '(' if line[idx..].starts_with("(e)") => {
                    self.push_token(
                        TokenKind::EventEndMarker,
                        line_number,
                        line_offset + idx,
                        3,
                        "(e)",
                    );
                    idx += 3;
                }
                '(' if line[idx..].starts_with("()") => {
                    self.push_token(
                        TokenKind::EventMarker,
                        line_number,
                        line_offset + idx,
                        2,
                        "()",
                    );
                    idx += 2;
                }
                '[' if line[idx..].starts_with("[]") => {
                    self.push_token(
                        TokenKind::TaskMarker,
                        line_number,
                        line_offset + idx,
                        2,
                        "[]",
                    );
                    idx += 2;
                }
                '[' => {
                    if let Some((width, command)) = parse_bracket_command(&line[idx..]) {
                        if idx == indent && is_task_marker_code(&command) {
                            let span =
                                self.span(line_number, line_offset + idx, line_offset + idx + width);
                            self.document.push_token(SyntaxToken::new(
                                TokenKind::TaskMarker,
                                span,
                                format!("[{}]", command),
                            ));
                            idx += width;
                            continue;
                        }

                        let span =
                            self.span(line_number, line_offset + idx, line_offset + idx + width);
                        self.document.push_token(SyntaxToken::new(
                            TokenKind::BracketCommand,
                            span,
                            command.clone(),
                        ));
                        idx += width;
                        if command != "call" || idx != indent + width {
                            while idx < line.len() && bytes[idx] as char == ' ' {
                                idx += 1;
                            }
                            if idx < line.len() && !line[idx..].starts_with("//") {
                                let span =
                                    self.span(line_number, line_offset + idx, line_offset + line.len());
                                let text = line[idx..].trim_end().to_string();
                                if !text.is_empty() {
                                    self.document.push_token(SyntaxToken::new(
                                        TokenKind::CommandValue,
                                        span,
                                        text,
                                    ));
                                }
                                idx = line.len();
                            }
                        }
                    } else {
                        let ch = &line[idx..idx + 1];
                        self.push_unknown(line_number, line_offset + idx, ch);
                        idx += 1;
                    }
                }
                '@' => {
                    self.document.diagnostics.push(
                        ParseDiagnostic::new(
                            ParseDiagnosticSeverity::Error,
                            "Legacy `@...` commands are not supported; use bracket commands",
                        )
                        .with_span(self.span(line_number, line_offset + idx, line_offset + idx + 1))
                        .with_code("DOGL1004"),
                    );
                    self.push_unknown(line_number, line_offset + idx, "@");
                    idx += 1;
                }
                '"' => {
                    let start = idx;
                    idx += 1;
                    while idx < line.len() && bytes[idx] as char != '"' {
                        idx += 1;
                    }
                    if idx >= line.len() {
                        self.document.diagnostics.push(
                            ParseDiagnostic::new(
                                ParseDiagnosticSeverity::Error,
                                "Unterminated string literal",
                            )
                            .with_span(self.span(line_number, line_offset + start, line_offset + line.len()))
                            .with_code("DOGL1003"),
                        );
                        let text = &line[start + 1..];
                        let span = self.span(line_number, line_offset + start, line_offset + line.len());
                        self.document
                            .push_token(SyntaxToken::new(TokenKind::StringLiteral, span, text));
                        break;
                    }

                    let end = idx + 1;
                    let text = &line[start + 1..idx];
                    let span = self.span(line_number, line_offset + start, line_offset + end);
                    self.document
                        .push_token(SyntaxToken::new(TokenKind::StringLiteral, span, text));
                    idx = end;
                }
                '{' => {
                    self.push_token(TokenKind::LeftBrace, line_number, line_offset + idx, 1, "{");
                    idx += 1;
                }
                '}' => {
                    self.push_token(TokenKind::RightBrace, line_number, line_offset + idx, 1, "}");
                    idx += 1;
                }
                '-' if idx + 1 < line.len() && (bytes[idx + 1] as char).is_ascii_digit() => {
                    let start = idx;
                    idx += 1;
                    idx = self.consume_number(line, idx);
                    let text = &line[start..idx];
                    let span = self.span(line_number, line_offset + start, line_offset + idx);
                    self.document
                        .push_token(SyntaxToken::new(TokenKind::Number, span, text));
                }
                ch if ch.is_ascii_digit() => {
                    let start = idx;
                    idx = self.consume_number(line, idx);
                    let text = &line[start..idx];
                    let span = self.span(line_number, line_offset + start, line_offset + idx);
                    self.document
                        .push_token(SyntaxToken::new(TokenKind::Number, span, text));
                }
                ch if is_identifier_start(ch) => {
                    let start = idx;
                    idx += ch.len_utf8();
                    while idx < line.len() {
                        let next_ch = line[idx..].chars().next().unwrap();
                        if is_identifier_char(next_ch) {
                            idx += next_ch.len_utf8();
                        } else {
                            break;
                        }
                    }
                    let text = &line[start..idx];
                    let kind = if text == "collab" {
                        TokenKind::KeywordCollab
                    } else if text == "layout" {
                        TokenKind::KeywordLayout
                    } else {
                        TokenKind::Identifier
                    };
                    let span = self.span(line_number, line_offset + start, line_offset + idx);
                    self.document.push_token(SyntaxToken::new(kind, span, text));
                }
                _ => {
                    let ch_len = ch.len_utf8();
                    let ch_str = &line[idx..idx + ch_len];
                    self.push_unknown(line_number, line_offset + idx, ch_str);
                    idx += ch_len;
                }
            }
        }
    }

    fn push_token(
        &mut self,
        kind: TokenKind,
        line_number: usize,
        absolute_start: usize,
        width: usize,
        text: &str,
    ) {
        let span = self.span(line_number, absolute_start, absolute_start + width);
        self.document.push_token(SyntaxToken::new(kind, span, text));
    }

    fn push_newline(&mut self, line_number: usize, offset: usize, has_newline: bool) {
        let end = if has_newline { offset + 1 } else { offset };
        let span = self.span(line_number, offset, end);
        self.document
            .push_token(SyntaxToken::new(TokenKind::Newline, span, ""));
    }

    fn push_trivia(
        &mut self,
        kind: SyntaxTriviaKind,
        line_number: usize,
        absolute_start: usize,
        absolute_end: usize,
        text: &str,
    ) {
        let span = self.span(line_number, absolute_start, absolute_end);
        self.document.push_trivia(SyntaxTrivia::new(kind, span, text));
    }

    fn push_unknown(&mut self, line_number: usize, absolute_start: usize, text: &str) {
        let span = self.span(line_number, absolute_start, absolute_start + text.len());
        self.document
            .push_token(SyntaxToken::new(TokenKind::Unknown, span, text));
        self.document.diagnostics.push(
            ParseDiagnostic::new(
                ParseDiagnosticSeverity::Error,
                format!("Unexpected token `{text}`"),
            )
            .with_span(span)
            .with_code("DOGL1004"),
        );
    }

    fn span(&self, line_number: usize, absolute_start: usize, absolute_end: usize) -> Span {
        Span::new(
            self.position(line_number, absolute_start),
            self.position(line_number, absolute_end),
        )
    }

    fn position(&self, line_number: usize, absolute_offset: usize) -> SourcePosition {
        let line_start = self
            .source
            .split_inclusive('\n')
            .take(line_number.saturating_sub(1))
            .map(str::len)
            .sum::<usize>();
        SourcePosition {
            offset: absolute_offset,
            line: line_number,
            column: absolute_offset.saturating_sub(line_start) + 1,
        }
    }

    fn position_for_offset(&self, absolute_offset: usize) -> SourcePosition {
        let mut running = 0usize;
        for (idx, raw_line) in self.source.split_inclusive('\n').enumerate() {
            let next = running + raw_line.len();
            if absolute_offset <= next {
                return SourcePosition {
                    offset: absolute_offset,
                    line: idx + 1,
                    column: absolute_offset.saturating_sub(running) + 1,
                };
            }
            running = next;
        }

        SourcePosition {
            offset: absolute_offset,
            line: self.source.lines().count().max(1),
            column: absolute_offset.saturating_sub(running) + 1,
        }
    }

    fn consume_number(&self, line: &str, mut idx: usize) -> usize {
        let bytes = line.as_bytes();
        let mut seen_dot = false;
        while idx < line.len() {
            let ch = bytes[idx] as char;
            if ch.is_ascii_digit() {
                idx += 1;
                continue;
            }
            if ch == '.' && !seen_dot {
                seen_dot = true;
                idx += 1;
                continue;
            }
            break;
        }
        idx
    }
}

fn is_identifier_start(ch: char) -> bool {
    ch.is_alphabetic() || ch == '_'
}

fn is_identifier_char(ch: char) -> bool {
    ch.is_alphanumeric() || ch == '_'
}

fn is_task_marker_code(command: &str) -> bool {
    matches!(command, "m" | "u" | "st" | "rt" | "se" | "sc" | "bu")
}

fn parse_bracket_command(input: &str) -> Option<(usize, String)> {
    let end = input.find(']')?;
    let name = &input[1..end];
    if !is_bracket_command_name(name) {
        return None;
    }
    Some((end + 1, name.to_string()))
}

fn is_bracket_command_name(name: &str) -> bool {
    !name.is_empty() && name.chars().all(|ch| ch.is_ascii_lowercase() || ch == '.')
}
