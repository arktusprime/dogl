use crate::{
    domain::{
        layout_from_grouped, Bounds, Collab, DoglFile, Element, Event, EventCode, Expression,
        Flow, FlowType, Gateway, GatewayCode, Lane, Layout, Pool, PoolLayoutData, Quadrant,
        Stage, Task, TaskCode, Uid, Identifiable, name_from_id,
    },
    resolver::{
        BindingSummary, NameResolutionSummary, NormalizationPass, ResolverDiagnostic,
        ResolverDiagnosticSeverity,
    },
    syntax::{ParseDiagnosticSeverity, SyntaxDocument, SyntaxKind, SyntaxNode, TokenKind},
};
use std::collections::{HashMap, HashSet};

/// Placeholder semantic output of the explicit lowering boundary between
/// resolver-owned intermediate data and the semantic domain.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct LoweredSemanticFile {
    pub semantic_file: Option<DoglFile>,
}

/// Placeholder resolver output that keeps binding, name resolution,
/// normalization, diagnostics, and semantic lowering as explicit phases.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ResolverOutput {
    /// Placeholder result of binding declarations and local names.
    pub bindings: BindingSummary,
    /// Placeholder result of resolving references after binding.
    pub resolution: NameResolutionSummary,
    /// Placeholder list of normalization passes run before lowering.
    pub normalization_passes: Vec<NormalizationPass>,
    /// Placeholder semantic-domain output produced only by explicit lowering.
    pub lowering: LoweredSemanticFile,
    /// Placeholder diagnostics emitted by resolver-owned stages.
    pub diagnostics: Vec<ResolverDiagnostic>,
}

pub fn resolve(document: &SyntaxDocument) -> ResolverOutput {
    let mut resolver = Resolver::new(document);
    resolver.resolve()
}

struct Resolver<'a> {
    document: &'a SyntaxDocument,
    allocator: UidAllocator,
    binding_summary: BindingSummary,
    resolution_summary: NameResolutionSummary,
    diagnostics: Vec<ResolverDiagnostic>,
}

impl<'a> Resolver<'a> {
    fn new(document: &'a SyntaxDocument) -> Self {
        Self {
            document,
            allocator: UidAllocator::default(),
            binding_summary: BindingSummary::default(),
            resolution_summary: NameResolutionSummary::default(),
            diagnostics: Vec::new(),
        }
    }

    fn resolve(&mut self) -> ResolverOutput {
        let has_syntax_errors = self
            .document
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.severity == ParseDiagnosticSeverity::Error);

        let semantic_file = if has_syntax_errors {
            None
        } else {
            self.lower_file()
        };

