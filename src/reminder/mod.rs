use tokio::{signal, sync::mpsc};

use crate::clients;

static EVERY: i64 = 10;

pub struct Reminder {
    clients: clients::Clients,
}

impl Reminder {
    pub fn new(clients: clients::Clients) -> Self {
        Self { clients }
    }
}

async fn wait_shot_down() {
    let (_, mut shutdown_recv) = mpsc::unbounded_channel::<()>();

    tokio::select! {
        _ = signal::ctrl_c() => {},
        _ = shutdown_recv.recv() => {},
    }
}

impl Reminder {
    // start reminder - reminder every EVERY seconds and not block the thread
    pub async fn run(&mut self) {
        tokio::select! {
            _ = self.start() => {
            },
            _ = wait_shot_down() => {
            },
        }
        self.start().await;
    }

    async fn start(&self) {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(EVERY as u64));
        loop {
            log::debug!("Tick");
            interval.tick().await;
            self.remind().await;
        }
    }

    pub async fn remind(&self) {
        log::debug!("Reminding");
    }
}
