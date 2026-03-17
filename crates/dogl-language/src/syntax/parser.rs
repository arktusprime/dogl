use super::{
    diagnostics::{ParseDiagnostic, ParseDiagnosticSeverity},
    nodes::{SyntaxDocument, SyntaxKind, SyntaxNode, SyntaxNodeId, TokenRange},
    source::Span,
    tokens::TokenKind,
};

pub fn parse(mut document: SyntaxDocument) -> SyntaxDocument {
    let mut parser = Parser::new(&mut document);
    parser.parse_file();
    document
}

struct Parser<'a> {
    document: &'a mut SyntaxDocument,
    cursor: usize,
}

impl<'a> Parser<'a> {
    fn new(document: &'a mut SyntaxDocument) -> Self {
        Self {
            document,
            cursor: 0,
        }
    }

    fn parse_file(&mut self) {
        let mut root = SyntaxNode::new(SyntaxKind::File);
        let start = self.cursor;

        self.skip_layout();
        while !self.at(TokenKind::Eof) {
            if self.at(TokenKind::KeywordCollab) {
                let child = self.parse_collab();
                root.push_child(child);
            } else if self.at(TokenKind::KeywordLayout) {
                let child = self.parse_layout_section();
                root.push_child(child);
            } else {
                self.error_current("Expected `collab` or `layout`");
                self.advance_to_line_end();
            }
            self.skip_layout();
        }

        let end = self.cursor;
        if end > start {
            root = root.with_token_range(TokenRange::new(start, end));
            if let Some(span) = self.range_span(start, end) {
                root = root.with_span(span);
            }
        }

        let root_id = self.document.push_node(root);
        self.document.set_root(root_id);
    }

    fn parse_collab(&mut self) -> SyntaxNodeId {
        let start = self.cursor;
        self.expect(TokenKind::KeywordCollab, "Expected `collab`");
        let collab_name = self.expect_text(TokenKind::Identifier, "Expected collaboration identifier");
        self.expect(TokenKind::Newline, "Expected newline after collaboration header");
        self.expect_block_indent("Expected indented collaboration body");

        let mut node = SyntaxNode::new(SyntaxKind::Collab).with_text_name(collab_name);
        while !self.at(TokenKind::Dedent) && !self.at(TokenKind::Eof) {
            self.skip_layout();
            if self.at(TokenKind::PoolMarker) {
                let child = self.parse_pool();
                node.push_child(child);
            } else if !self.at(TokenKind::Dedent) && !self.at(TokenKind::Eof) {
                self.error_current("Expected pool declaration");
                self.advance_to_line_end();
            }
        }

        self.expect(TokenKind::Dedent, "Expected end of collaboration block");
        self.finish_node(node, start, self.cursor)
    }

    fn parse_layout_section(&mut self) -> SyntaxNodeId {
        let start = self.cursor;
        self.expect(TokenKind::KeywordLayout, "Expected `layout`");
        self.expect(TokenKind::Newline, "Expected newline after `layout`");
        self.expect_block_indent("Expected indented layout body");

        let mut node = SyntaxNode::new(SyntaxKind::LayoutSection);
        while !self.at(TokenKind::Dedent) && !self.at(TokenKind::Eof) {
            self.skip_layout();
            if self.at(TokenKind::PoolMarker) {
                let child = self.parse_layout_pool();
                node.push_child(child);
            } else if !self.at(TokenKind::Dedent) && !self.at(TokenKind::Eof) {
                self.error_current("Expected pool layout entry");
                self.recover_invalid_line_with_block();
            }
        }

        self.expect(TokenKind::Dedent, "Expected end of layout block");
        self.finish_node(node, start, self.cursor)
    }