        ResolverOutput {
            bindings: self.binding_summary.clone(),
            resolution: self.resolution_summary.clone(),
            normalization_passes: vec![NormalizationPass::new("mvp_lowering")],
            lowering: LoweredSemanticFile { semantic_file },
            diagnostics: self.diagnostics.clone(),
        }
    }

    fn lower_file(&mut self) -> Option<DoglFile> {
        let root_id = self.document.root?;
        let root = self.document.node(root_id)?;
        let layout_section_id = root.children.iter().copied().find(|child_id| {
            self.document
                .node(*child_id)
                .is_some_and(|node| node.kind == SyntaxKind::LayoutSection)
        });

        let mut collabs = Vec::new();
        for child_id in &root.children {
            let Some(node) = self.document.node(*child_id) else {
                continue;
            };
            if node.kind != SyntaxKind::Collab {
                continue;
            }
            if let Some(collab) = self.lower_collab(node, layout_section_id) {
                collabs.push(collab);
            }
        }

        if collabs.is_empty() || self.has_errors() {
            None
        } else {
            Some(DoglFile::new(collabs))
        }
    }

    fn lower_collab(&mut self, node: &SyntaxNode, layout_section_id: Option<crate::syntax::SyntaxNodeId>) -> Option<Collab> {
        let collab_id = node.text_name.clone()?;
        self.binding_summary.bound_names += 1;

        let mut collab = Collab::new(self.allocator.next(), collab_id);
        let mut layout = Layout::default();
        let mut element_uids = HashMap::new();
        let mut pending_flows = Vec::new();

        for child_id in &node.children {
            let Some(pool_node) = self.document.node(*child_id) else {
                continue;
            };
            if pool_node.kind != SyntaxKind::Pool {
                continue;
            }
            if let Some(pool) = self.lower_pool(pool_node, &mut layout, &mut element_uids, &mut pending_flows) {
                collab.pools.push(pool);
            }
        }

        for pending in pending_flows {
            let Some(to_uid) = element_uids.get(&pending.target_id).copied() else {
                self.resolution_summary.unresolved_references += 1;
                self.diagnostics.push(ResolverDiagnostic::new(
                    ResolverDiagnosticSeverity::Error,
                    format!("Unknown flow target `{}`", pending.target_id),
                ));
                continue;
            };

            self.resolution_summary.resolved_references += 1;
            let flow = Flow::new(self.allocator.next(), pending.from_uid, to_uid, pending.flow_type);
            
            if pending.flow_type == FlowType::Message {
                if let Err(err) = collab.add_message_flow(flow) {
                    self.diagnostics.push(ResolverDiagnostic::new(
                        ResolverDiagnosticSeverity::Error,
                        format!("Failed to add message flow: {err:?}"),
                    ));
                }
            } else {
                let mut added = false;
                for pool in &mut collab.pools {
                    let mut in_pool = false;
                    for q in &pool.quadrants {
                        if q.elements.iter().any(|e| crate::domain::Identifiable::uid(e) == pending.from_uid) {
                            in_pool = true;
                            break;
                        }
                    }
                    
                    if in_pool {
                        if let Err(err) = pool.add_sequence_flow(flow.clone()) {
                            self.diagnostics.push(ResolverDiagnostic::new(
                                ResolverDiagnosticSeverity::Error,
                                format!("Failed to add sequence flow: {err:?}"),
                            ));
                        }
                        added = true;
                        break;
                    }
                }
                if !added {
                    self.diagnostics.push(ResolverDiagnostic::new(
                        ResolverDiagnosticSeverity::Error,
                        format!("Could not find pool for flow from uid {}", pending.from_uid),
                    ));
                }
            }
        }

        if let Some(layout_section_id) = layout_section_id {
            if let Some(layout_node) = self.document.node(layout_section_id) {
                if let Some(grouped_layout) = self.lower_layout_section(layout_node, &collab) {
                    for (uid, bounds) in grouped_layout.bounds_by_uid {
                        layout.set(uid, bounds);
                    }
                }
            }
        }

        if !layout.bounds_by_uid.is_empty() {
            collab.layout = Some(layout);
        }

        Some(collab)
    }

    fn lower_pool(
        &mut self,
        node: &SyntaxNode,
        layout: &mut Layout,
        element_uids: &mut HashMap<String, Uid>,
        pending_flows: &mut Vec<PendingFlow>,
    ) -> Option<Pool> {
        let pool_id = node.text_name.clone()?;
        self.binding_summary.bound_names += 1;

        let mut pool = Pool::new(self.allocator.next(), pool_id);
        self.capture_layout_bounds(node, pool.uid, layout);
        let mut lane_ids = HashSet::new();
        let mut stage_ids = HashSet::new();

        for child_id in &node.children {
            let Some(lane_node) = self.document.node(*child_id) else {
                continue;
            };
            if lane_node.kind != SyntaxKind::Lane {
                continue;
            }

            self.lower_lane(
                lane_node,
                &mut pool,
                layout,
                &mut lane_ids,
                &mut stage_ids,
                element_uids,
                pending_flows,
            );
        }

        Some(pool)
    }

    fn lower_lane(
        &mut self,
        node: &SyntaxNode,
        pool: &mut Pool,
        layout: &mut Layout,
        lane_ids: &mut HashSet<String>,
        stage_ids: &mut HashSet<String>,
        element_uids: &mut HashMap<String, Uid>,
        pending_flows: &mut Vec<PendingFlow>,
    ) {
        let Some(lane_id) = node.text_name.clone() else {
            return;
        };

        if !lane_ids.insert(lane_id.clone()) {
            self.binding_summary.unresolved_names += 1;
            self.diagnostics.push(ResolverDiagnostic::new(
                ResolverDiagnosticSeverity::Error,
                format!("Duplicate lane id `{lane_id}` in pool `{}`", pool.id),
            ));
            return;
        }

        self.binding_summary.bound_names += 1;
        let lane = Lane::new(self.allocator.next(), lane_id.clone());
        self.capture_layout_bounds(node, lane.uid, layout);
        pool.lanes.push(lane);

        for child_id in &node.children {
            let Some(stage_node) = self.document.node(*child_id) else {
                continue;
            };
            if stage_node.kind != SyntaxKind::Stage {
                continue;
            }

            self.lower_stage(
                stage_node,
                &lane_id,
                pool,
                layout,
                stage_ids,
                element_uids,
                pending_flows,
            );
        }
    }

    fn lower_stage(
        &mut self,
        node: &SyntaxNode,
        lane_id: &str,
        pool: &mut Pool,
        layout: &mut Layout,
        stage_ids: &mut HashSet<String>,
        element_uids: &mut HashMap<String, Uid>,
        pending_flows: &mut Vec<PendingFlow>,
    ) {
        let Some(stage_id) = node.text_name.clone() else {
            return;
        };

        if stage_ids.insert(stage_id.clone()) {
            self.binding_summary.bound_names += 1;
            let stage = Stage::new(self.allocator.next(), stage_id.clone());
            self.capture_layout_bounds(node, stage.uid, layout);
            pool.stages.push(stage);
        }

        let mut quadrant = Quadrant::new(self.allocator.next(), lane_id.to_string(), stage_id.clone());

        for child_id in &node.children {
            let Some(element_node) = self.document.node(*child_id) else {
                continue;
            };

            let lowered = match element_node.kind {
                SyntaxKind::Event => self.lower_event(element_node),
                SyntaxKind::Task => self.lower_task(element_node),
                SyntaxKind::Gateway => self.lower_gateway(element_node),
                _ => None,
            };

            let Some((element, flows)) = lowered else {
                continue;
            };

            let element_id = element.id().clone();
            if let Some(bounds) = self.collect_bounds(element_node) {
                layout.set(element.uid(), bounds);
            }
            if element_uids.insert(element_id.clone(), element.uid()).is_some() {
                self.binding_summary.unresolved_names += 1;
                self.diagnostics.push(ResolverDiagnostic::new(
                    ResolverDiagnosticSeverity::Error,
                    format!("Duplicate element id `{element_id}` in pool `{}`", pool.id),
                ));
                continue;
            }

            self.binding_summary.bound_names += 1;
            pending_flows.extend(flows);
            quadrant.elements.push(element);
        }

        pool.quadrants.push(quadrant);
    }

    fn lower_event(&mut self, node: &SyntaxNode) -> Option<(Element, Vec<PendingFlow>)> {
        let id = node.text_name.clone()?;
        let name = self.element_name(node, &id);
        let code = match self.first_token_kind(node) {
            Some(TokenKind::EventMarker) => EventCode::Inferred,
            Some(TokenKind::EventStartMarker) => EventCode::Start,
            Some(TokenKind::EventIntermediateMarker) => EventCode::Intermediate,
            Some(TokenKind::EventEndMarker) => EventCode::End,
            _ => EventCode::Inferred,
        };

        let uid = self.allocator.next();
        let expressions = self.collect_expressions(node);
        let pending_flows = self.collect_flows(node, uid);
        let event = Event {
            uid,
            id: id.clone(),
            name,
            code,
            expressions,
        };
        Some((Element::Event(event), pending_flows))
    }

    fn lower_task(&mut self, node: &SyntaxNode) -> Option<(Element, Vec<PendingFlow>)> {
        let id = node.text_name.clone()?;
        let name = self.element_name(node, &id);
        
        let first_token = self.token_slice(node).first();
        let code = if let Some(token) = first_token {
            if token.kind == TokenKind::TaskMarker {
                match token.text.as_str() {
                    "[]" => TaskCode::Generic,
                    "[m]" => TaskCode::Manual,
                    "[u]" => TaskCode::User,
                    "[st]" => TaskCode::Service,
                    "[rt]" => TaskCode::Receive,
                    "[se]" => TaskCode::Send,
                    "[sc]" => TaskCode::Script,
                    "[bu]" => TaskCode::BusinessRule,
                    _ => TaskCode::Generic,
                }
            } else if token.kind == TokenKind::BracketCommand && token.text == "call" {
                TaskCode::CallActivity
            } else {
                TaskCode::Generic
            }
        } else {
            TaskCode::Generic
        };

        let uid = self.allocator.next();
        let expressions = self.collect_expressions(node);
        self.validate_task(node, code);
        let pending_flows = self.collect_flows(node, uid);
        let task = Task {
            uid,
            id: id.clone(),
            name,
            code,
            call_target: (code == TaskCode::CallActivity).then(|| id.clone()),
            expressions,
        };
        Some((Element::Task(task), pending_flows))
    }

    fn lower_gateway(&mut self, node: &SyntaxNode) -> Option<(Element, Vec<PendingFlow>)> {
        let id = node.text_name.clone()?;
        let name = self.element_name(node, &id);
        let code = match self.first_token_kind(node) {
            Some(TokenKind::GatewayExclusiveMarker) => GatewayCode::Exclusive,
            Some(TokenKind::GatewayParallelMarker) => GatewayCode::Parallel,
            Some(TokenKind::GatewayEventBasedMarker) => GatewayCode::EventBased,
            Some(TokenKind::GatewayInclusiveMarker) => GatewayCode::Inclusive,
            Some(TokenKind::GatewayComplexMarker) => GatewayCode::Complex,
            _ => GatewayCode::Inclusive,
        };

        let uid = self.allocator.next();
        let expressions = self.collect_expressions(node);
        let dmn_ref = expressions
            .iter()
            .find(|expr| expr.key == "dmn")
            .map(|expr| expr.value.clone());
        let pending_flows = self.collect_flows(node, uid);
        let gateway = Gateway {
            uid,
            id: id.clone(),
            name,
            code,
            dmn_ref,
            expressions,
        };
        Some((Element::Gateway(gateway), pending_flows))
    }

    fn validate_task(&mut self, node: &SyntaxNode, code: TaskCode) {
        if code == TaskCode::CallActivity
            && node
                .text_name
                .as_deref()
                .map_or(true, |text| text.is_empty())
        {
            self.push_task_error(node, "Call activity task requires a target process identifier");
        }
    }

    fn push_task_error(&mut self, _node: &SyntaxNode, message: &str) {
        self.diagnostics
            .push(ResolverDiagnostic::new(ResolverDiagnosticSeverity::Error, message));
    }

    fn collect_expressions(&self, node: &SyntaxNode) -> Vec<Expression> {
        node.children
            .iter()
            .filter_map(|child_id| self.document.node(*child_id))
            .filter(|child| child.kind == SyntaxKind::Expression)
            .filter_map(|child| {
                let mut key = None;
                let mut value = None;
                for token in self.token_slice(child) {
                    match token.kind {
                        TokenKind::BracketCommand => key = Some(token.text.clone()),
                        TokenKind::CommandValue | TokenKind::StringLiteral | TokenKind::Identifier => {
                            value = Some(normalize_command_value(&token.text))
                        }
                        _ => {}
                    }
                }
                Some(Expression::new(key?, value?))
            })
            .collect()
    }

    fn collect_flows(&self, node: &SyntaxNode, from_uid: Uid) -> Vec<PendingFlow> {
        node.children
            .iter()
            .filter_map(|child_id| self.document.node(*child_id))
            .filter(|child| child.kind == SyntaxKind::Flow)
            .filter_map(|child| {
                let first_token = self.token_slice(child).first()?;
                let flow_type = match first_token.text.as_str() {
                    "->" | "~>" => FlowType::Message,
                    ".>" => FlowType::DataAssociation,
                    _ => FlowType::Sequence,
                };
                Some(PendingFlow {
                    from_uid,
                    target_id: child.text_name.clone()?,
                    flow_type,
                })
            })
            .collect()
    }

    fn collect_bounds(&mut self, node: &SyntaxNode) -> Option<Bounds> {
        let bounds_node = node
            .children
            .iter()
            .filter_map(|child_id| self.document.node(*child_id))
            .find(|child| child.kind == SyntaxKind::Bounds)?;

        let numbers: Vec<f64> = self
            .token_slice(bounds_node)
            .iter()
            .filter(|token| token.kind == TokenKind::Number)
            .filter_map(|token| token.text.parse::<f64>().ok())
            .collect();

        if numbers.len() != 4 {
            self.diagnostics.push(ResolverDiagnostic::new(
                ResolverDiagnosticSeverity::Error,
                "Bounds must contain exactly four numbers",
            ));
            return None;
        }

        match Bounds::new(numbers[0], numbers[1], numbers[2], numbers[3]) {
            Ok(bounds) => Some(bounds),
            Err(err) => {
                self.diagnostics.push(ResolverDiagnostic::new(
                    ResolverDiagnosticSeverity::Error,
                    err.to_string(),
                ));
                None
            }
        }
    }

    fn capture_layout_bounds(&mut self, node: &SyntaxNode, uid: Uid, layout: &mut Layout) {
        if let Some(bounds) = self.collect_bounds(node) {
            layout.set(uid, bounds);
        }
    }

    fn element_name(&self, node: &SyntaxNode, id: &str) -> String {
        node.display_name
            .clone()
            .unwrap_or_else(|| name_from_id(id))
    }

    fn lower_layout_section(&mut self, node: &SyntaxNode, collab: &Collab) -> Option<Layout> {
        let mut grouped = HashMap::<String, PoolLayoutData>::new();

        for child_id in &node.children {
            let Some(pool_node) = self.document.node(*child_id) else {
                continue;
            };
            if pool_node.kind != SyntaxKind::Pool {
                continue;
            }

            let Some(pool_id) = pool_node.text_name.clone() else {
                continue;
            };
            let entry = grouped.entry(pool_id.clone()).or_default();
            entry.pool = self.collect_bounds(pool_node);

            for lane_id in &pool_node.children {
                let Some(lane_node) = self.document.node(*lane_id) else {
                    continue;
                };
                if lane_node.kind != SyntaxKind::Lane {
                    continue;
                }

                let Some(lane_name) = lane_node.text_name.clone() else {
                    continue;
                };
                if let Some(bounds) = self.collect_bounds(lane_node) {
                    entry.lanes.insert(lane_name.clone(), bounds);
                }

                for stage_id in &lane_node.children {
                    let Some(stage_node) = self.document.node(*stage_id) else {
                        continue;
                    };
                    if stage_node.kind != SyntaxKind::Stage {
                        continue;
                    }

                    let Some(stage_name) = stage_node.text_name.clone() else {
                        continue;
                    };
                    if let Some(bounds) = self.collect_bounds(stage_node) {
                        entry.stages.insert(stage_name.clone(), bounds);
                    }

                    for element_id in &stage_node.children {
                        let Some(element_node) = self.document.node(*element_id) else {
                            continue;
                        };
                        if !matches!(
                            element_node.kind,
                            SyntaxKind::Event | SyntaxKind::Task | SyntaxKind::Gateway
                        ) {
                            continue;
                        }

                        let Some(element_name) = element_node.text_name.clone() else {
                            continue;
                        };
                        if let Some(bounds) = self.collect_bounds(element_node) {
                            entry.elements.insert(element_name, bounds);
                        }
                    }
                }
            }
        }

        if grouped.is_empty() {
            return None;
        }

        match layout_from_grouped(collab, &grouped) {
            Ok(layout) => Some(layout),
            Err(err) => {
                self.diagnostics.push(ResolverDiagnostic::new(
                    ResolverDiagnosticSeverity::Error,
                    err.to_string(),
                ));
                None
            }
        }
    }

    fn first_token_kind(&self, node: &SyntaxNode) -> Option<TokenKind> {
        self.token_slice(node).first().map(|token| token.kind)
    }

    fn token_slice(&self, node: &SyntaxNode) -> &[crate::syntax::SyntaxToken] {
        let Some(range) = node.token_range else {
            return &[];
        };
        self.document
            .tokens
            .get(range.start..range.end)
            .unwrap_or_default()
    }

    fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|diagnostic| diagnostic.severity == ResolverDiagnosticSeverity::Error)
    }
}

#[derive(Debug)]
struct PendingFlow {
    from_uid: Uid,
    target_id: String,
    flow_type: FlowType,
}

#[derive(Debug, Default)]
struct UidAllocator {
    next_uid: Uid,
}

impl UidAllocator {
    fn next(&mut self) -> Uid {
        self.next_uid += 1;
        self.next_uid
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
