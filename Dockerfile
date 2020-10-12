FROM rust:1
WORKDIR /app
COPY target/release/kekw_bot /app

CMD /app/kekw_bot