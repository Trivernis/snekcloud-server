/*
 * snekcloud node based network
 * Copyright (C) 2020 trivernis
 * See LICENSE for more information
 */

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HeartbeatSettings {
    pub output_file: Option<PathBuf>,
    pub interval_ms: u64,
    pub max_record_history: usize,
}

impl Default for HeartbeatSettings {
    fn default() -> Self {
        Self {
            output_file: None,
            interval_ms: 10000,
            max_record_history: 10,
        }
    }
}

impl HeartbeatSettings {
    pub fn interval(&self) -> Duration {
        Duration::from_millis(self.interval_ms as u64)
    }
}
