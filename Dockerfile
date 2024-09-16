FROM ghcr.io/huggingface/text-embeddings-inference:cpu-1.5
RUN apt-get update && apt-get install -y git build-essential libssl-dev pkg-config curl

RUN curl https://sh.rustup.rs -sSf | bash -s -- -y

ENV PATH="/root/.cargo/bin:${PATH}"

RUN git clone https://github.com/1rgs/tei-wrapper.git && \
  cd tei-wrapper && \
  cargo build --release

ENTRYPOINT []  
