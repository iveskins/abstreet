use crate::objects::ID;
use crate::ui::UI;
use ezgui::{Color, EventCtx, GfxCtx, Key};
use geom::{Duration, PolyLine};
use map_model::LANE_THICKNESS;
use sim::{AgentID, TripID};

pub enum RouteViewer {
    Inactive,
    Hovering(Duration, AgentID, PolyLine),
    Active(Duration, TripID, Option<PolyLine>),
    DebugAllRoutes(Duration, Vec<PolyLine>),
}

impl RouteViewer {
    pub fn event(&mut self, ctx: &mut EventCtx, ui: &mut UI) {
        match self {
            RouteViewer::Inactive => {
                if let Some(agent) = ui
                    .state
                    .primary
                    .current_selection
                    .and_then(|id| id.agent_id())
                {
                    if let Some(trace) =
                        ui.state
                            .primary
                            .sim
                            .trace_route(agent, &ui.state.primary.map, None)
                    {
                        *self = RouteViewer::Hovering(ui.state.primary.sim.time(), agent, trace);
                    }
                } else if ctx.input.modal_action("show/hide route for all agents") {
                    *self = debug_all_routes(ui);
                }
            }
            RouteViewer::Hovering(time, agent, _) => {
                // Argh, borrow checker.
                let agent = *agent;

                if *time != ui.state.primary.sim.time()
                    || ui.state.primary.current_selection != Some(ID::from_agent(agent))
                {
                    *self = RouteViewer::Inactive;
                    if let Some(new_agent) = ui
                        .state
                        .primary
                        .current_selection
                        .and_then(|id| id.agent_id())
                    {
                        if let Some(trace) =
                            ui.state
                                .primary
                                .sim
                                .trace_route(new_agent, &ui.state.primary.map, None)
                        {
                            *self = RouteViewer::Hovering(
                                ui.state.primary.sim.time(),
                                new_agent,
                                trace,
                            );
                        }
                    }
                }

                // If there's a current route, then there must be a trip.
                let trip = ui.state.primary.sim.agent_to_trip(agent).unwrap();
                // TODO agent might be stale here! Really need a second match after this or
                // something. Expressing a state machine like this isn't really great.
                if ctx
                    .input
                    .contextual_action(Key::R, &format!("show {}'s route", agent))
                {
                    *self = show_route(trip, ui);
                }
            }
            RouteViewer::Active(time, trip, _) => {
                // TODO Using the modal menu from parent is weird...
                if ctx.input.modal_action("stop showing agent's route") {
                    *self = RouteViewer::Inactive;
                } else if *time != ui.state.primary.sim.time() {
                    *self = show_route(*trip, ui);
                }
            }
            RouteViewer::DebugAllRoutes(time, _) => {
                if ctx.input.modal_action("show/hide route for all agents") {
                    *self = RouteViewer::Inactive;
                } else if *time != ui.state.primary.sim.time() {
                    *self = debug_all_routes(ui);
                }
            }
        }
    }

    pub fn draw(&self, g: &mut GfxCtx, ui: &UI) {
        match self {
            RouteViewer::Hovering(_, _, ref trace) => {
                g.draw_polygon(
                    ui.state.cs.get("route").alpha(0.5),
                    &trace.make_polygons(LANE_THICKNESS),
                );
            }
            RouteViewer::Active(_, _, Some(ref trace)) => {
                g.draw_polygon(
                    ui.state.cs.get_def("route", Color::RED.alpha(0.8)),
                    &trace.make_polygons(LANE_THICKNESS),
                );
            }
            RouteViewer::DebugAllRoutes(_, ref traces) => {
                for t in traces {
                    g.draw_polygon(ui.state.cs.get("route"), &t.make_polygons(LANE_THICKNESS));
                }
            }
            _ => {}
        }
    }
}

fn show_route(trip: TripID, ui: &UI) -> RouteViewer {
    let time = ui.state.primary.sim.time();
    if let Some(agent) = ui.state.primary.sim.trip_to_agent(trip) {
        // Trace along the entire route by passing in max distance
        if let Some(trace) = ui
            .state
            .primary
            .sim
            .trace_route(agent, &ui.state.primary.map, None)
        {
            RouteViewer::Active(time, trip, Some(trace))
        } else {
            println!("{} has no trace right now", agent);
            RouteViewer::Active(time, trip, None)
        }
    } else {
        println!(
            "{} has no agent associated right now; is the trip done?",
            trip
        );
        RouteViewer::Active(time, trip, None)
    }
}

fn debug_all_routes(ui: &mut UI) -> RouteViewer {
    let mut traces: Vec<PolyLine> = Vec::new();
    let trips: Vec<TripID> = ui
        .state
        .primary
        .sim
        .get_stats(&ui.state.primary.map)
        .canonical_pt_per_trip
        .keys()
        .cloned()
        .collect();
    for trip in trips {
        if let Some(agent) = ui.state.primary.sim.trip_to_agent(trip) {
            if let Some(trace) =
                ui.state
                    .primary
                    .sim
                    .trace_route(agent, &ui.state.primary.map, None)
            {
                traces.push(trace);
            }
        }
    }
    RouteViewer::DebugAllRoutes(ui.state.primary.sim.time(), traces)
}
