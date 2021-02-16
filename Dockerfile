FROM debian:stable-slim
WORKDIR /app
ENV PROD=1
COPY target/release/kekw_bot /app
RUN apt update

CMD /app/kekw_bot