use crate::{Config, ConfigError};
use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use std::sync::{Arc, RwLock};
use std::time::Duration;
use tokio::runtime::Handle;
use tokio::sync::broadcast;
use tokio::time::sleep;

#[derive(Debug, Clone)]
pub enum ConfigEvent {
    Changed,
    Error(String),
}

pub struct ConfigWatcher {
    config: Arc<RwLock<Config>>,
    events: broadcast::Receiver<ConfigEvent>,
    _watcher: RecommendedWatcher,
}

impl ConfigWatcher {
    pub fn start<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let config = match Config::load(&path) {
            Ok(c) => c,
            Err(e) => {
                // Non-fatal: a missing TTS api_key must not prevent the app from starting.
                let mut c = Config::parse(&path)?;
                if c.tts.mode != "off" && c.tts.api_key.is_empty() {
                    tracing::warn!("{e}; forcing tts.mode=off until api_key is set");
                    c.tts.mode = "off".into();
                } else {
                    return Err(e);
                }
                c
            }
        };
        let config = Arc::new(RwLock::new(config));
        let (sender, events) = broadcast::channel(16);
        let handle = Handle::try_current().map_err(|_| {
            ConfigError::Validation("ConfigWatcher::start requires a tokio runtime".into())
        })?;
        let sender_clone = sender.clone();
        let config_clone = config.clone();
        let path = path.as_ref().to_path_buf();
        let watch_path = path.clone();

        let mut watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
            if let Ok(event) = res {
                if event.kind.is_modify() {
                    let sender = sender_clone.clone();
                    let config = config_clone.clone();
                    let path = path.clone();
                    let handle = handle.clone();
                    handle.spawn(async move {
                        sleep(Duration::from_millis(100)).await; // debounce
                        match Config::load(&path) {
                            Ok(new_config) => {
                                *config.write().unwrap() = new_config;
                                let _ = sender.send(ConfigEvent::Changed);
                            }
                            Err(e) => {
                                let _ = sender.send(ConfigEvent::Error(e.to_string()));
                            }
                        }
                    });
                }
            }
        })?;

        watcher.watch(watch_path.parent().unwrap(), RecursiveMode::NonRecursive)?;

        Ok(ConfigWatcher {
            config,
            events,
            _watcher: watcher,
        })
    }

    pub fn config(&self) -> &Arc<RwLock<Config>> {
        &self.config
    }

    pub fn events(&self) -> broadcast::Receiver<ConfigEvent> {
        self.events.resubscribe()
    }
}
