use axum::{
    Form, Router,
    extract::State,
    response::{Html, IntoResponse, Sse},
    routing::{get, post},
};
use futures::stream::{self, Stream};
use serde::{Deserialize, Serialize};
use std::{convert::Infallible, sync::Arc, time::Duration};
use supabase_client_rs::supabase_realtime_rs::{
    PostgresChangeEvent, PostgresChangesFilter, PostgresChangesPayload, RealtimeClient,
    RealtimeClientOptions,
};
use tokio::sync::{RwLock, broadcast};
use tokio_postgres::{Client, NoTls};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// Message structure matching our Postgres table
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Message {
    id: i64,
    text: String,
    created_at: String,
}

// Form data for submitting messages
#[derive(Deserialize)]
struct MessageForm {
    text: String,
}

// Shared application state
// Arc = Atomic Reference Counted (thread-safe shared ownership)
// RwLock = Multiple readers OR single writer
// broadcast::Sender = Sends messages to multiple receivers
#[derive(Clone)]
struct AppState {
    messages: Arc<RwLock<Vec<Message>>>,
    realtime_url: String,
    api_key: String,
    db_client: Arc<RwLock<Client>>,
    tx: broadcast::Sender<Message>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing for logs
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "chat_demo=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load environment variables
    dotenvy::dotenv().ok();
    let realtime_url = std::env::var("SUPABASE_REALTIME_URL")?;
    let api_key = std::env::var("SUPABASE_API_KEY")?;
    let database_url = std::env::var("DATABASE_URL")?;

    // Connect to Postgres
    let (client, connection) = tokio_postgres::connect(&database_url, NoTls).await?;

    // Spawn connection handler
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            tracing::error!("Database connection error: {}", e);
        }
    });

    tracing::info!("Connected to database");

    // Create broadcast channel for real-time message updates
    // 100 is the channel capacity (how many messages can be buffered)
    let (tx, _rx) = broadcast::channel::<Message>(100);

    let messages_as_row = client
        .query(
            "SELECT id, text, created_at::text FROM chat_public_demo ORDER BY created_at ASC",
            &[],
        )
        .await?;
    let messages: Vec<Message> = messages_as_row
        .into_iter()
        .map(|row| Message {
            id: row.get(0),
            text: row.get(1),
            created_at: row.get(2),
        })
        .collect();

    tracing::info!("Loaded {} existing messages", messages.len());
    // Initialize shared state
    let state = AppState {
        messages: Arc::new(RwLock::new(messages)),
        realtime_url: realtime_url.clone(),
        api_key: api_key.clone(),
        db_client: Arc::new(RwLock::new(client)),
        tx,
    };

    // Spawn Realtime listener in background task
    let state_clone = state.clone();
    tokio::spawn(async move {
        if let Err(e) = start_realtime_listener(state_clone).await {
            tracing::error!("Realtime listener error: {}", e);
        }
    });

    // Build Axum router
    let app = Router::new()
        .route("/", get(index_handler))
        .route("/messages", post(submit_message))
        .route("/events", get(sse_handler))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    // Start web server
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    tracing::info!("Server running on http://127.0.0.1:3000");

    axum::serve(listener, app).await?;

    Ok(())
}

// Start Realtime client and listen for Postgres changes
async fn start_realtime_listener(state: AppState) -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!("Starting Realtime listener...");

    // Connect to Supabase Realtime
    let client = RealtimeClient::new(
        &state.realtime_url,
        RealtimeClientOptions {
            api_key: state.api_key.clone(),
            ..Default::default()
        },
    )?;
    client.connect().await?;

    // Subscribe to chat_public_demo table changes
    let channel = client.channel("chat-changes", Default::default()).await;

    let mut rx = channel
        .on_postgres_changes(
            PostgresChangesFilter::new(PostgresChangeEvent::Insert, "public")
                .table("chat_public_demo"),
        )
        .await;

    channel.subscribe().await?;
    tracing::info!("Subscribed to chat_public_demo changes");

    // Listen for new messages
    while let Some(payload) = rx.recv().await {
        tracing::info!("Received Postgres change: {:?}", payload);

        match payload {
            PostgresChangesPayload::Insert(insert_payload) => {
                // Convert HashMap to JSON Value, then deserialize to Message struct
                let message = match serde_json::to_value(&insert_payload.new) {
                    Ok(value) => match serde_json::from_value::<Message>(value) {
                        Ok(msg) => msg,
                        Err(e) => {
                            tracing::error!("Failed to deserialize message from payload: {}", e);
                            continue;
                        }
                    },
                    Err(e) => {
                        tracing::error!("Failed to convert HashMap to JSON value: {}", e);
                        continue;
                    }
                };

                tracing::info!("New message: {:?}", message);

                // Add to shared state
                let mut messages = state.messages.write().await;
                messages.push(message.clone());

                // Broadcast to all SSE clients
                let _ = state.tx.send(message);
            }
            _ => {}
        }
    }

    Ok(())
}

