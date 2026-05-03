use crossterm::event::{self, Event as CrosstermEvent, KeyEvent};
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

pub enum Event {
    Input(KeyEvent),
    Tick,
}

pub struct Events {
    rx: mpsc::UnboundedReceiver<Event>,
}

impl Events {
    pub fn new(tick_rate: Duration) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        let input_tx = tx.clone();
        tokio::spawn(async move {
            loop {
                if event::poll(Duration::from_millis(50)).unwrap() {
                    if let CrosstermEvent::Key(key) = event::read().unwrap() {
                        let _ = input_tx.send(Event::Input(key));
                    }
                }
            }
        });

        let tick_tx = tx.clone();
        tokio::spawn(async move {
            let mut last_tick = Instant::now();
            loop {
                let timeout = tick_rate
                    .checked_sub(last_tick.elapsed())
                    .unwrap_or_else(|| Duration::from_secs(0));
                
                tokio::time::sleep(timeout).await;
                if last_tick.elapsed() >= tick_rate {
                    if tick_tx.send(Event::Tick).is_err() {
                        break;
                    }
                    last_tick = Instant::now();
                }
            }
        });

        Self { rx }
    }

    pub async fn next(&mut self) -> Option<Event> {
        self.rx.recv().await
    }
}
