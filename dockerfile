FROM rockylinux:8.9 AS builder

RUN dnf install -y gcc openssl-devel curl && \
    curl https://sh.rustup.rs -sSf | sh -s -- -y && \
    dnf clean all

ENV PATH=/root/.cargo/bin:$PATH

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo 'fn main() {}' > src/main.rs && cargo build --release

COPY . .
RUN cargo build --release

FROM rockylinux:8.9

WORKDIR /app
COPY --from=builder /app/target/release/operator_mappings_api .
COPY --from=builder /app/resources ./resources

ENV OPERATOR_MAPPINGS_FILE_PATH=/app/resources/operator_mappings.json  
ENV COUNTRY_BORDERS_CSV_FILE_PATH=/app/resources/country_borders.csv

EXPOSE 8080
ENTRYPOINT ["/app/operator_mappings_api"]
