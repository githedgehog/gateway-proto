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
