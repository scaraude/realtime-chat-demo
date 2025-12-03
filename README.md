# Realtime Chat Demo

A real-time chat application built with Rust and Supabase, demonstrating [supabase-client-rs](https://github.com/Scaraude/supabase-client-rs).

## Features

- Real-time message updates using Supabase Realtime (Postgres CDC)
- Built with [Axum](https://github.com/tokio-rs/axum) web framework
- Server-Sent Events (SSE) for live browser updates
- PostgREST for database operations via [supabase-client-rs](https://github.com/Scaraude/supabase-client-rs)

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
SUPABASE_URL=https://your-project.supabase.co
SUPABASE_API_KEY=your-anon-key-here
```

Find these values in your Supabase project settings:

- **SUPABASE_URL**: Project Settings → API → Project URL
- **SUPABASE_API_KEY**: Project Settings → API → anon/public key

### 4. Run

```bash
cargo run
```

Open http://127.0.0.1:3000 in multiple browser tabs to see real-time updates.

## Troubleshooting

**Messages not appearing in real-time**

- Verify the migration ran successfully in Supabase SQL Editor
- Check that Realtime is enabled for the table

**Connection errors**

- Ensure your SUPABASE_URL and SUPABASE_API_KEY are correct
- Verify your Supabase project is active (not paused)
- Check that your table has the correct name (`chat_public_demo`)

## License

MIT
