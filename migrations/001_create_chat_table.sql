-- Create chat_public_demo table
CREATE TABLE IF NOT EXISTS chat_public_demo (
    id BIGSERIAL PRIMARY KEY,
    text TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create function to auto-update updated_at
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Create trigger to auto-update updated_at on UPDATE
CREATE TRIGGER update_chat_public_demo_updated_at
    BEFORE UPDATE ON chat_public_demo
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Enable Row Level Security (RLS)
ALTER TABLE chat_public_demo ENABLE ROW LEVEL SECURITY;

-- Allow public read access (required for Realtime)
CREATE POLICY "Allow public read access"
    ON chat_public_demo
    FOR SELECT
    USING (true);

-- Allow public insert access (so anyone can post messages)
CREATE POLICY "Allow public insert access"
    ON chat_public_demo
    FOR INSERT
    WITH CHECK (true);

-- Enable Realtime for this table
ALTER PUBLICATION supabase_realtime ADD TABLE chat_public_demo;
