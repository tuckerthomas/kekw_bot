FROM debian:stable-slim
WORKDIR /app
ENV PROD=1
COPY target/release/kekw_bot /app
RUN apt update
RUN apt install -y sqlite3 libssl-dev

CMD /app/kekw_bot