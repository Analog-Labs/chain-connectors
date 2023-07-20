FROM docker.io/library/ubuntu:22.04

ARG VCS_REF=master
ARG BUILD_DATE=""
ARG REGISTRY_PATH=docker.io/analoglabs

# metadata
LABEL summary="Layer 1 image with all dependencies for Rust compilation." \
      name="${REGISTRY_PATH}/base-ci-linux" \
      maintainer="lohann@analog.one" \
      version="1.0.0" \
      description="libssl-dev, clang, libclang-dev, libsasl2-dev, lld, cmake, make, git, pkg-config \
curl, jq, time, lsof, rhash, rust stable, rust nightly, sccache, cargo-udeps, cargo-tarpaulin" \
      one.analog.image.vendor="Analog One Foundation" \
      one.analog.image.source="https://github.com/Analog-Labs/chain-connectors/blob/${VCS_REF}/\
ci/dockerfiles/base-ci-linux/Dockerfile" \
      one.analog.image.documentation="https://github.com/Analog-Labs/chain-connectors/blob/${VCS_REF}/\
ci/dockerfiles/base-ci-linux/README.md" \
      one.analog.image.revision="${VCS_REF}" \
      one.analog.image.created="${BUILD_DATE}"

WORKDIR /builds

# config for clang 15
COPY ./base-ci-linux-config /root/.cargo/config

ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH \
    RUST_VERSION=1.70.0 \
    RUST_NIGHTLY=2023-05-23 \
    CC=clang-15 \
    CXX=clang-15

# install tools and dependencies
RUN set -eux; \
	apt-get -y update; \
	dpkgArch="$(dpkg --print-architecture)"; \
	apt-get install -y --no-install-recommends \
		libssl-dev libsasl2-dev make cmake graphviz \
		git pkg-config curl time rhash ca-certificates jq \
		python3 python3-pip lsof ruby ruby-bundler git-restore-mtime xz-utils unzip gnupg protobuf-compiler && \
# add clang 15 repo
	curl -s https://apt.llvm.org/llvm-snapshot.gpg.key | gpg --dearmor -o /usr/share/keyrings/llvm-snapshot.gpg; \
	echo "deb [arch=${dpkgArch} signed-by=/usr/share/keyrings/llvm-snapshot.gpg] http://apt.llvm.org/jammy/ llvm-toolchain-jammy-15 main" >> /etc/apt/sources.list.d/llvm-toolchain-jammy-15.list; \
	apt-get -y update; \
	apt-get install -y --no-install-recommends \
		clang-15 lldb-15 lld-15 libclang-15-dev && \
# add non-root user
	groupadd -g 1000 nonroot && \
	useradd -u 1000 -g 1000 -s /bin/bash -m nonroot && \
# set a link to clang
	update-alternatives --install /usr/bin/cc cc /usr/bin/clang-15 100; \
# set a link to ldd
	update-alternatives --install /usr/bin/ld ld /usr/bin/ld.lld-15 100; \
# install rustup, use minimum components
	case "${dpkgArch##*-}" in \
        amd64) rustArch='x86_64-unknown-linux-gnu'; rustupSha256='0b2f6c8f85a3d02fde2efc0ced4657869d73fccfce59defb4e8d29233116e6db' ;; \
        armhf) rustArch='armv7-unknown-linux-gnueabihf'; rustupSha256='f21c44b01678c645d8fbba1e55e4180a01ac5af2d38bcbd14aa665e0d96ed69a' ;; \
        arm64) rustArch='aarch64-unknown-linux-gnu'; rustupSha256='673e336c81c65e6b16dcdede33f4cc9ed0f08bde1dbe7a935f113605292dc800' ;; \
        i386) rustArch='i686-unknown-linux-gnu'; rustupSha256='e7b0f47557c1afcd86939b118cbcf7fb95a5d1d917bdd355157b63ca00fc4333' ;; \
        *) echo >&2 "unsupported architecture: ${dpkgArch}"; exit 1 ;; \
    esac; \
    url="https://static.rust-lang.org/rustup/archive/1.26.0/${rustArch}/rustup-init"; \
	curl -L "$url" -o rustup-init; \
	echo "${rustupSha256} *rustup-init" | sha256sum -c -; \
	chmod +x rustup-init; \
	./rustup-init -y --no-modify-path --profile minimal --default-toolchain $RUST_VERSION --default-host ${rustArch}; \
	rm rustup-init; \
	chmod -R a+w $RUSTUP_HOME $CARGO_HOME; \
	chown -R root:nonroot ${RUSTUP_HOME} ${CARGO_HOME}; \
	chmod -R g+w ${RUSTUP_HOME} ${CARGO_HOME}; \
