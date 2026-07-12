#!/bin/bash

PROJECT_DIR=${PWD}

# would be used to forward ssh agent but does not work because of selinux policy
# instead we bind mount .ssh
# --volume $SSH_AUTH_SOCK:$SSH_AUTH_SOCK:Z \
# --env SSH_AUTH_SOCK=$SSH_AUTH_SOCK \

podman run \
  --rm \
  --interactive \
  --tty \
  --volume ${PROJECT_DIR}:/workspace:Z \
  --volume ${HOME}/.config/helix:/home/dev/.config/helix:Z,ro \
  --volume ${HOME}/.gitconfig:/home/dev/.gitconfig:Z,ro \
  --volume ${HOME}/.ssh:/home/dev/.ssh:Z,ro \
  --userns=keep-id \
  --name=rust-dev-container \
  rust-dev:latest