    fn parse_pool(&mut self) -> SyntaxNodeId {
        let start = self.cursor;
        self.expect(TokenKind::PoolMarker, "Expected pool marker");
        let pool_name = self.expect_text(TokenKind::Identifier, "Expected pool identifier");
        let bounds = self.parse_optional_bounds();
        self.expect(TokenKind::Newline, "Expected newline after pool declaration");
        self.expect_block_indent("Expected indented pool body");

        let mut node = SyntaxNode::new(SyntaxKind::Pool).with_text_name(pool_name);
        if let Some(bounds) = bounds {
            node.push_child(bounds);
        }
        while !self.at(TokenKind::Dedent) && !self.at(TokenKind::Eof) {
            self.skip_layout();
            if self.at(TokenKind::LaneMarker) {
                let child = self.parse_lane();
                node.push_child(child);
            } else if !self.at(TokenKind::Dedent) && !self.at(TokenKind::Eof) {
                self.error_current("Expected lane declaration");
                self.advance_to_line_end();
            }
        }

        self.expect(TokenKind::Dedent, "Expected end of pool block");
        self.finish_node(node, start, self.cursor)
    }

    fn parse_lane(&mut self) -> SyntaxNodeId {
        let start = self.cursor;
        self.expect(TokenKind::LaneMarker, "Expected lane marker");
        let lane_name = self.expect_text(TokenKind::Identifier, "Expected lane identifier");
        let bounds = self.parse_optional_bounds();
        self.expect(TokenKind::Newline, "Expected newline after lane declaration");
        self.expect_block_indent("Expected indented lane body");

        let mut node = SyntaxNode::new(SyntaxKind::Lane).with_text_name(lane_name);
        if let Some(bounds) = bounds {
            node.push_child(bounds);
        }
        while !self.at(TokenKind::Dedent) && !self.at(TokenKind::Eof) {
            self.skip_layout();
            if self.at(TokenKind::StageMarker) {
                let child = self.parse_stage();
                node.push_child(child);
            } else if !self.at(TokenKind::Dedent) && !self.at(TokenKind::Eof) {
                self.error_current("Expected stage declaration");
                self.advance_to_line_end();
            }
        }

        self.expect(TokenKind::Dedent, "Expected end of lane block");
        self.finish_node(node, start, self.cursor)
    }

    fn parse_stage(&mut self) -> SyntaxNodeId {
        let start = self.cursor;
        self.expect(TokenKind::StageMarker, "Expected stage marker");
        let stage_name = self.expect_text(TokenKind::Identifier, "Expected stage identifier");
        let bounds = self.parse_optional_bounds();
        self.expect(TokenKind::Newline, "Expected newline after stage declaration");
        self.expect_block_indent("Expected indented stage body");

        let mut node = SyntaxNode::new(SyntaxKind::Stage).with_text_name(stage_name);
        if let Some(bounds) = bounds {
            node.push_child(bounds);
        }
        while !self.at(TokenKind::Dedent) && !self.at(TokenKind::Eof) {
            self.skip_layout();
            match self.current_kind() {
                Some(kind) if kind.is_event_marker() => {
                    let child = self.parse_event();
                    node.push_child(child);
                }
                Some(kind) if kind.is_gateway_marker() => {
                    let child = self.parse_gateway();
                    node.push_child(child);
                }
                Some(TokenKind::TaskMarker) => {
                    let child = self.parse_task();
                    node.push_child(child);
                }
                Some(TokenKind::BracketCommand) if self.current_text_is("call") => {
                    let child = self.parse_call_task();
                    node.push_child(child);
                }
                Some(TokenKind::Dedent | TokenKind::Eof) | None => break,
                _ => {
                    self.error_current("Expected event, task, or gateway element");
                    self.recover_invalid_line_with_block();
                }
            }
        }

        self.expect(TokenKind::Dedent, "Expected end of stage block");
        self.finish_node(node, start, self.cursor)
    }

