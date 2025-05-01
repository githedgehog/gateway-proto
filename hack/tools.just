#
# Version handling
#

version_extra := ""
version_dirty := `[ -z "$(git status -s)" ] || echo "-$(LC_ALL=C tr -dc 'a-zA-Z0-9' < /dev/urandom | head -c 2)$(date +"%H%M")"`
version := `git describe --tags --dirty --always` + version_dirty + version_extra

# Print version that will be used in the build
version:
  @echo "Using version: {{version}}"

_is_clean:
  [ -z "`git status -s`" ] || exit 1

#
# Downloading all tools locally
#

localbin := "bin"
localpath := `pwd`
localbinpath := `pwd`/localbin

_localbin:
  @mkdir -p {{localbin}}

# Download all tools locally
tools: _protoc _protoc_gen_go _protoc_gen_go_grpc _actionlint _addlicense _skopeo _oras

arch_raw := `uname -m`
os_raw := `uname|tr '[:upper:]' '[:lower:]'`

protoc_version := "30.2"
protoc := localbin / "protoc" + "-" + protoc_version
@_protoc: _localbin
  [ -f {{protoc}} ] || just _protoc_install

protoc_arch := if arch_raw == "arm64" { "aarch_64" } else { arch_raw }
protoc_os := if os_raw == "darwin" { "osx" } else { os_raw }
@_protoc_install: _localbin
  echo "Downloading protoc v{{protoc_version}} {{protoc_arch}}/{{protoc_os}}"

  rm -rf {{localbin}}/protoc-tmp.zip {{localbin}}/protoc-tmp || true

  wget -O {{localbin}}/protoc-tmp.zip "https://github.com/protocolbuffers/protobuf/releases/download/v{{protoc_version}}/protoc-{{protoc_version}}-{{protoc_os}}-{{protoc_arch}}.zip"
  mkdir {{localbin}}/protoc-tmp
  unzip {{localbin}}/protoc-tmp.zip -d {{localbin}}/protoc-tmp
  mv {{localbin}}/protoc-tmp/bin/protoc {{protoc}}

  rm -rf {{localbin}}/protoc-tmp.zip {{localbin}}/protoc-tmp

  # go install helper
_goinstall PACKAGE VERSION BINNAME TARGET FLAGS="": _localbin
  #!/usr/bin/env bash
  set -euo pipefail

  echo "Installing go package: {{PACKAGE}}@{{VERSION}}..."
  GOBIN=`pwd`/{{localbin}} go install {{FLAGS}} {{PACKAGE}}@{{VERSION}}
  mv {{localbin}}/{{BINNAME}} {{TARGET}}

# actionlint
actionlint_version := "v1.7.3"
actionlint := localbin / "actionlint" + "-" + actionlint_version
@_actionlint: _localbin
  [ -f {{actionlint}} ] || just _goinstall "github.com/rhysd/actionlint/cmd/actionlint" {{actionlint_version}} "actionlint" {{actionlint}}

# addlicense
addlicense_version := "v1.1.1"
addlicense := localbin / "addlicense" + "-" + addlicense_version
@_addlicense: _localbin
  [ -f {{addlicense}} ] || just _goinstall "github.com/google/addlicense" {{addlicense_version}} "addlicense" {{addlicense}}

# protoc-gen-go
protoc_gen_go_version := "v1.36.6"
protoc_gen_go_bin := "protoc-gen-go" + "-" + protoc_gen_go_version
protoc_gen_go := localbin / protoc_gen_go_bin
@_protoc_gen_go: _localbin
  [ -f {{protoc_gen_go}} ] || just _goinstall "google.golang.org/protobuf/cmd/protoc-gen-go" {{protoc_gen_go_version}} "protoc-gen-go" {{protoc_gen_go}}
  ln -sf {{protoc_gen_go_bin}} {{localbin}}/protoc-gen-go

# protoc-gen-go-grpc
protoc_gen_go_grpc_version := "v1.5.1"
protoc_gen_go_grpc_bin := "protoc-gen-go-grpc" + "-" + protoc_gen_go_grpc_version
protoc_gen_go_grpc := localbin / protoc_gen_go_grpc_bin
@_protoc_gen_go_grpc: _localbin
  [ -f {{protoc_gen_go_grpc}} ] || just _goinstall "google.golang.org/grpc/cmd/protoc-gen-go-grpc" {{protoc_gen_go_grpc_version}} "protoc-gen-go-grpc" {{protoc_gen_go_grpc}}
  ln -sf {{protoc_gen_go_grpc_bin}} {{localbin}}/protoc-gen-go-grpc

# skopeo
skopeo_version := "v1.16.1"
skopeo := localbin / "skopeo" + "-" + skopeo_version
@_skopeo: _localbin
  [ -f {{skopeo}} ] || just _goinstall "github.com/containers/skopeo/cmd/skopeo" {{skopeo_version}} "skopeo" {{skopeo}} "--tags containers_image_openpgp,exclude_graphdriver_btrfs"

# oras
oras_version := "v1.2.0"
oras := localbin / "oras" + "-" + oras_version
@_oras: _localbin
  [ -f {{oras}} ] || just _goinstall "oras.land/oras/cmd/oras" {{oras_version}} "oras" {{oras}}

#
# Common recipes
#

_license_headers: _addlicense
  @{{addlicense}} -l apache -s=only -c Hedgehog -ignore ".github/**" -ignore "config/**" -ignore "**/*.yaml" -ignore "**/*.yml" -ignore "vendor/**" -y 2025 .

# Run linter for GitHub Actions
lint-gha: _actionlint
   {{actionlint}} -verbose

#
# Docker recipes
#

oci := ""
oras_insecure := if oci == "http" { "--plain-http" } else { "" }
skopeo_dest_insecure := if oci == "http" { "--dest-tls-verify=false" } else { "" }

_docker-build bin: build
  cp bin/{{bin}} config/docker/{{bin}}/
  cd config/docker/{{bin}}/ && docker build --platform=linux/amd64 -t {{oci_repo}}/{{oci_prefix}}/{{bin}}:{{version}} -f Dockerfile .

skopeo_copy_flags := if env("DOCKER_HOST", "") != "" { "--src-daemon-host " + env_var("DOCKER_HOST") } else { "" }
_docker-push bin: _skopeo
  {{skopeo}} --insecure-policy copy {{skopeo_copy_flags}} {{skopeo_dest_insecure}} docker-daemon:{{oci_repo}}/{{oci_prefix}}/{{bin}}:{{version}} docker://{{oci_repo}}/{{oci_prefix}}/{{bin}}:{{version}}
