# diviner

Diviner is a [FoundationDB style simulation testing](https://apple.github.io/foundationdb/testing.html) framework for Rust. It includes 2 parts:

* A [Future executor](https://rust-lang.github.io/async-book/02_execution/04_executor.html) that is designed to be single threaded and deterministic. The goal here is to enable deterministic simulations in Rust.
* Wrappers over existing Rust async IO libraries. When building normally, the wrappers will use the actual implementation, but when building with `simulation` feature enabled, the wrappers will use mocked implementations that integrate with above Future executor to enable deterministic testing. The wrappers mentioned here, might inclue (but are not limited to):
    + Time related functions, such as sleep or now;
    + Network related modules, such as TCPListener, TCPStream;
    + File IO related modules;

If you find the above term confusing, [this video](https://www.youtube.com/watch?v=4fFDFbi3toc) might help explain what diviner provides.

The goal here, is to enable deterministic testing on any Rust code satisfying the following rules:

1. The code is written in [async/await](https://rust-lang.github.io/async-book/01_getting_started/04_async_await_primer.html) style;
2. The code uses wrappers provided by diviner to perform all the necessary IOs;

# Examples

To illustrate what the library does, several examples are provided in the repository:

* [simple](https://github.com/xxuejie/diviner/blob/f368ad6fe1cb367ef65ed21f6f220a367328c184/examples/simple.rs): a minimal piece of code written in async/await style;
* [time-manipulation](https://github.com/xxuejie/diviner/blob/f368ad6fe1cb367ef65ed21f6f220a367328c184/examples/time-manipulation.rs): time manipulation example, here we are testing sleeping for a whole day, but the simulation code would manipulate time and finish immediately;
* [execution-order](https://github.com/xxuejie/diviner/blob/f368ad6fe1cb367ef65ed21f6f220a367328c184/examples/execution-order.rs): a poorly implemented agent, diviner can use different seeds to test different execution orders of futures. So later you can determinisically debug the code with the seed that will trigger the errors;
