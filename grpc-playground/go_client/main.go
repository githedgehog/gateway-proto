package main

import (
	"context"
	"fmt"
	"log"
	"time"

	pb "go_client/dataplane/grpc"

	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"
)

func main() {
	conn, err := grpc.Dial("localhost:50051", grpc.WithTransportCredentials(insecure.NewCredentials()))
	if err != nil {
		log.Fatalf("did not connect: %v", err)
	}
	defer conn.Close()
	client := pb.NewConfigServiceClient(conn)

	ctx, cancel := context.WithTimeout(context.Background(), time.Second*5)
	defer cancel()

	// 🔧 Constructing a minimal sane config
	newConfig := &pb.GatewayConfig{
		Devices: []*pb.Device{
			{
				Name:    "eth0",
				Index:   1,
				Ipaddr:  "192.168.1.10",
				Pciaddr: "0000:03:00.0",
				Type:    pb.IfType_IF_TYPE_UPLINK,
			},
		},
		Peerings: []*pb.Peering{
			{
				Name: "peer-group-1",
				Entries: map[string]*pb.PeeringEntry{
					"10.0.0.0/24": {
						Ips: []*pb.PeeringIPs{{Cidr: "10.0.0.0/24"}},
						As:  []*pb.PeeringAs{{Cidr: "65001"}},
					},
				},
			},
		},
		Vrfs: []*pb.VRF{
			{
				Name: "vrf-vpc-1",
				Router: &pb.RouterConfig{
					Asn:      "65000",
					RouterId: "192.168.1.1",
					Neighbors: []*pb.BgpNeighbor{
						{
							Address:         "10.0.0.1",
							RemoteAsn:       "65001",
							AddressFamilies: []string{"ipv4"},
						},
					},
					Options: []*pb.BgpAddressFamilyOptions{
						{
							RedistributeConnected: true,
							Ipv4Enable:            true,
						},
					},
					RouteMaps: []*pb.RouteMap{
						{
							Name:             "EXPORT_ALL",
							MatchPrefixLists: []string{"0.0.0.0/0"},
							Action:           "permit",
							Sequence:         10,
						},
					},
				},
				Vpc: &pb.VPC{
					Id:   "greatest-vpc-1",
					Name: "vpc-1",
					Vni:  1001,
					Subnets: []*pb.Subnet{
						{
							Cidr: "192.168.10.0/24",
							Name: "subnet-1",
						},
					},
				},
			},
		},
	}

	// 🔄 Send the new config
	updateResp, err := client.UpdateConfig(ctx, &pb.UpdateConfigRequest{Config: newConfig})
	if err != nil {
		log.Fatalf("could not update config: %v", err)
	}
	fmt.Println("Update response:", updateResp.Message)

	// ✅ Fetch updated config
	res, err := client.GetConfig(ctx, &pb.GetConfigRequest{})
	if err != nil {
		log.Fatalf("could not get config: %v", err)
	}

	fmt.Printf("Updated GatewayConfig: %+v\n", res)
}
