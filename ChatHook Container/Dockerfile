FROM archlinux:base
RUN pacman -Syu --noconfirm netcat curl wget rust openssl pkg-config

COPY ./entrypoint.sh /entrypoint.sh
WORKDIR /chathook
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]