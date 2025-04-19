FROM ubuntu:24.04

ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH

EXPOSE 8095
EXPOSE 8096

RUN apt-get update && apt-get install -y \
    curl \
    git \
    cmake \
    build-essential \
    libssl-dev \
    pkg-config

RUN curl -fsSL https://deb.nodesource.com/setup_22.x | bash - && \
    apt-get install -y nodejs && \
    npm install -g npm@latest

RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --no-modify-path && \
    ln -s /usr/local/cargo/bin/* /usr/local/bin/

WORKDIR /app

RUN git clone https://github.com/distantmagic/paddler.git .

RUN make

RUN mv target/release/paddler /usr/local/bin/paddler

ENTRYPOINT ["paddler"]
