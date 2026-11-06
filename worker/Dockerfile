FROM ghcr.io/anomalyco/opencode:2.1.2
LABEL org.opencontainers.image.source="https://github.com/graize/graize"

USER root
RUN mkdir -p /workspace /home/opencode/.config/opencode /home/opencode/.local/share/opencode \
    && chown -R 10001:10001 /workspace /home/opencode

ENV HOME=/home/opencode
WORKDIR /workspace

USER 10001:10001

EXPOSE 4096

ENTRYPOINT ["opencode"]
CMD ["serve", "--hostname", "0.0.0.0", "--port", "4096"]