# versions
	rustup show; \
	rustup --version; \
	cargo --version; \
	rustc --version; \
# install sccache
	cargo install sccache --features redis --version 0.3.3; \
# cargo clean up
# removes compilation artifacts cargo install creates (>250M)
	rm -rf "${CARGO_HOME}/registry" "${CARGO_HOME}/git" /root/.cache/sccache; \
# apt clean up
	apt-get autoremove -y; \
	apt-get clean; \
	rm -rf /var/lib/apt/lists/*

# cache handler
ENV RUSTC_WRAPPER=sccache \
    # show backtraces
    RUST_BACKTRACE=1

# install tools and dependencies
RUN set -eux && \
	dpkgArch="$(dpkg --print-architecture)"; \
	case "${dpkgArch##*-}" in \
		amd64) rustArch='x86_64-unknown-linux-gnu'; rustTargetArch='x86_64-unknown-linux-musl' ;; \
		armhf) rustArch='armv7-unknown-linux-gnueabihf'; rustTargetArch='armv7-unknown-linux-musleabi' ;; \
		arm64) rustArch='aarch64-unknown-linux-gnu'; rustTargetArch='aarch64-unknown-linux-musl' ;; \
		i386) rustArch='i686-unknown-linux-gnu'; rustTargetArch='i686-unknown-linux-musl' ;; \
		*) echo >&2 "unsupported architecture: ${dpkgArch}"; exit 1 ;; \
	esac; \
# install `rust-src` component for ui test
	rustup component add rust-src rustfmt clippy && \
# install wasm target into default (stable) toolchain
    rustup target add wasm32-unknown-unknown && \
# install linux-musl target into default (stable) toolchain
    rustup target add ${rustTargetArch} && \
# install specific Rust nightly, default is stable, use minimum components
	rustup toolchain install nightly-${RUST_NIGHTLY} --profile minimal --component rustfmt clippy && \
# install wasm target into nightly toolchain
    rustup target add wasm32-unknown-unknown --toolchain "nightly-${RUST_NIGHTLY}" && \
# install linux-musl target into nightly toolchain
    rustup target add ${rustTargetArch} --toolchain "nightly-${RUST_NIGHTLY}" && \
# "alias" pinned nightly toolchain as nightly
	ln -s /usr/local/rustup/toolchains/nightly-${RUST_NIGHTLY}-${rustArch} /usr/local/rustup/toolchains/nightly-${rustArch} && \
# install wasm-pack
    cargo install wasm-pack --version 0.12.1 && \
# install cargo deny
	cargo install cargo-deny --version 0.13.9 && \
# install dprint
	cargo +nightly install dprint --version 0.38.3 && \
# install udeps
	cargo +nightly install cargo-udeps --version 0.1.40 && \
# install tarpaulin
    cargo +nightly install cargo-tarpaulin --version 0.26.1 && \
# versions
	rustup show && \
	cargo --version && \
# apt clean up
	apt-get autoremove -y && \
	apt-get clean && \
	rm -rf /var/lib/apt/lists/* && \
# cargo clean up
# removes compilation artifacts cargo install creates (>250M)
	rm -rf "${CARGO_HOME}/registry" "${CARGO_HOME}/git" /root/.cache/sccache
