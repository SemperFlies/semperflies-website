FROM rust:latest as builder

# RUN USER=root cargo new --lib semperflies
WORKDIR /semperflies
# COPY ./Cargo.toml ./Cargo.toml
# RUN cargo build --release 
# RUN rm src/*.rs


COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo fetch

 
COPY . .


# RUN rm ./target/release/semperflies
RUN cargo build --bin server --release
RUN cargo build --bin save_products --release


FROM linuxcontainers/debian-slim:latest
ARG APP=/usr/src/app

RUN apt-get update \
    && apt-get install -y ca-certificates tzdata libssl3 \
    && rm -rf /var/lib/apt/lists/*

EXPOSE 443

ENV TZ=Etc/UTC \
    APP_USER=appuser

RUN groupadd $APP_USER \
    && useradd -g $APP_USER $APP_USER \
    && mkdir -p ${APP}

COPY --from=builder /semperflies/target/release/server ${APP}/server
COPY --from=builder /semperflies/target/release/save_products ${APP}/save_products

RUN chown -R $APP_USER:$APP_USER ${APP}

WORKDIR ${APP}

USER root

ADD public ./public
RUN chown -R $APP_USER:$APP_USER ./public
RUN chmod u+rwx /usr/src/app/public


USER $APP_USER
RUN chmod -R 755 ./public  

ADD migrations ./migrations
ADD templates ./templates
ADD certifications ./certifications

CMD ["./server"]

