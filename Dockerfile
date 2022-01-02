# Build any of the included examples
#
# This Dockerfile builds any opf the included examples through use of the build arg
# `example`.
#
# To build the rbroadlink-cli, for example, use the following command:
#
# docker build -t rbroadlink-cli:latest --build-arg example=rbroadlink-cli .

from rust:slim as builder

ARG example

WORKDIR /app

# Add musl tools for static compilation
RUN apt update && apt install -y musl-tools

# Add the target for static build
RUN rustup target add x86_64-unknown-linux-musl

# Copy in the source
COPY Cargo.* /app/
COPY src/ /app/src/
COPY examples/ /app/examples/

# Build the example
RUN cargo build --release --example ${example} --target x86_64-unknown-linux-musl --features ${example}
RUN cp /app/target/x86_64-unknown-linux-musl/release/examples/${example} /example

# Create minimal container
from scratch

ARG example

COPY --from=builder /example /app
ENTRYPOINT ["/app"]
