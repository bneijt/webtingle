FROM circleci/rust:latest as build

COPY  --chown=circleci:circleci . /opt/build
WORKDIR /opt/build
RUN cargo build --release

FROM debian:buster-slim
RUN mkdir -p /opt/app
WORKDIR /opt/app
RUN apt-get update && apt-get install --no-install-recommends -y \
  ca-certificates \
  && apt-get clean \
  && rm -rf /var/lib/apt/lists/*
ENV PORT=8080
USER daemon
COPY --from=build /opt/build/target/release/webtingle .
CMD ["/opt/app/webtingle"]