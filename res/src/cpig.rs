use async_bincode::AsyncBincodeStream;
use pigment::{rpc, view::{self, View}};
use structopt::StructOpt;
use tokio::net::TcpStream;

#[derive(Debug, StructOpt)]
struct Opt {
    /// Address of the Pigment server.
    #[structopt(short, long, default_value = "127.0.0.1:4211")]
    host: std::net::SocketAddr,

    #[structopt(subcommand)]
    request: Option<Request>,

    /// Shell to generate completions for.
    #[structopt(long, possible_values = &structopt::clap::Shell::variants())]
    completions: Option<String>,
}

#[derive(Debug, StructOpt)]
enum Request {
    Update {
        #[structopt(short, long, env = "CANVAS_TOKEN")]
        token: String,

        #[structopt(short, long, env = "CANVAS_BASE_URL")]
        canvas: String,

        #[structopt(short, long, env = "CANVAS_USER")]
        user: canvas_lms::Id,
    },
    Query {

    },
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

    let transport = AsyncBincodeStream::from(TcpStream::connect(opt.host).await?).for_async();

    if let Some(req_opt) = opt.request {
        let rpc_req: rpc::Request = match req_opt {
            Request::Update { token, canvas, user } => {
                rpc::Request::Update {
                    canvas_token: token,
                    view: View {
                        truth: view::Canvas {
                            base_url: canvas,
                        },
                        viewer: view::Viewer::User(user),
                    }
                }
            }
            Request::Query { } => {
                todo!()
            }
        };
        rpc_req.send(&mut transport).await?;
    }

    Ok(())
}