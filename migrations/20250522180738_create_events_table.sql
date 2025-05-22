CREATE TABLE IF NOT EXISTS events (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL,
    result TEXT               -- برنده واقعی، null یعنی هنوز نتیجه نیومده
);
