use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};
use std::sync::RwLock;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum State {
    Closed,
    Open,
    HalfOpen,
}

pub struct CircuitBreaker {
    state: RwLock<State>,
    failure_count: AtomicU64,
    failure_threshold: u64,
    reset_timeout: Duration,
    last_failure_time: RwLock<Option<Instant>>,
}

impl CircuitBreaker {
    pub fn new(failure_threshold: u64, reset_timeout: Duration) -> Self {
        Self {
            state: RwLock::new(State::Closed),
            failure_count: AtomicU64::new(0),
            failure_threshold,
            reset_timeout,
            last_failure_time: RwLock::new(None),
        }
    }

    pub fn state(&self) -> State {
        let state = *self.state.read().unwrap();
        if state == State::Open {
            let last_failure = self.last_failure_time.read().unwrap();
            if let Some(time) = *last_failure {
                if time.elapsed() >= self.reset_timeout {
                    // 状态超时，进入半开状态尝试
                    drop(state);
                    let mut state_mut = self.state.write().unwrap();
                    *state_mut = State::HalfOpen;
                    return State::HalfOpen;
                }
            }
        }
        state
    }

    pub fn allow_request(&self) -> bool {
        match self.state() {
            State::Closed => true,
            State::HalfOpen => true, // 半开状态允许单个请求试探
            State::Open => false,
        }
    }

    pub fn record_success(&self) {
        let mut state = self.state.write().unwrap();
        if *state == State::HalfOpen {
            *state = State::Closed;
            self.failure_count.store(0, Ordering::SeqCst);
        }
    }

    pub fn record_failure(&self) {
        let count = self.failure_count.fetch_add(1, Ordering::SeqCst) + 1;
        if count >= self.failure_threshold {
            let mut state = self.state.write().unwrap();
            *state = State::Open;
            let mut last_failure = self.last_failure_time.write().unwrap();
            *last_failure = Some(Instant::now());
        }
    }
}
