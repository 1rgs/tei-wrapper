FROM ghcr.io/huggingface/text-embeddings-inference:1.5
RUN apt-get update && apt-get install -y \
  git \
  build-essential \
  libssl-dev \
# install cargo
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"


RUN git clone https://github.com/1rgs/tei-wrapper.git && \
  cd tei-wrapper && \
  cargo build --release

ENTRYPOINT []  
CMD ["sh", "-c", "MAX_CLIENT_BATCH_SIZE=4096 MAX_BATCH_REQUESTS=256 AUTO_TRUNCATE=true MODEL_ID=/bore/model_cache/embedder_merchant_matching API_KEY=${TEI_AUTH_SECRET} ./tei-wrapper/target/release/tei_wrapper"]