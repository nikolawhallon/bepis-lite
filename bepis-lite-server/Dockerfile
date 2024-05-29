FROM ubuntu:22.04 as builder

LABEL maintainer="Nikola Whallon <nikola@deepgram.com>"

ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update && \
	apt-get install -y --no-install-recommends \
		ca-certificates \
		clang \
		curl \
		libpq-dev \
		libssl-dev \
		pkg-config

RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain stable && \
	. $HOME/.cargo/env

COPY . /bepis-lite-server

RUN . $HOME/.cargo/env && \
	cargo install --path /bepis-lite-server --root /

FROM ubuntu:22.04

LABEL maintainer="Nikola Whallon <nikola@deepgram.com>"

ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update && \
	apt-get install -y --no-install-recommends \
		ca-certificates \
		libpq5 \
		libssl3 && \
	apt-get clean

COPY --from=builder /bin/bepis-lite-server /bin/bepis-lite-server

ENTRYPOINT ["/bin/bepis-lite-server"]
