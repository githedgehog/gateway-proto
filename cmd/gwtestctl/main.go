// Copyright 2025 Hedgehog
// SPDX-License-Identifier: Apache-2.0

package main

import (
	"context"
	"fmt"
	"log/slog"
	"os"
	"strings"
	"time"

	"github.com/lmittmann/tint"
	"github.com/mattn/go-isatty"
	"github.com/samber/lo"
	"github.com/urfave/cli/v2"
	"go.githedgehog.com/gateway-proto/pkg/gwtestctl"
	"go.githedgehog.com/gateway-proto/pkg/version"
)

func main() {
	if err := Run(context.Background()); err != nil {
		slog.Error(err.Error())
		os.Exit(1)
	}
}

func Run(ctx context.Context) error {
	var verbose, brief bool
	verboseFlag := &cli.BoolFlag{
		Name:        "verbose",
		Aliases:     []string{"v"},
		Usage:       "verbose output (includes debug)",
		Destination: &verbose,
	}
	briefFlag := &cli.BoolFlag{
		Name:        "brief",
		Aliases:     []string{"b"},
		Usage:       "brief output (only warn and error)",
		Destination: &brief,
	}

	before := func(installLog bool) cli.BeforeFunc {
		return func(_ *cli.Context) error {
			if verbose && brief {
				return cli.Exit("verbose and brief are mutually exclusive", 1)
			}

			logLevel := slog.LevelInfo
			if verbose {
				logLevel = slog.LevelDebug
			} else if brief {
				logLevel = slog.LevelWarn
			}

			logW := os.Stderr
			slog.SetDefault(slog.New(tint.NewHandler(logW, &tint.Options{
				Level:      logLevel,
				TimeFormat: time.TimeOnly,
				NoColor:    !isatty.IsTerminal(logW.Fd()),
			})))

			args := []any{
				"version", version.Version,
			}

			slog.Info("Hedgehog Gateway Test Client", args...)

			return nil
		}
	}

	defaultFlags := []cli.Flag{
		verboseFlag,
		briefFlag,
	}

	target := ""
	targetFlags := []cli.Flag{
		&cli.StringFlag{
			Name:        "target",
			Aliases:     []string{"t"},
			Usage:       "gRPC client/server target: unix:///path/to/socket or tcp://host:port or tcp://: for random port",
			Destination: &target,
			Required:    true,
		},
	}

	configFile := ""
	configFileFlags := []cli.Flag{
		&cli.StringFlag{
			Name:        "config-file",
			Aliases:     []string{"f"},
			Usage:       "path to config file",
			Destination: &configFile,
			Required:    true,
		},
	}

	cli.VersionFlag.(*cli.BoolFlag).Aliases = []string{"V"}
	app := &cli.App{
		Name:  "gwtestctl",
		Usage: "Simple Hedgehog Gateway Dataplate gRPC client and fake server for testing",
		UsageText: strings.TrimSpace(strings.ReplaceAll(`
			Run fake server:
			  gwtestctl server -t unix://tmp/gateway.sock # unix socket
			  gwtestctl server -t tcp://localhost:5123 # tcp socket
			  gwtestctl server -t tcp://:5123 # tcp socket on all interfaces
			  gwtestctl server -t tcp://:0 # tcp socket on all interfaces with random port

			And run client:
			  gwtestctl get-config -t unix://tmp/gateway.sock # read current config from server
			  gwtestctl get-config -t tcp://:5123 > config.yaml # save current config to file
			  gwtestctl update-config -t tcp://:5123 -f config.yaml # update config on server
			  gwtestctl get-config-gen -t tcp://:5123 # read current config generation from server

			Config is a YAML representation of the GatewayConfig protobuf message, e.g.:
			  generation: 42
			  underlay:
			    vrf:
			    - name: vrf1
			      interfaces:
			      - name: eth0
			        type: IF_TYPE_VLAN
		`, "			", "")),
		Version:                version.Version,
		Suggest:                true,
		UseShortOptionHandling: true,
		EnableBashCompletion:   true,
		Commands: []*cli.Command{
			{
				Name:    "get-config",
				Aliases: []string{"get"},
				Usage:   "get config",
				Flags:   flatten(defaultFlags, targetFlags),
				Before:  before(true),
				Action: func(_ *cli.Context) error {
					if err := gwtestctl.DoGetConfig(ctx, target, false); err != nil {
						return fmt.Errorf("getting config: %w", err)
					}

					return nil
				},
			},
			{
				Name:    "get-config-gen",
				Aliases: []string{"gen"},
				Usage:   "get config generation",
				Flags:   flatten(defaultFlags, targetFlags),
				Before:  before(true),
				Action: func(_ *cli.Context) error {
					if err := gwtestctl.DoGetConfigGeneration(ctx, target, false); err != nil {
						return fmt.Errorf("getting config gen: %w", err)
					}

					return nil
				},
			},
			{
				Name:    "update-config",
				Aliases: []string{"set"},
				Usage:   "update config from file",
				Flags:   flatten(defaultFlags, targetFlags, configFileFlags),
				Before:  before(true),
				Action: func(_ *cli.Context) error {
					if err := gwtestctl.DoUpdateConfig(ctx, target, configFile, false); err != nil {
						return fmt.Errorf("updating config: %w", err)
					}

					return nil
				},
			},
			{
				Name:    "fake-server",
				Aliases: []string{"server"},
				Usage:   "run fake server",
				Flags:   flatten(defaultFlags, targetFlags),
				Before:  before(true),
				Action: func(_ *cli.Context) error {
					if err := gwtestctl.DoFakeServer(ctx, target); err != nil {
						return fmt.Errorf("running fake server: %w", err)
					}

					return nil
				},
			},
		},
	}

	return app.Run(os.Args) //nolint:wrapcheck
}

func flatten[T any, Slice ~[]T](collection ...Slice) Slice {
	return lo.Flatten(collection)
}
