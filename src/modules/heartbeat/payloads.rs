/*
 * snekcloud node based network
 * Copyright (C) 2020 trivernis
 * See LICENSE for more information
 */

use serde::{Deserialize, Serialize};
use std::ops::Add;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Serialize, Deserialize)]
pub struct HeartbeatPayload {
    pub node_id: String,
    beat_at: u64,
}

impl HeartbeatPayload {
    pub fn now(node_id: String) -> Self {
        let start = SystemTime::now();
        Self {
            node_id,
            beat_at: start.duration_since(UNIX_EPOCH).unwrap().as_millis() as u64,
        }
    }

    pub fn get_beat_time(&self) -> SystemTime {
        UNIX_EPOCH.add(Duration::from_millis(self.beat_at))
    }
}
