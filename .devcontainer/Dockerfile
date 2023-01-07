FROM ubuntu:22.04

ENV NODE_VERSION 16.16
ENV GO_VERSION 1.19
ENV PROTOC_VERSION 21.12

RUN apt-get update -y && apt-get upgrade -y
RUN apt-get install -y snap
RUN apt-get install -y git build-essential ca-certificates g++ make curl wget unzip
RUN apt-get install -y zsh tmux neovim thefuck
RUN chsh -s /bin/zsh

# install rustup & cargo + tools
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y
RUN . $HOME/.cargo/env && rustup default nightly
ENV PATH /root/.cargo/bin:$PATH
RUN cargo install lsd gitui

# install NVM & NPM
ENV NVM_DIR /usr/local/nvm
RUN mkdir -p /usr/local/nvm
ENV NODE_PATH $NVM_DIR/v$NODE_VERSION/lib/node_modules
ENV PATH $NVM_DIR/v$NODE_VERSION/bin:$PATH

RUN curl https://raw.githubusercontent.com/creationix/nvm/master/install.sh | bash \
    && . $NVM_DIR/nvm.sh && nvm install $NODE_VERSION \
    && nvm alias default $NODE_VERSION

# install go
ENV GOROOT /usr/local/go
ENV GOPATH $HOME/go
ENV PATH $GOPATH/bin:$GOROOT/bin:$PATH 
RUN wget https://go.dev/dl/go$GO_VERSION.linux-amd64.tar.gz
RUN tar -xvf go$GO_VERSION.linux-amd64.tar.gz
RUN mv go /usr/local
RUN rm ./go$GO_VERSION.linux-amd64.tar.gz

# install protobuf
RUN curl -LO https://github.com/protocolbuffers/protobuf/releases/download/v$PROTOC_VERSION/protoc-$PROTOC_VERSION-linux-x86_64.zip
RUN unzip protoc-$PROTOC_VERSION-linux-x86_64.zip -d ./protoc
RUN mv ./protoc/bin/protoc /usr/local/bin
RUN chmod +x /usr/local/bin/protoc
RUN rm -rf ./protoc && rm protoc-$PROTOC_VERSION-linux-x86_64.zip

ENTRYPOINT ["/bin/zsh"]
