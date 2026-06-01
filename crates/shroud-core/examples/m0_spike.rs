use arti_client::{TorClient, TorClientConfig};
use tor_hsservice::config::OnionServiceConfigBuilder;
use tor_cell::relaycell::msg::Connected;
use futures::StreamExt;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Initialize tracing/logging
    tracing_subscriber::fmt::init();
    println!("Starting shroud-speak M0 spike...");

    // 2. Build Tor config and explicitly enable onion addresses
    let mut config_builder = TorClientConfig::builder();
    config_builder
        .address_filter()
        .allow_onion_addrs(true);
    let config = config_builder.build()?;

    // 3. Bootstrapping TorClient
    let tor_client = TorClient::create_bootstrapped(config).await?;
    println!("Tor client bootstrapped successfully!");

    // 4. Verify Vanguards / DoS hardening state
    println!("Verifying Vanguards and DoS hardening status...");
    let runtime_config = tor_client.config();
    println!("Configured Vanguards State: {:?}", runtime_config.vanguards());

    // 5. Configure the onion service (nickname speak_spike, no hyphens)
    let svc_cfg = OnionServiceConfigBuilder::default()
        .nickname("speak_spike".to_owned().try_into()?)
        .build()?;

    // 6. Launch the onion service
    let (running_service, rend_requests) = tor_client.launch_onion_service(svc_cfg)?;
    println!("Onion service launched successfully!");

    // 7. Retrieve and print the .onion address using onion_name()
    let hsid = running_service
        .onion_name()
        .ok_or_else(|| anyhow::anyhow!("Failed to retrieve HsId from running service"))?;
    let onion_address = hsid.to_string();
    println!("\n========================================");
    println!("ONION SERVICE HOSTED!");
    println!("Address: {}", onion_address);
    println!("========================================\n");

    // 8. Spawn task to handle incoming streams
    let mut stream_requests = tor_hsservice::handle_rend_requests(rend_requests);
    tokio::spawn(async move {
        while let Some(stream_req) = stream_requests.next().await {
            tokio::spawn(async move {
                println!("Incoming stream request received!");
                let connected_msg = Connected::new_empty();
                match stream_req.accept(connected_msg).await {
                    Ok(mut data_stream) => {
                        println!("Stream accepted successfully. Reading bytes...");
                        let mut buf = [0u8; 1024];
                        match data_stream.read(&mut buf).await {
                            Ok(n) => {
                                let received = String::from_utf8_lossy(&buf[..n]);
                                println!("Service Received: {}", received);
                                
                                // Echo back the message
                                let response = format!("Echo: {}", received);
                                if let Err(e) = data_stream.write_all(response.as_bytes()).await {
                                    eprintln!("Failed to write echo back: {:?}", e);
                                }
                            }
                            Err(e) => eprintln!("Failed to read from stream: {:?}", e),
                        }
                    }
                    Err(e) => eprintln!("Failed to accept stream request: {:?}", e),
                }
            });
        }
    });

    // 9. Dial the onion service with a timeout/retry loop to handle descriptor publication delay
    println!("Dialing self at {} (waiting for descriptor publication)...", onion_address);
    let mut client_stream = None;
    let max_attempts = 36; // 3 minutes total (5s delay between retries)
    
    for attempt in 1..=max_attempts {
        match tor_client.connect((onion_address.as_str(), 80)).await {
            Ok(stream) => {
                client_stream = Some(stream);
                println!("Connection established on attempt {}!", attempt);
                break;
            }
            Err(e) => {
                if attempt == max_attempts {
                    return Err(anyhow::anyhow!("Failed to connect after {} attempts: {:?}", max_attempts, e));
                }
                println!("Attempt {} failed (descriptor publishing): {}. Retrying in 5s...", attempt, e);
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
        }
    }
    
    let mut client_stream = client_stream.expect("Stream should be populated");

    // 10. Write bytes and read response
    let msg = "Hello through Tor onion service!";
    client_stream.write_all(msg.as_bytes()).await?;
    client_stream.flush().await?;

    let mut resp_buf = [0u8; 1024];
    let n = client_stream.read(&mut resp_buf).await?;
    println!("Client Received Response: {}", String::from_utf8_lossy(&resp_buf[..n]));

    println!("M0 spike successful!");
    Ok(())
}
