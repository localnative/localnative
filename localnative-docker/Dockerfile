FROM rust:1.55.0-bullseye as builder

RUN apt-get update && apt-get install -y tree vim tig less sqlite3
RUN mkdir -p /root/repos/localnative &&\
    cd /root/repos/localnative &&\
    git clone https://gitlab.com/localnative/localnative.git &&\
    cd localnative/localnative-rs/localnative_cli &&\
    cargo build
RUN ln -s /bin/sed /bin/gsed
