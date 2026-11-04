use super::Session;
use amml_launcher_bridge::{BridgeError, FaultKindView, FaultView};
use std::collections::BTreeMap;
use std::sync::Arc;
use tauri::State;

// Runs a bridge call off the window thread and turns its error into the fault the window words.
pub struct Blocking;

impl Blocking {
    pub async fn run<T, F>(session: &State<'_, Arc<Session>>, call: F) -> Result<T, FaultView>
    where
        T: Send + 'static,
        F: FnOnce(&Session) -> Result<T, BridgeError> + Send + 'static,
    {
        let session = Arc::clone(session.inner());
        match tauri::async_runtime::spawn_blocking(move || call(&session)).await {
            Ok(Ok(value)) => Ok(value),
            Ok(Err(error)) => Err(FaultView::from(error)),
            Err(error) => Err(Self::fault(error.to_string())),
        }
    }

    // an `operation-failed` fault with the text on its code line
    pub fn fault(text: String) -> FaultView {
        FaultView::new(FaultKindView::OperationFailed, Some(text), BTreeMap::new())
    }
}
