use tracing::{info};
use tokio::sync::mpsc;

mod core;
mod peripherals;
mod ui;

use core::system::{VirtualMCU, SystemEvent};

#[tokio::main]
async fn main() {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_target(false) // hides the module name
        .compact() // shortens log formatting
        .init(); // activates the logging backend

    info!("Starting Virtual Peripheral Simulator...");
    
    // Create channel between "MCU core" and "main controller"
    let (tx, mut rx) = mpsc::channel::<SystemEvent>(32);
    
    // Create MCU
    let mcu = VirtualMCU::new(200); // 200ms tick
    
    // Spawn MCU task
    tokio::spawn(async move {
        mcu.run(tx).await;
    });

    // Receive and handle events
    while let Some(event) = rx.recv().await {
        match event {
            SystemEvent::Tick(count) => info!("Tick {}", count),
            SystemEvent::Interrupt(src) => info!("Interrupt from {}", src),
            SystemEvent::Stop => {
                info!("System stopping gracefully.");
                break;
            }
        }
    }

    info!("Simulation ended.");
}

    
   
  
 
    



