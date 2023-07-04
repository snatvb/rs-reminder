use tokio::{signal, sync::mpsc};

use crate::{clients, state::events::Event};

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
    let (shutdown_send, mut shutdown_recv) = mpsc::unbounded_channel::<()>();

    tokio::select! {
        _ = signal::ctrl_c() => {
            log::info!("Reminder shutdown by ctrl-c");
            let _ = shutdown_send.send(()).unwrap();
        },
        _ = shutdown_recv.recv() => {
            log::info!("Reminder shutdown");
        },
    }
}

impl Reminder {
    // start reminder - reminder every EVERY seconds and not block the thread
    pub async fn run(&mut self) {
        tokio::select! {
            _ = self.start() => { },
            _ = wait_shot_down() => { },
        }
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
        self.clients.handle_event(Event::Remind).await;
    }
}
