FROM rust:latest
ARG USER_UID=1000
ARG USER_GID=$USER_UID
ARG USERNAME=rust
# Create the user
RUN groupadd --gid $USER_GID $USERNAME \
    && useradd --uid $USER_UID --gid $USER_GID -m $USERNAME 
USER $USERNAME