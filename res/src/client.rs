#![feature(result_flattening)]

use async_bincode::AsyncBincodeStream;
use futures::StreamExt;
use miette::{Context, IntoDiagnostic};
use pigment::{
    rpc::*,
    view::{self, View},
};
use structopt::StructOpt;
use tokio::net::TcpStream;

#[derive(Debug, StructOpt)]
struct Opt {
    /// Address of the Pigment server.
    #[structopt(short, long, default_value = "127.0.0.1:4211")]
    host: std::net::SocketAddr,

    #[structopt(subcommand)]
    request: Option<Verb>,

    /// Shell to generate completions for.
    #[structopt(long, possible_values = &structopt::clap::Shell::variants())]
    completions: Option<String>,
}

#[derive(Debug, StructOpt)]
enum Verb {
    Update {
        #[structopt(short, long, env = "CANVAS_TOKEN")]
        token: String,

        #[structopt(short, long, env = "CANVAS_BASE_URL")]
        canvas: String,

        #[structopt(short, long, env = "CANVAS_USER")]
        user: canvas_lms::Id,
    },
    Query {},
}

#[tokio::main]
async fn main() -> miette::Result<()> {
    pretty_env_logger::init();

    let opt = Opt::from_args();

    if let Some(shell) = opt.completions {
        Opt::clap().gen_completions_to(
            "cpig",
            shell.parse().unwrap(),
            &mut std::io::stdout().lock(),
        );
        return Ok(());
    }

    let mut transport = AsyncBincodeStream::<_, Result<Response, String>, Request, _>::from(
        TcpStream::connect(opt.host)
            .await
            .into_diagnostic()
            .wrap_err("while connecting to host")?,
    )
    .for_async()
    .map(|r| r.map_err(|e| e.to_string()).flatten());

    if let Some(req_opt) = opt.request {
        let rpc_req: Request = match req_opt {
            Verb::Update {
                token,
                canvas,
                user,
            } => Request::Update {
                canvas_token: token,
                view: View {
                    truth: view::Canvas { base_url: canvas },
                    viewer: view::Viewer::User(user),
                },
            },
            Verb::Query {} => {
                todo!()
            }
        };

        let mut resps  = rpc_req
            .send(&mut transport)
            .await
            .into_diagnostic()
            .wrap_err("while sending request")?;

        while let Some(resp) = resps.next().await {
            println!("{:?}", resp);
        }
    }

    Ok(())
}
