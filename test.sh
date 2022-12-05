IMAGE=zhangchi0104/dotender:test
CONTAINER_WORKSPACE=/home/rust/dotender
docker run \
    -v $(PWD)/dotender:$CONTAINER_WORKSPACE/dotender \
    -v $(PWD)/dotender_lib:$CONTAINER_WORKSPACE/dotender_lib \
    -v $(PWD)/Cargo.toml:$CONTAINER_WORKSPACE/Cargo.toml \
    -v $(PWD)/Cargo.lock:$CONTAINER_WORKSPACE/Cargo.lock \
    -w $CONTAINER_WORKSPACE \
    --rm -t $IMAGE cargo test
