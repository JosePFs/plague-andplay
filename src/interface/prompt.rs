use std::fmt::Write as _;
use std::io::stdout;
use std::time::{Duration, Instant};

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
use termimad::MadSkin;

use crate::application::use_case::get_risk_summary::GetRiskSummaryUseCaseResult;
use crate::domain::plague::PlagueType;

const AGENT_MODEL: &str = "qwen/qwen3.5-35b-a3b";
const REDRAW_MIN_INTERVAL: Duration = Duration::from_millis(1000);
const REDRAW_MIN_NEW_BYTES: usize = 1024;

pub async fn run_prompt(risk_summary: &GetRiskSummaryUseCaseResult) -> Result<()> {
    let client = openai::Client::from_env().completions_api();

    let agent = client
        .agent(AGENT_MODEL)
        .preamble(
            "You act as an expert in plant pests and diseases, with a focus on pine phytosanitary management. \
Always write your entire reply in clear, professional Spanish. \
Use Markdown; prefer tables when they make the analysis easier to read. \
Be **consistent** across runs: for the same input data, keep the **same report structure** (section order, table columns, and depth). \
Avoid varying intros, closings, or decorative filler; state facts and conclusions plainly.",
        )
        .temperature(0.0)
        .build();

    let user_prompt = build_user_prompt(risk_summary);
    let mut stream = agent.stream_prompt(user_prompt).multi_turn(8).await;

    let skin = MadSkin::default();
    let mut full_text = String::new();
    let mut last_redraw = Instant::now()
        .checked_sub(REDRAW_MIN_INTERVAL)
        .unwrap_or_else(Instant::now);
    let mut last_painted_len: usize = 0;

    while let Some(item) = stream.next().await {
        let item = item.map_err(|e| anyhow::anyhow!("{e}"))?;
        match item {
            MultiTurnStreamItem::StreamAssistantItem(content) => match content {
                StreamedAssistantContent::Text(t) => {
                    full_text.push_str(t.text());
                    maybe_repaint_stream(
                        &skin,
                        &full_text,
                        &mut last_redraw,
                        &mut last_painted_len,
                        false,
                    )?;
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
                    maybe_repaint_stream(
                        &skin,
                        &full_text,
                        &mut last_redraw,
                        &mut last_painted_len,
                        false,
                    )?;
                }
                StreamedAssistantContent::ReasoningDelta { reasoning, .. } => {
                    full_text.push_str(&reasoning);
                    maybe_repaint_stream(
                        &skin,
                        &full_text,
                        &mut last_redraw,
                        &mut last_painted_len,
                        false,
                    )?;
                }
                StreamedAssistantContent::Final(_) => {
                    repaint_stream_full(
                        &skin,
                        &full_text,
                        &mut last_redraw,
                        &mut last_painted_len,
                    )?;
                }
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
                repaint_stream_full(&skin, &full_text, &mut last_redraw, &mut last_painted_len)?;
            }
            _ => {}
        }
    }

    repaint_stream_full(&skin, &full_text, &mut last_redraw, &mut last_painted_len)?;

    Ok(())
}

