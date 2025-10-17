use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::interval;
use tracing::{info, Level};
use tracing_subscriber;

mod peripherals;
mod monitor;

/// Events exchanged across the system.
#[derive(Debug, Clone)]
enum SystemEvent {
    Tick(u64),
    Interrupt(String),
    Stop,
}

/// Simple virtual MCU that sends Tick events periodically.
/// (We keep it small: it will also send a Stop after some ticks.)
struct VirtualMCU {
    tick_count: u64,
}

impl VirtualMCU {
    fn new() -> Self {
        VirtualMCU { tick_count: 0 }
    }

    async fn run(&mut self, tx: mpsc::Sender<SystemEvent>, tick_ms: u64, max_ticks: u64) {
        let mut ticker = interval(Duration::from_millis(tick_ms));
        loop {
            ticker.tick().await;
            self.tick_count += 1;

            // Post a Tick event
            if tx.send(SystemEvent::Tick(self.tick_count)).await.is_err() {
                // receiver dropped
                break;
            }

            // stop condition
            if self.tick_count >= max_ticks {
                let _ = tx.send(SystemEvent::Stop).await;
                break;
            }
        }
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    // Channel between producers (MCU + peripherals) and the main consumer
    let (tx, mut rx) = mpsc::channel::<SystemEvent>(64);
    let (bcast_tx, _) = tokio::sync::broadcast::channel::<SystemEvent>(64);

    // Clone tx for each producer we spawn
    let tx_for_mcu  = tx.clone();
    let tx_for_uart = tx.clone();
    let tx_for_gpio = tx.clone();

    // Start system monitor
    let monitor_rx = bcast_tx.subscribe();
    monitor::spawn_monitor(monitor_rx);

    // Spawn the virtual MCU core
    tokio::spawn(async move {
        let mut mcu = VirtualMCU::new();
        // tick every 200 ms, stop after 60 ticks
        mcu.run(tx_for_mcu, 200, 60).await;
    });

    // Spawn UART simulator (periodic bytes every 500 ms)
    peripherals::spawn_uart(tx_for_uart, 500);

    // Spawn GPIO simulator (avg event every 700 ms)
    peripherals::spawn_gpio(tx_for_gpio, 700);

    // Main event loop (consumer)
    while let Some(event) = rx.recv().await {
        // Send a copy of the event to broadcast monitor
        let _ = bcast_tx.send(event.clone());

        match event {
            SystemEvent::Tick(count) => {
                info!("Tick: {}", count);
            }
            SystemEvent::Interrupt(payload) => {
                // We receive both GPIO and UART interrupts here.
                // In a real firmware you'd route by source, or decode payload.
                info!("Peripheral interrupt: {}", payload);
            }
            SystemEvent::Stop => {
                info!("Simulation stop requested.");
                break;
            }
        }
    }
    info!("Simulation finished.");
}