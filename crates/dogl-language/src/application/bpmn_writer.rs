use std::collections::HashMap;

use crate::domain::{
    Bounds, Collab, DoglFile, Element, EventCode, Flow, FlowType, GatewayCode, Layout, Pool,
    TaskCode, Uid,
};
use crate::domain::Identifiable;
use crate::domain::name_from_id;

use super::{ApplicationError, BpmnExport};

pub fn render(file: &DoglFile) -> Result<BpmnExport, ApplicationError> {
    if file.collabs.len() != 1 {
        return Err(ApplicationError::Serialize(
            "MVP BPMN export supports exactly one collaboration per .dogl file".to_string(),
        ));
    }

    let collab = &file.collabs[0];
    if collab.pools.len() != 1 {
        return Err(ApplicationError::Serialize(
            "MVP BPMN export supports exactly one pool per collaboration".to_string(),
        ));
    }
    let layout = collab.layout.as_ref().ok_or_else(|| {
        ApplicationError::Serialize(
            "BPMN export requires an existing layout in the .dogl file".to_string(),
        )
    })?;
    let pool = &collab.pools[0];

    let element_records = collect_export_elements(pool, layout)?;
    let flow_records = collect_export_flows(pool, &element_records)?;
    
    let mut message_flow_records = Vec::new();
    for (index, mflow) in collab.message_flows.iter().enumerate() {
        message_flow_records.push(ExportFlow {
            id: format!("MessageFlow_{:03}", index + 1),
            flow: mflow,
        });
    }

    let ids = ExportIds::new(collab, pool, &element_records, &flow_records, &message_flow_records);

    let mut xml = String::new();
    xml.push_str(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
    xml.push('\n');
    xml.push_str(&format!(
        r#"<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL" xmlns:bpmndi="http://www.omg.org/spec/BPMN/20100524/DI" xmlns:dc="http://www.omg.org/spec/DD/20100524/DC" xmlns:di="http://www.omg.org/spec/DD/20100524/DI" id="{}" targetNamespace="http://dogl.dev/bpmn" exporter="DOGL" exporterVersion="0.0.0">"#,
        ids.definitions_id
    ));
    xml.push('\n');
    xml.push_str(&format!(
        r#"  <bpmn:collaboration id="{}" name="{}">"#,
        ids.collaboration_id,
        escape_attr(collab.id.as_str())
    ));
    xml.push('\n');
    xml.push_str(&format!(
        r#"    <bpmn:participant id="{}" name="{}" processRef="{}" />"#,
        ids.participant_id,
        escape_attr(pool.id.as_str()),
        ids.process_id
    ));
    xml.push('\n');
    for record in &message_flow_records {
        let source_id = ids
            .node_ids
            .get(&record.flow.from_uid)
            .expect("source node id must exist for message flow");
        let target_id = ids
            .node_ids
            .get(&record.flow.to_uid)
            .expect("target node id must exist for message flow");
        xml.push_str(&format!(
            r#"    <bpmn:messageFlow id="{}" sourceRef="{}" targetRef="{}" />"#,
            record.id, source_id, target_id
        ));
        xml.push('\n');
    }
    xml.push_str("  </bpmn:collaboration>\n");

    xml.push_str(&format!(
        r#"  <bpmn:process id="{}" name="{}" isExecutable="true">"#,
        ids.process_id,
        escape_attr(collab.id.as_str())
    ));
    xml.push('\n');

    xml.push_str(&format!(r#"    <bpmn:laneSet id="{}">"#, ids.lane_set_id));
    xml.push('\n');
    for lane in &pool.lanes {
        let lane_xml_id = ids
            .lane_ids
            .get(&lane.uid)
            .expect("lane id must exist");
        xml.push_str(&format!(
            r#"      <bpmn:lane id="{}" name="{}">"#,
            lane_xml_id,
            escape_attr(lane.id.as_str())
        ));
        xml.push('\n');
        for record in element_records.iter().filter(|record| record.lane_id == lane.id) {
            let node_id = ids
                .node_ids
                .get(&record.uid)
                .expect("node id must exist");
            xml.push_str(&format!(
                r#"        <bpmn:flowNodeRef>{}</bpmn:flowNodeRef>"#,
                node_id
            ));
            xml.push('\n');
        }
        xml.push_str("      </bpmn:lane>\n");
    }
    xml.push_str("    </bpmn:laneSet>\n");

    let mut incoming = HashMap::<Uid, Vec<&str>>::new();
    let mut outgoing = HashMap::<Uid, Vec<&str>>::new();
    for record in &flow_records {
        incoming
            .entry(record.flow.to_uid)
            .or_default()
            .push(record.id.as_str());
        outgoing
            .entry(record.flow.from_uid)
            .or_default()
            .push(record.id.as_str());
    }

    for record in &element_records {
        let node_id = ids
            .node_ids
            .get(&record.uid)
            .expect("node id must exist");
        render_element_opening(&mut xml, record, node_id, &incoming, &outgoing)?;
        xml.push('\n');
        for flow_id in incoming.get(&record.uid).into_iter().flatten() {
            xml.push_str(&format!(r#"      <bpmn:incoming>{}</bpmn:incoming>"#, flow_id));
            xml.push('\n');
        }
        for flow_id in outgoing.get(&record.uid).into_iter().flatten() {
            xml.push_str(&format!(r#"      <bpmn:outgoing>{}</bpmn:outgoing>"#, flow_id));
            xml.push('\n');
        }
        render_element_closing(&mut xml, record)?;
        xml.push('\n');
    }

    for record in &flow_records {
        let source_id = ids
            .node_ids
            .get(&record.flow.from_uid)
            .expect("source node id must exist");
        let target_id = ids
            .node_ids
            .get(&record.flow.to_uid)
            .expect("target node id must exist");
        xml.push_str(&format!(
            r#"    <bpmn:sequenceFlow id="{}" sourceRef="{}" targetRef="{}" />"#,
            record.id, source_id, target_id
        ));
        xml.push('\n');
    }
    xml.push_str("  </bpmn:process>\n");

    xml.push_str(r#"  <bpmndi:BPMNDiagram id="BPMNDiagram_1">"#);
    xml.push('\n');
    xml.push_str(&format!(
        r#"    <bpmndi:BPMNPlane id="BPMNPlane_1" bpmnElement="{}">"#,
        ids.collaboration_id
    ));
    xml.push('\n');

    let pool_bounds = required_bounds(layout, pool.uid, "pool")?;
    xml.push_str(&format!(
        r#"      <bpmndi:BPMNShape id="{}_di" bpmnElement="{}" isHorizontal="true">"#,
        ids.participant_id, ids.participant_id
    ));
    xml.push('\n');
    xml.push_str(&format!(
        r#"        <dc:Bounds x="{}" y="{}" width="{}" height="{}" />"#,
        format_number(pool_bounds.x()),
        format_number(pool_bounds.y()),
        format_number(pool_bounds.w()),
        format_number(pool_bounds.h())
    ));
    xml.push('\n');
    xml.push_str("      </bpmndi:BPMNShape>\n");

    for lane in &pool.lanes {
        let lane_bounds = required_bounds(layout, lane.uid, "lane")?;
        let lane_xml_id = ids
            .lane_ids
            .get(&lane.uid)
            .expect("lane id must exist");
        xml.push_str(&format!(
            r#"      <bpmndi:BPMNShape id="{}_di" bpmnElement="{}" isHorizontal="true">"#,
            lane_xml_id, lane_xml_id
        ));
        xml.push('\n');
        xml.push_str(&format!(
            r#"        <dc:Bounds x="{}" y="{}" width="{}" height="{}" />"#,
            format_number(lane_bounds.x()),
            format_number(lane_bounds.y()),
            format_number(lane_bounds.w()),
            format_number(lane_bounds.h())
        ));
        xml.push('\n');
        xml.push_str("      </bpmndi:BPMNShape>\n");
    }

    for record in &element_records {
        let node_id = ids
            .node_ids
            .get(&record.uid)
            .expect("node id must exist");
        xml.push_str(&format!(
            r#"      <bpmndi:BPMNShape id="{}_di" bpmnElement="{}"{}>"#,
            node_id,
            node_id,
            if matches!(record.element, Element::Gateway(_)) {
                r#" isMarkerVisible="true""#
            } else {
                ""
            }
        ));
        xml.push('\n');
        xml.push_str(&format!(
            r#"        <dc:Bounds x="{}" y="{}" width="{}" height="{}" />"#,
            format_number(record.bounds.x()),
            format_number(record.bounds.y()),
            format_number(record.bounds.w()),
            format_number(record.bounds.h())
        ));
        xml.push('\n');
        if matches!(record.element, Element::Task(_) | Element::Gateway(_)) {
            xml.push_str("        <bpmndi:BPMNLabel />\n");
        }
        xml.push_str("      </bpmndi:BPMNShape>\n");
    }

    for record in &flow_records {
        let source = element_records
            .iter()
            .find(|element| element.uid == record.flow.from_uid)
            .expect("source element must exist");
        let target = element_records
            .iter()
            .find(|element| element.uid == record.flow.to_uid)
            .expect("target element must exist");
        let waypoints = derive_waypoints(&source.bounds, &target.bounds);
        xml.push_str(&format!(
            r#"      <bpmndi:BPMNEdge id="{}_di" bpmnElement="{}">"#,
            record.id, record.id
        ));
        xml.push('\n');
        for (x, y) in waypoints {
            xml.push_str(&format!(
                r#"        <di:waypoint x="{}" y="{}" />"#,
                format_number(x),
                format_number(y)
            ));
            xml.push('\n');
        }
        xml.push_str("      </bpmndi:BPMNEdge>\n");
    }

    for record in &message_flow_records {
        let source = element_records
            .iter()
            .find(|element| element.uid == record.flow.from_uid)
            .expect("source element must exist for message flow");
        let target = element_records
            .iter()
            .find(|element| element.uid == record.flow.to_uid)
            .expect("target element must exist for message flow");
        let waypoints = derive_waypoints(&source.bounds, &target.bounds);
        xml.push_str(&format!(
            r#"      <bpmndi:BPMNEdge id="{}_di" bpmnElement="{}">"#,
            record.id, record.id
        ));
        xml.push('\n');
        for (x, y) in waypoints {
            xml.push_str(&format!(
                r#"        <di:waypoint x="{}" y="{}" />"#,
                format_number(x),
                format_number(y)
            ));
            xml.push('\n');
        }
        xml.push_str("      </bpmndi:BPMNEdge>\n");
    }

    xml.push_str("    </bpmndi:BPMNPlane>\n");
    xml.push_str("  </bpmndi:BPMNDiagram>\n");
    xml.push_str("</bpmn:definitions>\n");

    Ok(BpmnExport { xml })
}

#[derive(Debug)]
struct ExportElement<'a> {
    uid: Uid,
    lane_id: &'a str,
    element: &'a Element,
    bounds: Bounds,
}

#[derive(Debug)]
struct ExportFlow<'a> {
    id: String,
    flow: &'a Flow,
}

struct ExportIds {
    definitions_id: String,
    collaboration_id: String,
    participant_id: String,
    process_id: String,
    lane_set_id: String,
    lane_ids: HashMap<Uid, String>,
    node_ids: HashMap<Uid, String>,
}

impl ExportIds {
    fn new(collab: &Collab, pool: &Pool, elements: &[ExportElement<'_>], flows: &[ExportFlow<'_>], _message_flows: &[ExportFlow<'_>]) -> Self {
        let mut lane_ids = HashMap::new();
        for lane in &pool.lanes {
            lane_ids.insert(lane.uid, prefixed_id("Lane", &lane.uid.to_string()));
        }

        let mut node_ids = HashMap::new();
        for element in elements {
            node_ids.insert(element.uid, element_xml_id(element.element, element.uid));
        }
        for (index, flow) in flows.iter().enumerate() {
            debug_assert_eq!(flow.id, format!("Flow_{:03}", index + 1));
        }

        Self {
            definitions_id: prefixed_id("Definitions", &collab.uid.to_string()),
            collaboration_id: prefixed_id("Collaboration", &collab.uid.to_string()),
            participant_id: prefixed_id("Participant", &pool.uid.to_string()),
            process_id: prefixed_id("Process", &collab.uid.to_string()),
            lane_set_id: prefixed_id("LaneSet", &pool.uid.to_string()),
            lane_ids,
            node_ids,
        }
    }
}

fn collect_export_elements<'a>(
    pool: &'a Pool,
    layout: &Layout,
) -> Result<Vec<ExportElement<'a>>, ApplicationError> {
    let mut elements = Vec::new();
    for quadrant in &pool.quadrants {
        for element in &quadrant.elements {
            if matches!(element, Element::Artifact(_)) {
                return Err(ApplicationError::Serialize(
                    "MVP BPMN export does not support artifact export yet".to_string(),
                ));
            }
            let bounds = required_bounds(layout, element.uid(), "element")?.clone();
            elements.push(ExportElement {
                uid: element.uid(),
                lane_id: quadrant.lane_id.as_str(),
                element,
                bounds,
            });
        }
    }
    if elements.is_empty() {
        return Err(ApplicationError::Serialize(
            "BPMN export requires at least one flow element".to_string(),
        ));
    }
    Ok(elements)
}

fn collect_export_flows<'a>(
    pool: &'a Pool,
    elements: &[ExportElement<'_>],
) -> Result<Vec<ExportFlow<'a>>, ApplicationError> {
    let known_uids: HashMap<_, _> = elements.iter().map(|element| (element.uid, element)).collect();
    let mut flows = Vec::new();
    for (index, flow) in pool.sequence_flows.iter().enumerate() {
        if flow.flow_type != FlowType::Sequence {
            return Err(ApplicationError::Serialize(format!(
                "MVP BPMN export does not support flow type `{:?}` yet",
                flow.flow_type
            )));
        }
        if !known_uids.contains_key(&flow.from_uid) || !known_uids.contains_key(&flow.to_uid) {
            return Err(ApplicationError::Serialize(
                "BPMN export encountered a flow with an unknown endpoint".to_string(),
            ));
        }
        flows.push(ExportFlow {
            id: format!("Flow_{:03}", index + 1),
            flow,
        });
    }
    Ok(flows)
}

fn render_element_opening(
    xml: &mut String,
    record: &ExportElement<'_>,
    node_id: &str,
    incoming: &HashMap<Uid, Vec<&str>>,
    outgoing: &HashMap<Uid, Vec<&str>>,
) -> Result<(), ApplicationError> {
    match record.element {
        Element::Event(event) => match event.code {
            EventCode::Start => xml.push_str(&format!(
                r#"    <bpmn:startEvent id="{}" name="{}">"#,
                node_id,
                escape_attr(exported_element_name(record.element))
            )),
            EventCode::End => xml.push_str(&format!(
                r#"    <bpmn:endEvent id="{}" name="{}">"#,
                node_id,
                escape_attr(exported_element_name(record.element))
            )),
            EventCode::Intermediate | EventCode::Inferred => xml.push_str(&format!(
                r#"    <bpmn:intermediateCatchEvent id="{}" name="{}">"#,
                node_id,
                escape_attr(exported_element_name(record.element))
            )),
        },
        Element::Task(task) => match task.code {
            TaskCode::CallActivity => xml.push_str(&format!(
                r#"    <bpmn:callActivity id="{}" name="{}" calledElement="{}">"#,
                node_id,
                escape_attr(exported_element_name(record.element)),
                prefixed_id(
                    "Process",
                    task.call_target
                        .as_deref()
                        .unwrap_or(task.id.as_str())
                )
            )),
            _ => xml.push_str(&format!(
                r#"    <bpmn:{} id="{}" name="{}">"#,
                task_tag(task.code),
                node_id,
                escape_attr(exported_element_name(record.element))
            )),
        },
        Element::Gateway(gateway) => {
            xml.push_str(&format!(
                r#"    <bpmn:{} id="{}" name="{}" gatewayDirection="{}">"#,
                gateway_tag(gateway.code),
                node_id,
                escape_attr(exported_element_name(record.element)),
                gateway_direction(
                    incoming.get(&record.uid).map_or(0, Vec::len),
                    outgoing.get(&record.uid).map_or(0, Vec::len),
                )
            ));
        }
        Element::Artifact(_) => unreachable!("artifact support is filtered earlier"),
    }
    Ok(())
}

fn render_element_closing(
    xml: &mut String,
    record: &ExportElement<'_>,
) -> Result<(), ApplicationError> {
    match record.element {
        Element::Event(event) => match event.code {
            EventCode::Start => xml.push_str("    </bpmn:startEvent>"),
            EventCode::End => xml.push_str("    </bpmn:endEvent>"),
            EventCode::Intermediate | EventCode::Inferred => xml.push_str("    </bpmn:intermediateCatchEvent>"),
        },
        Element::Task(task) => {
            xml.push_str(&format!("    </bpmn:{}>", task_tag(task.code)));
        }
        Element::Gateway(gateway) => {
            xml.push_str(&format!("    </bpmn:{}>", gateway_tag(gateway.code)));
        }
        Element::Artifact(_) => unreachable!("artifact support is filtered earlier"),
    }
    Ok(())
}

fn task_tag(code: TaskCode) -> &'static str {
    match code {
        TaskCode::Generic => "task",
        TaskCode::Manual => "manualTask",
        TaskCode::User => "userTask",
        TaskCode::Service => "serviceTask",
        TaskCode::Receive => "receiveTask",
        TaskCode::Send | TaskCode::SendMessage => "sendTask",
        TaskCode::Script => "scriptTask",
        TaskCode::BusinessRule => "businessRuleTask",
        TaskCode::ReceiveMessage => "receiveTask",
        TaskCode::CallActivity => "callActivity",
    }
}

fn gateway_tag(code: GatewayCode) -> &'static str {
    match code {
        GatewayCode::Inclusive => "inclusiveGateway",
        GatewayCode::Exclusive => "exclusiveGateway",
        GatewayCode::Parallel => "parallelGateway",
        GatewayCode::Complex => "complexGateway",
        GatewayCode::EventBased => "eventBasedGateway",
    }
}

fn gateway_direction(incoming: usize, outgoing: usize) -> &'static str {
    if incoming <= 1 && outgoing > 1 {
        "Diverging"
    } else if incoming > 1 && outgoing <= 1 {
        "Converging"
    } else if incoming > 1 && outgoing > 1 {
        "Mixed"
    } else {
        "Unspecified"
    }
}

fn exported_element_name(element: &Element) -> &str {
    match element {
        Element::Event(event) => explicit_alias_or_id(&event.name, &event.id),
        Element::Task(task) => explicit_alias_or_id(&task.name, &task.id),
        Element::Gateway(gateway) => explicit_alias_or_id(&gateway.name, &gateway.id),
        Element::Artifact(artifact) => explicit_alias_or_id(&artifact.name, &artifact.id),
    }
}

fn explicit_alias_or_id<'a>(name: &'a str, id: &'a str) -> &'a str {
    if name == name_from_id(id) {
        id
    } else {
        name
    }
}

fn element_xml_id(element: &Element, uid: Uid) -> String {
    match element {
        Element::Event(event) => match event.code {
            EventCode::Start => prefixed_id("StartEvent", &uid.to_string()),
            EventCode::End => prefixed_id("EndEvent", &uid.to_string()),
            EventCode::Intermediate => prefixed_id("IntermediateEvent", &uid.to_string()),
            EventCode::Inferred => prefixed_id("Event", &uid.to_string()),
        },
        Element::Task(task) => match task.code {
            TaskCode::CallActivity => prefixed_id("CallActivity", &uid.to_string()),
            _ => prefixed_id("Activity", &uid.to_string()),
        },
        Element::Gateway(_) => prefixed_id("Gateway", &uid.to_string()),
        Element::Artifact(_) => prefixed_id("Artifact", &uid.to_string()),
    }
}

fn required_bounds<'a>(
    layout: &'a Layout,
    uid: Uid,
    subject: &str,
) -> Result<&'a Bounds, ApplicationError> {
    layout.get(uid).ok_or_else(|| {
        ApplicationError::Serialize(format!(
            "BPMN export requires {} bounds for uid `{}`",
            subject, uid
        ))
    })
}

