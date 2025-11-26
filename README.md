# 🦀 Realtime Chat Demo

A simple real-time chat application built with Rust, demonstrating [supabase-realtime-rs](https://github.com/Scaraude/supabase-realtime-rs) in action.

## Features

- 💬 Real-time message updates using Supabase Realtime (Postgres CDC)
- 🚀 Built with [Axum](https://github.com/tokio-rs/axum) web framework
- 📡 Server-Sent Events (SSE) for live browser updates
- 🗄️ Direct Postgres connection for message persistence
- 🎨 Clean, minimal UI

## Architecture

```
User submits message
    ↓
Axum POST handler
    ↓
Insert into Postgres (chat_public_demo table)
    ↓
Postgres triggers INSERT event
    ↓
Supabase Realtime broadcasts event
    ↓
supabase-realtime-rs receives event
    ↓
Updates shared state
    ↓
SSE streams to all connected browsers
    ↓
UI updates automatically
```

## Setup

### 1. Create Supabase Project

Go to [supabase.com](https://supabase.com) and create a new project.

### 2. Run Database Migration

Execute the SQL in `migrations/001_create_chat_table.sql` in your Supabase SQL Editor. This will:
- Create the `chat_public_demo` table
- Set up RLS policies for public read/insert
- Enable Realtime for the table
- Add auto-update trigger for `updated_at`

### 3. Configure Environment

```bash
cp .env.example .env
```

Edit `.env` with your Supabase credentials:

```env
SUPABASE_REALTIME_URL=wss://your-project.supabase.co/realtime/v1
SUPABASE_API_KEY=your-anon-key-here
DATABASE_URL=postgresql://postgres:[password]@db.your-project.supabase.co:5432/postgres
```

**Where to find these values:**
- **SUPABASE_REALTIME_URL**: Project Settings → API → Realtime URL (replace `https` with `wss` and add `/realtime/v1`)
- **SUPABASE_API_KEY**: Project Settings → API → `anon` `public` key
- **DATABASE_URL**: Project Settings → Database → Connection String → URI (use Direct connection)

### 4. Run the Application

```bash
cargo run
```

Open http://127.0.0.1:3000 in your browser(s).

## How It Works

### Realtime Listener ([main.rs](src/main.rs#L111-L154))

The app spawns a background task that:
1. Connects to Supabase Realtime
2. Subscribes to `chat_public_demo` table INSERT events
3. Parses incoming messages and adds them to shared state

```rust
let mut rx = channel
    .on_postgres_changes(
        PostgresChangesFilter::new(PostgresChangeEvent::Insert, "public")
            .table("chat_public_demo"),
    )
    .await;

while let Some(payload) = rx.recv().await {
    // Parse and store message
}
```

### Message Submission ([main.rs](src/main.rs#L256-L277))

When a user submits the form:
1. Axum receives POST request
2. Inserts message into Postgres
3. Postgres triggers Realtime event (automatically)
4. User sees message appear via SSE

### Server-Sent Events ([main.rs](src/main.rs#L279-L307))

Each browser connection gets a continuous stream:
1. Every second, checks for new messages in shared state
2. Sends new messages to the browser
3. Browser JavaScript appends to the DOM

## Key Rust Concepts Demonstrated

1. **Arc<RwLock<T>>** - Thread-safe shared state
   - `Arc` = Multiple ownership across threads
   - `RwLock` = Multiple readers OR single writer

2. **Tokio spawn** - Background async tasks
   - Realtime listener runs independently
   - Database connection handler runs independently

3. **Axum extractors** - Type-safe request handling
   - `State` for shared application state
   - `Form` for parsing form data

4. **Stream unfold** - Infinite async iteration
   - Used for Server-Sent Events
   - Properly handles lock dropping to avoid borrow checker issues

## Testing

Open multiple browser tabs to http://127.0.0.1:3000 and send messages. You should see:
- Messages appear in all tabs in real-time
- New tabs load existing messages from shared state
- Network tab shows SSE connection streaming updates

## Troubleshooting

**"failed to lookup address information"**
- Check your `DATABASE_URL` is correct
- Try using the "Session pooler" connection string instead

**Messages not appearing in real-time:**
- Verify the SQL migration ran successfully
- Check that Realtime is enabled: `ALTER PUBLICATION supabase_realtime ADD TABLE chat_public_demo;`
- Ensure RLS policies allow SELECT on the table

**Connection refused:**
- Make sure your Supabase project is awake (not paused)
- Verify firewall isn't blocking WebSocket connections

## Production Considerations

This is a **demo application**. For production use, consider:

- [ ] Add authentication (Supabase Auth integration)
- [ ] Rate limiting on message submission
- [ ] Message validation and sanitization (XSS protection)
- [ ] Pagination for loading old messages
- [ ] Better error handling and user feedback
- [ ] Graceful shutdown handling
- [ ] Connection pool for Postgres
- [ ] TLS for database connection (use `tokio-postgres-rustls`)

## Learn More

- [supabase-realtime-rs docs](https://docs.rs/supabase-realtime-rs)
- [Axum documentation](https://docs.rs/axum)
- [Tokio tutorial](https://tokio.rs/tokio/tutorial)
- [Supabase Realtime docs](https://supabase.com/docs/guides/realtime)

## License

MIT
