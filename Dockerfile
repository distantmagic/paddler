FROM ubuntu:22.04

RUN apt-get update && apt-get install -y \
    git \
    build-essential \
    cmake \
    curl \
    && rm -rf /var/lib/apt/lists/*

RUN curl -fsSL https://deb.nodesource.com/setup_18.x | bash - \
    && apt-get install -y nodejs \
    && npm install -g npm@latest

RUN curl -LO https://golang.org/dl/go1.20.6.linux-amd64.tar.gz \
    && tar -C /usr/local -xzf go1.20.6.linux-amd64.tar.gz \
    && rm go1.20.6.linux-amd64.tar.gz

ENV PATH=$PATH:/usr/local/go/bin

WORKDIR /app

RUN git clone https://github.com/distantmagic/paddler.git .

RUN make

RUN mv ./paddler-bin-linux-x64 /usr/local/bin/paddler

CMD ["paddler", "balancer"]
