-- Marketplace schema for Pawtner API.
-- This mirrors the infra demo schema to keep table and column names aligned.

CREATE TABLE IF NOT EXISTS marketplace_users (
  id UUID PRIMARY KEY,
  keycloak_username TEXT NOT NULL UNIQUE,
  role TEXT NOT NULL CHECK (role IN ('merchant', 'client')),
  email TEXT NOT NULL UNIQUE,
  display_name TEXT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS merchant_profiles (
  merchant_user_id UUID PRIMARY KEY REFERENCES marketplace_users(id) ON DELETE CASCADE,
  profile_code TEXT NOT NULL UNIQUE,
  label_score INTEGER NOT NULL CHECK (label_score BETWEEN 0 AND 100),
  is_certified BOOLEAN NOT NULL DEFAULT FALSE,
  is_family_style BOOLEAN NOT NULL DEFAULT FALSE,
  location TEXT NOT NULL,
  specialties TEXT[] NOT NULL DEFAULT '{}'
);

CREATE TABLE IF NOT EXISTS marketplace_offers (
  id UUID PRIMARY KEY,
  offer_code TEXT NOT NULL UNIQUE,
  merchant_user_id UUID NOT NULL REFERENCES marketplace_users(id) ON DELETE RESTRICT,
  name TEXT NOT NULL,
  animal_type TEXT NOT NULL CHECK (animal_type IN ('dog', 'cat', 'horse')),
  breed TEXT NOT NULL,
  gender TEXT NOT NULL CHECK (gender IN ('M', 'F')),
  birth_date DATE NOT NULL,
  price_eur NUMERIC(10, 2) NOT NULL CHECK (price_eur >= 0),
  location TEXT NOT NULL,
  listing_type TEXT NOT NULL CHECK (listing_type IN ('sale', 'stud')),
  image_url TEXT NOT NULL,
  cycle_status TEXT CHECK (cycle_status IN ('rest', 'heat', 'pregnancy', 'nursing')),
  is_available_for_club BOOLEAN NOT NULL DEFAULT FALSE,
  description TEXT NOT NULL,
  status TEXT NOT NULL DEFAULT 'published' CHECK (status IN ('draft', 'published', 'archived')),
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS merchant_reviews (
  id UUID PRIMARY KEY,
  review_code TEXT NOT NULL UNIQUE,
  merchant_user_id UUID NOT NULL REFERENCES marketplace_users(id) ON DELETE CASCADE,
  author_name TEXT NOT NULL,
  rating INTEGER NOT NULL CHECK (rating BETWEEN 1 AND 5),
  comment TEXT NOT NULL,
  reviewed_at DATE NOT NULL
);

CREATE TABLE IF NOT EXISTS marketplace_orders (
  id UUID PRIMARY KEY,
  order_code TEXT NOT NULL UNIQUE,
  client_user_id UUID NOT NULL REFERENCES marketplace_users(id) ON DELETE RESTRICT,
  merchant_user_id UUID NOT NULL REFERENCES marketplace_users(id) ON DELETE RESTRICT,
  offer_id UUID NOT NULL REFERENCES marketplace_offers(id) ON DELETE RESTRICT,
  status TEXT NOT NULL CHECK (status IN ('pending', 'confirmed', 'completed', 'cancelled')),
  amount_eur NUMERIC(10, 2) NOT NULL CHECK (amount_eur >= 0),
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS marketplace_monthly_sales_metrics (
  id UUID PRIMARY KEY,
  metric_code TEXT NOT NULL UNIQUE,
  metric_year INTEGER NOT NULL CHECK (metric_year >= 2000),
  month_index INTEGER NOT NULL CHECK (month_index BETWEEN 1 AND 12),
  merchant_user_id UUID REFERENCES marketplace_users(id) ON DELETE CASCADE,
  amount_eur NUMERIC(10, 2) NOT NULL CHECK (amount_eur >= 0)
);

