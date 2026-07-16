FROM redhat/ubi8:latest AS builder

ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH

RUN yum update -y && yum install -y curl git which gcc && yum clean all

RUN ARCH=$(uname -m) && \
    if [ "$ARCH" = "aarch64" ]; then RUSTUP_SHA256="9732d6c5e2a098d3521fca8145d826ae0aaa067ef2385ead08e6feac88fa5792"; \
    elif [ "$ARCH" = "x86_64" ]; then RUSTUP_SHA256="4acc9acc76d5079515b46346a485974457b5a79893cfb01112423c89aeb5aa10"; \
    else echo "Unsupported architecture: $ARCH" && exit 1; fi && \
    RUSTUP_URL="https://static.rust-lang.org/rustup/archive/1.29.0/${ARCH}-unknown-linux-gnu/rustup-init" && \
    curl --proto '=https' --tlsv1.2 -sSf -o rustup-init "$RUSTUP_URL" && \
    echo "${RUSTUP_SHA256} *rustup-init" | sha256sum -c - && \
    chmod +x rustup-init && \
    ./rustup-init -y --no-modify-path --profile minimal --default-toolchain nightly && \
    rm rustup-init && \
    chmod -R a+w "$RUSTUP_HOME" "$CARGO_HOME" && rustc --version

SHELL ["/bin/bash", "-c"]
WORKDIR /build

COPY . .

RUN cargo build --profile prod && cargo test

FROM scratch AS binary-export

COPY --from=builder /build/target/prod/irma-viz /export/
