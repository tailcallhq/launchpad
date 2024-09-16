ARG RUST_VERSION=1.80.1
FROM rust:${RUST_VERSION}-slim-bookworm AS builder
RUN apt-get update && apt-get install -y pkg-config libssl-dev protobuf-compiler && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY . .
RUN cargo build --release

ARG GITHUB_TOKEN=""
FROM debian:bookworm-slim AS runtime
# environment variables
ENV SERVER_HOST=0.0.0.0
ENV SERVER_PORT=8080
ENV TZ=Etc/UTC
ENV APP_USER=appuser
ENV NVM_DIR=/home/$APP_USER/.nvm
ENV NVM_VERSION=v0.40.1
ENV NODE_VERSION=22.3.0
ENV APP_FOLDER=/usr/src/app
ENV GITHUB_TOKEN=${GITHUB_TOKEN}
# https://www.pulumi.com/docs/pulumi-cloud/access-management/access-tokens/
# ENV PULUMI_ACCESS_TOKEN
# ENV AWS_ACCESS_KEY_ID
# ENV AWS_SECRET_ACCESS_KEY
# ENV AWS_REGION
ENV DOWNLOAD_URL=https://github.com/tailcallhq/tailcall/releases/download/v0.111.2/tailcall-aws-lambda-bootstrap
# dependencies
RUN apt-get update && apt-get install -y ca-certificates tzdata curl && rm -rf /var/lib/apt/lists/*
# add rootless user
RUN groupadd $APP_USER && useradd -m -g $APP_USER $APP_USER && mkdir -p $APP_FOLDER
# switch to user mode
USER $APP_USER
# pulumi
RUN curl -fsSL https://get.pulumi.com | sh
ENV PATH /home/$APP_USER/.pulumi/bin:$PATH
RUN pulumi plugin install resource aws
# node
RUN curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/$NVM_VERSION/install.sh | bash \
    && . $NVM_DIR/nvm.sh \
    && nvm install $NODE_VERSION \
    && nvm alias default $NODE_VERSION \
    && nvm use default
# copy required files
COPY --from=builder /app/target/release/tailcall-launchpad $APP_FOLDER/tailcall-launchpad
COPY deployments $APP_FOLDER/deployments
# give data to user
USER root
RUN chown -R $APP_USER:$APP_USER $APP_FOLDER
# switch to user mode
USER $APP_USER
# install node dependencies
WORKDIR $APP_FOLDER/deployments
RUN . $NVM_DIR/nvm.sh && nvm use default && npm i
RUN curl -Lo tailcall $DOWNLOAD_URL
# set runtime
ENV PATH /home/$APP_USER/.nvm/versions/node/v$NODE_VERSION/bin:$PATH
WORKDIR $APP_FOLDER
ENTRYPOINT ["./tailcall-launchpad"]