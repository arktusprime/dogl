use std::collections::HashMap;

use crate::domain::{
    Collab, DoglFile, Element, EventCode, GatewayCode, Layout, Pool, TaskCode,
};
use crate::domain::Identifiable;

use super::ApplicationError;

const INDENT: &str = "    ";

pub fn render(file: &DoglFile) -> Result<String, ApplicationError> {
    let mut lines = Vec::<String>::new();

    for (collab_index, collab) in file.collabs.iter().enumerate() {
        if collab_index > 0 {
            lines.push(String::new());
        }
        render_collab_body(collab, &mut lines);
    }

    let any_layout = file.collabs.iter().any(|collab| collab.layout.is_some());
    if any_layout {
        if !lines.is_empty() {
            lines.push(String::new());
        }
        lines.push("layout".to_string());
        for collab in &file.collabs {
            render_collab_layout(collab, &mut lines)?;
        }
    }

    Ok(lines.join("\n") + "\n")
}

fn render_collab_body(collab: &Collab, lines: &mut Vec<String>) {
    lines.push(format!("collab {}", collab.id));
    for pool in &collab.pools {
        lines.push(indent(1, format!("== {}", pool.id)));
        let lane_groups = lane_stage_groups(pool);
        for lane in &pool.lanes {
            lines.push(indent(2, format!("-- {}", lane.id)));
            if let Some(stages) = lane_groups.get(&lane.id) {
                for stage in stages {
                    lines.push(indent(3, format!("|| {}", stage.stage_id)));
                    let outgoing = outgoing_targets(pool);
                    for element in &stage.elements {
                        lines.push(indent(4, render_element_header(element)));
                        if let Some(target_ids) = outgoing.get(&element.uid()) {
                            for target_id in target_ids {
                                lines.push(indent(5, format!("=> {target_id}")));
                            }
                        }
                    }
                }
            }
        }
    }
}

fn render_collab_layout(collab: &Collab, lines: &mut Vec<String>) -> Result<(), ApplicationError> {
    let layout = collab
        .layout
        .as_ref()
        .ok_or_else(|| ApplicationError::Serialize("cannot write layout section without layout data".to_string()))?;

    for pool in &collab.pools {
        lines.push(indent(
            1,
            format!("== {}{}", pool.id, render_layout_bounds(layout, pool.uid)),
        ));
        let lane_groups = lane_stage_groups(pool);
        for lane in &pool.lanes {
            lines.push(indent(
                2,
                format!("-- {}{}", lane.id, render_layout_bounds(layout, lane.uid)),
            ));
            if let Some(stages) = lane_groups.get(&lane.id) {
                for stage in stages {
                    lines.push(indent(3, format!("|| {}", stage.stage_id)));
                    for element in &stage.elements {
                        lines.push(indent(
                            4,
                            format!(
                                "{}{}",
                                render_element_marker(element),
                                render_layout_bounds(layout, element.uid())
                            ),
                        ));
                    }
                }
            }
        }
    }

    Ok(())
}

fn render_element_header(element: &Element) -> String {
    let mut line = render_element_marker(element);
    let expressions = match element {
        Element::Event(event) => &event.expressions,
        Element::Task(task) => &task.expressions,
        Element::Gateway(gateway) => &gateway.expressions,
        Element::Artifact(artifact) => &artifact.expressions,
    };

    for expression in expressions {
        line.push(' ');
        line.push_str(&format!("[{}] {}", expression.key, expression.value));
    }

    line
}

fn render_element_marker(element: &Element) -> String {
    match element {
        Element::Event(event) => match event.code {
            EventCode::Start => format!("(s) {}", event.id),
            EventCode::Intermediate => format!("(i) {}", event.id),
            EventCode::End => format!("(e) {}", event.id),
            EventCode::Inferred => format!("() {}", event.id),
        },
        Element::Task(task) => {
            if task.code == TaskCode::CallActivity {
                format!(
                    "[call] {}",
                    task.call_target.as_deref().unwrap_or(task.id.as_str())
                )
            } else {
                format!("[] {}", task.id)
            }
        }
        Element::Gateway(gateway) => match gateway.code {
            GatewayCode::Inclusive => format!("<> {}", gateway.id),
            GatewayCode::Exclusive => format!("<x> {}", gateway.id),
            GatewayCode::Parallel => format!("<p> {}", gateway.id),
            GatewayCode::Complex => format!("<c> {}", gateway.id),
            GatewayCode::EventBased => format!("<eb> {}", gateway.id),
        },
        Element::Artifact(artifact) => format!("{{}} {}", artifact.id),
    }
}

fn render_layout_bounds(layout: &Layout, uid: crate::domain::Uid) -> String {
    let Some(bounds) = layout.get(uid) else {
        return String::new();
    };
    format!(
        " {{{} {} {} {}}}",
        format_number(bounds.x()),
        format_number(bounds.y()),
        format_number(bounds.w()),
        format_number(bounds.h())
    )
}

fn format_number(value: f64) -> String {
    if value.fract().abs() < 1e-9 {
        format!("{}", value as i64)
    } else {
        let mut formatted = format!("{value:.2}");
        while formatted.contains('.') && formatted.ends_with('0') {
            formatted.pop();
        }
        if formatted.ends_with('.') {
            formatted.pop();
        }
        formatted
    }
}

fn indent(level: usize, text: String) -> String {
    format!("{}{}", INDENT.repeat(level), text)
}

fn outgoing_targets(pool: &Pool) -> HashMap<crate::domain::Uid, Vec<String>> {
    let mut ids_by_uid = HashMap::new();
    for quadrant in &pool.quadrants {
        for element in &quadrant.elements {
            ids_by_uid.insert(element.uid(), element.id().to_string());
        }
    }

    let mut outgoing = HashMap::<crate::domain::Uid, Vec<String>>::new();
    for flow in &pool.sequence_flows {
        if let Some(target_id) = ids_by_uid.get(&flow.to_uid) {
            outgoing
                .entry(flow.from_uid)
                .or_default()
                .push(target_id.clone());
        }
    }
    outgoing
}

struct LaneStageGroup<'a> {
    stage_id: &'a str,
    elements: Vec<&'a Element>,
}

fn lane_stage_groups<'a>(pool: &'a Pool) -> HashMap<String, Vec<LaneStageGroup<'a>>> {
    let mut groups = HashMap::<String, Vec<LaneStageGroup<'a>>>::new();
    for quadrant in &pool.quadrants {
        groups
            .entry(quadrant.lane_id.clone())
            .or_default()
            .push(LaneStageGroup {
                stage_id: quadrant.stage_id.as_str(),
                elements: quadrant.elements.iter().collect(),
            });
    }
    groups
}
