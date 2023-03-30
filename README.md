## What is this?

`stair-stepper` is a small Rust web service built with `axum`. It doesn't do anything useful.

The purpose of the project is to act as a sandbox in which to play with different HTTP workloads and observe the impact
on the memory profile.

I wanted to develop an intuition about how heap fragmentation might be occurring in a much larger codebase with many
more dependencies on external services, making rapid iteration more of a challenge. 

Additionally I wanted to get an idea for what the difference might be when switching from the system default allocator
versus [jemalloc](https://github.com/tikv/jemallocator) before introducing it in our production systems.

By experimenting with different aspects of the workload, I was able to compare and contrast the allocators.

## Disclaimer

This project is _written very poorly_ and for the most part this is _on purpose_. There's needless cloning all over
the place, the actual work being done by the sole HTTP handler doesn't make sense... It's fine. Don't worry about it.

The point here is to just make sure "things are happening" and by _"things"_ I guess I mean _"allocations."_
Since this is a study of memory fragmentation and its mitigation, allocating and freeing heap memory is key to the
reproduction.

At any rate, do not refer to this as an example of _how to write an axum app_.

## Prerequisites

To play in this sandbox you will need:

- A rust toolchain (using [rustup] is recommended)
- A "unix" system (the `jemalloc` crate **doesn't work on Windows**, sorry!)

With rust installed, you should be able to install
[drill], a load testing tool, via cargo:

```
$ cargo install drill
```

You should be ready to use this project once you have `rust`, `cargo`, and `drill` installed.

## How to Use

The idea here is to run the web server then run "simulated load" through it so you can watch how the process memory
changes over time.

### Running the web server

To run the web server using the **default allocator**:

```
$ cargo run
```

To run the web server **using `jemalloc`**:

```
$ cargo run --features jemalloc
```

The web server has a "sleep" in the request handling code path and you can configure the duration with the `SLEEP_MS`
env var. This is useful for simulating cases where a remote system (like a database) is having increased latency, which
in turn increases the latency of the request handler.

Eg. to run the the server with a 1.2 sec sleep duration:
```
$ env SLEEP_MS=1200 cargo run
```

### Generating load with `drill`

[drill] is configured via `benchmark.yml` which contains some settings to control:

- the overall number of requests to send (more on this below)
- the concurrency (ie, how big of a pool of clients to use)
- rampup ("time, in seconds, `drill` will take to start all iterations", per the docs)

The actual requests `drill` will send are configured each in their own files under `plans/`.

There are several "tags" defined to group these by "size" ranging: `tiny`, `small`, `medium`, `large`.
The smallest request body is around 30kb, and the largest is around 2Mb.

Tags can be specified when running `drill` allowing us to mix and match, but note that the total number of requests is
the product of the _number of tags selected_ and the _iterations_.

Running without specifying any tags at all, for example, will result in at total of `12,500` requests being sent:

```
$ drill --benchmark benchmark.yml --stats
```

Meanwhile, running with a pair of tags will reduce to total request count to `5,000`:


```
$ drill --benchmark benchmark.yml --stats --tags small,medium
```


### Tips for experimenting

The actual outcome of these load tests is dependent on a number of factors including the system specs of the machine, 
the default allocator's implementation for the system, possibly even the version of rust and/or libc, and so on.

It's a moving target.

Anecdotally, I saw good contrasts between `jemalloc` and the default allocator with:

- `SLEEP_MS=450`
- `--tags medium`

The rest of the benchmark config as seen in the repo by default, e.g.:
```
concurrency: 16
base: 'http://localhost:3000'
iterations: 2500
rampup: 1
```

With these settings, I ran `drill` repeatedly with a brief pause between batches. With the default allocator I was able
to push the process memory of `stair-stepper` to over 700Mb, whereas with `jemalloc` the memory would float between
200Mb and 370Mb (with a brief spike of around 430Mb).

With `jemalloc` the memory usage was a lot more variable, but didn't continue to climb past a certain point.
Using the default allocator, the ceiling was much higher.

I recommend playing with the benchmark config parameters, `SLEEP_MS`, to see what works on your system.

[drill]: https://github.com/fcsonline/drill
[rustup]: https://rustup.rs/
