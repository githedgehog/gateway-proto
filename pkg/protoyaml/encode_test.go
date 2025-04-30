package protoyaml_test

import (
	"testing"

	"github.com/stretchr/testify/require"
	"go.githedgehog.com/gateway-proto/pkg/dataplane"
	"go.githedgehog.com/gateway-proto/pkg/protoyaml"
)

func TestMarshalUnmarshalYAML(t *testing.T) {
	for _, test := range []struct {
		name  string
		input *dataplane.GatewayConfig
		err   bool
	}{
		{
			name: "valid input",
			input: &dataplane.GatewayConfig{
				Generation: 42,
				Underlay: &dataplane.Underlay{
					Vrf: []*dataplane.VRF{
						{
							Name: "vrf1",
							Interfaces: []*dataplane.Interface{
								{
									Name: "eth0",
									Type: dataplane.IfType_IF_TYPE_VLAN,
								},
							},
						},
					},
				},
			},
		},
	} {
		t.Run(test.name, func(t *testing.T) {
			data, err := protoyaml.MarshalYAML(test.input)
			if test.err {
				require.Error(t, err)

				return
			}

			require.NoError(t, err)
			require.NotNil(t, data)

			actual := &dataplane.GatewayConfig{}
			err = protoyaml.UnmarshalYAML(data, actual)
			require.NoError(t, err)

			require.Equal(t, test.input, actual)
		})
	}
}

func TestUnmarshal(t *testing.T) {
	for _, test := range []struct {
		name  string
		input string
		check func(t *testing.T, actual *dataplane.GatewayConfig)
	}{
		{
			name: "default-value-enum-explicit",
			input: `
generation: 42
underlay:
  vrf:
  - interfaces:
    - name: eth0
      type: IF_TYPE_ETHERNET
    name: vrf1
`,
			check: func(t *testing.T, actual *dataplane.GatewayConfig) {
				require.Equal(t, uint64(42), actual.Generation)
				require.Len(t, actual.Underlay.Vrf, 1)
				require.Equal(t, "vrf1", actual.Underlay.Vrf[0].Name)
				require.Len(t, actual.Underlay.Vrf[0].Interfaces, 1)
				require.Equal(t, "eth0", actual.Underlay.Vrf[0].Interfaces[0].Name)
				require.Equal(t, dataplane.IfType_IF_TYPE_ETHERNET, actual.Underlay.Vrf[0].Interfaces[0].Type)
			},
		},
		{
			name: "default-value-enum-implicit",
			input: `
generation: 42
underlay:
  vrf:
  - interfaces:
    - name: eth0
    name: vrf1
`,
			check: func(t *testing.T, actual *dataplane.GatewayConfig) {
				require.Equal(t, uint64(42), actual.Generation)
				require.Len(t, actual.Underlay.Vrf, 1)
				require.Equal(t, "vrf1", actual.Underlay.Vrf[0].Name)
				require.Len(t, actual.Underlay.Vrf[0].Interfaces, 1)
				require.Equal(t, "eth0", actual.Underlay.Vrf[0].Interfaces[0].Name)
				require.Equal(t, dataplane.IfType_IF_TYPE_ETHERNET, actual.Underlay.Vrf[0].Interfaces[0].Type)
			},
		},
		{
			name: "non-default-value-enum",
			input: `
generation: 42
underlay:
  vrf:
  - interfaces:
    - name: eth0
      type: IF_TYPE_VLAN
    name: vrf1
`,
			check: func(t *testing.T, actual *dataplane.GatewayConfig) {
				require.Equal(t, uint64(42), actual.Generation)
				require.Len(t, actual.Underlay.Vrf, 1)
				require.Equal(t, "vrf1", actual.Underlay.Vrf[0].Name)
				require.Len(t, actual.Underlay.Vrf[0].Interfaces, 1)
				require.Equal(t, "eth0", actual.Underlay.Vrf[0].Interfaces[0].Name)
				require.Equal(t, dataplane.IfType_IF_TYPE_VLAN, actual.Underlay.Vrf[0].Interfaces[0].Type)
			},
		},
		{
			name: "uint-as-string",
			input: `
generation: "42"
`,
			check: func(t *testing.T, actual *dataplane.GatewayConfig) {
				require.Equal(t, uint64(42), actual.Generation)
			},
		},
	} {
		t.Run(test.name, func(t *testing.T) {
			actual := &dataplane.GatewayConfig{}
			err := protoyaml.UnmarshalYAML([]byte(test.input), actual)
			require.NoError(t, err)

			test.check(t, actual)
		})
	}
}
