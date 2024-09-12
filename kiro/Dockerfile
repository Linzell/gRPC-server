FROM rust:latest

ARG CERT_DOMAIN

RUN apt-get update \
    && DEBIAN_FRONTEND=noninteractive \
    apt-get install --no-install-recommends --assume-yes \
    protobuf-compiler
RUN apt-get install pkg-config make g++ libssl-dev --assume-yes
RUN apt-get install -y libssl-dev --assume-yes

WORKDIR /back-rust

COPY ["Cargo.toml", "Cargo.lock", "/back-rust/"]
COPY src /back-rust/src
# COPY email-templates /back-rust/email-templates
COPY .cargo /back-rust/.cargo

# COPY generate-certs.sh /back-rust/generate-certs.sh
# COPY ca.pe[m] /back-rust/certs/ca.pem
# COPY cert.pe[m] /back-rust/certs/cert
# COPY key.pe[m] /back-rust/certs/key.pem
# RUN chmod +x ./generate-certs.sh
# RUN ./generate-certs.sh $CERT_DOMAIN
# RUN rm -f /back-rust/certs/ca.pem /back-rust/generate-certs.sh

RUN cargo build --release

# Environment variable for the port number
ENV PORT 3000
# Inform Docker that the container listens on the specified port at runtime.
EXPOSE $PORT

ENTRYPOINT [ "/bin/bash", "-c", "cargo run --release" ]

# Use HEALTHCHECK instruction to tell Docker how to test a container to check that it is still working
HEALTHCHECK --interval=10s --timeout=5s --start-period=10s --retries=3 \
    CMD grpcurl -insecure http://localhost:$PORT/grpc.health.v1.Health/Check || exit 1
