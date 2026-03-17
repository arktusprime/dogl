//! Semantic validation beyond parse-time diagnostics.

use std::collections::{HashMap, HashSet};

use crate::{
    domain::{DoglFile, Element, EventCode, Pool, TaskCode, Uid},
    syntax::{Span, SyntaxDocument, SyntaxKind},
};
use crate::domain::Identifiable;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationDiagnostic {
    pub severity: ValidationSeverity,
    pub message: String,
    pub span: Option<Span>,
    pub metadata: ValidationDiagnosticMetadata,
}

impl ValidationDiagnostic {
    pub fn new(severity: ValidationSeverity, message: impl Into<String>) -> Self {
        Self {
            severity,
            message: message.into(),
            span: None,
            metadata: ValidationDiagnosticMetadata::default(),
        }
    }

    pub fn with_span(mut self, span: Span) -> Self {
        self.span = Some(span);
        self
    }

    pub fn with_code(mut self, code: &'static str) -> Self {
        self.metadata.code = Some(code);
        self
    }

    pub fn with_related_span(mut self, span: Span) -> Self {
        self.metadata.related_spans.push(span);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ValidationDiagnosticMetadata {
    pub code: Option<&'static str>,
    pub related_spans: Vec<Span>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationSeverity {
    Error,
    Warning,
    Note,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ValidationReport {
    pub diagnostics: Vec<ValidationDiagnostic>,
}

impl ValidationReport {
    pub fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|diagnostic| diagnostic.severity == ValidationSeverity::Error)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum SourceKey {
    File,
    Collab {
        collab_id: String,
    },
    Pool {
        collab_id: String,
        pool_id: String,
    },
    Element {
        collab_id: String,
        pool_id: String,
        element_id: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ValidationSourceMap {
    spans: HashMap<SourceKey, Span>,
}

impl ValidationSourceMap {
    pub fn from_syntax(document: &SyntaxDocument) -> Self {
        let mut source_map = Self::default();
        let Some(root_id) = document.root else {
            return source_map;
        };
        let Some(root) = document.node(root_id) else {
            return source_map;
        };

        source_map.remember(SourceKey::File, root.span);
        for collab_id in &root.children {
            let Some(collab_node) = document.node(*collab_id) else {
                continue;
            };
            if collab_node.kind != SyntaxKind::Collab {
                continue;
            }

            let Some(collab_name) = collab_node.text_name.clone() else {
                continue;
            };

            source_map.remember(
                SourceKey::Collab {
                    collab_id: collab_name.clone(),
                },
                collab_node.span,
            );

            for pool_id in &collab_node.children {
                let Some(pool_node) = document.node(*pool_id) else {
                    continue;
                };
                if pool_node.kind != SyntaxKind::Pool {
                    continue;
                }

                let Some(pool_name) = pool_node.text_name.clone() else {
                    continue;
                };

                source_map.remember(
                    SourceKey::Pool {
                        collab_id: collab_name.clone(),
                        pool_id: pool_name.clone(),
                    },
                    pool_node.span,
                );

                for lane_id in &pool_node.children {
                    let Some(lane_node) = document.node(*lane_id) else {
                        continue;
                    };
                    if lane_node.kind != SyntaxKind::Lane {
                        continue;
                    }

                    for stage_id in &lane_node.children {
                        let Some(stage_node) = document.node(*stage_id) else {
                            continue;
                        };
                        if stage_node.kind != SyntaxKind::Stage {
                            continue;
                        }

                        for element_id in &stage_node.children {
                            let Some(element_node) = document.node(*element_id) else {
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

                            source_map.remember(
                                SourceKey::Element {
                                    collab_id: collab_name.clone(),
                                    pool_id: pool_name.clone(),
                                    element_id: element_name,
                                },
                                element_node.span,
                            );
                        }
                    }
                }
            }
        }

        source_map
    }

    pub fn file_span(&self) -> Option<Span> {
        self.spans.get(&SourceKey::File).copied()
    }

    pub fn collab_span(&self, collab_id: &str) -> Option<Span> {
        self.spans
            .get(&SourceKey::Collab {
                collab_id: collab_id.to_string(),
            })
            .copied()
    }

    pub fn pool_span(&self, collab_id: &str, pool_id: &str) -> Option<Span> {
        self.spans
            .get(&SourceKey::Pool {
                collab_id: collab_id.to_string(),
                pool_id: pool_id.to_string(),
            })
            .copied()
    }

    pub fn element_span(&self, collab_id: &str, pool_id: &str, element_id: &str) -> Option<Span> {
        self.spans
            .get(&SourceKey::Element {
                collab_id: collab_id.to_string(),
                pool_id: pool_id.to_string(),
                element_id: element_id.to_string(),
            })
            .copied()
    }

    fn remember(&mut self, key: SourceKey, span: Option<Span>) {
        let Some(span) = span else {
            return;
        };
        self.spans.entry(key).or_insert(span);
    }
}

pub fn validate(file: &DoglFile) -> ValidationReport {
    Validator::new(file, None).validate()
}

pub fn validate_with_source_map(
    file: &DoglFile,
    source_map: &ValidationSourceMap,
) -> ValidationReport {
    Validator::new(file, Some(source_map)).validate()
}

struct Validator<'a> {
    file: &'a DoglFile,
    source_map: Option<&'a ValidationSourceMap>,
    diagnostics: Vec<ValidationDiagnostic>,
}

impl<'a> Validator<'a> {
    fn new(file: &'a DoglFile, source_map: Option<&'a ValidationSourceMap>) -> Self {
        Self {
            file,
            source_map,
            diagnostics: Vec::new(),
        }
    }

    fn validate(mut self) -> ValidationReport {
        if self.file.collabs.is_empty() {
            self.push_file_error(
                "DOGL2000",
                "A .dogl file must contain at least one collaboration",
            );
        }

        let mut first_collab_spans = HashMap::<String, Option<Span>>::new();
        let mut seen_collabs = HashSet::<String>::new();
        for collab in &self.file.collabs {
            if !seen_collabs.insert(collab.id.clone()) {
                self.push_collab_error(
                    &collab.id,
                    "DOGL2001",
                    format!("Duplicate collaboration id `{}`", collab.id),
                    first_collab_spans.get(&collab.id).copied().flatten(),
                );
            } else {
                first_collab_spans.insert(collab.id.clone(), self.collab_span(&collab.id));
            }

            if collab.pools.is_empty() {
                self.push_collab_error(
                    &collab.id,
                    "DOGL2002",
                    format!("Collaboration `{}` must contain at least one pool", collab.id),
                    None,
                );
            }

            self.validate_collab(collab);
        }

        ValidationReport {
            diagnostics: self.diagnostics,
        }
    }

    fn validate_collab(&mut self, collab: &crate::domain::Collab) {
        let mut first_pool_spans = HashMap::<String, Option<Span>>::new();
        let mut seen_pools = HashSet::<String>::new();
        for pool in &collab.pools {
            if !seen_pools.insert(pool.id.clone()) {
                self.push_pool_error(
                    &collab.id,
                    &pool.id,
                    "DOGL2003",
                    format!(
                        "Duplicate pool id `{}` in collaboration `{}`",
                        pool.id, collab.id
                    ),
                    first_pool_spans.get(&pool.id).copied().flatten(),
                );
            } else {
                first_pool_spans
                    .insert(pool.id.clone(), self.pool_span(&collab.id, &pool.id));
            }

            self.validate_pool(&collab.id, pool);
        }
    }

    fn validate_pool(&mut self, collab_id: &str, pool: &Pool) {
        let elements: Vec<&Element> = pool
            .quadrants
            .iter()
            .flat_map(|quadrant| quadrant.elements.iter())
            .collect();
        if elements.is_empty() {
            self.push_pool_error(
                collab_id,
                &pool.id,
                "DOGL2004",
                format!("Pool `{}` must contain at least one element", pool.id),
                None,
            );
            return;
        }

        let known_uids: HashSet<_> = elements.iter().map(|element| element.uid()).collect();
        let elements_by_uid: HashMap<Uid, &Element> =
            elements.iter().map(|element| (element.uid(), *element)).collect();
        let mut incoming = HashMap::new();
        let mut outgoing = HashMap::new();
        let mut adjacency = HashMap::<Uid, Vec<Uid>>::new();
        for flow in &pool.sequence_flows {
            let from_known = known_uids.contains(&flow.from_uid);
            let to_known = known_uids.contains(&flow.to_uid);
            if !known_uids.contains(&flow.from_uid) {
                self.push_pool_error(
                    collab_id,
                    &pool.id,
                    "DOGL2205",
                    format!(
                        "Sequence flow in pool `{}` references unknown source uid `{}`",
                        pool.id, flow.from_uid
                    ),
                    None,
                );
            }
            if !known_uids.contains(&flow.to_uid) {
                self.push_pool_error(
                    collab_id,
                    &pool.id,
                    "DOGL2206",
                    format!(
                        "Sequence flow in pool `{}` references unknown target uid `{}`",
                        pool.id, flow.to_uid
                    ),
                    None,
                );
            }
            if from_known {
                *outgoing.entry(flow.from_uid).or_insert(0usize) += 1;
            }
            if to_known {
                *incoming.entry(flow.to_uid).or_insert(0usize) += 1;
            }
            if from_known && to_known {
                adjacency.entry(flow.from_uid).or_default().push(flow.to_uid);
                adjacency.entry(flow.to_uid).or_default().push(flow.from_uid);
            }
        }

        for element in &elements {
            let incoming_count = incoming.get(&element.uid()).copied().unwrap_or(0);
            let outgoing_count = outgoing.get(&element.uid()).copied().unwrap_or(0);
            self.validate_element(collab_id, &pool.id, element, incoming_count, outgoing_count);
        }

        self.validate_pool_components(collab_id, &pool.id, &elements_by_uid, &adjacency);
    }

    fn validate_element(
        &mut self,
        collab_id: &str,
        pool_id: &str,
        element: &Element,
        incoming_count: usize,
        outgoing_count: usize,
    ) {
        match element {
            Element::Task(task) => {
                if task.code == TaskCode::CallActivity
                    && task
                        .call_target
                        .as_deref()
                        .is_none_or(|target| target.trim().is_empty())
                {
                    self.push_element_error(
                        collab_id,
                        pool_id,
                        &task.id,
                        "DOGL2100",
                        format!(
                            "Call activity `{}` must declare a target process identifier",
                            task.id
                        ),
                        None,
                    );
                }
                if task.code != TaskCode::CallActivity && task.call_target.is_some() {
                    self.push_element_error(
                        collab_id,
                        pool_id,
                        &task.id,
                        "DOGL2101",
                        format!(
                            "Task `{}` sets `call_target`, but only `[call]` tasks may do that",
                            task.id
                        ),
                        None,
                    );
                }
                if incoming_count == 0 {
                    self.push_element_error(
                        collab_id,
                        pool_id,
                        &task.id,
                        "DOGL2207",
                        format!(
                            "Element `{}` must have an incoming sequence flow",
                            task.id
                        ),
                        None,
                    );
                }
                if outgoing_count == 0 {
                    self.push_element_error(
                        collab_id,
                        pool_id,
                        &task.id,
                        "DOGL2208",
                        format!(
                            "Element `{}` must have an outgoing sequence flow",
                            task.id
                        ),
                        None,
                    );
                }
            }
            Element::Event(event) => match event.code {
                EventCode::Start => {
                    if incoming_count > 0 {
                        self.push_element_error(
                            collab_id,
                            pool_id,
                            &event.id,
                            "DOGL2200",
                            format!("Start event `{}` must not have incoming sequence flows", event.id),
                            None,
                        );
                    }
                    if outgoing_count == 0 {
                        self.push_element_error(
                            collab_id,
                            pool_id,
                            &event.id,
                            "DOGL2201",
                            format!("Start event `{}` must have an outgoing sequence flow", event.id),
                            None,
                        );
                    }
                }
                EventCode::End => {
                    if outgoing_count > 0 {
                        self.push_element_error(
                            collab_id,
                            pool_id,
                            &event.id,
                            "DOGL2202",
                            format!("End event `{}` must not have outgoing sequence flows", event.id),
                            None,
                        );
                    }
                    if incoming_count == 0 {
                        self.push_element_error(
                            collab_id,
                            pool_id,
                            &event.id,
                            "DOGL2203",
                            format!("End event `{}` must have an incoming sequence flow", event.id),
                            None,
                        );
                    }
                }
                EventCode::Intermediate | EventCode::Inferred => {
                    if incoming_count == 0 {
                        self.push_element_error(
                            collab_id,
                            pool_id,
                            &event.id,
                            "DOGL2207",
                            format!(
                                "Element `{}` must have an incoming sequence flow",
                                event.id
                            ),
                            None,
                        );
                    }
                    if outgoing_count == 0 {
                        self.push_element_error(
                            collab_id,
                            pool_id,
                            &event.id,
                            "DOGL2208",
                            format!("Element `{}` must have an outgoing sequence flow", event.id),
                            None,
                        );
                    }
                }
            },
            Element::Gateway(gateway) => {
                if incoming_count == 0 {
                    self.push_element_error(
                        collab_id,
                        pool_id,
                        &gateway.id,
                        "DOGL2207",
                        format!(
                            "Element `{}` must have an incoming sequence flow",
                            gateway.id
                        ),
                        None,
                    );
                }
                if outgoing_count == 0 {
                    self.push_element_error(
                        collab_id,
                        pool_id,
                        &gateway.id,
                        "DOGL2208",
                        format!(
                            "Element `{}` must have an outgoing sequence flow",
                            gateway.id
                        ),
                        None,
                    );
                }
            }
            Element::Artifact(artifact) => {
                if incoming_count == 0 {
                    self.push_element_error(
                        collab_id,
                        pool_id,
                        &artifact.id,
                        "DOGL2207",
                        format!(
                            "Element `{}` must have an incoming sequence flow",
                            artifact.id
                        ),
                        None,
                    );
                }
                if outgoing_count == 0 {
                    self.push_element_error(
                        collab_id,
                        pool_id,
                        &artifact.id,
                        "DOGL2208",
                        format!(
                            "Element `{}` must have an outgoing sequence flow",
                            artifact.id
                        ),
                        None,
                    );
                }
            }
        }
    }

    fn validate_pool_components(
        &mut self,
        collab_id: &str,
        pool_id: &str,
        elements_by_uid: &HashMap<Uid, &Element>,
        adjacency: &HashMap<Uid, Vec<Uid>>,
    ) {
        let mut visited = HashSet::<Uid>::new();
        let mut ordered_uids: Vec<_> = elements_by_uid.keys().copied().collect();
        ordered_uids.sort_unstable();

        for start_uid in ordered_uids {
            if !visited.insert(start_uid) {
                continue;
            }

            let mut stack = vec![start_uid];
            let mut component = vec![start_uid];
            while let Some(uid) = stack.pop() {
                for neighbor in adjacency.get(&uid).into_iter().flatten() {
                    if visited.insert(*neighbor) {
                        stack.push(*neighbor);
                        component.push(*neighbor);
                    }
                }
            }

            let mut has_start = false;
            let mut has_end = false;
            for uid in &component {
                let Some(element) = elements_by_uid.get(uid) else {
                    continue;
                };
                match element {
                    Element::Event(event) if event.code == EventCode::Start => has_start = true,
                    Element::Event(event) if event.code == EventCode::End => has_end = true,
                    _ => {}
                }
            }

            let Some(anchor) = component
                .iter()
                .filter_map(|uid| elements_by_uid.get(uid).copied())
                .min_by_key(|element| element.uid())
            else {
                continue;
            };

            if !has_start {
                self.push_element_error(
                    collab_id,
                    pool_id,
                    anchor.id(),
                    "DOGL2210",
                    format!(
                        "Graph containing element `{}` must contain a Start event",
                        anchor.id()
                    ),
                    None,
                );
            }
            if !has_end {
                self.push_element_error(
                    collab_id,
                    pool_id,
                    anchor.id(),
                    "DOGL2211",
                    format!(
                        "Graph containing element `{}` must contain an End event",
                        anchor.id()
                    ),
                    None,
                );
            }
        }
    }

    fn push_file_error(&mut self, code: &'static str, message: impl Into<String>) {
        let mut diagnostic = ValidationDiagnostic::new(ValidationSeverity::Error, message).with_code(code);
        if let Some(span) = self.source_map.and_then(ValidationSourceMap::file_span) {
            diagnostic = diagnostic.with_span(span);
        }
        self.diagnostics.push(diagnostic);
    }

    fn push_collab_error(
        &mut self,
        collab_id: &str,
        code: &'static str,
        message: impl Into<String>,
        related_span: Option<Span>,
    ) {
        let mut diagnostic = ValidationDiagnostic::new(ValidationSeverity::Error, message).with_code(code);
        if let Some(span) = self.collab_span(collab_id) {
            diagnostic = diagnostic.with_span(span);
        }
        if let Some(span) = related_span {
            diagnostic = diagnostic.with_related_span(span);
        }
        self.diagnostics.push(diagnostic);
    }

    fn push_pool_error(
        &mut self,
        collab_id: &str,
        pool_id: &str,
        code: &'static str,
        message: impl Into<String>,
        related_span: Option<Span>,
    ) {
        let mut diagnostic = ValidationDiagnostic::new(ValidationSeverity::Error, message).with_code(code);
        if let Some(span) = self.pool_span(collab_id, pool_id) {
            diagnostic = diagnostic.with_span(span);
        }
        if let Some(span) = related_span {
            diagnostic = diagnostic.with_related_span(span);
        }
        self.diagnostics.push(diagnostic);
    }

    fn push_element_error(
        &mut self,
        collab_id: &str,
        pool_id: &str,
        element_id: &str,
        code: &'static str,
        message: impl Into<String>,
        related_span: Option<Span>,
    ) {
        let mut diagnostic = ValidationDiagnostic::new(ValidationSeverity::Error, message).with_code(code);
        if let Some(span) = self.element_span(collab_id, pool_id, element_id) {
            diagnostic = diagnostic.with_span(span);
        }
        if let Some(span) = related_span {
            diagnostic = diagnostic.with_related_span(span);
        }
        self.diagnostics.push(diagnostic);
    }

    fn collab_span(&self, collab_id: &str) -> Option<Span> {
        self.source_map
            .and_then(|source_map| source_map.collab_span(collab_id))
    }

    fn pool_span(&self, collab_id: &str, pool_id: &str) -> Option<Span> {
        self.source_map
            .and_then(|source_map| source_map.pool_span(collab_id, pool_id))
    }

    fn element_span(&self, collab_id: &str, pool_id: &str, element_id: &str) -> Option<Span> {
        self.source_map
            .and_then(|source_map| source_map.element_span(collab_id, pool_id, element_id))
    }
}
