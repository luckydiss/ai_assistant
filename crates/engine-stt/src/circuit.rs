use crate::CircuitState;
use std::sync::atomic::{AtomicU64, AtomicU8, Ordering};
use std::time::{Duration, Instant};

pub struct CircuitBreaker {
    state: AtomicU8,
    failure_count: AtomicU64,
    failure_threshold: u64,
    timeout_duration: Duration,
    last_failure: std::sync::Mutex<Option<Instant>>,
}

impl CircuitBreaker {
    pub fn new(failure_threshold: u64, timeout_secs: u64) -> Self {
        Self {
            state: AtomicU8::new(Self::state_id(CircuitState::Closed)),
            failure_count: AtomicU64::new(0),
            failure_threshold,
            timeout_duration: Duration::from_secs(timeout_secs),
            last_failure: std::sync::Mutex::new(None),
        }
    }

    fn state_id(state: CircuitState) -> u8 {
        match state {
            CircuitState::Closed => 0,
            CircuitState::Open => 1,
            CircuitState::HalfOpen => 2,
        }
    }

    fn state_from_id(id: u8) -> CircuitState {
        match id {
            0 => CircuitState::Closed,
            1 => CircuitState::Open,
            _ => CircuitState::HalfOpen,
        }
    }

    pub async fn allow_request(&self) -> bool {
        let state = Self::state_from_id(self.state.load(Ordering::SeqCst));

        match state {
            CircuitState::Closed => true,
            CircuitState::HalfOpen => true,
            CircuitState::Open => {
                let elapsed = self
                    .last_failure
                    .lock()
                    .map(|guard| guard.map(|last| last.elapsed()))
                    .ok()
                    .flatten();

                match elapsed {
                    Some(elapsed) if elapsed >= self.timeout_duration => {
                        self.state
                            .store(Self::state_id(CircuitState::HalfOpen), Ordering::SeqCst);
                        true
                    }
                    _ => false,
                }
            }
        }
    }

    pub async fn record_success(&self) {
        self.state
            .store(Self::state_id(CircuitState::Closed), Ordering::SeqCst);
        self.failure_count.store(0, Ordering::SeqCst);
    }

    pub async fn record_failure(&self) {
        let failures = self.failure_count.fetch_add(1, Ordering::SeqCst) + 1;

        if failures >= self.failure_threshold {
            self.state
                .store(Self::state_id(CircuitState::Open), Ordering::SeqCst);
            if let Ok(mut guard) = self.last_failure.lock() {
                *guard = Some(Instant::now());
            }
        }
    }

    pub fn current_state(&self) -> CircuitState {
        Self::state_from_id(self.state.load(Ordering::SeqCst))
    }
}