fn derive_waypoints(source: &Bounds, target: &Bounds) -> Vec<(f64, f64)> {
    let source_right = source.x() + source.w();
    let source_top = source.y();
    let source_bottom = source.y() + source.h();
    let source_center_x = source.x() + source.w() / 2.0;
    let source_center_y = source.y() + source.h() / 2.0;
    
    let target_left = target.x();
    let target_top = target.y();
    let target_bottom = target.y() + target.h();
    let target_center_x = target.x() + target.w() / 2.0;
    let target_center_y = target.y() + target.h() / 2.0;

    let cell_width = 200.0;
    let cell_height = 160.0;

    let source_cell_top = source_center_y - cell_height / 2.0;
    let source_cell_bottom = source_center_y + cell_height / 2.0;

    let target_cell_left = target_center_x - cell_width / 2.0;
    let target_cell_right = target_center_x + cell_width / 2.0;
    let target_cell_top = target_center_y - cell_height / 2.0;
    let target_cell_bottom = target_center_y + cell_height / 2.0;

    let is_forward = target_center_x > source_center_x;

    if is_forward {
        if (source_center_y - target_center_y).abs() < f64::EPSILON {
            vec![(source_right, source_center_y), (target_left, target_center_y)]
        } else {
            // Forward flows that go up or down run along the left edge of the target cell.
            let channel_x = target_cell_left;
            vec![
                (source_right, source_center_y),
                (channel_x, source_center_y),
                (channel_x, target_center_y),
                (target_left, target_center_y),
            ]
        }
    } else {
        // Backward flows: target is to the left or shares the column.
        let backward_channel_x = target_cell_right; // Along the right edge of the target cell.

        if (source_center_x - target_center_x).abs() < f64::EPSILON {
            // Same column.
            if target_center_y < source_center_y {
                vec![(source_center_x, source_top), (target_center_x, target_bottom)]
            } else {
                vec![(source_center_x, source_bottom), (target_center_x, target_top)]
            }
        } else if target_center_y < source_center_y - 1.0 {
            // Target is ABOVE
            let channel_y1 = source_cell_top; // Along the top edge of the source cell.
            let channel_y2 = target_cell_bottom; // Along the bottom edge of the target cell.
            vec![
                (source_center_x, source_top),
                (source_center_x, channel_y1),
                (backward_channel_x, channel_y1),
                (backward_channel_x, channel_y2),
                (target_center_x, channel_y2),
                (target_center_x, target_bottom),
            ]
        } else if target_center_y > source_center_y + 1.0 {
            // Target is BELOW
            let channel_y1 = source_cell_bottom; // Along the bottom edge of the source cell.
            let channel_y2 = target_cell_top; // Along the top edge of the target cell.
            vec![
                (source_center_x, source_bottom),
                (source_center_x, channel_y1),
                (backward_channel_x, channel_y1),
                (backward_channel_x, channel_y2),
                (target_center_x, channel_y2),
                (target_center_x, target_top),
            ]
        } else {
            // Target is SAME LEVEL
            let channel_y = source_cell_bottom.max(target_cell_bottom); // Along the bottom of both cells.
            vec![
                (source_center_x, source_bottom),
                (source_center_x, channel_y),
                (target_center_x, channel_y),
                (target_center_x, target_bottom),
            ]
        }
    }
}

