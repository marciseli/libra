// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_config::config::{LoggerConfig, NodeConfig, NodeConfigHelpers};
use libra_logger::prelude::*;
use slog_scope::GlobalLoggerGuard;
use std::path::Path;

pub fn load_config_from_path(config: Option<&Path>) -> NodeConfig {
    // Load the config
    let node_config = if let Some(path) = config {
        info!("Loading node config from: {}", path.display());
        NodeConfig::load(path).expect("NodeConfig")
    } else {
        info!("Loading test configs");
        NodeConfigHelpers::get_single_node_test_config(false /* random ports */)
    };

    // Node configuration contains important ephemeral port information and should
    // not be subject to being disabled as with other logs
    println!("Using node config {:?}", &node_config);

    node_config
}

pub fn setup_metrics(peer_id: &str, node_config: &NodeConfig) {
    if let Some(metrics_dir) = node_config.get_metrics_dir() {
        libra_metrics::dump_all_metrics_to_file_periodically(
            &metrics_dir,
            &format!("{}.metrics", peer_id),
            node_config.metrics.collection_interval_ms,
        );
    }
}

pub fn setup_executable(
    config: Option<&Path>,
    no_logging: bool,
) -> (NodeConfig, Option<GlobalLoggerGuard>) {
    crash_handler::setup_panic_handler();
    let mut _logger = set_default_global_logger(no_logging, &LoggerConfig::default());

    let config = load_config_from_path(config);

    // Reset the global logger using config (for chan_size currently).
    // We need to drop the global logger guard first before resetting it.
    _logger = None;
    let logger = set_default_global_logger(no_logging, &config.logger);
    for network in &config.networks {
        setup_metrics(&network.peer_id, &config);
    }

    (config, logger)
}

fn set_default_global_logger(
    is_logging_disabled: bool,
    logger_config: &LoggerConfig,
) -> Option<GlobalLoggerGuard> {
    if is_logging_disabled {
        return None;
    }

    Some(libra_logger::set_default_global_logger(
        logger_config.is_async,
        Some(logger_config.chan_size),
    ))
}