fn build_user_prompt(risk_summary: &GetRiskSummaryUseCaseResult) -> String {
    let mut out = String::new();

    let _ = writeln!(
        out,
        "## Instructions\n\
\n\
**Response language:** Spanish only (for the entire answer).\n\
\n\
- Use **only** the pests/diseases described under «Context: relevant pests and diseases» (each `id` and its associated text).\n\
- For each location and date, interpret **risk** using the weather table and the **IDs flagged at risk**, matching each ID to its context entry.\n\
- **Risk rules:** **Diseases** — mean temperature within the stated range, mean relative humidity ≥ minimum, and if minimum daily precipitation (mm) > 0 then the **sum of timestep precipitation for that day** must be ≥ that minimum. **Pests** — same temperature and humidity checks; **`min_precipitation` is not used**; if **maximum daily precipitation (mm)** appears in context, a day is not flagged when the daily sum **exceeds** that maximum.\n\
- Present the analysis in **Markdown tables** when helpful.\n\
- As a pine phytosanitary expert, state whether you recommend **action**, **monitoring**, or **no action**, reasoning from the data and context thresholds.\n\
- **Report shape (mandatory):** Use **exactly** these top-level `##` sections **in this order** (Spanish headings as written): \
`## Resumen`, then `## Riesgo por lugar y fecha` (tables: location, date, weather, plague name), \
then `## Recomendaciones fitosanitarias`, then if and only if needed `## Limitaciones e incertidumbre`. \
Do not add other top-level sections or reorder them.\n\
"
    );

    let _ = writeln!(out, "## Context: relevant pests and diseases\n");
    if risk_summary.risk_plagues().is_empty() {
        let _ = writeln!(
            out,
            "_No pests or diseases at risk according to the supplied data._\n"
        );
    } else {
        for plague in risk_summary.risk_plagues() {
            let _ = writeln!(out, "### `{}`\n", plague.id);
            let _ = writeln!(out, "{}\n", plague.text);
            let _ = writeln!(
                out,
                "- Reference: temperature {:.1}–{:.1} °C, minimum mean relative humidity {:.1} %.\n\
- Type: {}.\n",
                plague.metadata.min_temp,
                plague.metadata.max_temp,
                plague.metadata.min_humidity,
                plague_kind(&plague.metadata.r#type),
            );
            match plague.metadata.r#type {
                PlagueType::Disease => {
                    let _ = writeln!(
                        out,
                        "- Engine uses **minimum daily precipitation sum (mm)** {:.1}; if 0, precipitation is not required for the flag.\n",
                        plague.metadata.min_precipitation,
                    );
                }
                PlagueType::Pest => {
                    let max_line = match plague.metadata.max_precipitation {
                        Some(m) => format!(
                            "- Engine **maximum daily precipitation sum (mm)** {:.1} (days above are not flagged); `min_precipitation` in JSON is ignored.\n",
                            m
                        ),
                        None => "- Engine does not apply a daily precipitation ceiling; `min_precipitation` in JSON is ignored.\n"
                            .to_string(),
                    };
                    let _ = write!(out, "{max_line}");
                }
            }
        }
    }

    let _ = writeln!(out, "## Weather data and risk by location\n");
    for place_summary in risk_summary.place_forecast_risk_summaries() {
        let place = &place_summary.place;
        let _ = writeln!(
            out,
            "### Location: {} ({})\n",
            place.name, place.municipality
        );

        let mut dates: Vec<String> = place_summary.daily_risk_summary.keys().cloned().collect();
        dates.sort();

        for date in dates {
            let Some(day) = place_summary.daily_risk_summary.get(&date) else {
                continue;
            };
            let _ = writeln!(out, "#### Date: {date}\n");
            let _ = writeln!(
                out,
                "| Variable | Value |\n|----------|-------|\n\
                | Mean temperature (°C) | {:.2} |\n\
                | Mean relative humidity (%) | {:.2} |\n\
                | Daily precipitation sum (mm, timestep sum) | {:.2} |\n",
                day.average_temperature,
                day.average_relative_humidity,
                day.precipitation_amount_accumulated,
            );

            if day.risk_plagues.is_empty() {
                let _ = writeln!(
                    out,
                    "_No context pests or diseases at risk for this day._\n"
                );
            } else {
                let _ = writeln!(out, "Pests at risk (link to context entries):\n");
                for pid in &day.risk_plagues {
                    let id_str: String = pid.clone().into();
                    let _ = writeln!(out, "- {}", plague_label_for_id(risk_summary, &id_str));
                }
                let _ = writeln!(out);
            }
        }
    }

    out
}

fn plague_kind(t: &PlagueType) -> &'static str {
    match t {
        PlagueType::Pest => "pest",
        PlagueType::Disease => "disease",
    }
}

fn plague_label_for_id(risk_summary: &GetRiskSummaryUseCaseResult, id: &str) -> String {
    risk_summary
        .risk_plagues()
        .iter()
        .find(|p| p.id == id)
        .map(|p| {
            let preview: String = p.text.chars().take(120).collect();
            let suffix = if p.text.chars().count() > 120 {
                "…"
            } else {
                ""
            };
            format!("`{id}` — {preview}{suffix}")
        })
        .unwrap_or_else(|| format!("`{id}` (no matching entry in supplied context)"))
}

fn clear_screen() -> Result<(), anyhow::Error> {
    let mut stdout = stdout();
    execute!(stdout, Clear(ClearType::All), MoveTo(0, 0))?;
    Ok(())
}

fn maybe_repaint_stream(
    skin: &MadSkin,
    full_text: &str,
    last_redraw: &mut Instant,
    last_painted_len: &mut usize,
    force: bool,
) -> Result<()> {
    let new_bytes = full_text.len().saturating_sub(*last_painted_len);
    let due_by_time = last_redraw.elapsed() >= REDRAW_MIN_INTERVAL;
    if force || due_by_time || new_bytes >= REDRAW_MIN_NEW_BYTES {
        repaint_stream_full(skin, full_text, last_redraw, last_painted_len)?;
    }
    Ok(())
}

fn repaint_stream_full(
    skin: &MadSkin,
    full_text: &str,
    last_redraw: &mut Instant,
    last_painted_len: &mut usize,
) -> Result<()> {
    clear_screen()?;
    skin.print_text(full_text);
    *last_redraw = Instant::now();
    *last_painted_len = full_text.len();
    Ok(())
}
