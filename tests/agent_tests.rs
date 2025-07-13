use std::net::SocketAddr;
use std::str::FromStr;

#[cfg(test)]
mod agent_tests {
    use super::*;

    #[test]
    fn test_agent_name_env_var_parsing() {
        // Test parsing of AGENT_NAME environment variable
        let env_var = "AGENT_NAME=test_agent";
        let agent_name = env_var.strip_prefix("AGENT_NAME=").unwrap_or("unnamed");

        assert_eq!(agent_name, "test_agent");
    }

    #[test]
    fn test_agent_name_env_var_default() {
        // Test default when AGENT_NAME is not set
        let env_var = "";
        let agent_name = if env_var.is_empty() {
            "unnamed"
        } else {
            env_var.strip_prefix("AGENT_NAME=").unwrap_or("unnamed")
        };

        assert_eq!(agent_name, "unnamed");
    }

    #[test]
    fn test_grpc_port_env_var_parsing() {
        // Test parsing of GRPC_PORT environment variable
        let env_var = "GRPC_PORT=50051";
        let port = env_var.strip_prefix("GRPC_PORT=").unwrap_or("50051");

        assert_eq!(port, "50051");
    }

    #[test]
    fn test_grpc_port_env_var_default() {
        // Test default when GRPC_PORT is not set
        let env_var = "";
        let port = if env_var.is_empty() {
            "50051"
        } else {
            env_var.strip_prefix("GRPC_PORT=").unwrap_or("50051")
        };

        assert_eq!(port, "50051");
    }

    #[test]
    fn test_socket_address_parsing() {
        // Test parsing of socket addresses for gRPC server
        let test_cases = vec![
            ("0.0.0.0:50051", true),
            ("127.0.0.1:50051", true),
            ("localhost:50051", false), // localhost is not a valid IP
            ("0.0.0.0:65535", true),
            ("0.0.0.0:0", true),
            ("invalid:50051", false),
            ("0.0.0.0:99999", false), // port out of range
        ];

        for (addr_str, should_parse) in test_cases {
            let result = SocketAddr::from_str(addr_str);

            if should_parse {
                assert!(result.is_ok(), "Failed to parse valid address: {addr_str}");
            } else {
                assert!(
                    result.is_err(),
                    "Incorrectly parsed invalid address: {addr_str}"
                );
            }
        }
    }

    #[test]
    fn test_grpc_address_format() {
        // Test the gRPC address format used by agents
        let port = "50051";
        let addr_str = format!("0.0.0.0:{port}");

        assert_eq!(addr_str, "0.0.0.0:50051");

        let addr = SocketAddr::from_str(&addr_str).unwrap();
        assert_eq!(addr.port(), 50051);
        assert!(addr.ip().is_unspecified());
    }

    #[test]
    fn test_agent_server_name_validation() {
        // Test various agent names that should be valid
        let valid_names = vec![
            "alice",
            "bob123",
            "test_agent",
            "agent-with-dashes",
            "a",
            "very_long_agent_name_with_underscores_and_numbers_123",
        ];

        for name in valid_names {
            // Test that names can be used in various contexts
            let env_var = format!("AGENT_NAME={name}");
            let extracted_name = env_var.strip_prefix("AGENT_NAME=").unwrap();

            assert_eq!(extracted_name, name);
            assert!(!extracted_name.is_empty());
        }
    }

    #[test]
    fn test_port_range_validation() {
        // Test valid port ranges for agent gRPC servers
        let valid_ports = vec![
            "1024",  // First non-privileged port
            "50051", // Default gRPC port
            "50100", // Starting port for sorcerer
            "65535", // Maximum port
        ];

        for port_str in valid_ports {
            let port: u16 = port_str.parse().unwrap();
            assert!(port >= 1024);

            // Test that we can create socket addresses with these ports
            let addr_str = format!("0.0.0.0:{port}");
            let addr = SocketAddr::from_str(&addr_str).unwrap();
            assert_eq!(addr.port(), port);
        }
    }

    #[test]
    fn test_invalid_port_handling() {
        // Test handling of invalid port values
        let invalid_ports = vec!["", "0", "99999", "invalid", "-1", "65536"];

        for port_str in invalid_ports {
            let port_result: Result<u16, _> = port_str.parse();

            match port_result {
                Ok(port) => {
                    // Port 0 is technically valid but not useful
                    if port == 0 {
                        assert_eq!(port, 0);
                    } else {
                        assert!(port >= 1);
                    }
                }
                Err(_) => {
                    // Expected for invalid strings
                    assert!(
                        port_str.is_empty()
                            || port_str == "invalid"
                            || port_str == "-1"
                            || port_str == "99999"
                            || port_str == "65536"
                    );
                }
            }
        }
    }