    fn parse_event(&mut self) -> SyntaxNodeId {
        let start = self.cursor;
        self.advance();
        let event_name = self.expect_text(TokenKind::Identifier, "Expected event identifier");
        let bounds = self.parse_optional_bounds();

        let mut node = SyntaxNode::new(SyntaxKind::Event).with_text_name(event_name);
        if let Some(bounds) = bounds {
            node.push_child(bounds);
        }
        while self.at(TokenKind::BracketCommand) && !self.current_text_is("call") {
            let expression = self.parse_expression();
            node.push_child(expression);
        }

        self.expect(TokenKind::Newline, "Expected newline after event");
        self.parse_element_children(&mut node);
        self.finish_node(node, start, self.cursor)
    }

    fn parse_task(&mut self) -> SyntaxNodeId {
        let start = self.cursor;
        self.expect(TokenKind::TaskMarker, "Expected task marker");
        let task_name = self.expect_text(TokenKind::Identifier, "Expected task identifier");
        let bounds = self.parse_optional_bounds();

        let mut node = SyntaxNode::new(SyntaxKind::Task).with_text_name(task_name);
        if let Some(bounds) = bounds {
            node.push_child(bounds);
        }
        while self.at(TokenKind::BracketCommand) && !self.current_text_is("call") {
            let expression = self.parse_expression();
            node.push_child(expression);
        }

        self.expect(TokenKind::Newline, "Expected newline after task");
        self.parse_element_children(&mut node);
        self.finish_node(node, start, self.cursor)
    }

    fn parse_call_task(&mut self) -> SyntaxNodeId {
        let start = self.cursor;
        self.expect(TokenKind::BracketCommand, "Expected `[call]`");
        let task_name = self.expect_text(TokenKind::Identifier, "Expected called process identifier");
        let bounds = self.parse_optional_bounds();

        let mut node = SyntaxNode::new(SyntaxKind::Task).with_text_name(task_name);
        if let Some(bounds) = bounds {
            node.push_child(bounds);
        }
        while self.at(TokenKind::BracketCommand) && !self.current_text_is("call") {
            let expression = self.parse_expression();
            node.push_child(expression);
        }

        self.expect(TokenKind::Newline, "Expected newline after call activity");
        self.parse_element_children(&mut node);
        self.finish_node(node, start, self.cursor)
    }

    fn parse_gateway(&mut self) -> SyntaxNodeId {
        let start = self.cursor;
        self.advance();
        let gateway_name = self.expect_text(TokenKind::Identifier, "Expected gateway identifier");
        let bounds = self.parse_optional_bounds();

        let mut node = SyntaxNode::new(SyntaxKind::Gateway).with_text_name(gateway_name);
        if let Some(bounds) = bounds {
            node.push_child(bounds);
        }
        while self.at(TokenKind::BracketCommand) && !self.current_text_is("call") {
            let expression = self.parse_expression();
            node.push_child(expression);
        }

        self.expect(TokenKind::Newline, "Expected newline after gateway");
        self.parse_element_children(&mut node);
        self.finish_node(node, start, self.cursor)
    }

    fn parse_element_children(&mut self, node: &mut SyntaxNode) {
        if !self.at(TokenKind::Indent) {
            return;
        }
        self.advance();

        while !self.at(TokenKind::Dedent) && !self.at(TokenKind::Eof) {
            self.skip_layout();
            match self.current_kind() {
                Some(TokenKind::FlowArrow) => {
                    let child = self.parse_flow();
                    node.push_child(child);
                }
                Some(TokenKind::BracketCommand) => {
                    let child = self.parse_expression_line();
                    node.push_child(child);
                }
                Some(TokenKind::Dedent | TokenKind::Eof) | None => break,
                _ => {
                    self.error_current("Expected flow or expression inside element block");
                    self.recover_invalid_line_with_block();
                }
            }
        }

        self.expect(TokenKind::Dedent, "Expected end of element block");
    }

    fn parse_flow(&mut self) -> SyntaxNodeId {
        let start = self.cursor;
        self.expect(TokenKind::FlowArrow, "Expected flow arrow");
        let target = self.expect_text(TokenKind::Identifier, "Expected flow target identifier");
        self.expect(TokenKind::Newline, "Expected newline after flow");

        let node = SyntaxNode::new(SyntaxKind::Flow).with_text_name(target);
        self.finish_node(node, start, self.cursor)
    }

