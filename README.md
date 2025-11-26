# Realtime Chat Demo

A real-time chat application built with Rust and Supabase, demonstrating [supabase-realtime-rs](https://github.com/Scaraude/supabase-realtime-rs).

## Features

- Real-time message updates using Supabase Realtime (Postgres CDC)
- Built with [Axum](https://github.com/tokio-rs/axum) web framework
- Server-Sent Events (SSE) for live browser updates
- Direct Postgres connection for message persistence

## How It Works

Messages are inserted into Postgres → Postgres triggers CDC event → Supabase Realtime broadcasts → Server receives update → SSE streams to all connected browsers

## Setup

### 1. Create a Supabase project

Create a new project at [supabase.com](https://supabase.com).

### 2. Run the migration

Execute the SQL in [migrations/001_create_chat_table.sql](migrations/001_create_chat_table.sql) in your Supabase SQL Editor.

### 3. Configure environment

```bash
cp .env.example .env
```

Edit `.env` with your Supabase credentials:

```env
SUPABASE_REALTIME_URL=wss://your-project.supabase.co/realtime/v1
SUPABASE_API_KEY=your-anon-key-here
DATABASE_URL=postgresql://postgres:[password]@db.your-project.supabase.co:5432/postgres
```

Find these values in your Supabase project settings:

- **SUPABASE_REALTIME_URL**: API → Realtime URL (replace `https` with `wss` and add `/realtime/v1`)
- **SUPABASE_API_KEY**: API → anon/public key
- **DATABASE_URL**: Database → Connection String → URI (Direct connection)

### 4. Run

```bash
cargo run
```

Open http://127.0.0.1:3000 in multiple browser tabs to see real-time updates.

## Troubleshooting

**Messages not appearing in real-time**

- Verify the migration ran successfully in Supabase SQL Editor
- Check that Realtime is enabled for the table

**Database connection errors**

- Ensure your DATABASE_URL is correct
- Verify your Supabase project is active (not paused)

## License

MIT
