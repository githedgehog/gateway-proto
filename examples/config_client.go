package main

import (
	"context"
	"encoding/json"
	"flag"
	"fmt"
	"log"
	"os"
	"time"

	pb "go.githedgehog.com/gateway-proto/pkg/dataplane"

	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"
	"google.golang.org/protobuf/encoding/protojson"
	"gopkg.in/yaml.v3"
)

var (
	serverAddr = flag.String("server", "localhost:20051", "The server address in the format of host:port")
	configFile = flag.String("config", "gateway_config.yaml", "Path to YAML configuration file")
	verifyOnly = flag.Bool("verify", false, "Verify configuration file without connecting to server")
)

// SMATOV: gRPC don't support YAML directly, so we need to convert YAML to JSON
func yamlToJSON(yamlData []byte) ([]byte, error) {
	var body interface{}
	if err := yaml.Unmarshal(yamlData, &body); err != nil {
		return nil, fmt.Errorf("error unmarshaling YAML: %v", err)
	}

	jsonData, err := json.Marshal(body)
	if err != nil {
		return nil, fmt.Errorf("error marshaling to JSON: %v", err)
	}

	return jsonData, nil
}

func loadConfigFromFile(filePath string) (*pb.GatewayConfig, error) {
	yamlData, err := os.ReadFile(filePath)
	if err != nil {
		return nil, fmt.Errorf("error reading config file: %v", err)
	}

	jsonData, err := yamlToJSON(yamlData)
	if err != nil {
		return nil, err
	}

	config := &pb.GatewayConfig{}

	err = protojson.Unmarshal(jsonData, config)
	if err != nil {
		return nil, fmt.Errorf("error unmarshaling to protobuf: %v", err)
	}

	if config.Device == nil {
		return nil, fmt.Errorf("missing device configuration")
	}
	if config.Underlay == nil {
		return nil, fmt.Errorf("missing underlay configuration")
	}
	if config.Overlay == nil {
		return nil, fmt.Errorf("missing overlay configuration")
	}

	return config, nil
}

func verifyConfig(filePath string) error {
	fmt.Printf("Verifying configuration file: %s\n", filePath)

	config, err := loadConfigFromFile(filePath)
	if err != nil {
		return fmt.Errorf("verification failed: %v", err)
	}

	fmt.Println("Configuration verification successful!")
	fmt.Printf("- Device hostname: %s\n", config.Device.Hostname)
	fmt.Printf("- Generation: %d\n", config.Generation)

	if len(config.Underlay.Vrf) > 0 {
		fmt.Printf("- VRF count: %d\n", len(config.Underlay.Vrf))
	}

	if len(config.Overlay.Vpcs) > 0 {
		fmt.Printf("- VPC count: %d\n", len(config.Overlay.Vpcs))
		for _, vpc := range config.Overlay.Vpcs {
			fmt.Printf("  - VPC: %s (VNI: %d)\n", vpc.Name, vpc.Vni)
		}
	}

	if len(config.Overlay.Peerings) > 0 {
		fmt.Printf("- VPC peering count: %d\n", len(config.Overlay.Peerings))
		for _, peering := range config.Overlay.Peerings {
			fmt.Printf("  - Peering: %s\n", peering.Name)
		}
	}

	return nil
}

func main() {
	flag.Parse()

	if *verifyOnly {
		if err := verifyConfig(*configFile); err != nil {
			log.Fatalf("Config verification failed: %v", err)
		}
		return
	}

	conn, err := grpc.NewClient(*serverAddr, grpc.WithTransportCredentials(insecure.NewCredentials()))
	if err != nil {
		log.Fatalf("Failed to connect: %v", err)
	}
	defer conn.Close()

	client := pb.NewConfigServiceClient(conn)

	ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
	defer cancel()

	genResponse, err := client.GetConfigGeneration(ctx, &pb.GetConfigGenerationRequest{})
	if err != nil {
		log.Fatalf("Could not get config generation: %v", err)
	}
	fmt.Printf("Current config generation: %d\n", genResponse.Generation)

	config, err := loadConfigFromFile(*configFile)
	if err != nil {
		log.Fatalf("Failed to load configuration: %v", err)
	}

	config.Generation = genResponse.Generation + 1
	fmt.Printf("Updating with new config generation: %d\n", config.Generation)

	updateResponse, err := client.UpdateConfig(ctx, &pb.UpdateConfigRequest{Config: config})
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
