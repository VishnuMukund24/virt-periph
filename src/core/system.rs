use tokio::sync::mpsc;
use tokio::time::{self, Duration};
use tracing::{info, warn};

// Define messages that can occur in our simulated MCU
#[derive(Debug)]
pub enum SystemEvent {
    Tick(u64),
    Interrupt(&'static str),
    Stop,
}

// Virtual MCU system structure
pub struct VirtualMCU {
    tick_rate_ms: u64,
}

impl VirtualMCU {
    pub fn new(tick_rate_ms: u64) -> Self {
        Self { tick_rate_ms }
    }

    pub async fn run(&self, mut tx: mpsc::Sender<SystemEvent>) {
        let mut interval = time::interval(Duration::from_millis(self.tick_rate_ms));
        let mut tick_count = 0u64;

        loop {
            interval.tick().await;
            tick_count += 1;

            // Send tick event
            if tx.send(SystemEvent::Tick(tick_count)).await.is_err() {
                warn!("Receiver dropped! Stopping core loop.");
                break;
            }

            // Simulate random interrupt (like GPIO or UART event)
            if tick_count % 7 == 0 {
                let _ = tx.send(SystemEvent::Interrupt("UART_RX")).await;
            }

            // Stop simulation after 20 ticks
            if tick_count >= 20 {
                let _ = tx.send(SystemEvent::Stop).await;
                break;
            }
        }
    }
}