    fn parse_expression_line(&mut self) -> SyntaxNodeId {
        let start = self.cursor;
        let key = self.expect_text(TokenKind::BracketCommand, "Expected bracket command");
        let value = self.expect_command_value();
        self.expect(TokenKind::Newline, "Expected newline after expression");

        let node = SyntaxNode::new(SyntaxKind::Expression).with_text_name(format!("{key}={value}"));
        self.finish_node(node, start, self.cursor)
    }

    fn parse_expression(&mut self) -> SyntaxNodeId {
        let start = self.cursor;
        let key = self.expect_text(TokenKind::BracketCommand, "Expected bracket command");
        let value = self.expect_command_value();

        let node = SyntaxNode::new(SyntaxKind::Expression).with_text_name(format!("{key}={value}"));
        self.finish_node(node, start, self.cursor)
    }

    fn parse_optional_bounds(&mut self) -> Option<SyntaxNodeId> {
        if !self.at(TokenKind::LeftBrace) {
            return None;
        }

        let start = self.cursor;
        self.expect(TokenKind::LeftBrace, "Expected `{` to start bounds");
        self.expect_number("Expected bounds x coordinate");
        self.expect_number("Expected bounds y coordinate");
        self.expect_number("Expected bounds width");
        self.expect_number("Expected bounds height");
        self.expect(TokenKind::RightBrace, "Expected `}` to end bounds");

        Some(self.finish_node(SyntaxNode::new(SyntaxKind::Bounds), start, self.cursor))
    }

    fn parse_layout_pool(&mut self) -> SyntaxNodeId {
        let start = self.cursor;
        self.expect(TokenKind::PoolMarker, "Expected pool marker");
        let pool_name = self.expect_text(TokenKind::Identifier, "Expected pool identifier");
        let bounds = self.parse_optional_bounds();
        self.expect(TokenKind::Newline, "Expected newline after pool layout entry");
        self.expect_block_indent("Expected indented pool layout body");

        let mut node = SyntaxNode::new(SyntaxKind::Pool).with_text_name(pool_name);
        if let Some(bounds) = bounds {
            node.push_child(bounds);
        }

        while !self.at(TokenKind::Dedent) && !self.at(TokenKind::Eof) {
            self.skip_layout();
            if self.at(TokenKind::LaneMarker) {
                let child = self.parse_layout_lane();
                node.push_child(child);
            } else if !self.at(TokenKind::Dedent) && !self.at(TokenKind::Eof) {
                self.error_current("Expected lane layout entry");
                self.recover_invalid_line_with_block();
            }
        }

        self.expect(TokenKind::Dedent, "Expected end of pool layout block");
        self.finish_node(node, start, self.cursor)
    }

    fn parse_layout_lane(&mut self) -> SyntaxNodeId {
        let start = self.cursor;
        self.expect(TokenKind::LaneMarker, "Expected lane marker");
        let lane_name = self.expect_text(TokenKind::Identifier, "Expected lane identifier");
        let bounds = self.parse_optional_bounds();
        self.expect(TokenKind::Newline, "Expected newline after lane layout entry");
        self.expect_block_indent("Expected indented lane layout body");

        let mut node = SyntaxNode::new(SyntaxKind::Lane).with_text_name(lane_name);
        if let Some(bounds) = bounds {
            node.push_child(bounds);
        }

        while !self.at(TokenKind::Dedent) && !self.at(TokenKind::Eof) {
            self.skip_layout();
            if self.at(TokenKind::StageMarker) {
                let child = self.parse_layout_stage();
                node.push_child(child);
            } else if !self.at(TokenKind::Dedent) && !self.at(TokenKind::Eof) {
                self.error_current("Expected stage layout entry");
                self.recover_invalid_line_with_block();
            }
        }

        self.expect(TokenKind::Dedent, "Expected end of lane layout block");
        self.finish_node(node, start, self.cursor)
    }

