FROM rust
COPY . .
RUN apt-get update
RUN apt-get install -y wkhtmltopdf 
EXPOSE 8000
ENV ROCKET_ADDRESS=0.0.0.0
ENV ROCKET_PORT=8000

RUN rustup default nightly
RUN cargo build


CMD ["cargo", "run"]
