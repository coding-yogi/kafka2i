use futures::{StreamExt, FutureExt};
use tokio::{
    sync::mpsc,
    task::JoinHandle,
    runtime::Builder,
};
use color_eyre::Result;
use crossterm::event::{self, KeyEvent};

/// Terminal events.
#[derive(Clone, Copy, Debug)]
pub enum TuiEvent {
    Error,
    Tick,
    Render,
    Key(KeyEvent),
}

/// Terminal event handler.
#[derive(Debug)]
pub struct EventHandler {
    /// Event sender channel.
    _tx: mpsc::UnboundedSender<TuiEvent>,
    /// Event receiver channel.
    rx: mpsc::UnboundedReceiver<TuiEvent>,
    /// Event handler thread.
    task: Option<JoinHandle<()>>,
}

impl EventHandler {
    /// Constructs a new instance of [`EventHandler`].
    pub fn new(tick_rate: f64, frame_rate: f64) -> Self {
        
        // define tick rate
        let tick_delay = std::time::Duration::from_secs_f64(1.0 / tick_rate);
        let render_delay = std::time::Duration::from_secs_f64(1.0 / frame_rate);
        let (tx, rx) = mpsc::unbounded_channel();

        let _tx = tx.clone();

        let task = tokio::spawn(async move {
            let mut reader = event::EventStream::new();
            let mut tick_interval = tokio::time::interval(tick_delay);
            let mut render_interval = tokio::time::interval(render_delay);

            loop {
                let tick_delay = tick_interval.tick();
                let render_delay = render_interval.tick();
                let cs_event = reader.next().fuse();
                
                tokio::select! {
                    maybe_event = cs_event => {
                        match maybe_event {
                            Some(Ok(evt)) => {
                                match evt {
                                    crossterm::event::Event::Key(key) => {
                                        if key.kind == crossterm::event::KeyEventKind::Press {
                                            tx.send(TuiEvent::Key(key)).unwrap();
                                        }
                                    },
                                    _ => {},
                                }
                            }
                            Some(Err(_)) => {
                                tx.send(TuiEvent::Error).unwrap();
                            }
                            None => {},
                        }
                    },
                    _ = tick_delay => {
                        tx.send(TuiEvent::Tick).unwrap();
                    },
                    _ = render_delay => {
                        tx.send(TuiEvent::Render).unwrap();
                    },
                }
            }
        });

        Self { _tx, rx, task: Some(task) }
    }

    /// Receive the next event from the handler thread.
    ///
    /// This function will always block the current thread if
    /// there is no data available and it's possible for more data to be sent.
    pub fn next(&mut self) -> Result<TuiEvent> {
        let rt = Builder::new_current_thread().enable_all().build().unwrap();
        rt.block_on(self.rx.recv()).ok_or(color_eyre::eyre::eyre!("unable to get event"))
    }
}