    fn parse_layout_stage(&mut self) -> SyntaxNodeId {
        let start = self.cursor;
        self.expect(TokenKind::StageMarker, "Expected stage marker");
        let stage_name = self.expect_text(TokenKind::Identifier, "Expected stage identifier");
        let bounds = self.parse_optional_bounds();
        self.expect(TokenKind::Newline, "Expected newline after stage layout entry");
        self.expect_block_indent("Expected indented stage layout body");

        let mut node = SyntaxNode::new(SyntaxKind::Stage).with_text_name(stage_name);
        if let Some(bounds) = bounds {
            node.push_child(bounds);
        }

        while !self.at(TokenKind::Dedent) && !self.at(TokenKind::Eof) {
            self.skip_layout();
            match self.current_kind() {
                Some(kind) if kind.is_event_marker() => {
                    let child = self.parse_layout_event();
                    node.push_child(child);
                }
                Some(kind) if kind.is_gateway_marker() => {
                    let child = self.parse_layout_gateway();
                    node.push_child(child);
                }
                Some(TokenKind::TaskMarker) => {
                    let child = self.parse_layout_task();
                    node.push_child(child);
                }
                Some(TokenKind::BracketCommand) if self.current_text_is("call") => {
                    let child = self.parse_layout_task();
                    node.push_child(child);
                }
                Some(TokenKind::Dedent | TokenKind::Eof) | None => break,
                _ => {
                    self.error_current("Expected layout element entry");
                    self.recover_invalid_line_with_block();
                }
            }
        }

        self.expect(TokenKind::Dedent, "Expected end of stage layout block");
        self.finish_node(node, start, self.cursor)
    }

    fn parse_layout_event(&mut self) -> SyntaxNodeId {
        let start = self.cursor;
        self.advance();
        let event_name = self.expect_text(TokenKind::Identifier, "Expected event identifier");
        let bounds = self.parse_optional_bounds();
        self.expect(TokenKind::Newline, "Expected newline after event layout entry");

        let mut node = SyntaxNode::new(SyntaxKind::Event).with_text_name(event_name);
        if let Some(bounds) = bounds {
            node.push_child(bounds);
        }
        self.finish_node(node, start, self.cursor)
    }

    fn parse_layout_task(&mut self) -> SyntaxNodeId {
        let start = self.cursor;
        if self.at(TokenKind::TaskMarker) || (self.at(TokenKind::BracketCommand) && self.current_text_is("call")) {
            self.advance();
        }
        let task_name = self.expect_text(TokenKind::Identifier, "Expected task identifier");
        let bounds = self.parse_optional_bounds();
        self.expect(TokenKind::Newline, "Expected newline after task layout entry");

        let mut node = SyntaxNode::new(SyntaxKind::Task).with_text_name(task_name);
        if let Some(bounds) = bounds {
            node.push_child(bounds);
        }
        self.finish_node(node, start, self.cursor)
    }

    fn parse_layout_gateway(&mut self) -> SyntaxNodeId {
        let start = self.cursor;
        self.advance();
        let gateway_name = self.expect_text(TokenKind::Identifier, "Expected gateway identifier");
        let bounds = self.parse_optional_bounds();
        self.expect(TokenKind::Newline, "Expected newline after gateway layout entry");

        let mut node = SyntaxNode::new(SyntaxKind::Gateway).with_text_name(gateway_name);
        if let Some(bounds) = bounds {
            node.push_child(bounds);
        }
        self.finish_node(node, start, self.cursor)
    }

