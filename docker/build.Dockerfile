FROM jrottenberg/ffmpeg:9.0-ubuntu

RUN apt-get update && \
    apt-get -y install \
        mold \
        build-essential \
        pkg-config \
        gcc \
        clang \
        libfontconfig1-dev \
        curl \
        tar \
        gzip \
        bzip2

# TODO
