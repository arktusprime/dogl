use dogl_language::domain::{DoglFile, Collab, Pool, Quadrant, Lane, Element, Event, EventCode, Flow, FlowType};
use dogl_language::validation::validate;
use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    #[test]
    fn does_not_panic_on_arbitrary_graphs(
        nodes in prop::collection::vec(1u64..100u64, 0..20),
        edges in prop::collection::vec((1u64..100u64, 1u64..100u64), 0..40)
    ) {
        let mut pool = Pool::new(1000, "Pool1");
        pool.lanes.push(Lane::new(1001, "Lane1"));
        let mut quadrant = Quadrant::new(1002, "Lane1", "Stage1");
        
        for &node_id in &nodes {
            quadrant.elements.push(Element::Event(Event {
                uid: node_id,
                id: format!("E{}", node_id).into(),
                name: format!("Event {}", node_id),
                code: EventCode::Intermediate,
                expressions: vec![],
            }));
        }
        pool.quadrants.push(quadrant);

        let mut flow_id = 2000;
        for &(from, to) in &edges {
            pool.sequence_flows.push(Flow::new(flow_id, from, to, FlowType::Sequence));
            flow_id += 1;
        }

        let mut collab = Collab::new(500, "Collab1");
        collab.pools.push(pool);
        let file = DoglFile::new(vec![collab]);

        // Just checking that validate does not panic on any graph shape (cycles, disconnected components, etc.)
        let _report = validate(&file);
    }
}
