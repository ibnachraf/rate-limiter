use chrono::{DateTime, Utc};
use dashmap::DashMap;
use futures::future::join_all;
use std::sync::Arc;
use std::time::Duration;
use std::{collections::HashMap, thread};
use tokio::task::JoinHandle;

pub struct RateLimiter {
    pub rate: u64,   // number of requests per window
    pub window: u64, // window duration (in seconds)
    cache: DashMap<String, Vec<DateTime<Utc>>>,
}

impl RateLimiter {
    pub fn new(rate: u64, window: u64) -> RateLimiter {
        RateLimiter {
            rate,   // number of requests per window
            window, // window duration
            cache: DashMap::new(),
        }
    }

    pub fn is_authorized(&self, user_id: &String) -> bool {
        match self.cache.get_mut(user_id) {
            Some(mut visits) => {
                let now: DateTime<Utc> = Utc::now();
                visits.retain(|e| e.clone() + chrono::Duration::seconds(self.window as i64) > now);
                if visits.len() < self.rate.try_into().unwrap() {
                    visits.push(Utc::now());
                    true
                } else {
                    false
                }
            }
            None => {
                self.cache.insert(user_id.clone(), vec![Utc::now()]);
                true
            }
        }
    }
}

#[test]
fn test_rate_limiter() {
    let mut rate_limiter = RateLimiter::new(2, 1);
    let user_1: String = "1.0.0.0".into();
    let user_2: String = "2.0.0.0".into();

    assert!(rate_limiter.is_authorized(&user_1));
    assert!(rate_limiter.is_authorized(&user_1));
    assert!(rate_limiter.is_authorized(&user_2));
    assert!(rate_limiter.is_authorized(&user_2));
    assert!(!rate_limiter.is_authorized(&user_1));
    assert!(!rate_limiter.is_authorized(&user_2));

    thread::sleep(Duration::from_secs(3));
    assert!(rate_limiter.is_authorized(&user_1));
    assert!(rate_limiter.is_authorized(&user_2));
}

#[tokio::test]
async fn test_rate_limiter_concurrently() {
    let rate_limiter = Arc::new(RateLimiter::new(2, 10));
    let user_1: String = "1.0.0.0".into();
    let mut tasks: Vec<JoinHandle<bool>> = vec![];
    for _ in 0..3 {
        tasks.push(spawn_task( rate_limiter.clone(), user_1.clone()).await);
    }
    let result: Vec<Result<bool, tokio::task::JoinError>> = futures::future::join_all(tasks).await;
    assert_eq!(result.iter().filter(|e| e.is_ok()).count(), 3);
    let res: Vec<bool> = result.into_iter().map(|r| r.unwrap()).collect();
    let authorized_count = res.iter().filter(|&&v| v).count();
    let denied_count = res.iter().filter(|&&v| !v).count();

    assert_eq!(authorized_count, 2);
    assert_eq!(denied_count, 1);
}


async fn spawn_task(rate_limiter: Arc<RateLimiter>, user_1: String) -> JoinHandle<bool> {
    let rl = rate_limiter.clone();
    let user = user_1.clone();
    let task = tokio::spawn(async move {
        let is_authorized = rl.is_authorized(&user);
        is_authorized
    });
    task
}