    fn expect_command_value(&mut self) -> String {
        if self.at(TokenKind::CommandValue) {
            let value = self.expect_text(TokenKind::CommandValue, "Expected command value");
            normalize_command_value(&value)
        } else if self.at(TokenKind::Identifier) {
            let value = self.expect_text(TokenKind::Identifier, "Expected command value");
            normalize_command_value(&value)
        } else if self.at(TokenKind::StringLiteral) {
            let value = self.expect_text(TokenKind::StringLiteral, "Expected command value");
            normalize_command_value(&value)
        } else {
            self.error_current("Expected command value");
            String::new()
        }
    }

    fn finish_node(&mut self, mut node: SyntaxNode, start: usize, end: usize) -> SyntaxNodeId {
        if end > start {
            node = node.with_token_range(TokenRange::new(start, end));
            if let Some(span) = self.range_span(start, end) {
                node = node.with_span(span);
            }
        }
        self.document.push_node(node)
    }

    fn skip_layout(&mut self) {
        while matches!(self.current_kind(), Some(TokenKind::Newline)) {
            self.advance();
        }
    }

    fn expect(&mut self, kind: TokenKind, message: &str) {
        if self.at(kind) {
            self.advance();
        } else {
            self.error_current(message);
        }
    }

    fn expect_text(&mut self, kind: TokenKind, message: &str) -> String {
        if self.at(kind) {
            let text = self.document.tokens[self.cursor].text.clone();
            self.advance();
            text
        } else {
            self.error_current(message);
            String::new()
        }
    }

    fn expect_number(&mut self, message: &str) -> String {
        self.expect_text(TokenKind::Number, message)
    }

    fn expect_block_indent(&mut self, message: &str) {
        while self.at(TokenKind::Newline) {
            self.advance();
        }
        self.expect(TokenKind::Indent, message);
    }

    fn error_current(&mut self, message: &str) {
        let diagnostic = ParseDiagnostic::new(ParseDiagnosticSeverity::Error, message)
            .with_code("DOGL2001");
        let diagnostic = if let Some(span) = self.current_span() {
            diagnostic.with_span(span)
        } else {
            diagnostic
        };
        self.document.diagnostics.push(diagnostic);
    }

    fn advance_to_line_end(&mut self) {
        while !self.at(TokenKind::Newline) && !self.at(TokenKind::Dedent) && !self.at(TokenKind::Eof) {
            self.advance();
        }
        if self.at(TokenKind::Newline) {
            self.advance();
        }
    }

    fn recover_invalid_line_with_block(&mut self) {
        self.advance_to_line_end();
        if !self.at(TokenKind::Indent) {
            return;
        }

        let mut depth = 0usize;
        while !self.at(TokenKind::Eof) {
            match self.current_kind() {
                Some(TokenKind::Indent) => {
                    depth += 1;
                    self.advance();
                }
                Some(TokenKind::Dedent) => {
                    self.advance();
                    if depth == 0 {
                        break;
                    }
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                _ => self.advance(),
            }
        }
    }

    fn at(&self, kind: TokenKind) -> bool {
        matches!(self.current_kind(), Some(current) if current == kind)
    }

    fn advance(&mut self) {
        if self.cursor < self.document.tokens.len() {
            self.cursor += 1;
        }
    }

    fn current_kind(&self) -> Option<TokenKind> {
        self.document.tokens.get(self.cursor).map(|token| token.kind)
    }

    fn current_text_is(&self, expected: &str) -> bool {
        self.document
            .tokens
            .get(self.cursor)
            .is_some_and(|token| token.text == expected)
    }

    fn current_span(&self) -> Option<Span> {
        self.document.tokens.get(self.cursor).map(|token| token.span)
    }

    fn range_span(&self, start: usize, end: usize) -> Option<Span> {
        let first = self.document.tokens.get(start)?;
        let last = self.document.tokens.get(end.saturating_sub(1))?;
        Some(Span::new(first.span.start, last.span.end))
    }
}

fn normalize_command_value(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.len() >= 2 && trimmed.starts_with('"') && trimmed.ends_with('"') {
        trimmed[1..trimmed.len() - 1].to_string()
    } else {
        trimmed.to_string()
    }
}
