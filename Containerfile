# Start rust build process
FROM docker.io/library/rust:latest as builder

# Set working directory to /src, to keep things clean
WORKDIR /src

# Copy source files to /src
COPY . /src

# Build
RUN cargo install --path=./ && cargo clean -v

# Swap to fedora runtime for the final result
FROM fedora:latest

# Copy binary to resulting image
COPY --from=builder /usr/local/cargo/bin/ibis /bin/ibis

# Install postgresql library
RUN sudo dnf install -y libpq

# Set working dir to /app for mounting configs
WORKDIR /app

# Set command to execute
CMD ["ibis"]