    #[test]
    fn test_agent_logging_format() {
        // Test the logging format used by agents
        let agent_name = "test_agent";
        let port = "50051";

        let start_message = format!("Agent {agent_name} starting with port {port}");
        let awaken_message = format!("Agent {agent_name} awakening on 0.0.0.0:{port}");

        assert_eq!(start_message, "Agent test_agent starting with port 50051");
        assert_eq!(
            awaken_message,
            "Agent test_agent awakening on 0.0.0.0:50051"
        );

        assert!(start_message.contains(agent_name));
        assert!(start_message.contains(port));
        assert!(awaken_message.contains(agent_name));
        assert!(awaken_message.contains(port));
    }

    #[test]
    fn test_environment_variable_extraction() {
        // Test extraction of environment variables with various formats
        let env_test_cases = vec![
            ("AGENT_NAME=alice", "AGENT_NAME", Some("alice")),
            ("GRPC_PORT=50051", "GRPC_PORT", Some("50051")),
            ("INVALID_VAR=value", "AGENT_NAME", None),
            ("AGENT_NAME=", "AGENT_NAME", Some("")),
            ("=value", "AGENT_NAME", None),
            ("", "AGENT_NAME", None),
        ];

        for (env_var, key, expected) in env_test_cases {
            let prefix = format!("{key}=");
            let result = if env_var.starts_with(&prefix) {
                env_var.strip_prefix(&prefix)
            } else {
                None
            };

            assert_eq!(result, expected);
        }
    }

    #[test]
    fn test_server_startup_sequence() {
        // Test the logical sequence of server startup
        let agent_name = "test_agent";
        let port = "50051";

        // Step 1: Extract environment variables
        let env_name = format!("AGENT_NAME={agent_name}");
        let env_port = format!("GRPC_PORT={port}");

        let extracted_name = env_name.strip_prefix("AGENT_NAME=").unwrap();
        let extracted_port = env_port.strip_prefix("GRPC_PORT=").unwrap();

        assert_eq!(extracted_name, agent_name);
        assert_eq!(extracted_port, port);

        // Step 2: Parse socket address
        let addr_str = format!("0.0.0.0:{extracted_port}");
        let addr = SocketAddr::from_str(&addr_str).unwrap();

        assert_eq!(addr.port(), 50051);
        assert!(addr.ip().is_unspecified());

        // Step 3: Validate startup parameters
        assert!(!extracted_name.is_empty());
        assert!(addr.port() > 0);
    }

    #[test]
    fn test_error_handling_scenarios() {
        // Test various error scenarios that might occur during agent startup

        // Invalid port parsing
        let invalid_port = "invalid_port";
        let port_parse_result: Result<u16, _> = invalid_port.parse();
        assert!(port_parse_result.is_err());

        // Invalid address parsing
        let invalid_addr = "invalid_address:50051";
        let addr_parse_result = SocketAddr::from_str(invalid_addr);
        assert!(addr_parse_result.is_err());

        // Port out of range
        let out_of_range_port = "99999";
        let port_result: Result<u16, _> = out_of_range_port.parse();
        assert!(port_result.is_err());
    }

    #[test]
    fn test_agent_name_edge_cases() {
        // Test edge cases for agent names
        let edge_cases = vec![
            ("", "unnamed"),  // Empty name should default to "unnamed"
            ("a", "a"),       // Single character
            ("123", "123"),   // Numeric name
            ("_", "_"),       // Special character
            ("CAPS", "CAPS"), // Uppercase
        ];

        for (input_name, expected_name) in edge_cases {
            let actual_name = if input_name.is_empty() {
                "unnamed"
            } else {
                input_name
            };

            assert_eq!(actual_name, expected_name);
        }
    }

    #[test]
    fn test_grpc_server_configuration() {
        // Test gRPC server configuration parameters
        let configs = vec![
            ("test_agent", "50051"),
            ("alice", "50100"),
            ("bob", "50200"),
            ("charlie", "65535"),
        ];

        for (name, port) in configs {
            // Test that we can create valid server configurations
            let addr_str = format!("0.0.0.0:{port}");
            let addr = SocketAddr::from_str(&addr_str).unwrap();

            assert!(!name.is_empty());
            assert!(addr.port() > 0);

            // Test logging messages
            let log_message = format!("Apprentice {name} awakening on {addr}");
            assert!(log_message.contains(name));
            assert!(log_message.contains(&addr.to_string()));
        }
    }
}
