FROM docker.io/library/buildpack-deps:bookworm

WORKDIR /builds

ENV GOLANG_VERSION=1.20.6 \
    GOPATH=/go \
    NODE_VERSION=18.16.1 \
    YARN_VERSION=1.22.19 \
    SOLIDITY_VERSION=0.5.17 \
    PATH=$GOPATH/bin:/usr/local/go/bin:$PATH

# install tools and dependencies
RUN set -eux; \
	apt-get -y update; \
	dpkgArch="$(dpkg --print-architecture)"; arch="${dpkgArch##*-}"; \
	apt-get install -y --no-install-recommends \
        g++ gcc libc6-dev make pkg-config \
        build-essential jq \
        libboost-all-dev libz3-dev \
		libssl-dev libsasl2-dev cmake graphviz \
		git curl time rhash ca-certificates \
		python3 python3-pip lsof ruby ruby-bundler git-restore-mtime xz-utils unzip gnupg protobuf-compiler; \
# add non-root user
	groupadd -g 1000 nonroot ; \
	useradd -u 1000 -g 1000 -s /bin/bash -m nonroot ; \
# setup architecture specific variables
	case "$dpkgArch" in \
		amd64) \
            golangUrl='https://dl.google.com/go/go1.20.6.linux-amd64.tar.gz'; \
            goSha256='b945ae2bb5db01a0fb4786afde64e6fbab50b67f6fa0eb6cfa4924f16a7ff1eb'; \
            nodejsArch='x64';; \
		armhf) \
            golangUrl='https://dl.google.com/go/go1.20.6.linux-armv6l.tar.gz'; \
            goSha256='669902f5c8efefbd5d5fd078db01e34355af3693e48659b89593da7db367c488'; \
            nodejsArch='armv7l';; \
		arm64) \
            golangUrl='https://dl.google.com/go/go1.20.6.linux-arm64.tar.gz'; \
            goSha256='4e15ab37556e979181a1a1cc60f6d796932223a0f5351d7c83768b356f84429b'; \
            nodejsArch='arm64';; \
		i386) \
            golangUrl='https://dl.google.com/go/go1.20.6.linux-386.tar.gz'; \
            goSha256='2e27c9db1defbf4d58e907f9843bf60a1ce229688f8463bf24d6a0a19dc949de'; \
            nodejsArch='x86';; \
		*) echo >&2 "unsupported architecture: ${dpkgArch}"; exit 1 ;; \
	esac; \
# gpg keys listed at https://github.com/nodejs/node#release-keys
    for key in \
        4ED778F539E3634C779C87C6D7062848A1AB005C \
        141F07595B7B3FFE74309A937405533BE57C7D57 \
        74F12602B6F1C4E913FAA37AD3A89613643B6201 \
        DD792F5973C6DE52C432CBDAC77ABFA00DDBF2B7 \
        61FC681DFB92A079F1685E77973F295594EC4689 \
        8FCCA13FEF1D0C2E91008E09770F7A9A5AE15600 \
        C4F0DFFF4E8C1A8236409D08E73BC641CC11F4C8 \
        890C08DB8579162FEE0DF9DB8BEAB4DFCF555EF4 \
        C82FA3AE1CBEDC6BE46B9360C43CEC45C17AB93C \
        108F52B48DB57BB0CC439B2997B01419BD92F80A \
        6A010C5166006599AA17F08146C2130DFD2497F5 \
    ; do \
        gpg --batch --keyserver hkps://keys.openpgp.org --recv-keys "$key" || \
        gpg --batch --keyserver keyserver.ubuntu.com --recv-keys "$key" ; \
    done; \
# Install NodeJS
    curl -fsSLO --compressed "https://nodejs.org/dist/v$NODE_VERSION/node-v$NODE_VERSION-linux-$nodejsArch.tar.xz" && \
    curl -fsSLO --compressed "https://nodejs.org/dist/v$NODE_VERSION/SHASUMS256.txt.asc" && \
    gpg --batch --decrypt --output SHASUMS256.txt SHASUMS256.txt.asc && \
    grep " node-v$NODE_VERSION-linux-$nodejsArch.tar.xz\$" SHASUMS256.txt | sha256sum -c - && \
    tar -xJf "node-v$NODE_VERSION-linux-$nodejsArch.tar.xz" -C /usr/local --strip-components=1 --no-same-owner && \
    rm "node-v$NODE_VERSION-linux-$nodejsArch.tar.xz" SHASUMS256.txt.asc SHASUMS256.txt && \
    ln -s /usr/local/bin/node /usr/local/bin/nodejs && \
    node --version && \
    npm --version && \
# Install Yarn
    curl -fsSLO --compressed "https://yarnpkg.com/downloads/$YARN_VERSION/yarn-v$YARN_VERSION.tar.gz" && \
    curl -fsSLO --compressed "https://yarnpkg.com/downloads/$YARN_VERSION/yarn-v$YARN_VERSION.tar.gz.asc" && \
    gpg --batch --verify yarn-v$YARN_VERSION.tar.gz.asc yarn-v$YARN_VERSION.tar.gz && \
    mkdir -p /opt && \
    tar -xzf yarn-v$YARN_VERSION.tar.gz -C /opt/ && \
    ln -s /opt/yarn-v$YARN_VERSION/bin/yarn /usr/local/bin/yarn && \
    ln -s /opt/yarn-v$YARN_VERSION/bin/yarnpkg /usr/local/bin/yarnpkg && \
    rm yarn-v$YARN_VERSION.tar.gz.asc yarn-v$YARN_VERSION.tar.gz && \
    yarn --version && \
# Install Go
	wget -O go.tgz.asc "$golangUrl.asc"; \
    wget -O go.tgz "$golangUrl" --progress=dot:giga; \
    echo "$goSha256 *go.tgz" | sha256sum -c -; \
 \
# https://github.com/golang/go/issues/14739#issuecomment-324767697
	GNUPGHOME="$(mktemp -d)"; export GNUPGHOME; \
# https://www.google.com/linuxrepositories/
	gpg --batch --keyserver keyserver.ubuntu.com --recv-keys 'EB4C 1BFD 4F04 2F6D DDCC  EC91 7721 F63B D38B 4796'; \
# let's also fetch the specific subkey of that key explicitly that we expect "go.tgz.asc" to be signed by, just to make sure we definitely have it
	gpg --batch --keyserver keyserver.ubuntu.com --recv-keys '2F52 8D36 D67B 69ED F998  D857 78BD 6547 3CB3 BD13'; \
	gpg --batch --verify go.tgz.asc go.tgz; \
    gpgconf --kill all; \
	rm -rf "$GNUPGHOME" go.tgz.asc; \
	tar -C /usr/local -xzf go.tgz; \
	rm go.tgz; \
    go version && \
    mkdir -p "$GOPATH/src" "$GOPATH/bin" && chmod -R 1777 "$GOPATH" && \
# apt clean up
	apt-get autoremove -y; \
	apt-get clean; \
	rm -rf /var/lib/apt/lists/*

COPY ./compile-solidity.sh /builds/compile-solidity.sh

# Compile Solidity
RUN set -eux; \
    chmod +x compile-solidity.sh && \
    ./compile-solidity.sh "v$SOLIDITY_VERSION" && \
    mv /builds/solidity/build/solc/solc /usr/local/bin/solc && \
    rm -rf /builds/solidity && \
    rm compile-solidity.sh
