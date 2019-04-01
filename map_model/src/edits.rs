use crate::{ControlStopSign, ControlTrafficSignal, IntersectionID, LaneID, LaneType};
use serde_derive::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MapEdits {
    pub(crate) map_name: String,
    pub edits_name: String,
    pub lane_overrides: BTreeMap<LaneID, LaneType>,
    // TODO Storing the entire thing is maybe a bit dramatic, but works for now.
    pub stop_sign_overrides: BTreeMap<IntersectionID, ControlStopSign>,
    pub traffic_signal_overrides: BTreeMap<IntersectionID, ControlTrafficSignal>,
}

impl MapEdits {
    pub fn new(map_name: String) -> MapEdits {
        MapEdits {
            map_name,
            // Something has to fill this out later
            edits_name: "no_edits".to_string(),
            lane_overrides: BTreeMap::new(),
            stop_sign_overrides: BTreeMap::new(),
            traffic_signal_overrides: BTreeMap::new(),
        }
    }

    pub fn describe(&self) -> String {
        format!(
            "map edits \"{}\" ({} lanes, {} stop signs, {} traffic signals)",
            self.edits_name,
            self.lane_overrides.len(),
            self.stop_sign_overrides.len(),
            self.traffic_signal_overrides.len(),
        )
    }

    pub fn save(&self) {
        abstutil::save_object("edits", &self.map_name, &self.edits_name, self);
    }
}
