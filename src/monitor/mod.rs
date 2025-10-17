use std::collections::HashMap;
use tokio::sync::broadcast::Receiver;
use tokio::time::{sleep, Duration};
use tracing::info;

use crate::SystemEvent;

pub fn spawn_monitor(mut rx: Receiver<SystemEvent>) {
    tokio::spawn(async move {
        let mut tick_count = 0u64;
        let mut interrupts = HashMap::new();

        loop {
            tokio::select! {
                Ok(event) = rx.recv() => {
                    match event {
                        SystemEvent::Tick(_) => {
                            tick_count += 1;
                            info!("Monitor received a tick");
                        }
                        SystemEvent::Interrupt(msg) => {
                            *interrupts.entry(msg).or_insert(0) += 1;
                            info!("Monitor received an interrupt");
                        }
                        SystemEvent::Stop => {
                            info!("=== System Monitor Report ===");
                            info!("Total ticks: {}", tick_count);
                            for (name, count) in interrupts.iter() {
                                info!("Peripheral {:?}: {} events", name, count);
                            }
                            info!("==============================");
                            break;
                        }
                    }
                }

                _ = sleep(Duration::from_secs(2)) => {
                    info!("=== System Monitor Report ===");
                    info!("Total ticks: {}", tick_count);
                    for (name, count) in interrupts.iter() {
                        info!("Peripheral {:?}: {} events", name, count);
                    }
                    info!("==============================");
                }
            } // might not need select! because it only executes one branch at a time
        }
    });
}
