FROM ubuntu:24.04

RUN apt-get update && apt-get install -y build-essential pkgconf curl libusb-1.0-0-dev libudev-dev

ENV PATH="/root/.cargo/bin:/usr/local/cmake/bin:${PATH}"

RUN curl --proto '=https' --tlsv1.2 -o rust.sh https://sh.rustup.rs\
    && /bin/bash rust.sh -y