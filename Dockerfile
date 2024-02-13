FROM rust:1.76

WORKDIR /usr/src/app
COPY . .

RUN cargo install --path .

CMD ["rinha-backend-24-q1"]
