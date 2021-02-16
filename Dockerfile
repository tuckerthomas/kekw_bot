FROM postgres:13
WORKDIR /app
ENV PROD=1
COPY target/release/kekw_bot /app

CMD /app/kekw_bot