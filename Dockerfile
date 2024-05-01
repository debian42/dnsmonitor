# Build container
FROM amd64/rust:buster AS build
ENV TZ=Europe/Berlin
WORKDIR /work
ADD . .
USER root
RUN cargo build --release

# Main container
FROM debian:buster
LABEL quay.expires-after=12w
LABEL maintainer="systemteam/yoda"
ENV TZ=Europe/Berlin
COPY --from=build /work/entrypoint.sh /entrypoint.sh
COPY --from=build /work/target/release/dnsmonitor /dnsmonitor
RUN apt update && apt upgrade && chmod ugo+x /entrypoint.sh && chmod ugo+x /dnsmonitor
WORKDIR /
ENTRYPOINT [ "/entrypoint.sh" ]
EXPOSE 8080
