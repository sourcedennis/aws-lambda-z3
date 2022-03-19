# Amazon Lambda images should build upon `amazonlinux`.
# 
# We use a multi-stage build to eliminate build dependencies.
FROM amazonlinux:latest AS build


# -- Install Rust --

RUN curl https://sh.rustup.rs -sSf | /bin/sh -s -- -y

ENV PATH="/root/.cargo/bin:${PATH}"

RUN rustup install stable


# -- Install Z3 --

# `git` is to download Z3. `clang` and `make` to build it
RUN yum install -y git clang make &&\
    yum clean all &&\
    rm -rf /var/cache/yum

WORKDIR /root

# We download Z3 and install it to `/root/z3`.
# These build instructions were taken from:
# https://github.com/Z3Prover/z3/blob/master/README.md
RUN git clone --depth 1 https://github.com/Z3Prover/z3 z3-src &&\
    cd z3-src &&\
    python scripts/mk_make.py --prefix=/root/z3 &&\
    cd build &&\
    make -j 8 &&\
    make install

# Z3 headers used by the `z3-sys` Rust package
ENV CPATH="/root/z3/include"
# Used to link Z3 with our project
ENV LIBRARY_PATH="/root/z3/lib:${LIBRARY_PATH}"


# -- Setup our project --

RUN mkdir code

WORKDIR code

# Copy the local project into the container
# Note that the `.dockerignore` file ensures not everything gets copied.
COPY . .

RUN cargo build --release



# The final Docker stage, which contains only runtime requirements
FROM amazonlinux:latest

WORKDIR /root

# Give execution right to others. This ensures Lambda can execute our program.
RUN chmod o+rx /root

# Copy Z3 inside. We only copy the `lib` directory, as it contains the
# runtime dependencies.
COPY --from=build /root/z3/lib /root/z3/lib

# Copy our program inside
COPY --from=build /root/code/target/release/aws_lambda_z3 program

# Ensure our program can find the Z3 runtime dependencies
ENV LD_LIBRARY_PATH="/root/z3/lib:${LD_LIBRARY_PATH}"

# Tell Lambda to run the program upon spawning the container
ENTRYPOINT [ "./program" ]
