use diviner::Environment;

fn main() {
    let result = Environment::new()
        .block_on(async {
            let f = async { 7 };
            f.await
        })
        .expect("block on failure!");
    assert!(result == 7, "Returned result is not 7!");
}
