/*
 * snekcloud node based network
 * Copyright (C) 2020 trivernis
 * See LICENSE for more information
 */

use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NodesRefreshSettings {
    pub update_interval_ms: u64,
}

impl Default for NodesRefreshSettings {
    fn default() -> Self {
        Self {
            update_interval_ms: 3600000,
        }
    }
}

impl NodesRefreshSettings {
    pub fn update_interval(&self) -> Duration {
        Duration::from_millis(self.update_interval_ms)
    }
}
