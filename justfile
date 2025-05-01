set shell := ["bash", "-euo", "pipefail", "-c"]

import "hack/tools.just"

# Print list of available recipes
default:
  @just --list

export CGO_ENABLED := "0"
go_flags := "-ldflags=\"-w -s -X go.githedgehog.com/gateway-proto/pkg/version.Version=" + version + "\""

_gotools:
  go fmt ./...
  go vet {{go_flags}} ./...

# Run linters against code (incl. license headers)
lint: _license_headers _gotools

_path := `echo $PATH`
gen: _protoc _protoc_gen_go _protoc_gen_go_grpc && lint
  PATH={{_path}}:{{localbinpath}} {{protoc}} --go_out=. --go-grpc_out=. ./proto/dataplane.proto
  cargo build --features regenerate

test: gen
  go test -v ./...
  cargo test -- --nocapture

go_build := "go build " + go_flags
go_linux_build := "GOOS=linux GOARCH=amd64 " + go_build

build: _license_headers _gotools gen && version
  {{go_linux_build}} -o ./bin/gwtestctl ./cmd/gwtestctl

oci_repo := "127.0.0.1:30000"
oci_prefix := "githedgehog/gateway-proto"

docker-build: build (_docker-build "gwtestctl") && version

docker-push: docker-build (_docker-push "gwtestctl") && version

push: docker-push && version
