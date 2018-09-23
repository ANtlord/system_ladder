FROM alpine

RUN apk add --no-cache libc6-compat

COPY target/release/system_ladder /bin/qwe

WORKDIR /

ENTRYPOINT ["qwe"]