fn prefixed_id(prefix: &str, uid_str: &str) -> String {
    format!("{prefix}_{uid_str}")
}

fn escape_attr(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{
        Bounds, Collab, DoglFile, Element, Event, Flow, FlowType, Lane, Layout, Pool, Quadrant,
        Task, name_from_id,
    };

    #[test]
    fn derive_waypoints_prefers_straight_lines_for_same_row() {
        let source = Bounds::new(10.0, 20.0, 100.0, 80.0).unwrap();
        let target = Bounds::new(210.0, 20.0, 100.0, 80.0).unwrap();

        let waypoints = derive_waypoints(&source, &target);
        assert_eq!(waypoints, vec![(110.0, 60.0), (210.0, 60.0)]);
    }

    #[test]
    fn render_requires_existing_layout() {
        let file = DoglFile::new(vec![]);
        assert!(matches!(
            render(&file),
            Err(ApplicationError::Serialize(message))
                if message.contains("exactly one collaboration")
        ));
    }

    #[test]
    fn render_rejects_artifacts() {
        let mut pool = Pool::new(2, "MainPool");
        pool.lanes.push(Lane::new(3, "Ops"));
        let mut quadrant = Quadrant::new(4, "Ops", "Default");
        quadrant.elements.push(Element::Artifact(crate::domain::Artifact {
            uid: 10,
            id: "Doc".to_string(),
            name: name_from_id("Doc"),
            code: crate::domain::ArtifactCode::Default,
            expressions: vec![],
        }));
        pool.quadrants.push(quadrant);
        let mut collab = Collab::new(1, "ArtifactProcess");
        collab.pools.push(pool);
        collab.layout = Some(Layout::default());
        let file = DoglFile::new(vec![collab]);

        assert!(matches!(
            render(&file),
            Err(ApplicationError::Serialize(message))
                if message.contains("artifact export")
        ));
    }

    #[test]
    fn render_produces_call_activity_xml() {
        let mut pool = Pool::new(2, "MainPool");
        pool.lanes.push(Lane::new(3, "Ops"));
        let mut quadrant = Quadrant::new(4, "Ops", "Default");
        quadrant.elements.push(Element::Event(Event {
            uid: 10,
            id: "Start".to_string(),
            name: name_from_id("Start"),
            code: EventCode::Start,
            expressions: vec![],
        }));
        quadrant.elements.push(Element::Task(Task {
            uid: 11,
            id: "ChildProcess".to_string(),
            name: name_from_id("ChildProcess"),
            code: TaskCode::CallActivity,
            call_target: Some("ChildProcess".to_string()),
            expressions: vec![],
        }));
        quadrant.elements.push(Element::Event(Event {
            uid: 12,
            id: "Done".to_string(),
            name: name_from_id("Done"),
            code: EventCode::End,
            expressions: vec![],
        }));
        pool.sequence_flows.push(Flow::new(20, 10, 11, FlowType::Sequence));
        pool.sequence_flows.push(Flow::new(21, 11, 12, FlowType::Sequence));
        pool.quadrants.push(quadrant);

        let mut layout = Layout::default();
        layout.set(2, Bounds::new(0.0, 0.0, 600.0, 200.0).unwrap());
        layout.set(3, Bounds::new(0.0, 40.0, 600.0, 160.0).unwrap());
        layout.set(10, Bounds::new(80.0, 102.0, 36.0, 36.0).unwrap());
        layout.set(11, Bounds::new(220.0, 80.0, 100.0, 80.0).unwrap());
        layout.set(12, Bounds::new(420.0, 102.0, 36.0, 36.0).unwrap());

        let mut collab = Collab::new(1, "RefundProcess");
        collab.pools.push(pool);
        collab.layout = Some(layout);
        let file = DoglFile::new(vec![collab]);

        let export = render(&file).expect("bpmn export");
        assert!(export.xml.contains("<bpmn:callActivity"));
        assert!(export.xml.contains(r#"name="ChildProcess""#));
        assert!(!export.xml.contains(r#"name="Child process""#));
        assert!(export
            .xml
            .contains(r#"calledElement="Process_ChildProcess""#));
        assert!(export.xml.contains("<bpmndi:BPMNDiagram"));
    }
}
