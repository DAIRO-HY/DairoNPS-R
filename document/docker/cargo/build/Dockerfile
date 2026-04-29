FROM rust:1.93

WORKDIR /home/rust/src
#COPY ..
#
#RUN pwd
#RUN mkdir test123
#RUN cargo clean
#RUN cargo build --release --package test --bin test
ENTRYPOINT ["/bin/sh", "-c", "cargo --version && cargo clean && cargo build --release --package test --bin test"]