use diviner::{spawn, Environment};
use rand::prelude::*;
use std::sync::{Arc, Mutex};

struct Agent {
    value: u64,
}

impl Agent {
    async fn current_value(&self) -> u64 {
        self.value
    }

    async fn update(&mut self, value: u64) {
        self.value = value
    }
}

fn main() {
    let mut success_seed: Option<u64> = None;
    let mut failure_seed: Option<u64> = None;
    for _ in 0..1000 {
        let seed: u64 = random();
        let e = Environment::new_with_seed(seed);
        let result = e.block_on(async {
            let agent = Arc::new(Mutex::new(Agent { value: 12 }));
            let _a = {
                let agent2 = Arc::clone(&agent);
                spawn(async move { agent2.lock().unwrap().update(14).await })
            };
            let b = {
                let agent2 = Arc::clone(&agent);
                spawn(async move { agent2.lock().unwrap().current_value().await })
            };
            let value = b.await.expect("Task canceled").expect("Task failed");
            assert!(value == 14, "Update is not performed!");
        });
        match result {
            Ok(_) => {
                success_seed = Some(seed);
            }
            Err(_) => {
                failure_seed = Some(seed);
            }
        }
        if success_seed.is_some() && failure_seed.is_some() {
            break;
        }
    }
    if let Some(seed) = success_seed {
        println!("Task succeeded with seed: {}", seed);
    }
    if let Some(seed) = failure_seed {
        println!("Task failed with seed: {}", seed);
    }
}
