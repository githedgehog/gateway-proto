package main

import (
	"context"
	"flag"
	"fmt"
	"log"
	"time"

	pb "gateway-client/dataplane/grpc"

	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"
)

var (
	serverAddr = flag.String("server", "localhost:50051", "The server address in the format of host:port")
)

func main() {
	flag.Parse()

	// Set up a connection to the server
	conn, err := grpc.NewClient(*serverAddr, grpc.WithTransportCredentials(insecure.NewCredentials()))
	if err != nil {
		log.Fatalf("Failed to connect: %v", err)
	}
	defer conn.Close()

	client := pb.NewConfigServiceClient(conn)

	// Create context with timeout
	ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
	defer cancel()

	// Example: Get current config
	config, err := client.GetConfig(ctx, &pb.GetConfigRequest{})
	if err != nil {
		log.Fatalf("Could not get config: %v", err)
	}
	fmt.Printf("Current config generation: %d\n", config.Generation)

	// Example: Get config generation
	genResponse, err := client.GetConfigGeneration(ctx, &pb.GetConfigGenerationRequest{})
	if err != nil {
		log.Fatalf("Could not get config generation: %v", err)
	}
	fmt.Printf("Config generation: %d\n", genResponse.Generation)

	// Example: Update config (simplified)
	// In a real client, you'd construct a more complete config
	newConfig := &pb.GatewayConfig{
		Generation: genResponse.Generation + 1,
		Device: &pb.Device{
			Hostname: "gateway-1",
			Loglevel: pb.LogLevel_INFO,
			Driver:   pb.PacketDriver_KERNEL,
		},
	}

	updateResponse, err := client.UpdateConfig(ctx, &pb.UpdateConfigRequest{Config: newConfig})
	if err != nil {
		log.Fatalf("Could not update config: %v", err)
	}

	if updateResponse.Error == pb.Error_ERROR_NONE {
		fmt.Println("Config updated successfully")
	} else {
		fmt.Printf("Config update failed: %s (error code: %v)\n",
			updateResponse.Message, updateResponse.Error)
	}
}
