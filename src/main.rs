use anyhow::Result;
use rig::client::{CompletionClient, ProviderClient};
use rig::completion::Prompt as _;
use rig::providers::openai;
use rmcp::{ServiceExt, transport::TokioChildProcess};
use termimad::MadSkin;
use tokio::process::Command;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("info,{}=debug", env!("CARGO_CRATE_NAME")).into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let mut rmcp_client =
        ().serve(TokioChildProcess::new(Command::new("/opt/mcp_forecast"))?)
            .await?;

    let tools = rmcp_client.list_tools(Default::default()).await?.tools;

    let client = openai::Client::from_env();

    let agent = client
        .agent("nvidia/nemotron-3-nano")
        .rmcp_tools(tools, rmcp_client.peer().to_owned())
        .preamble("Eres un experto en plagas y enfermedades de las plantas. Responde en español de forma clara y concisa.")
        .build();

    let prompt = r#"Obtén los datos de previsión del tiempo referentes a temperatura, humedad y cantidad de lluvias en relación al riesgo de aparición de plagas en pinos en la localidad de carballo, en el municipio de carballo, entre los días 8 y 10 de abril de 2026.
                        Muestra los resultados en una tabla, que encaje con el tipo de informe, y dado que eres un experto en tratamientos fitosanitarios en pinos, recomienda, o no, actuaciones en función los datos obtenidos."#;

    let response = agent.prompt(prompt).await?;

    let skin = MadSkin::default();

    skin.print_text(&response);

    rmcp_client.close().await?;

    Ok(())
}
