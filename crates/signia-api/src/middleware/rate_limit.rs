use std::sync::OnceLock;
use std::time::{Duration, Instant};

use axum::extract::State;
use axum::http::Request;
use axum::middleware::Next;
use axum::response::Response;
use parking_lot::Mutex;

use crate::error::ApiError;
use crate::state::AppState;

static GLOBAL: OnceLock<Mutex<Bucket>> = OnceLock::new();

#[derive(Debug)]
struct Bucket {
    capacity: u32,
    tokens: f64,
    refill_per_sec: f64,
    last: Instant,
}

impl Bucket {
    fn new(rpm: u32) -> Self {
        let capacity = rpm.max(1);
        let refill_per_sec = (capacity as f64) / 60.0;
        Self { capacity, tokens: capacity as f64, refill_per_sec, last: Instant::now() }
    }

    fn allow(&mut self) -> bool {
        let now = Instant::now();
        let dt = now.duration_since(self.last);
        self.last = now;

        self.tokens = (self.tokens + dt.as_secs_f64() * self.refill_per_sec).min(self.capacity as f64);
        if self.tokens >= 1.0 {
            self.tokens -= 1.0;
            true
        } else {
            false
        }
    }
}

pub fn layer() -> tower::layer::util::Identity {
    // Implemented as route middleware in routes/mod.rs.
    tower::layer::util::Identity::new()
}

pub async fn enforce(State(state): State<AppState>, req: Request<axum::body::Body>, next: Next) -> Result<Response, ApiError> {
    if !state.cfg.rate_limit.enabled {
        return Ok(next.run(req).await);
    }
    let rpm = state.cfg.rate_limit.rpm;
    let bucket = GLOBAL.get_or_init(|| Mutex::new(Bucket::new(rpm)));
    let mut b = bucket.lock();
    if b.allow() {
        Ok(next.run(req).await)
    } else {
        Err(ApiError::RateLimited)
    }
}
