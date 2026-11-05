use super::Session;
use amml_launcher_bridge::RegistryRefresher;
use std::sync::Arc;

// Refreshes the registries once at start and then on a fixed interval.
pub struct RegistrySchedule;

impl RegistrySchedule {
    pub fn start(session: Arc<Session>) -> std::io::Result<()> {
        std::thread::Builder::new().name("launcher-registries".to_string()).spawn(move || {
            loop {
                if let Err(error) = session.refresh_registries() {
                    curia::warn!("the registries were not refreshed", { error: error.to_string() });
                }
                std::thread::sleep(RegistryRefresher::INTERVAL);
            }
        })?;
        Ok(())
    }
}
