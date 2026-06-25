#!/bin/bash

PROJECT_DIR=${PWD}

podman run \
  --volume ${HOME}/.config/helix:/root/.config/helix:Z \
  --volume ${PROJECT_DIR}:/project:Z \
  -w /project/ \
  --rm \
  --privileged \ # disable ?
  -it \
  rust-env:latest
