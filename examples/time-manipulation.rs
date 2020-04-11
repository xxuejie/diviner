use diviner::{
    time::{now, sleep},
    Environment,
};
use std::time::Duration;

fn main() {
    let result = Environment::new()
        .block_on(async {
            let a = now();
            sleep(Duration::from_secs(24 * 3600)).await;
            let b = now();
            b.duration_since(a)
                .expect("duration calculation error")
                .as_secs()
        })
        .expect("block_on failure");
    assert!(result >= 24 * 3600, "A day must have passed since sleep!");
}
