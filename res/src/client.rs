#![feature(result_flattening)]
#![feature(box_patterns)]

use async_bincode::AsyncBincodeStream;
use canvas_lms::DateTime;
use ebauche::rpc::*;
use futures::{future, StreamExt};
use miette::{Context, IntoDiagnostic};
use pigment::view::{self, View};
use pigment::ResourceKind;
use structopt::StructOpt;
use tokio::net::TcpStream;

#[derive(Debug, StructOpt)]
struct Opt {
    /// Address of the Ebauche server.
    #[structopt(short, long, default_value = "127.0.0.1:4211")]
    host: std::net::SocketAddr,

    /// Hide responses.
    #[structopt(long)]
    hide: bool,

    #[structopt(subcommand)]
    request: Option<Verb>,

    /// Shell to generate completions for.
    #[structopt(long, possible_values = &structopt::clap::Shell::variants())]
    completions: Option<String>,
}

#[derive(Debug, StructOpt)]
enum Verb {
    Fetch {
        #[structopt(short, long, env = "CANVAS_TOKEN")]
        token: String,

        #[structopt(short, long, env = "CANVAS_BASE_URL")]
        canvas: String,

        #[structopt(short, long, env = "CANVAS_USER")]
        user: canvas_lms::Id,
    },
    Diff {
        #[structopt(short, long, env = "CANVAS_BASE_URL")]
        canvas: String,

        #[structopt(short, long, env = "CANVAS_USER")]
        user: canvas_lms::Id,

        #[structopt(short, long)]
        since: DateTime,

        #[structopt(short, long)]
        kind: ResourceKind,
    },
}

#[tokio::main]
async fn main() -> miette::Result<()> {
    tracing_subscriber::fmt().pretty().init();

    let opt = Opt::from_args();

    if let Some(shell) = opt.completions {
        Opt::clap().gen_completions_to(
            "cpig",
            shell.parse().unwrap(),
            &mut std::io::stdout().lock(),
        );
        return Ok(());
    }

    tracing::info!("initiating transport...");
    let transport: &'static mut _ = Box::leak(Box::new(
        AsyncBincodeStream::<_, Result<Response, String>, Request, _>::from(
            TcpStream::connect(opt.host)
                .await
                .into_diagnostic()
                .wrap_err("failed connecting to host")?,
        )
        .for_async()
        .take_while(|e| {
            future::ready(match e {
                Err(box bincode::ErrorKind::Io(e)) => {
                    e.kind() != std::io::ErrorKind::ConnectionReset
                }
                _ => true,
            })
        }),
    ));

    if let Some(req_opt) = opt.request {
        let rpc_req: Request = match req_opt {
            Verb::Fetch {
                token,
                canvas,
                user,
            } => Request::FetchUpstream {
                canvas_token: token,
                view: View {
                    truth: view::Canvas { base_url: canvas },
                    viewer: view::Viewer::User(user),
                },
            },
            Verb::Diff {
                canvas,
                user,
                since,
                kind,
            } => Request::Diff {
                view: View {
                    truth: view::Canvas { base_url: canvas },
                    viewer: view::Viewer::User(user),
                },
                since,
                resource_kind: kind,
            },
        };

        tracing::info!("sending request...");
        let mut resps = rpc_req
            .send(transport)
            .await
            .into_diagnostic()
            .wrap_err("while sending request")?;

        tracing::info!("awaiting responses...");
        let mut count = 0;
        while let Some(resp) = resps.next().await {
            count += 1;
            match resp? {
                Ok(r) if !opt.hide => println!("{:#?}", r),
                Err(e) => tracing::error!("got error from host: {}", e),
                _ => {}
            }
        }
        tracing::info!(message = "finished recieving responses", count);
    }

    Ok(())
}
