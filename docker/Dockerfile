FROM public.ecr.aws/sam/build-provided.al2

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

ENV PATH="/root/.cargo/bin:$PATH"

WORKDIR /app