use anyhow::{Context, Result};
use lact_client::DaemonClient;
use lact_schema::args::{CliArgs, CliCommand};

pub fn run(args: CliArgs) -> Result<()> {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    rt.block_on(async move {
        let client = DaemonClient::connect().await?;

        match args.subcommand {
            CliCommand::ListGpus => list_gpus(&args, &client).await,
            CliCommand::Info => info(&args, &client).await,
            CliCommand::Snapshot => snapshot(&client).await,
        }
    })
}

async fn list_gpus(_: &CliArgs, client: &DaemonClient) -> Result<()> {
    let entries = client.list_devices().await?;
    for entry in entries {
        let id = entry.id;
        if let Some(name) = entry.name {
            println!("{id} ({name})");
        } else {
            println!("{id}");
        }
    }
    Ok(())
}

async fn info(args: &CliArgs, client: &DaemonClient) -> Result<()> {
    for id in extract_gpu_ids(args, client).await {
        let info = client.get_device_info(&id).await?;
        let pci_info = info.pci_info.context("GPU reports no pci info")?;

        if let Some(ref vendor) = pci_info.device_pci_info.vendor {
            println!("GPU Vendor: {vendor}");
        }
        if let Some(ref model) = pci_info.device_pci_info.model {
            println!("GPU Model: {model}");
        }
        println!("Driver in use: {}", info.driver);
        if let Some(ref vbios_version) = info.vbios_version {
            println!("VBIOS version: {vbios_version}");
        }
        println!("Link: {:?}", info.link_info);
    }
    Ok(())
}

async fn extract_gpu_ids(args: &CliArgs, client: &DaemonClient) -> Vec<String> {
    match args.gpu_id {
        Some(ref id) => vec![id.clone()],
        None => {
            let entries = client.list_devices().await.expect("Could not list GPUs");
            entries
                .into_iter()
                .map(|entry| entry.id.to_owned())
                .collect()
        }
    }
}

async fn snapshot(client: &DaemonClient) -> Result<()> {
    let path = client.generate_debug_snapshot().await?;
    println!("Generated debug snapshot in {path}");
    Ok(())
}
