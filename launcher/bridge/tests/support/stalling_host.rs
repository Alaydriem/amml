use std::net::TcpListener;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

// A host on the loopback that takes each connection and says nothing for `stall`, then closes it: a registry fetch
// to it takes about that long and then fails.
pub struct StallingHost {
    port: u16,
    accepted: Arc<AtomicUsize>,
}

impl StallingHost {
    pub fn start(stall: Duration) -> StallingHost {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let accepted = Arc::new(AtomicUsize::new(0));
        let count = Arc::clone(&accepted);
        std::thread::spawn(move || {
            for stream in listener.incoming().flatten() {
                count.fetch_add(1, Ordering::SeqCst);
                std::thread::spawn(move || {
                    std::thread::sleep(stall);
                    drop(stream);
                });
            }
        });
        StallingHost { port, accepted }
    }

    // how many connections the host has taken
    pub fn accepted(&self) -> usize {
        self.accepted.load(Ordering::SeqCst)
    }

    pub fn url(&self) -> String {
        format!("https://127.0.0.1:{}/", self.port)
    }
}
