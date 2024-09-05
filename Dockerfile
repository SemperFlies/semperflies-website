FROM rust:latest as builder

RUN USER=root cargo new --bin semperflies
WORKDIR ./semperflies
COPY ./Cargo.toml ./Cargo.toml
RUN cargo build --release
RUN rm src/*.rs
 
ADD . ./


RUN rm ./target/release/semperflies
RUN cargo build --release


FROM linuxcontainers/debian-slim:latest
ARG APP=/usr/src/app

RUN apt-get update \
    && apt-get install -y ca-certificates tzdata libssl3\
    && rm -rf /var/lib/apt/lists/*

EXPOSE ${PORT} 

ENV TZ=Etc/UTC \
    APP_USER=appuser

RUN groupadd $APP_USER \
    && useradd -g $APP_USER $APP_USER \
    && mkdir -p ${APP}

COPY --from=builder /semperflies/target/release/semperflies ${APP}/semperflies

RUN chown -R $APP_USER:$APP_USER ${APP}

WORKDIR ${APP}

USER root

ADD public ./public

RUN chown -R $APP_USER:$APP_USER ./public

USER $APP_USER
RUN chmod -R 755 ./public  

COPY setup_nfs_and_start_web.sh /usr/local/bin/entrypoint.sh
RUN chmod +x /usr/local/bin/entrypoint.sh

ADD migrations ./migrations
ADD templates ./templates


ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]

