# gateway-proto

[Gateway Dataplne gRPC API proto file](./proto/dataplane.proto)

## Simple Gateway Dataplate gRPC client and fake server for testing

Install using `GOBIN=. go install go.githedgehog.com/gateway-proto/cmd/gwtestctl@master` and run `./gwtestctl` to start
a fake server that implements the gRPC API defined in `proto/dataplane.proto` and just saves/returns config. It allows
to issue get/update calls to a real dataplane as well. Use `./gwtestctl -h` to see all options.
