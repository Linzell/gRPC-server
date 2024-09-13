# Copy binary stage
FROM --platform=$BUILDPLATFORM alpine:3.18.0 as binary

ARG TARGETPLATFORM

COPY target/x86_64-unknown-linux-musl/release/kiro /usr/bin/kiro-x86_64
COPY target/armv7-unknown-linux-musleabihf/release/kiro /usr/bin/kiro-armv7hf
COPY target/aarch64-unknown-linux-musl/release/kiro /usr/bin/kiro-aarch64

RUN case "$TARGETPLATFORM" in \
    "linux/amd64") \
    cp /usr/bin/kiro-x86_64 /usr/bin/kiro; \
    ;; \
    "linux/arm/v7") \
    cp /usr/bin/kiro-armv7hf /usr/bin/kiro; \
    ;; \
    "linux/arm64") \
    cp /usr/bin/kiro-aarch64 /usr/bin/kiro; \
    ;; \
    esac;

# Final stage
FROM alpine:3.18.0

RUN apk --no-cache add \
    ca-certificates

COPY --from=binary /usr/bin/kiro /usr/bin/kiro
USER nobody:nogroup
ENTRYPOINT ["/usr/bin/kiro"]
