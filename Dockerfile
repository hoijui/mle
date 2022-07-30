FROM ubuntu:18.04

RUN apt-get update; apt-get install -y ca-certificates; update-ca-certificates
ADD ./target/release/mle /bin/mle
RUN PATH=$PATH:/bin/mle
