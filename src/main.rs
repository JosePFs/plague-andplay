use std::io::stdout;

use anyhow::Result;
use crossterm::{
    cursor::MoveTo,
    execute,
    terminal::{Clear, ClearType},
};
use futures::StreamExt as _;
use rig::agent::MultiTurnStreamItem;
use rig::client::{CompletionClient, ProviderClient};
use rig::providers::openai;
use rig::streaming::{StreamedAssistantContent, StreamingPrompt};
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

    let client = openai::Client::from_env().completions_api();

    let agent = client
        .agent("nvidia/nemotron-3-nano")
        .rmcp_tools(tools, rmcp_client.peer().to_owned())
        .preamble("Eres un experto en plagas y enfermedades de las plantas. Responde en español de forma clara y concisa.")
        .build();

    let prompt = r#"Obtén los datos de previsión del tiempo referentes a temperatura, humedad y cantidad de lluvias en relación al riesgo de aparición de plagas en pinos en la localidad de carballo, en el municipio de carballo, entre los días 8 y 10 de abril de 2026.
                        Muestra los resultados en una tabla, que encaje con el tipo de informe, y dado que eres un experto en tratamientos fitosanitarios en pinos, recomienda, o no, actuaciones en función los datos obtenidos."#;

    let mut stream = agent.stream_prompt(prompt).multi_turn(8).await;

    let skin = MadSkin::default();
    let mut full_text = String::new();

    while let Some(item) = stream.next().await {
        let item = item.map_err(|e| anyhow::anyhow!("{e}"))?;
        match item {
            MultiTurnStreamItem::StreamAssistantItem(content) => match content {
                StreamedAssistantContent::Text(t) => {
                    full_text.push_str(t.text());
                    clear_screen()?;
                    skin.print_text(&full_text);
                }
                StreamedAssistantContent::ToolCall { tool_call, .. } => {
                    tracing::info!(
                        tool = %tool_call.function.name,
                        "Executing tool call"
                    );
                }
                StreamedAssistantContent::ToolCallDelta { .. } => {}
                StreamedAssistantContent::Reasoning(r) => {
                    full_text.push_str(&r.display_text());
                    clear_screen()?;
                    skin.print_text(&full_text);
                }
                StreamedAssistantContent::ReasoningDelta { reasoning, .. } => {
                    full_text.push_str(&reasoning);
                    clear_screen()?;
                    skin.print_text(&full_text);
                }
                StreamedAssistantContent::Final(_) => {}
                #[allow(unreachable_patterns)]
                _ => {}
            },
            MultiTurnStreamItem::StreamUserItem(_) => {
                tracing::debug!("Tool call sent to model");
            }
            MultiTurnStreamItem::FinalResponse(res) => {
                tracing::debug!(
                    output_len = res.response().len(),
                    input_tokens = res.usage().input_tokens,
                    output_tokens = res.usage().output_tokens,
                    "Final response added"
                );
            }
            #[allow(unreachable_patterns)]
            _ => {}
        }
    }

    rmcp_client.close().await?;

    Ok(())
}

fn clear_screen() -> Result<(), anyhow::Error> {
    let mut stdout = stdout();
    execute!(stdout, Clear(ClearType::All), MoveTo(0, 0))?;
    Ok(())
}
