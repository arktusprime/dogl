use std::collections::{HashMap, HashSet};

use crate::domain::{
    Bounds, Collab, DoglFile, Element, EventCode, GatewayCode, Layout, Pool, TaskCode, Uid,
};
use crate::domain::Identifiable;

const TASK_WIDTH: f64 = 100.0;
const TASK_HEIGHT: f64 = 80.0;
const EVENT_SIZE: f64 = 36.0;
const GATEWAY_SIZE: f64 = 50.0;
const ARTIFACT_WIDTH: f64 = 100.0;
const ARTIFACT_HEIGHT: f64 = 80.0;

const CELL_WIDTH: f64 = TASK_WIDTH * 2.0;
const CELL_HEIGHT: f64 = TASK_HEIGHT * 2.0;
const POOL_HEADER_WIDTH: f64 = 30.0;
const LANE_LABEL_WIDTH: f64 = 30.0;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LayoutError {
    Unsupported(&'static str),
    InvalidBounds(String),
}

impl std::fmt::Display for LayoutError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LayoutError::Unsupported(message) => write!(f, "{message}"),
            LayoutError::InvalidBounds(message) => write!(f, "{message}"),
        }
    }
}

impl std::error::Error for LayoutError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridBounds {
    pub x: usize,
    pub y: usize,
    pub w: usize,
    pub h: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LayoutComputation {
    pub pool_grid: GridBounds,
    pub lane_grids: HashMap<String, GridBounds>,
    pub element_grids: HashMap<Uid, GridBounds>,
    pub pixel_layout: Layout,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct GridCell {
    x: usize,
    y: usize,
}

#[derive(Debug, Clone)]
struct LaneState {
    id: String,
    rows: usize,
}

#[derive(Debug, Clone)]
struct ElementDescriptor<'a> {
    element: &'a Element,
    lane_id: String,
    decl_index: usize,
}

impl<'a> ElementDescriptor<'a> {
    fn uid(&self) -> Uid {
        self.element.uid()
    }
}

pub fn compute(file: &DoglFile) -> Result<DoglFile, LayoutError> {
    if file.collabs.len() != 1 {
        return Err(LayoutError::Unsupported(
            "POC3 layout supports exactly one collaboration per .dogl file",
        ));
    }

    let mut updated = file.clone();
    let layout = compute_collab(&updated.collabs[0])?;
    updated.collabs[0].layout = Some(layout.pixel_layout);
    Ok(updated)
}

pub fn compute_collab(collab: &Collab) -> Result<LayoutComputation, LayoutError> {
    if collab.pools.len() != 1 {
        return Err(LayoutError::Unsupported(
            "POC3 layout supports exactly one pool per collaboration",
        ));
    }

    let pool = &collab.pools[0];
    if pool.lanes.is_empty() {
        return Err(LayoutError::Unsupported(
            "POC3 layout requires at least one lane in the pool",
        ));
    }

    let lane_order = collect_lane_order(pool);
    let lane_index_by_id: HashMap<_, _> = lane_order
        .iter()
        .enumerate()
        .map(|(index, lane)| (lane.id.clone(), index))
        .collect();

    let descriptors = collect_element_descriptors(pool);
    if descriptors.is_empty() {
        return Err(LayoutError::Unsupported(
            "POC3 layout requires at least one placeable element",
        ));
    }

    let descriptors_by_uid: HashMap<_, _> =
        descriptors.iter().map(|descriptor| (descriptor.uid(), descriptor)).collect();
    let outgoing_by_uid = collect_outgoing_flows(pool);

    let mut lane_states: Vec<_> = lane_order
        .iter()
        .map(|lane| LaneState {
            id: lane.id.clone(),
            rows: 1,
        })
        .collect();
    let mut occupied = HashMap::<String, HashSet<(usize, usize)>>::new();
    let mut placed = HashMap::<Uid, GridCell>::new();
    let mut max_column = 0usize;

    place_start_events(
        &descriptors,
        &lane_order,
        &mut lane_states,
        &mut occupied,
        &mut placed,
    );

    let mut current_column = 0usize;
    while current_column <= max_column {
        let mut sources: Vec<_> = descriptors
            .iter()
            .filter_map(|descriptor| {
                let cell = placed.get(&descriptor.uid())?;
                (cell.x == current_column).then_some((descriptor, *cell))
            })
            .collect();
        sources.sort_by_key(|(descriptor, cell)| {
            (
                *lane_index_by_id
                    .get(&descriptor.lane_id)
                    .unwrap_or(&usize::MAX),
                cell.y,
                descriptor.decl_index,
            )
        });

        for (source, source_cell) in sources {
            let Some(targets) = outgoing_by_uid.get(&source.uid()) else {
                continue;
            };
            let is_gateway_fan_out =
                matches!(source.element, Element::Gateway(_)) && targets.len() > 1;
            let mut fan_out_anchor_row = None;

            for (branch_index, target_uid) in targets.iter().enumerate() {
                if is_gateway_fan_out && fan_out_anchor_row.is_none() {
                    if let Some(existing) = placed.get(target_uid) {
                        fan_out_anchor_row = Some(existing.y);
                    }
                }
                if placed.contains_key(target_uid) {
                    continue;
                }

                let Some(target) = descriptors_by_uid.get(target_uid) else {
                    continue;
                };
                let selected_lane = target.lane_id.clone();
                let base_row = if selected_lane == source.lane_id {
                    source_cell.y
                } else {
                    0
                };
                let preferred_row = if is_gateway_fan_out {
                    match fan_out_anchor_row {
                        Some(anchor_row) if branch_index > 0 => anchor_row + branch_index,
                        _ => base_row,
                    }
                } else {
                    base_row
                };

                let target_cell = place_in_lane(
                    target.uid(),
                    &selected_lane,
                    source_cell.x + 1,
                    preferred_row,
                    &mut lane_states,
                    &mut occupied,
                    &mut placed,
                );
                max_column = max_column.max(target_cell.x);
                if is_gateway_fan_out && fan_out_anchor_row.is_none() {
                    fan_out_anchor_row = Some(target_cell.y);
                }
            }
        }

        current_column += 1;
    }

    let pool_grid_width = placed.values().map(|cell| cell.x).max().unwrap_or(0) + 1;
    let mut lane_offsets = HashMap::<String, usize>::new();
    let mut next_lane_y = 0usize;
    for lane_state in &lane_states {
        lane_offsets.insert(lane_state.id.clone(), next_lane_y);
        next_lane_y += lane_state.rows;
    }

    let mut pixel_layout = Layout::default();
    let pool_bounds = bounds(0.0, 0.0, POOL_HEADER_WIDTH + LANE_LABEL_WIDTH + pool_grid_width as f64 * CELL_WIDTH, next_lane_y as f64 * CELL_HEIGHT)?;
    pixel_layout.set(pool.uid, pool_bounds);

    let mut lane_grids = HashMap::new();
    for lane in &lane_order {
        let lane_state = lane_states
            .iter()
            .find(|state| state.id == lane.id)
            .expect("lane state");
        let lane_y = *lane_offsets.get(&lane.id).expect("lane offset");
        lane_grids.insert(
            lane.id.clone(),
            GridBounds {
                x: 0,
                y: lane_y,
                w: pool_grid_width,
                h: lane_state.rows,
            },
        );
        let lane_bounds = bounds(
            POOL_HEADER_WIDTH,
            lane_y as f64 * CELL_HEIGHT,
            LANE_LABEL_WIDTH + pool_grid_width as f64 * CELL_WIDTH,
            lane_state.rows as f64 * CELL_HEIGHT,
        )?;
        pixel_layout.set(lane.uid, lane_bounds);
    }

    let mut element_grids = HashMap::new();
    for descriptor in &descriptors {
        let Some(cell) = placed.get(&descriptor.uid()) else {
            continue;
        };
        let lane_y = *lane_offsets
            .get(&descriptor.lane_id)
            .expect("lane offset for element");
        element_grids.insert(
            descriptor.uid(),
            GridBounds {
                x: cell.x,
                y: cell.y,
                w: 1,
                h: 1,
            },
        );

        let (shape_w, shape_h) = element_pixel_size(descriptor.element);
        let shape_bounds = bounds(
            POOL_HEADER_WIDTH + LANE_LABEL_WIDTH + cell.x as f64 * CELL_WIDTH + (CELL_WIDTH - shape_w) / 2.0,
            lane_y as f64 * CELL_HEIGHT
                + cell.y as f64 * CELL_HEIGHT
                + (CELL_HEIGHT - shape_h) / 2.0,
            shape_w,
            shape_h,
        )?;
        pixel_layout.set(descriptor.uid(), shape_bounds);
    }

    Ok(LayoutComputation {
        pool_grid: GridBounds {
            x: 0,
            y: 0,
            w: pool_grid_width,
            h: next_lane_y,
        },
        lane_grids,
        element_grids,
        pixel_layout,
    })
}

fn place_start_events(
    descriptors: &[ElementDescriptor<'_>],
    lane_order: &[&crate::domain::Lane],
    lane_states: &mut [LaneState],
    occupied: &mut HashMap<String, HashSet<(usize, usize)>>,
    placed: &mut HashMap<Uid, GridCell>,
) {
    for lane in lane_order {
        let start_events: Vec<_> = descriptors
            .iter()
            .filter(|descriptor| descriptor.lane_id == lane.id)
            .filter(|descriptor| {
                matches!(
                    descriptor.element,
                    Element::Event(event) if event.code == EventCode::Start
                )
            })
            .collect();
        for (index, descriptor) in start_events.iter().enumerate() {
            place_in_lane(
                descriptor.uid(),
                &lane.id,
                0,
                index,
                lane_states,
                occupied,
                placed,
            );
        }
    }
}

fn place_in_lane(
    uid: Uid,
    lane_id: &str,
    preferred_x: usize,
    preferred_y: usize,
    lane_states: &mut [LaneState],
    occupied: &mut HashMap<String, HashSet<(usize, usize)>>,
    placed: &mut HashMap<Uid, GridCell>,
) -> GridCell {
    let occupied_cells = occupied.entry(lane_id.to_string()).or_default();
    let lane_state = lane_states
        .iter_mut()
        .find(|state| state.id == lane_id)
        .expect("lane state must exist");
    let mut y = preferred_y;
    while occupied_cells.contains(&(preferred_x, y)) {
        y += 1;
    }
    lane_state.rows = lane_state.rows.max(y + 1);
    occupied_cells.insert((preferred_x, y));
    let cell = GridCell { x: preferred_x, y };
    placed.insert(uid, cell);
    cell
}

fn collect_lane_order(pool: &Pool) -> Vec<&crate::domain::Lane> {
    pool.lanes.iter().collect()
}

fn collect_element_descriptors(pool: &Pool) -> Vec<ElementDescriptor<'_>> {
    let mut descriptors = Vec::new();
    for quadrant in &pool.quadrants {
        for element in &quadrant.elements {
            descriptors.push(ElementDescriptor {
                element,
                lane_id: quadrant.lane_id.clone(),
                decl_index: descriptors.len(),
            });
        }
    }
    descriptors
}

fn collect_outgoing_flows(pool: &Pool) -> HashMap<Uid, Vec<Uid>> {
    let mut outgoing = HashMap::<Uid, Vec<Uid>>::new();
    for flow in &pool.sequence_flows {
        outgoing.entry(flow.from_uid).or_default().push(flow.to_uid);
    }
    outgoing
}

fn element_pixel_size(element: &Element) -> (f64, f64) {
    match element {
        Element::Event(_) => (EVENT_SIZE, EVENT_SIZE),
        Element::Gateway(gateway) => match gateway.code {
            GatewayCode::Inclusive
            | GatewayCode::Exclusive
            | GatewayCode::Parallel
            | GatewayCode::Complex
            | GatewayCode::EventBased => (GATEWAY_SIZE, GATEWAY_SIZE),
        },
        Element::Task(task) => match task.code {
            TaskCode::Generic
            | TaskCode::Manual
            | TaskCode::User
            | TaskCode::Service
            | TaskCode::Receive
            | TaskCode::Send
            | TaskCode::Script
            | TaskCode::BusinessRule
            | TaskCode::SendMessage
            | TaskCode::ReceiveMessage
            | TaskCode::CallActivity => (TASK_WIDTH, TASK_HEIGHT),
        },
        Element::Artifact(_) => (ARTIFACT_WIDTH, ARTIFACT_HEIGHT),
    }
}

fn bounds(x: f64, y: f64, w: f64, h: f64) -> Result<Bounds, LayoutError> {
    Bounds::new(x, y, w, h).map_err(|err| LayoutError::InvalidBounds(err.to_string()))
}
