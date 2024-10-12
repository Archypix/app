FROM rust:1.81
WORKDIR /app

# Libmysqlclient-dev is required for diesel
RUN apt-get update && apt-get install -y default-libmysqlclient-dev && rm -rf /var/lib/apt/lists/*

# Install cargo-watch for live reloading
RUN rustup component add rust-src
RUN cargo install cargo-watch

ENV ROCKET_ADDRESS=0.0.0.0
ENV ROCKET_PORT=80
EXPOSE 80

CMD ["sleep", "infinity"]
