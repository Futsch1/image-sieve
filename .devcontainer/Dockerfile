FROM ubuntu:20.04

WORKDIR /home/

COPY . .

ARG DEBIAN_FRONTEND=noninteractive
ENV TZ=Etc/UTC
RUN bash ./setup.sh

ENV PATH="/root/.cargo/bin:$PATH"