// Handler for main page
async fn index_handler(State(state): State<AppState>) -> impl IntoResponse {
    let messages = state.messages.read().await;

    let messages_html: String = messages
        .iter()
        .map(|msg| format!("<div class='message'>{}</div>", msg.text))
        .collect();

    Html(format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>Realtime Chat Demo</title>
    <style>
        body {{
            font-family: system-ui, -apple-system, sans-serif;
            max-width: 600px;
            margin: 40px auto;
            padding: 20px;
            background: #f5f5f5;
        }}
        h1 {{
            color: #333;
            text-align: center;
        }}
        #messages {{
            background: white;
            border-radius: 8px;
            padding: 20px;
            min-height: 300px;
            max-height: 500px;
            overflow-y: auto;
            margin-bottom: 20px;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
        }}
        .message {{
            padding: 10px;
            margin: 8px 0;
            background: #f0f0f0;
            border-radius: 4px;
        }}
        form {{
            display: flex;
            gap: 10px;
        }}
        input[type="text"] {{
            flex: 1;
            padding: 12px;
            border: 1px solid #ddd;
            border-radius: 4px;
            font-size: 16px;
        }}
        button {{
            padding: 12px 24px;
            background: #3b82f6;
            color: white;
            border: none;
            border-radius: 4px;
            cursor: pointer;
            font-size: 16px;
        }}
        button:hover {{
            background: #2563eb;
        }}
    </style>
</head>
<body>
    <h1>🦀 Realtime Chat Demo</h1>
    <div id="messages">{messages_html}</div>
    <form method="POST" action="/messages">
        <input type="text" name="text" placeholder="Type a message..." required autofocus>
        <button type="submit">Send</button>
    </form>

    <script>
        // Connect to Server-Sent Events for live updates
        const evtSource = new EventSource('/events');

        evtSource.addEventListener('message', function(event) {{
            const messagesDiv = document.getElementById('messages');
            const messageDiv = document.createElement('div');
            messageDiv.className = 'message';
            messageDiv.textContent = event.data;
            messagesDiv.appendChild(messageDiv);
            messagesDiv.scrollTop = messagesDiv.scrollHeight;
        }});

        evtSource.onerror = function() {{
            console.error('EventSource failed, reconnecting...');
        }};

        // Handle form submission via JavaScript to avoid page reload
        const form = document.querySelector('form');
        const input = document.querySelector('input[name="text"]');

        form.addEventListener('submit', async function(e) {{
            e.preventDefault();

            const text = input.value.trim();
            if (!text) return;

            // Submit the form data
            await fetch('/messages', {{
                method: 'POST',
                headers: {{
                    'Content-Type': 'application/x-www-form-urlencoded',
                }},
                body: 'text=' + encodeURIComponent(text)
            }});

            // Clear the input field
            input.value = '';
            input.focus();
        }});
    </script>
</body>
</html>"#,
        messages_html = messages_html
    ))
}

// Handler for message submission
async fn submit_message(
    State(state): State<AppState>,
    Form(form): Form<MessageForm>,
) -> impl IntoResponse {
    tracing::info!("Received message: {}", form.text);

    // Insert message into Postgres
    let client = state.db_client.read().await;
    match client
        .execute(
            "INSERT INTO chat_public_demo (text) VALUES ($1)",
            &[&form.text],
        )
        .await
    {
        Ok(_) => tracing::info!("Message inserted successfully"),
        Err(e) => tracing::error!("Failed to insert message: {}", e),
    }

    // Return 204 No Content - don't redirect, let SSE update the UI
    axum::http::StatusCode::NO_CONTENT
}

// Server-Sent Events handler for live updates
async fn sse_handler(
    State(state): State<AppState>,
) -> Sse<impl Stream<Item = Result<axum::response::sse::Event, Infallible>>> {
    // Subscribe to broadcast channel
    let rx = state.tx.subscribe();

    let stream = stream::unfold(rx, |mut rx| async move {
        // Wait for new messages from the broadcast channel
        match rx.recv().await {
            Ok(message) => {
                let event = axum::response::sse::Event::default().data(message.text);
                Some((Ok(event), rx))
            }
            Err(_) => {
                // Channel closed or lagged, keep connection alive
                let event = axum::response::sse::Event::default().comment("keepalive");
                Some((Ok(event), rx))
            }
        }
    });

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("keepalive"),
    )
}
