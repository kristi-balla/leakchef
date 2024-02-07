
# intermediate image for building
FROM rust:1.70 AS build

# the usual stuff: update, git, ssh
RUN apt-get update && apt-get upgrade -y && \
    apt-get install -y --no-install-recommends \
    git

WORKDIR /usr/src/server

# Build dependencies first to allow docker caching
COPY ./ ./
RUN rm -rf ./src && mkdir ./src && echo "fn main() {}" > ./src/main.rs
RUN cargo build --release && rm -rf ./src

# Build the actual project
COPY ./src ./src
RUN cargo build --release

# Final image
FROM rust:1.70-slim

RUN apt-get update && \
    apt-get upgrade -y && \
    apt-get -y install --no-install-recommends cmake && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /.
COPY --from=build /usr/src/server/target/release/server .

ENTRYPOINT ["./server"]
