FROM rust:buster

RUN apt -y update && apt -y install --no-install-recommends --no-install-suggests \
        git \
        nano \
        ssh \
 && apt -y autoremove \
 && apt -y clean \
 && rm -rf /var/lib/apt/lists/*

RUN groupadd -g 1000 dev \
 && useradd -g 1000 -u 1000 -m -s /bin/bash dev

USER dev
