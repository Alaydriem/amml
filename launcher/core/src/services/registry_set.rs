use crate::{HttpFetch, NameHit, NameSearch, RegistryClient, RegistryRef, RegistryStatus};

// Every registry the player has, in settings order (the official one first).
pub struct RegistrySet<H: HttpFetch> {
    clients: Vec<RegistryClient<H>>,
}

impl<H: HttpFetch> RegistrySet<H> {
    pub fn new(clients: Vec<RegistryClient<H>>) -> RegistrySet<H> {
        RegistrySet { clients }
    }

    pub fn refresh_all(&mut self) -> Vec<(RegistryRef, RegistryStatus)> {
        self.clients.iter_mut().map(|client| (client.registry().clone(), client.refresh())).collect()
    }

    pub fn clients_mut(&mut self) -> &mut [RegistryClient<H>] {
        &mut self.clients
    }

    // the root of the official registry, when it was fetched
    pub fn official_root(&self) -> Option<&amml_registry::RegistryRoot> {
        self.clients.iter().find(|client| client.registry().is_official()).and_then(|client| client.root())
    }

    pub fn official_mut(&mut self) -> Option<&mut RegistryClient<H>> {
        self.clients.iter_mut().find(|client| client.registry().is_official())
    }

    // Hits grouped by registry in settings order. A registry with no hits, or whose names fail, is left out.
    pub fn search(&mut self, query: &str) -> Vec<(RegistryRef, Vec<NameHit>)> {
        let mut found = Vec::new();
        for client in &mut self.clients {
            match client.names() {
                Ok(names) => {
                    let hits = NameSearch::search(&names.value, query);
                    if !hits.is_empty() {
                        found.push((client.registry().clone(), hits));
                    }
                }
                Err(error) => {
                    curia::warn!("a registry has no names", {
                        registry: client.registry().base.clone(),
                        error: error.to_string(),
                    });
                }
            }
        }
        found
    }
}
