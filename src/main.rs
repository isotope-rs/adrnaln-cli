
use std::path::Path;


use adrnaln::client::sequence::Sequence;
use adrnaln::config::Addresses;
use clap::{Parser, Subcommand};
use tokio::fs;



#[derive(Debug, clap::Args, Clone)]
struct ClientArgs {
    #[clap(short,long)]
    ip: Option<String>,
    #[clap(short, long)]
    port: Option<String>,
    #[clap(short, long)]
    file: Option<String>
}
#[derive(Debug, clap::Args, Clone)]
struct ServerArgs {
    #[clap(short, long)]
    port: Option<String>,
}
#[derive(Subcommand, Debug, Clone)]
pub enum Mode {
    Server(ServerArgs),
    Client(ClientArgs)
}
#[derive(Parser, Debug)]
#[command(version)]
pub struct Args {
    #[command(subcommand)]
    mode: Mode,

}
#[tokio::main]
async fn main() {
    let ars = Args::parse();

    match ars.mode {

        Mode::Server(args) => {

            if args.port.is_none() {
                println!("Port is required");
                return
            }
            let (tx, mut rx) = tokio::sync::mpsc::channel::<Sequence>(100);
            let (_kill_tx,kill_rx) = tokio::sync::oneshot::channel();
            let config = adrnaln::config::Configuration{
                addresses: Addresses { local_address: format!("0.0.0.0:{}", args.port.unwrap()).parse().unwrap(),
                    remote_address: "0.0.0.0:0".parse().unwrap() },
                sequence_tx: tx,
            };
            let mut server = adrnaln::server::Server::new(config);
            tokio::spawn(async move {
               loop {
                   match rx.recv().await {
                       None => {}
                       Some(seq) => {
                          write_sequence_to_file(".",seq).await
                       }
                   }

               }
            });

            server.start(kill_rx).await;

        }
        Mode::Client(args) => {
            if args.port.is_none() || args.ip.is_none() || args.file.is_none(){
                println!("Port, IP and File are required");
                return
            }
            let addresses = Addresses { local_address: "0.0.0.0:0".parse().unwrap(),
                remote_address: format!("{}:{}", args.ip.unwrap(), args.port.unwrap()).parse().unwrap() };

            let client = adrnaln::client::Client::new(addresses);
            let sequence = client.build_sequence_from_file(args.file.unwrap().as_str()).await;
            client.send_sequence(sequence.unwrap()).await.expect("Error sending sequence");
        }
    }
}
pub async fn write_sequence_to_file(file_path: &str, sequence: Sequence) {
    let mut bytes = vec![];
    let mut filename = "".to_string();
    for packets in &sequence.packets {
        bytes.extend(packets.clone().bytes);
        if filename.is_empty() {
            filename = packets.filename.clone();
        }
    }
    let path = Path::new(".").join(file_path).join(&filename);
    fs::write(path, &bytes)
        .await
        .expect("Could not write file!");
}
