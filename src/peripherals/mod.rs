use std::time::Duration;
use tokio::time::{interval, sleep};
use tokio::sync::mpsc;
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use tracing::info;

/// The public API: spawn the UART and GPIO simulator tasks,
/// each taking a clone of the event sender.
pub fn spawn_uart(tx: mpsc::Sender<crate::SystemEvent>, period_ms: u64) {
    // spawn a background async task for UART behavior
    tokio::spawn(async move {
        let mut ticker = interval(Duration::from_millis(period_ms));
        let mut byte_counter: u8 = 0;

        loop {
            ticker.tick().await;

            // Simulate receiving a byte (payload content)
            byte_counter = byte_counter.wrapping_add(1);
            let msg = format!("UART_BYTE:{}", byte_counter);

            // Send as an interrupt event (string payload)
            if tx.send(crate::SystemEvent::Interrupt(msg)).await.is_err() {
                info!("UART: main receiver dropped, ending UART task.");
                break;
            }

            // Optional: small jitter between bytes
            sleep(Duration::from_millis(5)).await;
        }
    });
}

pub fn spawn_gpio(tx: mpsc::Sender<crate::SystemEvent>, avg_interval_ms: u64) {
    tokio::spawn(async move {
        // ✅ Thread-safe random number generator
        let mut rng = StdRng::from_entropy();

        loop {
            // --- Simulate variable peripheral timing ----------------------
            // Generate a random interval ±50% around avg_interval_ms
            let jitter = rng.gen_range(
                (avg_interval_ms / 2)..=(avg_interval_ms + avg_interval_ms / 2)
            );
            sleep(Duration::from_millis(jitter)).await;

            // --- Simulate hardware event ---------------------------------
            // Toggle state (true/false)
            let state: bool = rng.gen_bool(0.5);
            let msg = format!("GPIO_STATE:{}", state);

            // Send event to main system
            if tx.send(crate::SystemEvent::Interrupt(msg)).await.is_err() {
                info!("GPIO: main receiver dropped, ending GPIO task.");
                break;
            }
        }
    });
}